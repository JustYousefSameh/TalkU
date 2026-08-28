use std::net::{SocketAddr, TcpListener, TcpStream};
use std::process::ExitCode;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use defguard_wireguard_rs::{
    key::Key, net::IpAddrMask, peer::Peer, InterfaceConfiguration, WGApi, WireguardInterfaceApi,
};

const DEFAULT_CONFIG: &str = "talkuwg.conf";

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
                peer = Some(PeerConfig::default());
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

fn start_wstunnel() -> Result<std::process::Child, String> {
    let args = [
        "client",
        "-L",
        "udp://51820:localhost:51820?timeout_sec=0",
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

/// Holds all long-lived state for the daemon so it can bring the tunnel up and
/// down on demand without exiting (which would otherwise close the leaked
/// WireGuardNT adapter handle). Keeping the WGApi alive inside the daemon is
/// exactly what makes the adapter persist while `up` is active, and taking it
/// out + `remove_interface` on `down` frees it cleanly.
struct Daemon {
    wgapi: Option<WGApi<defguard_wireguard_rs::Kernel>>,
    wstunnel: Option<std::process::Child>,
    status_listener: Option<TcpListener>,
}

impl Default for Daemon {
    fn default() -> Self {
        Self {
            wgapi: None,
            wstunnel: None,
            status_listener: None,
        }
    }
}

impl Daemon {
    fn is_up(&self) -> bool {
        self.wgapi.is_some()
    }

    fn up(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.is_up() {
            return Ok(());
        }
        log("up: start");
        let cfg = load_config()?;
        let name = ifname();

        #[cfg(not(target_os = "macos"))]
        let mut wgapi = WGApi::<defguard_wireguard_rs::Kernel>::new(name.clone())?;
        #[cfg(target_os = "macos")]
        let mut wgapi = WGApi::<defguard_wireguard_rs::Userspace>::new(name.clone())?;

        wgapi.create_interface()?;

        let mut peers = Vec::new();
        for p in &cfg.peers {
            let public_key = p.public_key.as_deref().ok_or("Peer missing PublicKey")?;
            let peer_key: Key = Key::from_str(public_key).map_err(|e| format!("Bad PublicKey: {e}"))?;
            let endpoint: SocketAddr = "127.0.0.1:51820".parse().unwrap();

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

        wgapi.configure_interface(&interface_config)?;
        wgapi.configure_peer_routing(&interface_config.peers)?;

        if !cfg.dns.is_empty() {
            let dns_ips = cfg
                .dns
                .iter()
                .map(|d| d.parse())
                .collect::<Result<Vec<_>, _>>()?;
            wgapi.configure_dns(&dns_ips, &[])?;
        }

        // Keep the api handle alive for the lifetime of this daemon process so
        // the WireGuardNT adapter persists while we are "up".
        self.wgapi = Some(wgapi);

        // Start wstunnel. Keep its handle so the tunnel keeps running.
        match start_wstunnel() {
            Ok(child) => {
                self.wstunnel = Some(child);
                log("up: wstunnel started");
            }
            Err(e) => {
                log(&format!("up: wstunnel failed to start: {e}"));
                eprintln!("warning: failed to start wstunnel: {e}");
            }
        }

        // Start the loopback status server if not already running.
        if self.status_listener.is_none() {
            let listener = TcpListener::bind("127.0.0.1:0")?;
            let port = listener.local_addr()?.port();
            let port_file = exe_dir()?.join("talku-cli.port");
            std::fs::write(&port_file, port.to_string())?;

            let running = Arc::new(AtomicBool::new(true));
            let thread_listener = listener.try_clone()?;
            std::thread::spawn(move || broadcast_status(thread_listener, running));
            self.status_listener = Some(listener);
            log("up: status server started");
        }

        log("up: done");
        Ok(())
    }

    fn down(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        log("down: start");

        // Kill wstunnel by its own handle.
        if let Some(mut child) = self.wstunnel.take() {
            let _ = child.kill();
            let _ = child.wait();
            log("down: wstunnel killed");
        }

        // Remove the interface now that we hold its api handle.
        if let Some(mut wgapi) = self.wgapi.take() {
            match wgapi.remove_interface() {
                Ok(()) => log("down: interface removed"),
                Err(e) => {
                    log(&format!("down: remove_interface failed: {e}"));
                    // Dropping handle should still free the adapter.
                }
            }
        }

        log("down: done");
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

fn broadcast_status(listener: TcpListener, running: Arc<AtomicBool>) {
    let mut clients: Vec<TcpStream> = Vec::new();
    loop {
        if !running.load(Ordering::SeqCst) {
            break;
        }
        // Accept new connections on the listener in the same loop we broadcast.
        listener.set_nonblocking(true).ok();
        loop {
            match listener.accept() {
                Ok((stream, _addr)) => {
                    stream.set_nonblocking(true).ok();
                    clients.push(stream);
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(_) => break,
            }
        }

        let line = match status_line() {
            Ok(l) => l,
            Err(e) => {
                let msg = format!("error {e}");
                log(&format!("up: status read failed: {e}"));
                msg
            }
        };
        clients.retain(|mut client| {
            use std::io::Write;
            let mut buf = line.clone();
            buf.push('\n');
            client.write_all(buf.as_bytes()).is_ok()
        });

        std::thread::sleep(std::time::Duration::from_secs(2));
    }
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

    let mut daemon = daemon.lock().unwrap();
    let response = match cmd {
        "up" => match daemon.up() {
            Ok(()) => "ok".to_string(),
            Err(e) => format!("error {e}"),
        },
        "down" => match daemon.down() {
            Ok(()) => "ok".to_string(),
            Err(e) => format!("error {e}"),
        },
        "status" => status_line().unwrap_or_else(|e| format!("error {e}")),
        other => format!("error unknown command: {other}"),
    };

    let mut buf = response;
    buf.push('\n');
    let _ = stream.write_all(buf.as_bytes());
}

fn run_daemon() -> ExitCode {
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
