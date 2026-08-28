use base64::Engine as _;
use serde::{Deserialize, Serialize};

use x25519_dalek::{PublicKey, StaticSecret};

const API_URL: &str = "https://talku.ddns.net:8000/";

#[derive(Serialize)]
struct ClientKey<'a> {
    #[serde(rename = "clientPubKey")]
    client_pub_key: &'a str,
    #[serde(rename = "apiKey")]
    api_key: &'a str,
}

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
}

pub struct Keypair {
    pub public_key: String,
    pub private_key: String,
}

impl Keypair {
    pub fn generate() -> Self {
        let secret = StaticSecret::random();
        let public = PublicKey::from(&secret);
        Self {
            public_key: base64::engine::general_purpose::STANDARD.encode(public.to_bytes()),
            private_key: base64::engine::general_purpose::STANDARD.encode(secret.to_bytes()),
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

pub async fn get_config_from_server(api_key: &str) -> Result<(ServerConfig, String), ConfigError> {
    // Generate public and private key
    let keypair = Keypair::generate();
    let public_key = keypair.public_key;
    let private_key = keypair.private_key;

    println!("Public Key: {public_key}");

    let client_key = ClientKey {
        client_pub_key: &public_key,
        api_key: api_key,
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
            kv.insert(format!("{section}.{}", k.trim()).to_lowercase(), v.trim().to_string());
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
    };
    let private_key = need(&kv, "interface.privatekey")?.to_string();

    Ok(Config { server, private_key })
}

/// Load the cached config from `path`. If it does not exist, fetch it from the
/// server and write it to `path`.
pub async fn load_or_fetch_config(
    path: &std::path::Path,
    api_key: &str,
) -> Result<Config, ConfigError> {
    match load_config(path) {
        Ok(config) => {
            println!("Loaded config from {}", path.display());
            Ok(config)
        }
        Err(_) => {
            println!("Config not found, fetching from server");
            let (server, private_key) = get_config_from_server(api_key).await?;
            let config = Config { server, private_key };
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            write_config(path, &config)?;
            Ok(config)
        }
    }
}
