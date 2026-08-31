mod config;
mod settings;

use std::{fs, os::windows::process::CommandExt};

use settings::SharedSettings;

use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Emitter, Manager,
};

fn helper_path(file_name: &str) -> Result<std::path::PathBuf, String> {
    let current_exe =
        std::env::current_exe().map_err(|e| format!("Failed to get current exe path: {e}"))?;
    let path = current_exe
        .parent()
        .ok_or_else(|| "Current exe has no parent directory".to_string())?
        .join(if cfg!(target_os = "windows") {
            file_name
        } else {
            file_name
        });
    Ok(path)
}

fn elevate_in_background(command: &str) {
    let helper = match helper_path("talku-cli.exe") {
        Ok(p) => p,
        Err(e) => {
            eprintln!("helper_path error: {e}");
            return;
        }
    };
    let cmd = command.to_string();
    println!("elevating talku-cli in background, command = '{cmd}'");
    std::thread::spawn(move || {
        let mut process = std::process::Command::new(&helper);
        process.arg(&cmd);
        let elevated = elevated_command::Command::new(process);
        match elevated.output() {
            Ok(out) => println!("elevated '{cmd}' returned status: {:?}", out.status),
            Err(e) => eprintln!("elevated '{cmd}' failed: {e}"),
        }
    });
}

/// Start the pre-enrolled "TalkUCLI" Windows service (see
/// `src-tauri/windows/hooks.nsh`) which runs `talku-cli` as SYSTEM with highest
/// privileges. STARTING a service requires admin, so this runs an ELEVATED
/// `talku-cli service-start` (which uses the `windows-service` crate to tell
/// the SCM to start the service). The elevation shows a UAC prompt; there is no
/// way to start a service without it unless the process is already elevated.
/// Returns true once the elevated start has been launched (the caller then
/// polls the named pipe until the service is reachable).
fn start_service() -> bool {
    elevate_in_background("service-start");
    true
}

/// Is the TalkUCLI service registered with the SCM? Uses the `windows-service`
/// crate directly (no `sc` subprocess). Querying/open is permitted for the
/// interactive (non-elevated) user, so no elevation is needed.
fn is_service_registered() -> bool {
    use windows_service::service::ServiceAccess;
    use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};

    let manager = match ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)
    {
        Ok(m) => m,
        Err(_) => return false,
    };
    manager
        .open_service(SERVICE_NAME, ServiceAccess::QUERY_STATUS)
        .is_ok()
}

/// Is the TalkUCLI service registered AND currently running? Used to avoid
/// needlessly re-starting an already-running service.
fn is_service_running() -> bool {
    use windows_service::service::ServiceAccess;
    use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};

    if !is_service_registered() {
        return false;
    }
    let manager = match ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)
    {
        Ok(m) => m,
        Err(_) => return false,
    };
    let service = match manager.open_service(SERVICE_NAME, ServiceAccess::QUERY_STATUS) {
        Ok(s) => s,
        Err(_) => return false,
    };
    match service.query_status() {
        Ok(status) => {
            matches!(
                status.current_state,
                windows_service::service::ServiceState::Running
            )
        }
        Err(_) => false,
    }
}

/// Named pipe the TalkUCLI service listens on (see talku-cli/src/main.rs).
const PIPE_NAME: &str = r"\\.\pipe\TalkUCLI";
/// Windows service name for the elevated daemon (see talku-cli/src/main.rs and
/// src-tauri/windows/hooks.nsh).
const SERVICE_NAME: &str = "TalkUCLI";

/// Open a connection to the TalkUCLI named pipe for reading and writing. The
/// pipe is created by the SYSTEM service with a DACL that allows the
/// interactive user to connect, so this works from the normal (non-elevated)
/// app.
async fn open_pipe() -> Result<tokio::net::windows::named_pipe::NamedPipeClient, String> {
    use tokio::net::windows::named_pipe::ClientOptions;
    match ClientOptions::new().open(PIPE_NAME) {
        Ok(client) => Ok(client),
        Err(e) => {
            println!("Failed to open named pipe {PIPE_NAME}: {e}");
            Err(format!("Failed to open named pipe {PIPE_NAME}: {e}"))
        }
    }
}

/// Persistent named-pipe connection, opened lazily on first use and reused for
/// every real command (websocket-style). Guarded by a tokio mutex so only one
/// request is in flight at a time; the guard is Send-safe across awaits.
static PIPE_CONN: std::sync::OnceLock<
    tokio::sync::Mutex<Option<tokio::net::windows::named_pipe::NamedPipeClient>>,
> = std::sync::OnceLock::new();

/// Read a single newline-terminated line directly from a named-pipe client,
/// chunk-by-chunk, stopping at the first `\n`. Mirrors `read_pipe_command` on
/// the daemon side and can be interleaved with writes on the same persistent
/// connection.
async fn read_pipe_line(
    stream: &mut tokio::net::windows::named_pipe::NamedPipeClient,
    timeout_ms: u64,
) -> Result<String, String> {
    use tokio::io::AsyncReadExt;
    use tokio::time::{timeout, Duration};

    let mut line = Vec::with_capacity(64);
    let mut chunk = [0u8; 256];
    loop {
        let n = timeout(Duration::from_millis(timeout_ms), stream.read(&mut chunk))
            .await
            .map_err(|_| "Timed out waiting for response".to_string())?
            .map_err(|e| format!("Failed to read response: {e}"))?;
        if n == 0 {
            if line.is_empty() {
                return Err("Connection closed by daemon".to_string());
            }
            break;
        }
        if let Some(pos) = chunk[..n].iter().position(|&b| b == b'\n') {
            line.extend_from_slice(&chunk[..pos]);
            break;
        }
        line.extend_from_slice(&chunk[..n]);
        if line.len() > 4096 {
            break;
        }
    }
    String::from_utf8(line)
        .map(|s| s.trim().to_string())
        .map_err(|e| format!("Invalid response encoding: {e}"))
}

/// Send one request over the named pipe and return the single-line reply. Each
/// call opens a fresh connection, sends `cmd\n`, and reads one line back, then
/// closes. Used only for the lightweight liveness `ping` probe.
async fn pipe_transact(cmd: &str, timeout_ms: u64) -> Result<String, String> {
    println!("  pipe_transact: {cmd}");
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::time::{timeout, Duration};

    let stream = open_pipe().await?;
    println!("pipe opened");

    let mut req = cmd.to_string();
    req.push('\n');

    let mut reader = BufReader::new(stream);
    reader
        .get_mut()
        .write_all(req.as_bytes())
        .await
        .map_err(|e| format!("Failed to send command: {e}"))?;

    let mut line = String::new();
    timeout(
        Duration::from_millis(timeout_ms),
        reader.read_line(&mut line),
    )
    .await
    .map_err(|_| format!("Timed out waiting for response to {cmd}"))?
    .map_err(|e| format!("Failed to read response: {e}"))?;

    println!("  pipe_transact: {cmd} -> {}", line.trim().to_string());
    Ok(line.trim().to_string())
}

/// Whether the TalkUCLI daemon is reachable. Opens the named pipe and expects a
/// `pong` reply to a `ping` probe. Never starts the service or elevates.
async fn is_daemon_alive() -> bool {
    matches!(pipe_transact("ping", 2000).await.as_deref(), Ok("pong"))
}

/// Make sure the elevated daemon (TalkUCLI service) is running. If the service
/// is registered but not running, start it via an ELEVATED `talku-cli
/// service-start` (starting a service requires admin, so a UAC prompt appears).
/// If no service is registered (dev shell), fall back to a one-time direct
/// elevation. Either way we then poll the named pipe until the daemon answers
/// `pong`.
async fn ensure_daemon() -> Result<(), String> {
    if is_daemon_alive().await {
        return Ok(());
    }

    // Single-flight: concurrent connect/disconnect must not both start the
    // service (or elevate) at once. Only the first caller acts; the rest wait.
    // tokio Mutex so its guard can be held across `.await` (Send-safe).
    static STARTING: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
    let _gate = STARTING.lock().await;

    // Re-check under the gate in case someone else already started it.
    if is_daemon_alive().await {
        return Ok(());
    }

    if is_service_registered() {
        if !is_service_running() {
            // Registered but stopped -> start it. Starting a service needs
            // admin, so this goes through an elevated helper (UAC prompt).
            start_service();
        }
        // If it is already running, the pipe server should come up on its own;
        // either way we fall through to the liveness poll below.
    } else {
        // No preinstalled service (dev shell): fall back to a one-time direct
        // elevation, which shows a UAC prompt. The installer registers the
        // service so that installed builds never reach this path.
        elevate_in_background("daemon");
    }

    for _ in 0..50 {
        if is_daemon_alive().await {
            return Ok(());
        }
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }

    Err("Failed to start the TalkUCLI service".to_string())
}

async fn send_command(cmd: &str) -> Result<String, String> {
    ensure_daemon().await?;
    send_command_impl(cmd).await
}

/// Send a command over the persistent named-pipe connection, waiting up to
/// `timeout_ms` for the reply.
async fn send_command_impl_timeout(cmd: &str, timeout_ms: u64) -> Result<String, String> {
    use tokio::io::AsyncWriteExt;
    use tokio::time::{timeout, Duration};

    let conn = PIPE_CONN.get_or_init(|| tokio::sync::Mutex::new(None));
    let mut guard = conn.lock().await;

    // Open the persistent connection lazily on first use.
    if guard.is_none() {
        *guard = Some(open_pipe().await?);
    }

    let stream = guard.as_mut().expect("connection just ensured");
    let req = format!("{cmd}\n");

    if let Err(e) = timeout(Duration::from_millis(timeout_ms), stream.write_all(req.as_bytes()))
        .await
        .map_err(|_| format!("Timed out sending command {cmd}"))?
    {
        // Connection is bad; drop it so the next call reopens.
        *guard = None;
        return Err(format!("Failed to send command: {e}"));
    }

    match read_pipe_line(stream, timeout_ms).await {
        Ok(line) => Ok(line),
        Err(e) => {
            // A failed read usually means the daemon closed the connection.
            *guard = None;
            Err(format!("{e} (for {cmd})"))
        }
    }
}

/// Send a command over the persistent named-pipe connection with a 10s default
/// reply timeout. Used for best-effort work (like disconnect-on-exit) where
/// starting the service would be undesirable.
async fn send_command_impl(cmd: &str) -> Result<String, String> {
    send_command_impl_timeout(cmd, 10000).await
}

/// Best-effort clean disconnect used right before the app exits. Only sends
/// `down` if a daemon is already alive (no service start), and never blocks the
/// exit on failures — the daemon cleans up stale state on its next `up`.
async fn disconnect_before_exit() -> Result<(), String> {
    if !is_daemon_alive().await {
        return Ok(()); // nothing to tear down
    }
    let resp = send_command_impl("down").await?;
    if resp.starts_with("error") {
        return Err(resp);
    }
    Ok(())
}

async fn read_status_line() -> Result<String, String> {
    // Status reads reuse the daemon's single named pipe (the "status" command
    // is served there), so there is no separate status broadcast port.
    let line = send_command_impl("status").await?;
    println!("{}", line);
    Ok(line)
}

async fn connect_vpn() -> Result<(), String> {
    let response = send_command("up").await?;
    if response.starts_with("error") {
        return Err(response);
    }
    Ok(())
}

#[tauri::command]
async fn get_vpn_status() -> Result<String, String> {
    read_status_line().await
}

#[tauri::command]
async fn disconnect_vpn() -> Result<(), String> {
    let response = send_command("down").await?;
    if response.starts_with("error") {
        return Err(response);
    }
    Ok(())
}

#[tauri::command]
async fn check_config_and_connect() -> Result<(), String> {
    let config_path = helper_path("talkuwg.conf")
        .map_err(|_| "Could not find talkuwg config path".to_string())?;

    // Compare the locally cached config version with the server's and, if the
    // server has a newer config, refetch it before connecting.
    config::ensure_config_up_to_date(&config_path)
        .await
        .map_err(|e| e.to_string())?;

    let _ = connect_vpn().await;

    Ok(())
}

#[derive(serde::Serialize)]
struct ProcessInfo {
    pid: u32,
    name: String,
    count: u32,
    cpu_percent: f32,
    memory_kb: u64,
}

/// Windows system/session-0 processes that must never show up in the Monitor
/// list, even the ones (like `csrss.exe`, `dwm.exe`) that also run in the
/// interactive session. Everything else running in session 0 is filtered out by
/// session id.
const SYSTEM_PROCESS_NAMES: &[&str] = &[
    "system",
    "system idle process",
    "registry",
    "smss.exe",
    "csrss.exe",
    "wininit.exe",
    "winlogon.exe",
    "services.exe",
    "lsass.exe",
    "svchost.exe",
    "lsm.exe",
    "fontdrvhost.exe",
    "dwm.exe",
    "conhost.exe",
    "sihost.exe",
    "dllhost.exe",
    "taskhostw.exe",
    "spoolsv.exe",
    "audiodg.exe",
];

/// List the processes currently visible to the app, sorted by name, for the
/// Monitor menu. Two rules keep the list useful:
///
/// 1. System processes are hidden — anything in session 0, the kernel/system
///    idle pseudo-processes, and known system executables by name.
/// 2. Only the *root* of each process tree is shown: a process with no
///    surviving parent, whose parent is a system process, or whose parent is
///    Explorer (the desktop shell launching user apps) counts as a root.
///    Everything spawned by another user process is hidden.
#[tauri::command]
fn list_processes() -> Result<Vec<ProcessInfo>, String> {
    use std::collections::HashMap;
    use sysinfo::System;

    let system = System::new_all();

    let is_system: HashMap<sysinfo::Pid, bool> = system
        .processes()
        .iter()
        .map(|(pid, p)| {
            let sys = pid.as_u32() <= 4
                || p.session_id().map_or(false, |s| s.as_u32() == 0)
                || SYSTEM_PROCESS_NAMES
                    .iter()
                    .any(|n| p.name().eq_ignore_ascii_case(n));
            (*pid, sys)
        })
        .collect();

    // Collapse multiple instances of the same executable into a single row and
    // keep the lowest PID (plus the instance count).
    let mut by_name: HashMap<String, (u32, u32, f32, u64)> = HashMap::new();
    for (pid, p) in system.processes() {
        if is_system.get(pid).copied().unwrap_or(true) {
            continue;
        }

        let is_root = match p.parent() {
            None => true, // orphaned / reparented to system -> treat as root
            Some(pp) => {
                if pp == *pid {
                    true
                } else if let Some(parent_proc) = system.processes().get(&pp) {
                    // An app launched from Explorer is the root of its own tree.
                    parent_proc.name().eq_ignore_ascii_case("explorer.exe")
                        || is_system.get(&pp).copied().unwrap_or(true)
                } else {
                    true // parent already gone -> root
                }
            }
        };
        if !is_root {
            continue;
        }

        let name = p.name().to_string();
        let entry =
            by_name
                .entry(name)
                .or_insert((pid.as_u32(), 0, p.cpu_usage(), p.memory() / 1024));
        if pid.as_u32() < entry.0 {
            entry.0 = pid.as_u32();
        }
        entry.1 += 1;
    }

    let mut list: Vec<ProcessInfo> = by_name
        .into_iter()
        .map(|(name, (pid, count, cpu_percent, memory_kb))| ProcessInfo {
            pid,
            name,
            count,
            cpu_percent,
            memory_kb,
        })
        .collect();
    list.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(list)
}

/// Tell the daemon to run its `unreachable` scan on a process. The scan runs in
/// the background on the daemon side (it never blocks the pipe for other
/// commands), so this returns right away with `ok`, `busy` (a scan is already
/// running), or an `error`. Results are not returned to the client.
#[tauri::command]
async fn collect_unreachable(process_name: String, seconds: Option<u32>) -> Result<(), String> {
    if process_name.trim().is_empty() {
        return Err("missing process name".to_string());
    }
    let secs = seconds.unwrap_or(30).max(1);
    let cmd = format!("unreachable {} {}", process_name.trim(), secs);
    let response = send_command(&cmd).await?;
    let response = response.trim();
    if response == "ok" {
        Ok(())
    } else if response == "busy" {
        Err("A scan is already running for another process".to_string())
    } else if response.starts_with("error") {
        Err(response.to_string())
    } else {
        Err(format!("Unexpected daemon response: {response}"))
    }
}

const API_URL: &str = "https://talku.ddns.net:8000/";
#[derive(serde::Deserialize)]
struct ConnectedUsersResponse {
    connected_users: i32,
}

#[tauri::command]
async fn get_connected_users_count() -> Result<i32, String> {
    let url = format!("{}connected_users/", API_URL);
    let response = reqwest::get(&url)
        .await
        .map_err(|e| format!("Failed to reach server: {}", e))?;

    let body = response
        .json::<ConnectedUsersResponse>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    Ok(body.connected_users)
}

/// Registry value name under which the app registers itself for autostart.
const AUTOSTART_REG_NAME: &str = "TalkU";

/// Path (`HKCU\Software\Microsoft\Windows\CurrentVersion\Run`) used to launch
/// the app at logon. Only touches the current user's key, so no elevation is
/// needed.
fn autostart_reg_path() -> winreg::RegKey {
    use winreg::enums::HKEY_CURRENT_USER;
    winreg::RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey_with_flags(
            r"Software\Microsoft\Windows\CurrentVersion\Run",
            winreg::enums::KEY_READ | winreg::enums::KEY_WRITE,
        )
        .unwrap_or_else(|_| {
            winreg::RegKey::predef(HKEY_CURRENT_USER)
                .create_subkey(r"Software\Microsoft\Windows\CurrentVersion\Run")
                .unwrap()
                .0
        })
}

/// Is the app currently set to launch on sign-in?
#[tauri::command]
fn get_launch_on_startup() -> Result<bool, String> {
    let reg = autostart_reg_path();
    match reg.get_value::<String, _>(AUTOSTART_REG_NAME) {
        Ok(v) => Ok(!v.is_empty()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(format!("failed to read autostart setting: {e}")),
    }
}

/// Enable or disable launching the app when the user signs in. Enabled writes
/// the full path to this executable (with `--autostart`, so a startup run
/// starts hidden/background); disabled removes the value.
#[tauri::command]
fn set_launch_on_startup(enabled: bool, app: tauri::AppHandle) -> Result<(), String> {
    let reg = autostart_reg_path();
    if enabled {
        let exe = std::env::current_exe()
            .map_err(|e| format!("failed to get exe path: {e}"))?;
        let cmd = format!("\"{}\" --autostart", exe.display());
        reg.set_value(AUTOSTART_REG_NAME, &cmd)
            .map_err(|e| format!("failed to enable autostart: {e}"))?;
    } else {
        match reg.delete_value(AUTOSTART_REG_NAME) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(format!("failed to disable autostart: {e}")),
        }
    }
    let _ = app;
    Ok(())
}

/// Absolute path to the JSON file that persists app settings (the monitored
/// games + the auto-connect flag), inside the OS app-config dir.
fn settings_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("could not resolve config dir: {e}"))?;
    Ok(dir.join("settings.json"))
}

#[tauri::command]
fn get_auto_connect(settings: tauri::State<'_, SharedSettings>) -> Result<bool, String> {
    Ok(settings
        .lock()
        .map(|s| s.auto_connect_on_game)
        .unwrap_or(false))
}

#[tauri::command]
fn set_auto_connect(
    enabled: bool,
    app: tauri::AppHandle,
    settings: tauri::State<'_, SharedSettings>,
) -> Result<(), String> {
    if let Ok(mut s) = settings.lock() {
        s.auto_connect_on_game = enabled;
        let path = settings_path(&app)?;
        s.save(&path);
    }
    Ok(())
}

#[tauri::command]
fn get_monitored_games(settings: tauri::State<'_, SharedSettings>) -> Result<Vec<String>, String> {
    Ok(settings
        .lock()
        .map(|s| s.monitored_games.clone())
        .unwrap_or_default())
}

#[tauri::command]
fn add_monitored_game(
    name: String,
    app: tauri::AppHandle,
    settings: tauri::State<'_, SharedSettings>,
) -> Result<(), String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("game name is required".to_string());
    }
    let normalized = settings::normalize_game(name);
    if let Ok(mut s) = settings.lock() {
        if !s.monitored_games.iter().any(|g| settings::normalize_game(g) == normalized) {
            s.monitored_games.push(name.to_string());
        }
        let path = settings_path(&app)?;
        s.save(&path);
    }
    Ok(())
}

#[tauri::command]
fn remove_monitored_game(
    name: String,
    app: tauri::AppHandle,
    settings: tauri::State<'_, SharedSettings>,
) -> Result<(), String> {
    let normalized = settings::normalize_game(&name);
    if let Ok(mut s) = settings.lock() {
        s.monitored_games
            .retain(|g| settings::normalize_game(g) != normalized);
        let path = settings_path(&app)?;
        s.save(&path);
    }
    Ok(())
}

/// Long-running background task that watches for monitored game processes and
/// notifies the UI when it should auto-connect (or disconnect) the VPN.
///
/// Polls every 2s. On the rising edge of "a monitored game is now running" it
/// emits a `game-connect` event; on the falling edge ("no monitored game running
/// anymore") it emits `game-disconnect`. The UI listens for these events and runs
/// the same connect/disconnect flow as a button click, so the app state, sounds,
/// and visual feedback all update as if the user had clicked.
///
/// Because events are only emitted on these edges:
/// - a manually-disconnected VPN is not force-reconnected while a game keeps
///   running (the user's manual action wins until the game is fully relaunched)
/// - the UI isn't spammed with redundant connect/disconnect every tick.
async fn watch_games(app: tauri::AppHandle, settings: SharedSettings) {
    use std::time::Duration;
    use sysinfo::System;

    let mut system = System::new();
    let mut game_was_running = false;
    let poll_interval = Duration::from_secs(2);

    loop {
        let (enabled, games) = {
            let s = settings.lock().unwrap_or_else(|p| p.into_inner());
            (s.auto_connect_on_game, s.monitored_games.clone())
        };

        let any_running = if enabled && !games.is_empty() {
            system.refresh_processes();
            let game_names: Vec<String> = games
                .iter()
                .map(|g| settings::normalize_game(g))
                .collect();
            system.processes().values().any(|p| {
                let name = settings::normalize_game(&p.name().to_string());
                game_names.contains(&name)
            })
        } else {
            false
        };

        if any_running && !game_was_running {
            // Rising edge: a monitored game just launched. Ask the UI to run its
            // normal connect flow. The window stays hidden in the tray if it was
            // already hidden (e.g. launched with --autostart).
            game_was_running = true;
            let _ = app.emit("game-connect", ());
        } else if !any_running && game_was_running {
            // Falling edge: last monitored game closed -> ask the UI to run its
            // normal disconnect flow.
            game_was_running = false;
            let _ = app.emit("game-disconnect", ());
        } else {
            // No edge; keep tracking current state (e.g. if settings changed).
            game_was_running = any_running;
        }

        tokio::time::sleep(poll_interval).await;
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(tauri_plugin_log::log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .manage(std::sync::Arc::new(std::sync::Mutex::new(
            settings::AppSettings::default(),
        )) as SharedSettings)
        .invoke_handler(tauri::generate_handler![
            get_connected_users_count,
            get_vpn_status,
            check_config_and_connect,
            disconnect_vpn,
            list_processes,
            collect_unreachable,
            get_launch_on_startup,
            set_launch_on_startup,
            get_auto_connect,
            set_auto_connect,
            get_monitored_games,
            add_monitored_game,
            remove_monitored_game
        ])
        .setup(|app| {
            if cfg!(target_os = "linux") {
                let cache_dir = app.path().cache_dir()?;
                let package_info = app.package_info();
                let app_name = package_info.name.as_str();
                let app_cache = cache_dir.join(app_name);
                if app_cache.exists() {
                    let _ = fs::remove_dir_all(&app_cache);
                }
            }

            // Load persisted settings (monitored games + auto-connect flag) into
            // the managed state, then start the background game watcher.
            {
                use tauri::Manager;
                let shared: SharedSettings = app.state::<SharedSettings>().inner().clone();
                if let Ok(path) = settings_path(app.handle()) {
                    let loaded = settings::AppSettings::load(&path);
                    if let Ok(mut s) = shared.lock() {
                        *s = loaded;
                    }
                }
                let watcher_settings = shared.clone();
                let watcher_app = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    watch_games(watcher_app, watcher_settings).await;
                });
            }

            let show_i = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
            let exit_i = MenuItem::with_id(app, "exit", "Exit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &exit_i])?;

            let _tray = TrayIconBuilder::with_id("main")
                .icon(
                    app.default_window_icon()
                        .cloned()
                        .expect("default window icon not found"),
                )
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::DoubleClick {
                        button: tauri::tray::MouseButton::Left,
                        ..
                    } = event
                    {
                        if let Some(window) = tray.app_handle().get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "exit" => {
                        // Disconnect cleanly (if connected) before exiting so the
                        // tunnel/adapter is torn down rather than left running.
                        let _ = tauri::async_runtime::block_on(disconnect_before_exit());
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            // When launched via the registry Run entry (`--autostart`), start
            // hidden in the tray rather than showing the window.
            #[cfg(windows)]
            if std::env::args().any(|a| a == "--autostart") {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
