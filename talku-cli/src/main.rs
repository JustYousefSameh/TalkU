use std::net::SocketAddr;
use std::process::ExitCode;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use windows_sys::Win32::Foundation::LocalFree;
use windows_sys::Win32::Security::Authorization::ConvertStringSecurityDescriptorToSecurityDescriptorW;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use defguard_wireguard_rs::{
    key::Key, net::IpAddrMask, peer::Peer, InterfaceConfiguration, WGApi, WireguardInterfaceApi,
};

const DEFAULT_CONFIG: &str = "talkuwg.conf";

/// Local UDP port that the wstunnel client tunnel endpoint binds once its
/// WebSocket connection to the remote server is established. The WireGuard
/// peer endpoint points here.
const TUNNEL_PORT: u16 = 51820;

fn ifname() -> String {
    if cfg!(target_os = "linux") || cfg!(target_os = "freebsd") {
        "wg0".into()
    } else {
        "TalkU".into()
    }
}

#[derive(Default)]
struct Config {
    private_key: Option<String>,
    addresses: Vec<String>,
    dns: Vec<String>,
    listen_port: Option<u16>,
    post_up: Option<String>,
    post_down: Option<String>,
    peers: Vec<PeerConfig>,
}

#[derive(Default)]
struct PeerConfig {
    public_key: Option<String>,
    endpoint: Option<String>,
    keepalive: Option<u16>,
    allowed_ips: Vec<String>,
}

fn parse_config(path: &str) -> Result<Config, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read config '{}': {e}", path))?;

    let mut config = Config::default();
    let mut section = String::new();
    let mut peer: Option<PeerConfig> = None;

    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].trim().to_lowercase();
            if section == "peer" {
                if let Some(p) = peer.replace(PeerConfig::default()) {
                    if p.public_key.is_some() {
                        config.peers.push(p);
                    }
                }
            }
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim().to_lowercase();
        let value = value.trim();

        match section.as_str() {
            "interface" => match key.as_str() {
                "privatekey" => config.private_key = Some(value.to_string()),
                "address" => config.addresses.extend(comma_split(value)),
                "dns" => config.dns.extend(comma_split(value)),
                "listenport" => {
                    config.listen_port = value
                        .parse::<u16>()
                        .map_err(|e| format!("Invalid ListenPort '{value}': {e}"))?
                        .into()
                }
                "postup" => config.post_up = Some(value.to_string()),
                "postdown" => config.post_down = Some(value.to_string()),
                _ => {}
            },
            "peer" => {
                let p = peer
                    .as_mut()
                    .ok_or("Peer key found outside [Peer] section")?;
                match key.as_str() {
                    "publickey" => p.public_key = Some(value.to_string()),
                    "endpoint" => p.endpoint = Some(value.to_string()),
                    "persistentkeepalive" => {
                        p.keepalive = value
                            .parse::<u16>()
                            .map_err(|e| format!("Invalid PersistentKeepalive '{value}': {e}"))?
                            .into()
                    }
                    "allowedips" => p.allowed_ips.extend(comma_split(value)),
                    _ => {}
                }
            }
            _ => {}
        }
    }

    if let Some(p) = peer {
        if p.public_key.is_some() {
            config.peers.push(p);
        }
    }

    Ok(config)
}

fn comma_split(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn load_config() -> Result<Config, String> {
    let path = match std::env::args().nth(2) {
        Some(p) => p,
        None => exe_dir()?
            .join(DEFAULT_CONFIG)
            .to_string_lossy()
            .into_owned(),
    };
    parse_config(&path)
}

fn exe_dir() -> Result<std::path::PathBuf, String> {
    let current_exe =
        std::env::current_exe().map_err(|e| format!("Failed to get current exe path: {e}"))?;
    current_exe
        .parent()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| "Current exe has no parent directory".to_string())
}

fn log(msg: &str) {
    use std::io::Write;
    let dir = match exe_dir() {
        Ok(d) => d,
        Err(_) => return,
    };
    let path = dir.join("talku-cli.log");
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let line = format!("[{now}] {msg}\n");
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    {
        let _ = f.write_all(line.as_bytes());
    }
}

/// Kill any orphaned `wstunnel.exe` left behind by a previous daemon that
/// exited without running `down` (otherwise port 51820 stays bound and a fresh
/// tunnel cannot start). Also reaps our own stale child handle if present.
fn kill_stale_wstunnel() {
    #[cfg(target_os = "windows")]
    {
        use std::process::Stdio;
        let mut cmd = std::process::Command::new("taskkill");
        cmd.args(["/F", "/IM", "wstunnel.exe"])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW: don't flash a console
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        if cmd.status().map(|s| s.success()).unwrap_or(false) {
            log("cleanup: killed stale wstunnel process");
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        use std::process::Stdio;
        if std::process::Command::new("pkill")
            .args(["-f", "wstunnel"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            log("cleanup: killed stale wstunnel process");
        }
    }
}

/// Remove any leftover `TalkU` WireGuard adapter from a previous session.
/// Opening the existing adapter and closing the handle (via `remove_interface`)
/// frees it, because WireGuardNT deletes the adapter once the last handle drops.
fn cleanup_stale_interface() {
    let name = ifname();
    let mut wgapi = match WGApi::<defguard_wireguard_rs::Kernel>::new(name.clone()) {
        Ok(a) => a,
        Err(e) => {
            log(&format!("cleanup: cannot open wg api: {e}"));
            return;
        }
    };
    match wgapi.create_interface() {
        Ok(()) => {
            let _ = wgapi.remove_interface();
            log("cleanup: removed stale wireguard adapter");
        }
        Err(e) => {
            log(&format!("cleanup: no stale wireguard adapter: {e}"));
        }
    }
}

fn start_wstunnel() -> Result<std::process::Child, String> {
    let args = [
        "client",
        "-L",
        &format!("udp://{TUNNEL_PORT}:localhost:{TUNNEL_PORT}?timeout_sec=0"),
        "wss://57.131.34.226:443",
    ];

    let exe = exe_dir()?.join(if cfg!(target_os = "windows") {
        "wstunnel.exe"
    } else {
        "wstunnel"
    });
    if !exe.exists() {
        return Err(format!("wstunnel not found at {:?}", exe));
    }

    std::process::Command::new(&exe)
        .args(args)
        .spawn()
        .map_err(|e| format!("Failed to start wstunnel: {e}"))
}

/// Wait until something has bound the local UDP `port`, i.e. wstunnel's tunnel
/// endpoint is listening. A throwaway UDP socket cannot bind a port that is
/// already in use, so `bind` succeeding means the port is still free and a
/// `bind` failure means wstunnel owns it (or the tunnel is being established).
fn wait_for_udp_port(port: u16, timeout: Duration) -> Result<(), String> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if std::net::UdpSocket::bind(("127.0.0.1", port)).is_err() {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    Err(format!(
        "no process bound 127.0.0.1:{port} within {}s",
        timeout.as_secs()
    ))
}

/// Holds all long-lived state for the daemon so it can bring the tunnel up and
/// down on demand without exiting (which would otherwise close the leaked
/// WireGuardNT adapter handle). Keeping the WGApi alive inside the daemon is
/// exactly what makes the adapter persist while `up` is active, and taking it
/// out + `remove_interface` on `down` frees it cleanly.
struct Daemon {
    wgapi: Option<WGApi<defguard_wireguard_rs::Kernel>>,
    wstunnel: Option<std::process::Child>,
    // Runs the slow WireGuardNT adapter teardown off the ctrl-handler thread
    // so `down()` (and hence UI disconnect) returns fast. `up()` joins any
    // in-flight teardown before creating a fresh interface, so the two never
    // race each other.
    teardown: Option<std::thread::JoinHandle<()>>,
    // Serializes all access to the WireGuard adapter. The status command
    // reads the adapter under this lock so it can't race `up()`'s
    // `set_config` and fail with ERROR_BAD_LENGTH.
    api_lock: Arc<Mutex<()>>,
}
impl Default for Daemon {
    fn default() -> Self {
        Self {
            wgapi: None,
            wstunnel: None,
            teardown: None,
            api_lock: Arc::new(Mutex::new(())),
        }
    }
}

impl Daemon {
    fn is_up(&self) -> bool {
        self.wgapi.is_some()
    }

    fn up(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.is_up() {
            log("up: already up");
            // The adapter is up. If the tunnel child died since then, restart it
            // so a retry can actually reconnect instead of "already up" but dead.
            if let Some(child) = self.wstunnel.as_mut() {
                if child.try_wait().ok().flatten().is_some() {
                    log("up: wstunnel died, restarting");
                    let new_child = start_wstunnel().map_err(|e| {
                        let msg = format!("wstunnel restart failed: {e}");
                        log(&msg);
                        msg
                    })?;
                    self.wstunnel = Some(new_child);
                    log("up: wstunnel restarted");
                }
            }
            return Ok(());
        }
        let _api_guard = match self.api_lock.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        let start = Instant::now();
        log("up: start");

        // If a previous `down` is still tearing down the WireGuardNT adapter
        // in the background, wait for it so we don't race the create below.
        // The teardown usually has a head start (disconnect happened earlier),
        // so this rarely blocks for long.
        if let Some(t) = self.teardown.take() {
            let _ = t.join();
            log("up: waited for previous teardown");
        }

        // Reap our own stale child handle if a previous `up` died part-way.
        // Kill but don't block on `wait()` (wstunnel exits slowly); the
        // `kill_stale_wstunnel()` force-kill below handles any remainder.
        if let Some(mut child) = self.wstunnel.take() {
            let _ = child.kill();
            drop(child);
            log("up: cleaned stale wstunnel handle");
        }

        // Make sure no leftover wstunnel or WireGuard adapter is still around
        // from a previous session before bringing the tunnel up fresh.
        kill_stale_wstunnel();
        cleanup_stale_interface();

        let cfg = load_config()?;
        let name = ifname();

        #[cfg(not(target_os = "macos"))]
        let mut wgapi = WGApi::<defguard_wireguard_rs::Kernel>::new(name.clone())?;
        #[cfg(target_os = "macos")]
        let mut wgapi = WGApi::<defguard_wireguard_rs::Userspace>::new(name.clone())?;

        wgapi.create_interface()?;
        log(&format!("up: interface created in {:?}", start.elapsed()));

        // Start wstunnel BEFORE configuring the interface. Its local UDP
        // tunnel endpoint (TUNNEL_PORT) only gets bound once its WebSocket
        // connection to the remote server is up, which is the slow part.
        // Spawning it first lets that happen while we prepare the adapter;
        // then `wait_for_udp_port` makes sure the endpoint is listening before
        // we configure the WireGuard peer. Otherwise the first handshake
        // fires into an unbound port, fails, and WireGuard backs off
        // ~1s, 2s, 4s... which is what made connecting take ~5-6s.
        match start_wstunnel() {
            Ok(child) => {
                self.wstunnel = Some(child);
                log(&format!("up: wstunnel started in {:?}", start.elapsed()));
            }
            Err(e) => {
                log(&format!("up: wstunnel failed to start: {e}"));
                // Roll back so a retry brings everything up from scratch instead
                // of leaving us in an "already up" state that never connects.
                if let Err(re) = wgapi.remove_interface() {
                    log(&format!("up: rollback remove_interface failed: {re}"));
                }
                return Err(e.into());
            }
        }

        let mut peers = Vec::new();
        for p in &cfg.peers {
            let public_key = p.public_key.as_deref().ok_or("Peer missing PublicKey")?;
            let peer_key: Key =
                Key::from_str(public_key).map_err(|e| format!("Bad PublicKey: {e}"))?;
            let endpoint: SocketAddr = format!("127.0.0.1:{TUNNEL_PORT}").parse().unwrap();

            let mut peer = Peer::new(peer_key);
            peer.endpoint = Some(endpoint);
            peer.persistent_keepalive_interval = p.keepalive;
            for ip in &p.allowed_ips {
                peer.allowed_ips.push(IpAddrMask::from_str(ip)?);
            }
            peers.push(peer);
        }

        let prvkey = cfg
            .private_key
            .clone()
            .ok_or("Interface missing PrivateKey")?;
        let addresses = cfg
            .addresses
            .iter()
            .map(|a| a.parse())
            .collect::<Result<Vec<_>, _>>()?;

        let interface_config = InterfaceConfiguration {
            name: name.clone(),
            prvkey,
            addresses,
            port: cfg.listen_port.unwrap_or(54321),
            peers,
            mtu: None,
            fwmark: None,
        };

        // Block until wstunnel's tunnel endpoint is actually listening. If the
        // tunnel never comes up, fail loudly instead of silently never
        // handshaking (and roll everything back).
        wait_for_udp_port(TUNNEL_PORT, Duration::from_secs(20)).map_err(|e| {
            log(&format!(
                "up: tunnel port {TUNNEL_PORT} not ready after {:?}: {e}",
                start.elapsed()
            ));
            e
        })?;
        log(&format!("up: tunnel port ready in {:?}", start.elapsed()));

        wgapi.configure_interface(&interface_config)?;
        log(&format!(
            "up: interface configured in {:?}",
            start.elapsed()
        ));
        wgapi.configure_peer_routing(&interface_config.peers)?;

        if !cfg.dns.is_empty() {
            let dns_ips = cfg
                .dns
                .iter()
                .map(|d| d.parse())
                .collect::<Result<Vec<_>, _>>()?;
            wgapi.configure_dns(&dns_ips, &[])?;
            log(&format!("up: dns configured in {:?}", start.elapsed()));
        }

        // Keep the api handle alive for the lifetime of this daemon process so
        // the WireGuardNT adapter persists while we are "up".
        self.wgapi = Some(wgapi);

        log(&format!("up: done in {:?}", start.elapsed()));
        Ok(())
    }

    fn down(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let _api_guard = match self.api_lock.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        let start = Instant::now();
        log("down: start");

        // Kill wstunnel by its own handle, but don't block on `wait()` —
        // wstunnel can take ~4s to fully exit. The next `up()` force-kills
        // any lingering wstunnel by name, so detaching here keeps `down()`
        // (and thus UI disconnect -> reconnect) fast.
        let wstunnel_killed = if let Some(mut child) = self.wstunnel.take() {
            log(&format!(
                "down: wstunnel child killed (elapsed {:?})",
                start.elapsed()
            ));
            let _ = child.kill();
            drop(child);
            true
        } else {
            log("down: no wstunnel child to kill");
            false
        };

        // Remove the interface now that we hold its api handle. WireGuardNT
        // adapter teardown (closing the adapter) can take ~4s, so run it on a
        // background thread and return to the client immediately — this is what
        // makes disconnect -> reconnect feel instant. The teardown thread
        // re-acquires `api_lock` for its whole duration so the status reader
        // cannot `get_config` on the adapter while it's being closed (which
        // would otherwise race and panic). `up()` joins this thread before
        // recreating the adapter, so create never races it either.
        let interface_removed = if let Some(wgapi) = self.wgapi.take() {
            log(&format!(
                "down: removing interface (elapsed {:?})",
                start.elapsed()
            ));
            let lock = self.api_lock.clone();
            self.teardown = Some(std::thread::spawn(move || {
                let _guard = lock.lock();
                let mut wgapi = wgapi;
                let _ = wgapi.remove_interface();
                log("down: teardown thread done");
            }));
            log(&format!(
                "down: teardown dispatched (elapsed {:?})",
                start.elapsed()
            ));
            true
        } else {
            log("down: no interface to remove");
            false
        };

        // Safety net: clear any orphaned wstunnel/adapter left by a crashed
        // daemon that we never owned a handle to. When we just tore down our
        // own resources cleanly this is redundant, and the create+remove probe
        // in `cleanup_stale_interface` is slow (~1s), which made a quick
        // disconnect -> reconnect feel unresponsive. Only fall through to it
        // when something was already missing.
        if !interface_removed {
            cleanup_stale_interface();
        }
        if !wstunnel_killed {
            kill_stale_wstunnel();
        }

        log(&format!("down: done in {:?}", start.elapsed()));
        Ok(())
    }
}

fn status_line() -> Result<String, Box<dyn std::error::Error>> {
    let name = ifname();

    #[cfg(not(target_os = "macos"))]
    let mut wgapi = WGApi::<defguard_wireguard_rs::Kernel>::new(name.clone())?;
    #[cfg(target_os = "macos")]
    let mut wgapi = WGApi::<defguard_wireguard_rs::Userspace>::new(name.clone())?;

    wgapi.create_interface()?;
    let host = wgapi.read_interface_data()?;

    let now = std::time::SystemTime::now();
    let mut recent_secs: Option<u64> = None;
    for peer in host.peers.values() {
        if let Some(t) = peer.last_handshake {
            if let Ok(d) = now.duration_since(t) {
                recent_secs = Some(d.as_secs());
            }
        }
    }

    let state = match recent_secs {
        Some(secs) if secs < 180 => format!("connected {secs}"),
        _ => "not_connected".to_string(),
    };
    Ok(state)
}

/// Given a target process name (e.g. `GTA5.exe`), listen over the target for
/// `duration_secs` seconds and collect every remote `ip:port` the process is
/// stuck trying to reach — TCP sockets in `SYN_SENT`. Sampling every ~500ms
/// catches endpoints that only stay in SYN_SENT briefly, so endpoints that
/// *become* unreachable at any point inside the window are all reported, not
/// just whatever was stuck at t=0. `duration_secs == 0` means a single instant
/// snapshot.
///
/// Process name → PID mapping is done with `sysinfo` (case-insensitive, accepts
/// the name with or without the `.exe` suffix) and re-resolved every sample, so
/// a process that starts mid-window is still followed. Socket enumeration is
/// done with `netstat2`, filtered by the resolved PIDs. Returns the union
/// (deduplicated) of all `ip:port` targets seen in SYN_SENT during the window.
fn unreachable_ips(process_name: &str, duration_secs: u64) -> Result<Vec<String>, String> {
    use netstat2::*;
    use std::time::{Duration, Instant};
    use sysinfo::System;

    let requested = process_name.trim().to_lowercase();
    let requested_plain = requested.strip_suffix(".exe").unwrap_or(&requested).to_string();

    // 1. Map process name -> PIDs. If nothing matches yet (e.g. the process
    //    hasn't started, or the name is slightly wrong) we DON'T bail: the loop
    //    below keeps re-resolving PIDs for the whole window, so a process that
    //    launches mid-window is still picked up.
    let mut system = System::new();
    system.refresh_processes();
    let mut pids: Vec<u32> = system
        .processes()
        .iter()
        .filter_map(|(pid, process)| {
            let name = process.name().to_lowercase();
            let name_plain = name.strip_suffix(".exe").unwrap_or(&name);
            if name == requested || name_plain == requested_plain {
                Some(pid.as_u32())
            } else {
                None
            }
        })
        .collect();

    // 2. Watch the socket table across the window, accumulating the union of
    //    SYN_SENT targets owned by the process.
    let sample_interval = Duration::from_millis(500);
    let deadline = Instant::now() + Duration::from_secs(duration_secs);
    let mut ips: Vec<String> = Vec::new();

    loop {
        let sockets = get_sockets_info(
            AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6,
            ProtocolFlags::TCP,
        )
        .map_err(|e| format!("netstat failed: {e}"))?;

        for si in &sockets {
            let owned = si.associated_pids.iter().any(|p| pids.contains(p));
            if !owned {
                continue;
            }
            if let ProtocolSocketInfo::Tcp(tcp) = &si.protocol_socket_info {
                if tcp.state == TcpState::SynSent {
                    let remote = format!("{}:{}", tcp.remote_addr, tcp.remote_port);
                    if !ips.contains(&remote) {
                        ips.push(remote);
                    }
                }
            }
        }

        if Instant::now() >= deadline {
            break;
        }
        std::thread::sleep(sample_interval);

        // Refresh the PID map so a process that starts (or restarts) during the
        // window is followed.
        system.refresh_processes();
        pids = system
            .processes()
            .iter()
            .filter_map(|(pid, process)| {
                let name = process.name().to_lowercase();
                let name_plain = name.strip_suffix(".exe").unwrap_or(&name);
                if name == requested || name_plain == requested_plain {
                    Some(pid.as_u32())
                } else {
                    None
                }
            })
            .collect();
    }
    Ok(ips)
}

/// Replace characters that Windows forbids in file names so a process name or
/// `ip:port` pair can safely appear in the report file name.
fn sanitize_file_name(s: &str) -> String {
    s.chars()
        .map(|c| {
            if matches!(
                c,
                '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '\n' | '\r'
            ) {
                '_'
            } else {
                c
            }
        })
        .collect()
}

/// Save a capture to `<process-name>_<ip:port>_<ip:port>...txt` next to the
/// executable (one endpoint per line). Returns the full path written.
fn write_unreachable_report(
    process_name: &str,
    ips: &[String],
) -> Result<std::path::PathBuf, String> {
    let mut base = sanitize_file_name(process_name.trim());
    if !ips.is_empty() {
        base.push('_');
        base.push_str(&sanitize_file_name(&ips.join("_")));
    }
    let path = exe_dir()?.join(format!("{base}.txt"));
    let mut content = String::new();
    for ip in ips {
        content.push_str(ip);
        content.push('\n');
    }
    std::fs::write(&path, content).map_err(|e| format!("failed to write report: {e}"))?;
    Ok(path)
}

/// Run `talku-cli daemon` (the dev/manual fallback when the TalkUCLI service is
/// not registered/enrolled). It serves the exact same named-pipe protocol
/// (`\\.\pipe\TalkUCLI` → `ping`/`up`/`down`/`status`) that the Windows service
/// serves in-process, so the app talks to it identically whether the daemon is
/// running as the service or as a manually-started process.
#[cfg(windows)]
fn run_daemon() -> ExitCode {
    let daemon = Arc::new(Mutex::new(Daemon::default()));
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("failed to build tokio runtime: {e}");
            return ExitCode::from(1);
        }
    };
    log("daemon: starting named-pipe server on \\\\.\\pipe\\TalkUCLI");
    let _ = runtime.block_on(run_pipe_server(daemon));
    ExitCode::SUCCESS
}

#[cfg(not(windows))]
fn run_daemon() -> ExitCode {
    eprintln!("daemon (named pipe) mode is only supported on Windows");
    ExitCode::from(1)
}

const SERVICE_NAME: &str = "TalkUCLI";
const PIPE_NAME: &str = r"\\.\pipe\TalkUCLI";
/// Win32 ERROR_FAILED_SERVICE_CONTROLLER_CONNECT, returned by
/// `service_dispatcher::start` when this process was NOT started by the
/// Service Control Manager (i.e. it was run from a shell). We use that to fall
/// back to the normal CLI argument mode in development / manual debugging.
const ERROR_FAILED_SERVICE_CONTROLLER_CONNECT: i32 = 1063;

// Registers the service entry point that the Service Control Manager calls.
#[cfg(windows)]
windows_service::define_windows_service!(ffi_service_main, service_main);

/// Entry point invoked by the SCM on a dedicated thread. `arguments` is the
/// service-specific argument vector; we spawn a tokio named-pipe server that
/// serves `ping`/`up`/`down`/`status` over `\\.\pipe\TalkUCLI`.
#[cfg(windows)]
fn service_main(_arguments: Vec<std::ffi::OsString>) {
    use windows_service::service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
        ServiceType,
    };
    use windows_service::service_control_handler::{self, ServiceControlHandlerResult};

    let (shutdown_tx, shutdown_rx) = std::sync::mpsc::channel::<()>();

    // The SCM handler runs on a different callback thread; it signals shutdown
    // through the channel and we (the service thread) block on the receiving end.
    let event_handler = move |control_event: ServiceControl| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Stop | ServiceControl::Shutdown => {
                let _ = shutdown_tx.send(());
                ServiceControlHandlerResult::NoError
            }
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    let status_handle = match service_control_handler::register(SERVICE_NAME, event_handler) {
        Ok(h) => h,
        Err(e) => {
            log(&format!("service: register handler failed: {e}"));
            return;
        }
    };

    // Tell the SCM we are running and accept Stop/Shutdown requests.
    if let Err(e) = status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: std::time::Duration::default(),
        process_id: None,
    }) {
        log(&format!("service: set running status failed: {e}"));
        return;
    }

    let daemon = Arc::new(Mutex::new(Daemon::default()));
    let daemon_for_pipe = daemon.clone();

    // Host the tokio named-pipe server on a background thread. We keep the
    // service thread blocked here so the service stays alive until stopped.
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(r) => r,
        Err(e) => {
            log(&format!("service: tokio runtime failed: {e}"));
            let _ = status_handle.set_service_status(service_stopped_status());
            return;
        }
    };

    let _runtime_thread = {
        let runtime = runtime;
        std::thread::spawn(move || {
            let _ = runtime.block_on(run_pipe_server(daemon_for_pipe));
        })
    };

    log("service: running (waiting for pipe clients)");

    // Block until the SCM tells us to stop.
    let _ = shutdown_rx.recv();
    log("service: stop requested");

    if let Err(e) = status_handle.set_service_status(service_stopped_status()) {
        log(&format!("service: set stopped status failed: {e}"));
    }
}

#[cfg(windows)]
fn service_stopped_status() -> windows_service::service::ServiceStatus {
    use windows_service::service::{ServiceExitCode, ServiceState, ServiceStatus, ServiceType};
    ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Stopped,
        controls_accepted: windows_service::service::ServiceControlAccept::STOP,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: std::time::Duration::default(),
        process_id: None,
    }
}

/// Runs the tokio named-pipe server for the life of the service/daemon. Accepts
/// clients one at a time but never stops: after each connection is fully
/// served, a fresh server instance is created so the pipe stays open and future
/// clients keep working. Also logs a heartbeat every 2s proving the daemon is
/// still alive and the pipe is still listening.
#[cfg(windows)]
async fn run_pipe_server(daemon: Arc<Mutex<Daemon>>) {
    use std::ffi::c_void;
    use std::ptr;

    use tokio::net::windows::named_pipe::ServerOptions;
    use tokio::time;

    use windows_sys::Win32::Foundation::LocalFree;
    use windows_sys::Win32::Security::Authorization::{
        ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1,
    };
    use windows_sys::Win32::Security::SECURITY_ATTRIBUTES;

    // Heartbeat
    let mut interval = time::interval(time::Duration::from_secs(2));
    tokio::spawn(async move {
        loop {
            interval.tick().await;
            log("service: alive (pipe listening)");
        }
    });

    let sddl = "D:(A;;GA;;;WD)\0";
    let wide_sddl: Vec<u16> = sddl.encode_utf16().collect();
    let mut sd_ptr = ptr::null_mut();

    let success = unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            wide_sddl.as_ptr(),
            SDDL_REVISION_1,
            &mut sd_ptr,
            ptr::null_mut(),
        )
    };

    if success == 0 {
        log(&format!(
            "service: SDDL conversion failed: {}",
            std::io::Error::last_os_error()
        ));
        return;
    }

    // 2. WRAP the Security Descriptor in a SECURITY_ATTRIBUTES struct
    let mut sa = SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: sd_ptr,
        bInheritHandle: 0,
    };

    // Create the initial pipe instance
    let mut server = match unsafe {
        ServerOptions::new()
            .first_pipe_instance(true)
            .create_with_security_attributes_raw(PIPE_NAME, &mut sa as *mut _ as *mut c_void)
    } {
        Ok(s) => s,
        Err(e) => {
            log(&format!("service: failed to create initial pipe: {e}"));
            unsafe {
                LocalFree(sd_ptr as _);
            }
            return;
        }
    };

    log("service: pipe server listening");

    loop {
        if let Err(e) = server.connect().await {
            log(&format!("service: pipe connect failed: {e}"));
            break;
        }

        let daemon = daemon.clone();
        tokio::spawn(handle_pipe_conn(daemon, server));

        // 3. APPLY the custom security attributes directly inline (No closure needed!)
        match unsafe {
            ServerOptions::new()
                .create_with_security_attributes_raw(PIPE_NAME, &mut sa as *mut _ as *mut c_void)
        } {
            Ok(next) => server = next,
            Err(e) => {
                log(&format!(
                    "service: failed to create next pipe instance: {e}"
                ));
                break;
            }
        }
    }

    // Clean up the security descriptor if the loop ever breaks
    unsafe {
        LocalFree(sd_ptr as _);
    }
}

/// Serves a single persistent named-pipe connection: loops reading
/// newline-terminated commands, dispatching each against the shared `Daemon`,
/// and writing back a newline-terminated reply for each, until the client
/// disconnects. Protocol: `ping` -> `pong`, else `up`/`down`/`status` ->
/// `ok`/`error ...`. This keeps the connection open (websocket-style) so one
/// client can issue many commands without reconnecting.
#[cfg(windows)]
async fn handle_pipe_conn(
    daemon: Arc<Mutex<Daemon>>,
    mut stream: tokio::net::windows::named_pipe::NamedPipeServer,
) {
    use tokio::io::AsyncWriteExt;

    loop {
        let Some(cmd) = read_pipe_command(&mut stream).await else {
            break; // client disconnected or error
        };
        if cmd.is_empty() {
            continue;
        }
        log(&format!("command received: {cmd}"));

        // Liveness probe: answered instantly, before taking the daemon mutex or
        // api_lock, so the app can detect a running daemon even mid-teardown.
        if cmd == "ping" {
            if let Err(e) = stream.write_all(b"pong\n").await {
                log(&format!("service: write failed on ping: {e}"));
                break;
            }
            continue;
        }

        // `unreachable <process-name> [seconds]`: listen over the given process
        // for `seconds` (default 30) and report the remote endpoints it is
        // stuck trying to connect to (SYN_SENT). Doesn't touch the daemon, so
        // handle it outside the daemon lock; `sysinfo`/`netstat2` are
        // blocking, so run them on a blocking thread.
        if let Some(rest) = cmd.strip_prefix("unreachable") {
            let mut parts = rest.split_whitespace();
            let name = parts.next().unwrap_or("").to_string();
            let secs = parts
                .next()
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(30);
            let response = if name.is_empty() {
                "error missing process name".to_string()
            } else {
                match tokio::task::spawn_blocking(move || unreachable_ips(&name, secs)).await {
                    Ok(Ok(ips)) => {
                        if ips.is_empty() {
                            "none".to_string()
                        } else {
                            ips.join(",")
                        }
                    }
                    Ok(Err(e)) => format!("error {e}"),
                    Err(e) => format!("error {e}"),
                }
            };
            let mut buf = response;
            buf.push('\n');
            if let Err(e) = stream.write_all(buf.as_bytes()).await {
                log(&format!("service: unreachable write failed: {e}"));
                break;
            }
            continue;
        }

        let response = {
            let mut daemon = match daemon.lock() {
                Ok(g) => g,
                Err(p) => p.into_inner(),
            };
            let cmd_ref = cmd.as_str();
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| match cmd_ref {
                "up" => match daemon.up() {
                    Ok(()) => "ok".to_string(),
                    Err(e) => format!("error {e}"),
                },
                "down" => match daemon.down() {
                    Ok(()) => "ok".to_string(),
                    Err(e) => format!("error {e}"),
                },
                "status" => {
                    let _guard = match daemon.api_lock.lock() {
                        Ok(g) => g,
                        Err(p) => p.into_inner(),
                    };
                    status_line().unwrap_or_else(|e| format!("error {e}"))
                }
                other => format!("error unknown command: {other}"),
            })) {
                Ok(resp) => resp,
                Err(_) => "error internal".to_string(),
            }
        };

        let mut buf = response;
        buf.push('\n');
        if let Err(e) = stream.write_all(buf.as_bytes()).await {
            log(&format!("service: write failed: {e}"));
            break;
        }
    }
    log("service: client disconnected");
}

/// Read a single newline-terminated command directly from the pipe stream,
/// chunk-by-chunk, stopping at the first `\n`. Returns `None` if the client
/// disconnects or a read error occurs. Reading straight from `stream` (rather
/// than through a BufReader that would borrow it) lets the caller freely
/// interleave reads and writes on the same persistent connection.
#[cfg(windows)]
async fn read_pipe_command(
    stream: &mut tokio::net::windows::named_pipe::NamedPipeServer,
) -> Option<String> {
    use tokio::io::AsyncReadExt;

    let mut line = Vec::with_capacity(64);
    let mut chunk = [0u8; 256];
    loop {
        let n = match stream.read(&mut chunk).await {
            Ok(0) => {
                if line.is_empty() {
                    return None;
                }
                break;
            }
            Ok(n) => n,
            Err(_) => return None,
        };
        if let Some(pos) = chunk[..n].iter().position(|&b| b == b'\n') {
            line.extend_from_slice(&chunk[..pos]);
            break;
        }
        line.extend_from_slice(&chunk[..n]);
        if line.len() > 4096 {
            break;
        }
    }
    String::from_utf8(line).ok().map(|s| s.trim().to_string())
}

fn main() -> ExitCode {
    // Resolve all runtime files (WireGuard DLL, config, wstunnel, pid/port
    // files) relative to this exe regardless of how it is launched. This
    // process may be started by the Service Control Manager, a scheduled task,
    // or the installer, all of which set the CWD to something unrelated, so we
    // pin the working directory to the directory containing this executable.
    if let Ok(dir) = exe_dir() {
        let _ = std::env::set_current_dir(&dir);
    }

    // On Windows, first try to let the Service Control Manager drive this
    // process as the TalkUCLI service. If we were not launched by the SCM,
    // `start` returns ERROR_FAILED_SERVICE_CONTROLLER_CONNECT quickly and we
    // fall through to the normal CLI argument handling (dev/debug mode).
    #[cfg(windows)]
    {
        use windows_service::Error;
        match windows_service::service_dispatcher::start(SERVICE_NAME, ffi_service_main) {
            // Not launched by the SCM (running interactively) -> fall through
            // to the normal CLI argument handling.
            Err(Error::Winapi(e))
                if e.raw_os_error() == Some(ERROR_FAILED_SERVICE_CONTROLLER_CONNECT) => {}
            Ok(()) => return ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("service_dispatcher::start error: {e}");
                return ExitCode::from(1);
            }
        }
    }

    cli_main()
}

#[cfg(not(windows))]
fn main() -> ExitCode {
    if let Ok(dir) = exe_dir() {
        let _ = std::env::set_current_dir(&dir);
    }
    cli_main()
}

/// Start the TalkUCLI service via the SCM using the `windows-service` crate.
/// Starting a service requires admin, so the app drives this through an
/// ELEVATED `talku-cli service-start` invocation (see `start_service` in
/// src-tauri/src/lib.rs) rather than calling it from the non-elevated app.
#[cfg(windows)]
fn cli_service_start() -> ExitCode {
    use windows_service::service::ServiceAccess;
    use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};

    let manager = match ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)
    {
        Ok(m) => m,
        Err(e) => {
            eprintln!("service-start: open SCM failed: {e}");
            return ExitCode::from(1);
        }
    };
    let service = match manager.open_service(SERVICE_NAME, ServiceAccess::START) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("service-start: open service failed: {e}");
            return ExitCode::from(1);
        }
    };
    match service.start(&[] as &[&std::ffi::OsStr]) {
        Ok(()) => {
            println!("service-start: started {SERVICE_NAME}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("service-start: start failed: {e}");
            ExitCode::from(1)
        }
    }
}

#[cfg(not(windows))]
fn cli_service_start() -> ExitCode {
    eprintln!("service-start is only supported on Windows");
    ExitCode::from(1)
}

fn cli_main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: talku-cli <daemon|service-start|up|down|status|unreachable> [config-path|process-name]");
        return ExitCode::from(1);
    }

    match args[1].as_str() {
        "daemon" => run_daemon(),
        "service-start" => cli_service_start(),
        "up" => {
            let mut d = Daemon::default();
            match d.up() {
                Ok(()) => {
                    println!("ok");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("error: {e}");
                    ExitCode::from(1)
                }
            }
        }
        "down" => {
            let mut d = Daemon::default();
            match d.down() {
                Ok(()) => {
                    println!("ok");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("error: {e}");
                    ExitCode::from(1)
                }
            }
        }
        "status" => match status() {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("error: {e}");
                ExitCode::from(1)
            }
        },
        "unreachable" => {
            let name = args.get(2).cloned().unwrap_or_default();
            if name.is_empty() {
                eprintln!("usage: talku-cli unreachable <process-name> [seconds]");
                return ExitCode::from(1);
            }
            let secs = args
                .get(3)
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(30);
            if secs > 0 {
                eprintln!("monitoring '{name}' for {secs}s ...");
            }
            match unreachable_ips(&name, secs).and_then(|ips| {
                let path = write_unreachable_report(&name, &ips)?;
                Ok((ips, path))
            }) {
                Ok((ips, path)) => {
                    eprintln!(
                        "saved {} endpoint(s) to {}",
                        ips.len(),
                        path.display()
                    );
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("error: {e}");
                    ExitCode::from(1)
                }
            }
        }
        other => {
            eprintln!("unknown command: {other}");
            ExitCode::from(1)
        }
    }
}

fn status() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", status_line()?);
    Ok(())
}
