use std::net::{SocketAddr, TcpListener, TcpStream};
use std::process::ExitCode;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

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
            let peer_key: Key = Key::from_str(public_key).map_err(|e| format!("Bad PublicKey: {e}"))?;
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
        log(&format!("up: interface configured in {:?}", start.elapsed()));
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
            log(&format!("down: wstunnel child killed (elapsed {:?})", start.elapsed()));
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
            log(&format!("down: removing interface (elapsed {:?})", start.elapsed()));
            let lock = self.api_lock.clone();
            self.teardown = Some(std::thread::spawn(move || {
                let _guard = lock.lock();
                let mut wgapi = wgapi;
                let _ = wgapi.remove_interface();
                log("down: teardown thread done");
            }));
            log(&format!("down: teardown dispatched (elapsed {:?})", start.elapsed()));
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

fn ctrl_port_path() -> Result<std::path::PathBuf, String> {
    Ok(exe_dir()?.join("talku-cli.ctrl.port"))
}

fn write_pid() -> Result<(), String> {
    let pid = std::process::id();
    let pid_file = exe_dir()?.join("talku-cli.pid");
    std::fs::write(&pid_file, pid.to_string())
        .map_err(|e| format!("Failed to write pid file: {e}"))
}

fn handle_conn(daemon: Arc<Mutex<Daemon>>, mut stream: TcpStream) {
    use std::io::{BufRead, BufReader, Write};

    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(10)))
        .ok();
    let mut reader = BufReader::new(stream.try_clone().ok().unwrap());
    let mut line = String::new();
    if reader.read_line(&mut line).is_err() {
        return;
    }
    let cmd = line.trim();

    // Liveness probe: answered instantly, before taking the daemon mutex or
    // api_lock, so the UI can detect a running daemon even mid-teardown
    // (when the adapter close holds api_lock for ~4s).
    if cmd == "ping" {
        let _ = stream.write_all(b"pong\n");
        return;
    }

    let mut daemon = match daemon.lock() {
        Ok(g) => g,
        Err(p) => p.into_inner(), // poisoned by an earlier panic; recover and continue
    };
    let response = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = &mut *daemon;
        match cmd {
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
        }
    })) {
        Ok(resp) => resp,
        Err(_) => "error internal".to_string(),
    };

    let mut buf = response;
    buf.push('\n');
    let _ = stream.write_all(buf.as_bytes());
}

fn run_daemon() -> ExitCode {
    // Capture panic messages and stderr into the log file so failures in the
    // daemon (spawned hidden/elevated) are visible instead of silently dropped.
    let log_path = exe_dir()
        .map(|d| d.join("talku-cli.panic.log"))
        .unwrap_or_else(|_| std::path::PathBuf::from("talku-cli.panic.log"));
    let hook_path = log_path.clone();
    std::panic::set_hook(Box::new(move |info| {
        use std::io::Write;
        let msg = format!("PANIC: {info}\n");
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&hook_path)
        {
            let _ = f.write_all(msg.as_bytes());
        }
    }));
    if let Ok(f) = std::fs::OpenOptions::new().create(true).append(true).open(&log_path) {
        let _ = f; // just ensure the file is creatable
    }

    let daemon = Arc::new(Mutex::new(Daemon::default()));

    let listener = match TcpListener::bind("127.0.0.1:0") {
        Ok(l) => l,
        Err(e) => {
            eprintln!("failed to bind ctrl listener: {e}");
            return ExitCode::from(1);
        }
    };
    let port = match listener.local_addr() {
        Ok(a) => a.port(),
        Err(e) => {
            eprintln!("failed to get ctrl port: {e}");
            return ExitCode::from(1);
        }
    };

    if let Ok(path) = ctrl_port_path() {
        let _ = std::fs::write(&path, port.to_string());
    }
    let _ = write_pid();

    log(&format!("daemon: listening on 127.0.0.1:{port}"));
    println!("daemon ready on 127.0.0.1:{port}");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let daemon = daemon.clone();
                std::thread::spawn(move || handle_conn(daemon, stream));
            }
            Err(e) => log(&format!("daemon: accept error: {e}")),
        }
    }

    ExitCode::SUCCESS
}

fn main() -> ExitCode {
    // Resolve all runtime files (WireGuard DLL, config, wstunnel, pid/port
    // files) relative to this exe regardless of how it is launched. This
    // process may be started by elevated-command, a scheduled task, or the
    // installer, all of which set the CWD to something unrelated, so we pin the
    // working directory to the directory containing this executable.
    if let Ok(dir) = exe_dir() {
        let _ = std::env::set_current_dir(&dir);
    }

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: talku-cli <daemon|up|down|status> [config-path]");
        return ExitCode::from(1);
    }

    match args[1].as_str() {
        "daemon" => run_daemon(),
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
