//! Auto-update functionality for QuickRun.
//!
//! Provides commands to check for updates from GitHub releases and initiate
//! the update process.

use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;

/// GitHub repository owner
const GITHUB_OWNER: &str = "Swatto86";
/// GitHub repository name  
const GITHUB_REPO: &str = "QuickRun";

/// Information about an available update.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    /// Whether an update is available
    pub available: bool,
    /// The latest version available (e.g., "1.2.0")
    pub version: String,
    /// Release notes/body from the GitHub release
    pub body: String,
    /// Current application version
    pub current_version: String,
    /// URL to the GitHub release page
    pub release_url: String,
    /// URL to download the installer directly (exe or msi)
    pub installer_url: Option<String>,
}

/// Response from GitHub releases API
#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    body: Option<String>,
    html_url: String,
    assets: Vec<GitHubAsset>,
}

/// Asset attached to a GitHub release
#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

/// Parse a semantic version string into (major, minor, patch) tuple.
fn parse_semver(version: &str) -> Option<(u32, u32, u32)> {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 {
        return None;
    }

    let major = parts[0].parse::<u32>().ok()?;
    let minor = parts[1].parse::<u32>().ok()?;
    let patch = parts[2].parse::<u32>().ok()?;

    Some((major, minor, patch))
}

/// Compare two semantic versions. Returns:
/// - `1` if a > b
/// - `-1` if a < b  
/// - `0` if a == b
fn compare_versions(a: &str, b: &str) -> i32 {
    let Some((a_maj, a_min, a_pat)) = parse_semver(a) else {
        return 0;
    };
    let Some((b_maj, b_min, b_pat)) = parse_semver(b) else {
        return 0;
    };

    if a_maj != b_maj {
        return if a_maj > b_maj { 1 } else { -1 };
    }
    if a_min != b_min {
        return if a_min > b_min { 1 } else { -1 };
    }
    if a_pat != b_pat {
        return if a_pat > b_pat { 1 } else { -1 };
    }
    0
}

/// Find the Windows installer asset from a list of release assets.
/// Prefers NSIS .exe files.
fn find_installer_asset(assets: &[GitHubAsset]) -> Option<String> {
    // Look for NSIS installer (contains "setup" or similar in the name, ends with .exe)
    for asset in assets {
        let name_lower = asset.name.to_lowercase();
        if name_lower.contains("quickrun") && name_lower.ends_with(".exe") && !name_lower.contains("portable") {
            return Some(asset.browser_download_url.clone());
        }
    }

    // Fallback: any .exe that isn't portable
    for asset in assets {
        let name_lower = asset.name.to_lowercase();
        if name_lower.ends_with(".exe") && !name_lower.contains("portable") {
            return Some(asset.browser_download_url.clone());
        }
    }

    None
}

/// Check for updates by querying the GitHub releases API.
///
/// Returns information about whether an update is available and details
/// about the latest release.
pub async fn check_for_update_impl() -> Result<UpdateInfo, String> {
    let current_version = env!("CARGO_PKG_VERSION");
    let api_url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        GITHUB_OWNER, GITHUB_REPO
    );

    eprintln!("[Updater] Checking for updates at: {}", api_url);

    // Create HTTP client with appropriate headers
    let client = reqwest::Client::builder()
        .user_agent(format!("QuickRun/{}", current_version))
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    // Fetch latest release info
    let response = client
        .get(&api_url)
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch release info: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();

        // Handle 404 specifically - usually means no releases exist yet
        if status == reqwest::StatusCode::NOT_FOUND {
            eprintln!(
                "[Updater] No releases found on GitHub - repository may not have any published releases yet"
            );
            return Ok(UpdateInfo {
                available: false,
                version: current_version.to_string(),
                body: String::new(),
                current_version: current_version.to_string(),
                release_url: format!(
                    "https://github.com/{}/{}/releases",
                    GITHUB_OWNER, GITHUB_REPO
                ),
                installer_url: None,
            });
        }

        return Err(format!("GitHub API returned error {}: {}", status, body));
    }

    let release: GitHubRelease = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse release JSON: {}", e))?;

    // Extract version from tag (strip 'v' prefix if present)
    let latest_version = release
        .tag_name
        .strip_prefix('v')
        .unwrap_or(&release.tag_name)
        .to_string();

    // Compare versions
    let is_newer = compare_versions(&latest_version, current_version) > 0;

    eprintln!(
        "[Updater] Current version: {}, Latest version: {}, Update available: {}",
        current_version, latest_version, is_newer
    );

    let installer_url = find_installer_asset(&release.assets);

    Ok(UpdateInfo {
        available: is_newer,
        version: latest_version,
        body: release.body.unwrap_or_default(),
        current_version: current_version.to_string(),
        release_url: release.html_url,
        installer_url,
    })
}

/// Download the installer and launch it, or open the release page as fallback.
///
/// The installer is downloaded to the system temp directory and then launched.
/// After launching, the application should exit to allow the installer to run.
pub async fn download_and_install_impl(update_info: UpdateInfo) -> Result<(), String> {
    // If we have a direct installer URL, try to download and run it
    if let Some(installer_url) = &update_info.installer_url {
        eprintln!("[Updater] Downloading installer from: {}", installer_url);
        match download_and_launch_installer(installer_url).await {
            Ok(_) => {
                eprintln!("[Updater] Installer launched successfully");
                return Ok(());
            }
            Err(e) => {
                eprintln!(
                    "[Updater] Failed to download/launch installer: {}. Falling back to browser.",
                    e
                );
            }
        }
    }

    // Fallback: open the release page in the default browser
    eprintln!(
        "[Updater] Opening release page in browser: {}",
        update_info.release_url
    );
    open_url_in_browser(&update_info.release_url)?;

    Ok(())
}

/// Download an installer from URL and launch it.
async fn download_and_launch_installer(url: &str) -> Result<(), String> {
    let current_version = env!("CARGO_PKG_VERSION");

    // Create HTTP client
    let client = reqwest::Client::builder()
        .user_agent(format!("QuickRun/{}", current_version))
        .timeout(std::time::Duration::from_secs(300)) // 5 minute timeout for download
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    // Start download
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Failed to start download: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Download failed with status: {}",
            response.status()
        ));
    }

    // Determine filename from URL
    let filename = url
        .split('/')
        .next_back()
        .unwrap_or("quickrun-setup.exe")
        .to_string();

    // Get temp directory
    let temp_dir = env::temp_dir();
    let installer_path: PathBuf = temp_dir.join(&filename);

    eprintln!(
        "[Updater] Downloading to: {}",
        installer_path.display()
    );

    // Download the file
    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to download file: {}", e))?;

    // Write to temp file
    std::fs::write(&installer_path, &bytes)
        .map_err(|e| format!("Failed to write installer: {}", e))?;

    eprintln!(
        "[Updater] Download complete ({} bytes). Launching installer...",
        bytes.len()
    );

    // Launch the installer using cmd /C start
    // This detaches the process so it continues after we exit
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        const DETACHED_PROCESS: u32 = 0x00000008;

        std::process::Command::new("cmd")
            .args(["/C", "start", "", installer_path.to_str().unwrap_or("")])
            .creation_flags(CREATE_NO_WINDOW | DETACHED_PROCESS)
            .spawn()
            .map_err(|e| format!("Failed to launch installer: {}", e))?;

        eprintln!("[Updater] Installer launched successfully");
    }

    #[cfg(not(windows))]
    {
        return Err("Update installation is only supported on Windows".to_string());
    }

    Ok(())
}

/// Open a URL in the system's default browser.
fn open_url_in_browser(url: &str) -> Result<(), String> {
    #[cfg(windows)]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn()
            .map_err(|e| format!("Failed to open browser: {}", e))?;
    }

    #[cfg(not(windows))]
    {
        return Err("Opening browser is only supported on Windows".to_string());
    }

    Ok(())
}
