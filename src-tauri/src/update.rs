use std::{io::Write, path::PathBuf, time::Duration};

use tauri::ipc::Channel;
use tauri_plugin_opener::OpenerExt;

/// Where TalkU releases are published. Users download new versions from here.
const RELEASES_URL: &str = "https://github.com/JustYousefSameh/TalkU/releases";

/// Every release ships its NSIS installer under this name (see the v3.0 release).
const INSTALLER_ASSET: &str = "TalkU.exe";

/// Download progress sent to the frontend after every chunk. The frontend
/// works out the percentage and speed from this.
#[derive(serde::Serialize)]
pub struct DownloadProgress {
    received: u64,
    total: u64,
}

/// Tells the frontend whether a newer TalkU is out. Returns its version
/// (e.g. `"3.1"`), or `None` if we're already up to date.
#[tauri::command]
pub async fn check_for_update(app: tauri::AppHandle) -> Result<Option<String>, String> {
    Ok(newer_release_tag(&app)
        .await?
        .map(|tag| tag.trim_start_matches(['v', 'V']).to_string()))
}

/// Downloads the newest installer, starts it and closes TalkU, reporting
/// download progress through `on_progress`.
///
/// The frontend doesn't get to pick what we download: we look up the latest
/// release again here, so this can only ever fetch an official TalkU
/// installer from GitHub.
#[tauri::command]
pub async fn install_update(
    app: tauri::AppHandle,
    on_progress: Channel<DownloadProgress>,
) -> Result<(), String> {
    let tag = newer_release_tag(&app)
        .await?
        .ok_or("TalkU is already up to date")?;
    let installer = download_installer(&tag, &on_progress).await?;

    // Going through the shell (instead of spawning the process directly) lets
    // Windows show the UAC prompt if the installer asks for admin rights. If
    // this fails (e.g. the user says "No" to UAC) we bail out while the VPN is
    // still connected, so a failed update never leaves anyone offline.
    app.opener()
        .open_path(installer.to_string_lossy(), None::<&str>)
        .map_err(|e| format!("Failed to start the installer: {e}"))?;

    // Same clean disconnect as the tray's "Exit". The installer stops the
    // service anyway, but this tears the tunnel down properly first.
    let _ = crate::disconnect_before_exit().await;
    app.exit(0);
    Ok(())
}

/// Asks GitHub for the newest release tag and returns it only if it's newer
/// than the version we're running.
///
/// Visiting `releases/latest` makes GitHub redirect to the newest release's
/// tag page, so we just read that redirect instead of calling the GitHub API.
/// The API only allows 60 requests per hour per IP, and many of our users
/// share one public IP, so it would start failing for them.
async fn newer_release_tag(app: &tauri::AppHandle) -> Result<Option<String>, String> {
    let client = reqwest::Client::builder()
        // We want the redirect itself, not the page it points to.
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;
    let response = client
        .head(format!("{RELEASES_URL}/latest"))
        .send()
        .await
        .map_err(|e| format!("Failed to reach GitHub: {e}"))?;
    let location = response
        .headers()
        .get(reqwest::header::LOCATION)
        .and_then(|v| v.to_str().ok())
        .ok_or("GitHub did not return the latest release location")?;
    let tag = location
        .rsplit_once("/tag/")
        .map(|(_, tag)| tag)
        .ok_or_else(|| format!("Unexpected latest release location: {location}"))?;

    // `is_newer` only passes plain version tags like `v3.1`, so the tag is
    // safe to put in URLs and file names.
    let current = app.package_info().version.to_string();
    Ok(is_newer(tag, &current).then(|| tag.to_string()))
}

/// Downloads the installer of `tag` into the temp folder and returns its path.
///
/// Security comes from where we download: a fixed GitHub URL over HTTPS, with
/// plain HTTP refused even on redirects. That's the same trust as a user
/// downloading the release in their browser. We also make sure every byte
/// arrived, so a dropped connection never leaves a half-written installer.
async fn download_installer(
    tag: &str,
    on_progress: &Channel<DownloadProgress>,
) -> Result<PathBuf, String> {
    let client = reqwest::Client::builder()
        .https_only(true)
        .connect_timeout(Duration::from_secs(10))
        // No total timeout, slow connections can take a while. We only give
        // up if the download stalls.
        .read_timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;
    let mut response = client
        .get(format!("{RELEASES_URL}/download/{tag}/{INSTALLER_ASSET}"))
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|e| format!("Failed to download the update: {e}"))?;
    let total = response
        .content_length()
        .filter(|&len| len > 0)
        .ok_or("GitHub did not say how big the update is")?;

    let path = std::env::temp_dir().join(format!("TalkU-{tag}-setup.exe"));
    let mut file = std::fs::File::create(&path)
        .map_err(|e| format!("Failed to save the update: {e}"))?;
    let mut received: u64 = 0;
    let result = async {
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|e| format!("The download was interrupted: {e}"))?
        {
            file.write_all(&chunk)
                .map_err(|e| format!("Failed to save the update: {e}"))?;
            received += chunk.len() as u64;
            let _ = on_progress.send(DownloadProgress { received, total });
        }
        if received != total {
            return Err("The download was incomplete, please try again".to_string());
        }
        file.flush()
            .map_err(|e| format!("Failed to save the update: {e}"))
    }
    .await;

    // Never leave a broken installer lying around.
    if result.is_err() {
        drop(file);
        let _ = std::fs::remove_file(&path);
    }
    result.map(|()| path)
}

/// Splits a version like `v3.1` or `3.1.0` into its numbers. Anything else
/// (`v3.1-beta`, `Release`, ...) gives `None`, so odd tags never show an
/// update to users.
fn version_parts(version: &str) -> Option<Vec<u64>> {
    version
        .trim_start_matches(['v', 'V'])
        .split('.')
        .map(|part| {
            // Check the digits ourselves: Rust's number parsing would also
            // accept something like `+3`.
            if !part.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            part.parse().ok()
        })
        .collect()
}

/// Our tags look like `v3.0` while the app version looks like `3.0.0`, so
/// missing numbers count as zero (`3.0` == `3.0.0`).
fn is_newer(tag: &str, current: &str) -> bool {
    let (Some(mut latest), Some(mut current)) = (version_parts(tag), version_parts(current)) else {
        return false;
    };
    let len = latest.len().max(current.len());
    latest.resize(len, 0);
    current.resize(len, 0);
    latest > current
}

#[cfg(test)]
mod tests {
    use super::is_newer;

    #[test]
    fn compares_release_tags_with_app_version() {
        assert!(!is_newer("v3.0", "3.0.0"));
        assert!(is_newer("v3.1", "3.0.0"));
        assert!(is_newer("v3.0.1", "3.0.0"));
        assert!(is_newer("v10.0", "9.9.9"));
        assert!(!is_newer("v2.3", "3.0.0"));
        assert!(!is_newer("Release", "3.0.0"));
        assert!(!is_newer("v3.1/../../evil", "3.0.0"));
        assert!(!is_newer("v+3.1", "3.0.0"));
        assert!(!is_newer("v3.1-beta", "3.0.0"));
    }
}
