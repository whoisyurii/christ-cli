use serde::Deserialize;
use std::fs;
use std::process::Command;

const REPO: &str = "whoisyurii/christ-cli";

#[derive(Deserialize)]
struct GithubRelease {
    tag_name: String,
}

pub fn current_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

fn target_triple() -> &'static str {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    { "aarch64-apple-darwin" }
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    { "x86_64-apple-darwin" }
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    { "x86_64-unknown-linux-gnu" }
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    { "aarch64-unknown-linux-gnu" }
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    { "x86_64-pc-windows-msvc" }
}

pub async fn check_latest_version() -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent("christ-cli")
        .build()
        .map_err(|e| e.to_string())?;

    let url = format!(
        "https://api.github.com/repos/{}/releases/latest",
        REPO
    );

    let release: GithubRelease = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to check for updates: {}", e))?
        .json()
        .await
        .map_err(|e| format!("Failed to parse release info: {}", e))?;

    // Strip the "v" prefix
    let version = release.tag_name.trim_start_matches('v').to_string();
    Ok(version)
}

pub async fn run_update(check_only: bool) -> Result<(), String> {
    println!("Checking for updates...");

    let current = current_version();
    let latest = check_latest_version().await?;

    if current == latest {
        println!("You're on the latest version (v{}).", current);
        return Ok(());
    }

    println!("Current: v{}  Latest: v{}", current, latest);

    if check_only {
        println!("Run `christ update` to install v{}.", latest);
        return Ok(());
    }

    let target = target_triple();
    let ext = if cfg!(windows) { "zip" } else { "tar.gz" };
    let url = format!(
        "https://github.com/{}/releases/download/v{}/christ-{}.{}",
        REPO, latest, target, ext
    );

    println!("Downloading christ v{} for {}...", latest, target);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .user_agent("christ-cli")
        .build()
        .map_err(|e| e.to_string())?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Download failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Download failed: HTTP {} — release may not exist for {}",
            response.status(),
            target
        ));
    }

    let total_size = response.content_length().unwrap_or(0);
    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Download failed: {}", e))?;

    if total_size > 0 {
        let mb = bytes.len() as f64 / 1024.0 / 1024.0;
        println!("Downloaded {:.1}MB", mb);
    }

    // Write to temp file
    let temp_dir = std::env::temp_dir();
    let archive_path = temp_dir.join(format!("christ-update.{}", ext));
    fs::write(&archive_path, &bytes)
        .map_err(|e| format!("Failed to write temp file: {}", e))?;

    // Extract
    let extract_dir = temp_dir.join("christ-update-extract");
    let _ = fs::remove_dir_all(&extract_dir);
    fs::create_dir_all(&extract_dir)
        .map_err(|e| format!("Failed to create temp dir: {}", e))?;

    if cfg!(windows) {
        Command::new("tar")
            .args(["-xf", &archive_path.to_string_lossy(), "-C", &extract_dir.to_string_lossy()])
            .output()
            .map_err(|e| format!("Extract failed: {}", e))?;
    } else {
        Command::new("tar")
            .args(["xzf", &archive_path.to_string_lossy(), "-C", &extract_dir.to_string_lossy()])
            .output()
            .map_err(|e| format!("Extract failed: {}", e))?;
    }

    let binary_name = if cfg!(windows) { "christ.exe" } else { "christ" };
    let new_binary = extract_dir.join(binary_name);

    if !new_binary.exists() {
        return Err("Extracted binary not found".to_string());
    }

    // Replace the current binary
    let current_exe = std::env::current_exe()
        .map_err(|e| format!("Cannot determine current binary path: {}", e))?;

    // On Unix, we can replace the running binary by renaming
    let backup_path = current_exe.with_extension("old");
    let _ = fs::remove_file(&backup_path); // Clean any previous backup

    fs::rename(&current_exe, &backup_path)
        .map_err(|e| format!("Failed to backup current binary: {}", e))?;

    match fs::copy(&new_binary, &current_exe) {
        Ok(_) => {
            // Set executable permissions on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = fs::set_permissions(&current_exe, fs::Permissions::from_mode(0o755));
            }
            let _ = fs::remove_file(&backup_path);
            let _ = fs::remove_dir_all(&extract_dir);
            let _ = fs::remove_file(&archive_path);
            println!("Updated to v{}! Restart to use the new version.", latest);
            Ok(())
        }
        Err(e) => {
            // Restore backup
            let _ = fs::rename(&backup_path, &current_exe);
            Err(format!("Failed to install new binary: {}", e))
        }
    }
}
