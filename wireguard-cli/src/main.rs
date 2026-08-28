use std::{net::SocketAddr, process::ExitCode, str::FromStr};

use defguard_wireguard_rs::{
    key::Key, net::IpAddrMask, peer::Peer, InterfaceConfiguration, WGApi, WireguardInterfaceApi,
};
use x25519_dalek::{EphemeralSecret, PublicKey};

fn ifname() -> String {
    if cfg!(target_os = "linux") || cfg!(target_os = "freebsd") {
        "wg0".into()
    } else {
        "utun3".into()
    }
}

fn up() -> Result<(), Box<dyn std::error::Error>> {
    let name = ifname();

    #[cfg(not(target_os = "macos"))]
    let mut wgapi = WGApi::<defguard_wireguard_rs::Kernel>::new(name.clone())?;
    #[cfg(target_os = "macos")]
    let mut wgapi = WGApi::<defguard_wireguard_rs::Userspace>::new(name.clone())?;

    wgapi.create_interface()?;

    let secret = EphemeralSecret::random();
    let key = PublicKey::from(&secret);
    let peer_key: Key = key.as_ref().try_into().unwrap();
    let mut peer = Peer::new(peer_key.clone());

    log::info!("endpoint");
    let endpoint: SocketAddr = "10.10.10.10:55001".parse().unwrap();
    peer.endpoint = Some(endpoint);
    peer.persistent_keepalive_interval = Some(25);
    peer.allowed_ips.push(IpAddrMask::from_str("10.6.0.0/24")?);
    peer.allowed_ips
        .push(IpAddrMask::from_str("192.168.22.0/24")?);

    let interface_config = InterfaceConfiguration {
        name: name.clone(),
        prvkey: "AAECAwQFBgcICQoLDA0OD/Dh0sO0pZaHeGlaSzwtHg8=".to_string(),
        addresses: vec!["10.6.0.30".parse().unwrap()],
        port: 12345,
        peers: vec![peer],
        mtu: None,
        fwmark: None,
    };

    wgapi.configure_interface(&interface_config)?;
    wgapi.configure_peer_routing(&interface_config.peers)?;

    Ok(())
}

fn down() -> Result<(), Box<dyn std::error::Error>> {
    let name = ifname();

    #[cfg(not(target_os = "macos"))]
    let mut wgapi = WGApi::<defguard_wireguard_rs::Kernel>::new(name.clone())?;
    #[cfg(target_os = "macos")]
    let mut wgapi = WGApi::<defguard_wireguard_rs::Userspace>::new(name.clone())?;

    wgapi.remove_interface()?;

    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: wireguard-cli <up|down>");
        return ExitCode::from(1);
    }

    let result = match args[1].as_str() {
        "up" => up(),
        "down" => down(),
        other => {
            eprintln!("unknown command: {other}");
            return ExitCode::from(1);
        }
    };

    match result {
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
