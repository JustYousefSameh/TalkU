use std::fs;

use tauri::Manager;

fn helper_path() -> Result<std::path::PathBuf, String> {
    let current_exe = std::env::current_exe()
        .map_err(|e| format!("Failed to get current exe path: {e}"))?;
    let path = current_exe
        .parent()
        .ok_or_else(|| "Current exe has no parent directory".to_string())?
        .join(if cfg!(target_os = "windows") {
            "wireguard-cli.exe"
        } else {
            "wireguard-cli"
        });
    Ok(path)
}

fn elevate_helper(command: &str) -> Result<(), String> {
    let helper = helper_path()?;
    let status = runas::Command::new(&helper)
        .arg(command)
        .status()
        .map_err(|e| format!("Failed to elevate wireguard helper: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "wireguard command '{command}' failed with status: {:?}",
            status.code()
        ))
    }
}

#[tauri::command]
async fn connect_vpn() -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(|| {
        let output = elevate_helper("up")?;
        Ok(output)
    })
    .await
    .map_err(|e| format!("Spawning connect task failed: {e}"))?
}

#[tauri::command]
async fn disconnect_vpn() -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(|| {
        let output = elevate_helper("down")?;
        Ok(output)
    })
    .await
    .map_err(|e| format!("Spawning disconnect task failed: {e}"))?
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
