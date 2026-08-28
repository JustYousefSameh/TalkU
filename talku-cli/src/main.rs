use std::{net::SocketAddr, process::ExitCode, str::FromStr};

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
    // ShellExecuteW (used by elevated-command/runas) always starts the helper with
    // its working directory set to %SystemRoot%\System32, so a relative config path
    // would resolve against System32 and fail. Resolve the default config relative to
    // the exe's own directory instead of the current working directory.
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

fn up() -> Result<(), Box<dyn std::error::Error>> {
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

    // Leak the wgapi so its adapter handle is not closed. A WireGuardNT adapter
    // created by this process is deleted when the last handle is dropped, so we
    // must keep it alive for the lifetime of this process (making this a
    // long-running helper). The OS removes the adapter when this process exits.
    Box::leak(Box::new(wgapi));

    // Start wstunnel. Keep its handle alive so the tunnel keeps running; it is
    // killed by name on down.
    match start_wstunnel() {
        Ok(child) => {
            Box::leak(Box::new(child));
        }
        Err(e) => {
            log(&format!("up: wstunnel failed to start: {e}"));
            eprintln!("warning: failed to start wstunnel: {e}");
        }
    }

    // Start the loopback status server (broadcasts handshake state every 2s).
    if let Err(e) = start_status_server() {
        log(&format!("up: status server failed: {e}"));
        return Err(e);
    }

    // Write our own PID so `down` (a separate elevated process) can terminate
    // this daemon. Killing this process closes the leaked adapter handle, which
    // is what makes Windows actually delete the WireGuardNT adapter.
    if let Err(e) = write_pid() {
        log(&format!("up: failed to write pid file: {e}"));
    }

    log("up: running (daemon started)");
    Ok(())
}

fn write_pid() -> Result<(), String> {
    let pid = std::process::id();
    let pid_file = exe_dir()?.join("talku-cli.pid");
    std::fs::write(&pid_file, pid.to_string())
        .map_err(|e| format!("Failed to write pid file: {e}"))
}

fn down() -> Result<(), Box<dyn std::error::Error>> {
    log("down: start");

    // Kill wstunnel by image name (daemon also leaks its handle).
    #[cfg(windows)]
    {
        std::process::Command::new("taskkill")
            .args(["/IM", "wstunnel.exe", "/F"])
            .status()
            .ok();
    }

    // Kill the running `up` daemon via the PID it wrote. When the daemon
    // process exits, its leaked WireGuardNT adapter handle is closed, which is
    // what makes Windows actually delete the adapter. This is the reliable
    // teardown (a separate `remove_interface` alone cannot remove an adapter
    // whose handle is still held open by the live daemon).
    #[cfg(windows)]
    {
        if let Ok(pid_file) = exe_dir().map(|d| d.join("talku-cli.pid")) {
            if let Ok(pid_text) = std::fs::read_to_string(&pid_file) {
                if let Ok(pid) = pid_text.trim().parse::<i32>() {
                    let status = std::process::Command::new("taskkill")
                        .args(["/PID", &pid.to_string(), "/F"])
                        .status()
                        .ok();
                    log(&format!(
                        "down: taskkill /PID {pid} /F -> {:?}",
                        status.map(|s| s.code())
                    ));
                }
            }
        }
    }

    // Best-effort: try to remove the interface directly too.
    let name = ifname();
    #[cfg(not(target_os = "macos"))]
    let mut wgapi = WGApi::<defguard_wireguard_rs::Kernel>::new(name.clone())?;
    #[cfg(target_os = "macos")]
    let mut wgapi = WGApi::<defguard_wireguard_rs::Userspace>::new(name.clone())?;

    match wgapi.remove_interface() {
        Ok(()) => log("down: interface removed"),
        Err(e) => log(&format!("down: remove_interface failed (daemon exit should free it): {e}")),
    }

    Ok(())
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

fn status() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", status_line()?);
    Ok(())
}

fn broadcast_status(listener: std::net::TcpListener) {
    let mut clients: Vec<std::net::TcpStream> = Vec::new();
    loop {
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

fn start_status_server() -> Result<(), Box<dyn std::error::Error>> {
    let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    let port_file = exe_dir()?.join("talku-cli.port");
    std::fs::write(&port_file, port.to_string())?;

    std::thread::spawn(move || broadcast_status(listener));
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: talku-cli <up|down|status> [config-path]");
        return ExitCode::from(1);
    }

    let result = match args[1].as_str() {
        "up" => up(),
        "down" => down(),
        "status" => status(),
        other => {
            eprintln!("unknown command: {other}");
            return ExitCode::from(1);
        }
    };

    match result {
        Ok(()) => {
            if args[1].as_str() == "up" {
                // Keep the process (and the leaked adapter + wstunnel handles)
                // alive so the tunnel keeps running. The status server thread
                // broadcasts handshake state every 2s. The adapter is removed on
                // process exit.
                loop {
                    std::thread::sleep(std::time::Duration::from_secs(3600));
                }
            }
            println!("ok");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: {e}");
            log(&format!("{} command FAILED: {e}", args[1]));
            ExitCode::from(1)
        }
    }
}
