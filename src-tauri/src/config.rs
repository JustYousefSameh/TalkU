use serde::{Deserialize, Serialize};

use defguard_wireguard_rs::key::Key;

const API_URL: &str = "https://talku.ddns.net:8000/";

#[derive(Serialize)]
struct ClientKey<'a> {
    #[serde(rename = "clientPubKey")]
    client_pub_key: &'a str,
    #[serde(rename = "apiKey")]
    api_key: &'a str,
    #[serde(rename = "clientVersion")]
    client_version: f32,
}

/// The client app version reported to the server. Keep this in sync with the
/// app's shipped version (e.g. "2.4" => 2.4); the server rejects clients below
/// `requiredVersion`.
const CURRENT_APP_VERSION: f32 = 2.4;

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerConfig {
    #[serde(rename = "serverKey")]
    pub server_key: String,
    pub address: String,
    #[serde(rename = "allowedIps")]
    pub allowed_ips: Vec<String>,
    #[serde(rename = "remoteIp")]
    pub remote_ip: String,
    pub endpoint: String,
    #[serde(rename = "presKeepAlive")]
    pub pres_keep_alive: u16,
    pub dns: String,
    #[serde(rename = "wstunnelRemotePort")]
    pub wstunnel_remote_port: String,
    #[serde(rename = "configVersion", default)]
    pub config_version: u64,
}

pub struct Keypair {
    pub public_key: String,
    pub private_key: String,
}

impl Keypair {
    pub fn generate() -> Self {
        let private = Key::generate();
        let public = private.public_key();
        Self {
            public_key: public.to_string(),
            private_key: private.to_string(),
        }
    }
}

#[derive(Debug)]
pub struct ConfigError(pub String);

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to get config from server: {}", self.0)
    }
}

impl std::error::Error for ConfigError {}

impl From<ConfigError> for String {
    fn from(e: ConfigError) -> Self {
        e.to_string()
    }
}

pub async fn get_config_from_server() -> Result<(ServerConfig, String), ConfigError> {
    // Generate public and private key
    let keypair = Keypair::generate();
    let public_key = keypair.public_key;
    let private_key = keypair.private_key;

    println!("Public Key: {public_key}");

    let client_key = ClientKey {
        client_pub_key: &public_key,
        api_key: "z~WXkukTav2^dodr5#9",
        client_version: CURRENT_APP_VERSION,
    };

    let client = reqwest::Client::new();
    let response = client
        .post(format!("{API_URL}exchange_keys/"))
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .json(&client_key)
        .send()
        .await
        .map_err(|e| ConfigError(e.to_string()))?;

    let server_config: ServerConfig = response
        .json()
        .await
        .map_err(|e| ConfigError(e.to_string()))?;

    Ok((server_config, private_key))
}

/// Query the server for the current client-config version number. Returns the
/// number, or `None` when the server can't be reached (so a transient network
/// failure never blocks connecting with a cached config).
pub async fn fetch_config_version() -> Result<u64, ConfigError> {
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{API_URL}config_version/"))
        .send()
        .await
        .map_err(|e| ConfigError(e.to_string()))?;

    if !response.status().is_success() {
        return Err(ConfigError(format!("HTTP {}", response.status())));
    }

    #[derive(serde::Deserialize)]
    struct VersionResponse {
        config_version: u64,
    }

    let body: VersionResponse = response
        .json()
        .await
        .map_err(|e| ConfigError(e.to_string()))?;

    Ok(body.config_version)
}

/// A complete client configuration: the server-provided settings plus the
/// generated client private key.
#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub server: ServerConfig,
    pub private_key: String,
}

fn write_config(path: &std::path::Path, config: &Config) -> Result<(), ConfigError> {
    let mut s = String::new();
    s.push_str("[Interface]\n");
    s.push_str(&format!("PrivateKey = {}\n", config.private_key));
    s.push_str(&format!("Address = {}\n", config.server.address));
    s.push_str(&format!("DNS = {}\n", config.server.dns));
    s.push_str("\n[Peer]\n");
    s.push_str(&format!("PublicKey = {}\n", config.server.server_key));
    s.push_str(&format!("Endpoint = {}\n", config.server.endpoint));
    s.push_str(&format!(
        "AllowedIPs = {}\n",
        config.server.allowed_ips.join(",")
    ));
    s.push_str(&format!(
        "PersistentKeepalive = {}\n",
        config.server.pres_keep_alive
    ));
    s.push_str("\n[Extra]\n");
    s.push_str(&format!("RemoteIp = {}\n", config.server.remote_ip));
    s.push_str(&format!(
        "WstunnelRemotePort = {}\n",
        config.server.wstunnel_remote_port
    ));
    s.push_str(&format!("ConfigVersion = {}\n", config.server.config_version));
    std::fs::write(path, s).map_err(|e| ConfigError(e.to_string()))
}

pub fn load_config(path: &std::path::Path) -> Result<Config, ConfigError> {
    let content = std::fs::read_to_string(path).map_err(|e| ConfigError(e.to_string()))?;
    let mut section = String::new();
    let mut kv: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    for raw in content.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].to_string();
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            kv.insert(
                format!("{section}.{}", k.trim()).to_lowercase(),
                v.trim().to_string(),
            );
        }
    }

    fn need<'a>(
        kv: &'a std::collections::HashMap<String, String>,
        key: &str,
    ) -> Result<&'a str, ConfigError> {
        kv.get(key)
            .map(|s| s.as_str())
            .ok_or_else(|| ConfigError(format!("missing {key}")))
    }

    let server = ServerConfig {
        server_key: need(&kv, "peer.publickey")?.to_string(),
        address: need(&kv, "interface.address")?.to_string(),
        allowed_ips: need(&kv, "peer.allowedips")?
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
        remote_ip: need(&kv, "extra.remoteip")?.to_string(),
        endpoint: need(&kv, "peer.endpoint")?.to_string(),
        pres_keep_alive: need(&kv, "peer.persistentkeepalive")?
            .parse()
            .map_err(|e| ConfigError(format!("invalid keepalive: {e}")))?,
        dns: need(&kv, "interface.dns")?.to_string(),
        wstunnel_remote_port: need(&kv, "extra.wstunnelremoteport")?.to_string(),
        config_version: kv
            .get("extra.configversion")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
    };
    let private_key = need(&kv, "interface.privatekey")?.to_string();

    Ok(Config {
        server,
        private_key,
    })
}

/// Load the cached config version from disk (0 when the file is missing or does
/// not carry a version).
fn cached_config_version(path: &std::path::Path) -> u64 {
    match load_config(path) {
        Ok(config) => config.server.config_version,
        Err(_) => 0,
    }
}

/// Check the server's config version against the locally cached one and, if the
/// server reports a newer version (or there is no cached config at all), re-run
/// `exchange_keys/` to fetch a fresh config and overwrite the cached file. This
/// is what makes config changes on the server reach installed clients: bump
/// `config_version` on the backend and the next connect picks the new config up.
pub async fn ensure_config_up_to_date(path: &std::path::Path) -> Result<(), ConfigError> {
    // If there is no usable cached config, just fetch a fresh one (stamping the
    // current server version so we don't re-register on the next connect).
    if load_config(path).is_err() {
        let remote_version = match fetch_config_version().await {
            Ok(v) => v,
            Err(e) => {
                println!("Could not fetch a config version ({e}); fetching config anyway");
                0
            }
        };
        let (server, private_key) = get_config_from_server().await?;
        let mut server = server;
        server.config_version = remote_version;
        let config = Config {
            server,
            private_key,
        };
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        write_config(path, &config)?;
        println!("Fetched missing config from server");
        return Ok(());
    }

    // Compare the cached version with the server's. Only refetch when the server
    // is reachable AND reports a version newer than what we have cached. A
    // transient network failure (fetch_config_version Err) leaves the cache
    // untouched so we can still connect offline.
    let remote_version = match fetch_config_version().await {
        Ok(v) => v,
        Err(e) => {
            println!("Could not check config version (using cached): {e}");
            return Ok(());
        }
    };

    if remote_version > cached_config_version(path) {
        println!(
            "Server config version {remote_version} is newer than cached {}; refetching",
            cached_config_version(path)
        );
        let (server, private_key) = get_config_from_server().await?;
        let mut server = server;
        server.config_version = remote_version;
        let config = Config {
            server,
            private_key,
        };
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        write_config(path, &config)?;
    } else {
        println!(
            "Config is up to date (cached version {})",
            cached_config_version(path)
        );
    }

    Ok(())
}
