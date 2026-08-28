mod config;

use std::fs;

use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager,
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

fn talku_cli_dir() -> Result<std::path::PathBuf, String> {
    let helper = helper_path("talku-cli.exe")?;
    helper
        .parent()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| "Helper exe has no parent directory".to_string())
}

fn ctrl_port() -> Result<u16, String> {
    let port_file = talku_cli_dir()?.join("talku-cli.ctrl.port");
    std::fs::read_to_string(&port_file)
        .map_err(|e| format!("Failed to read ctrl port file: {e}"))?
        .trim()
        .parse()
        .map_err(|e| format!("Invalid ctrl port: {e}"))
}

fn is_daemon_alive() -> bool {
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::time::Duration;

    let port = match ctrl_port() {
        Ok(p) => p,
        Err(_) => return false,
    };
    let Ok(mut stream) = TcpStream::connect(("127.0.0.1", port)) else {
        return false;
    };
    stream.set_read_timeout(Some(Duration::from_secs(2))).ok();
    let _ = stream.write_all(b"status\n");
    let mut buf = [0u8; 64];
    stream.read(&mut buf).ok().is_some()
}

/// Make sure the elevated daemon is running. If it is not, elevate it once in
/// the background (this is the only time a UAC prompt appears) and wait until
/// its control socket is ready. Once the daemon stays alive, all later
/// up/down/status commands go over loopback with no further elevation.
fn ensure_daemon() -> Result<(), String> {
    if is_daemon_alive() {
        return Ok(());
    }

    elevate_in_background("daemon");

    for _ in 0..50 {
        if is_daemon_alive() {
            return Ok(());
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }

    Err("Failed to start the elevated helper".to_string())
}

fn send_command(cmd: &str) -> Result<String, String> {
    use std::io::{BufRead, BufReader, Write};
    use std::net::TcpStream;
    use std::time::Duration;

    ensure_daemon()?;
    let port = ctrl_port()?;
    let mut stream = TcpStream::connect(("127.0.0.1", port))
        .map_err(|e| format!("Failed to connect to helper: {e}"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .map_err(|e| format!("Failed to set read timeout: {e}"))?;

    let mut req = cmd.to_string();
    req.push('\n');
    stream
        .write_all(req.as_bytes())
        .map_err(|e| format!("Failed to send command: {e}"))?;

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .map_err(|e| format!("Failed to read response: {e}"))?;

    Ok(line.trim().to_string())
}

fn read_status_line() -> Result<String, String> {
    use std::io::{BufRead, BufReader};
    use std::net::TcpStream;
    use std::time::Duration;

    let port_file = talku_cli_dir()?.join("talku-cli.port");
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

fn connect_vpn() -> Result<(), String> {
    let response = send_command("up")?;
    if response.starts_with("error") {
        return Err(response);
    }
    Ok(())
}

#[tauri::command]
fn get_vpn_status() -> Result<String, String> {
    read_status_line()
}

#[tauri::command]
fn disconnect_vpn() -> Result<(), String> {
    let response = send_command("down")?;
    if response.starts_with("error") {
        return Err(response);
    }
    Ok(())
}

#[tauri::command]
async fn check_config_and_connect() -> Result<(), String> {
    let config_path = helper_path("talkuwg.conf")
        .map_err(|_| "Could not find talkuwg config path".to_string())?;

    let _config = config::load_or_fetch_config(&config_path)
        .await
        .map_err(|e| e.to_string())?;

    let _ = connect_vpn();

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
            check_config_and_connect,
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
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

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
