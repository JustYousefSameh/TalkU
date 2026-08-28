use std::fs;

use tauri::Manager;

fn helper_path() -> Result<std::path::PathBuf, String> {
    let current_exe =
        std::env::current_exe().map_err(|e| format!("Failed to get current exe path: {e}"))?;
    let path = current_exe
        .parent()
        .ok_or_else(|| "Current exe has no parent directory".to_string())?
        .join(if cfg!(target_os = "windows") {
            "talku-cli.exe"
        } else {
            "talku-cli"
        });
    println!("helper_path = {:?}", path);
    Ok(path)
}

fn elevate_in_background(command: &str) {
    let helper = match helper_path() {
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

fn helper_dir() -> Result<std::path::PathBuf, String> {
    let helper = helper_path()?;
    helper
        .parent()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| "Helper exe has no parent directory".to_string())
}

fn read_status_line() -> Result<String, String> {
    use std::io::{BufRead, BufReader};
    use std::net::TcpStream;
    use std::time::Duration;

    let port_file = helper_dir()?.join("talku-cli.port");
    let port: u16 = std::fs::read_to_string(&port_file)
        .map_err(|e| format!("Failed to read status port file: {e}"))?
        .trim()
        .parse()
        .map_err(|e| format!("Invalid status port: {e}"))?;

    let stream = TcpStream::connect(("127.0.0.1", port))
        .map_err(|e| format!("Failed to connect to status server: {e}"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .map_err(|e| format!("Failed to set read timeout: {e}"))?;

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .map_err(|e| format!("Failed to read status line: {e}"))?;

    println!("{}", line);

    Ok(line.trim().to_string())
}

#[tauri::command]
fn connect_vpn() -> Result<(), String> {
    elevate_in_background("up");
    Ok(())
}

#[tauri::command]
fn get_vpn_status() -> Result<String, String> {
    read_status_line()
}

#[tauri::command]
fn disconnect_vpn() -> Result<(), String> {
    elevate_in_background("down");
    Ok(())
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

    println!("{}", body.connected_users);

    Ok(body.connected_users)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(tauri_plugin_log::log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_connected_users_count,
            get_vpn_status,
            connect_vpn,
            disconnect_vpn
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
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
