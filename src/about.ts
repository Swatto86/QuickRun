// about.ts - QuickRun About window logic
//
// This file manages the About dialog UI.
//
// Features:
// - Displays app version dynamically
// - Check for updates functionality
// - Links to GitHub repository
//
// Architecture:
// - Calls Rust backend via Tauri commands for version info and updates
// - Displays update status with visual feedback

import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

const currentWindow = getCurrentWebviewWindow();
const checkUpdatesBtn = document.getElementById("check-updates-btn") as HTMLButtonElement;
const updateStatus = document.getElementById("update-status") as HTMLParagraphElement;
const closeBtn = document.getElementById("close-btn") as HTMLButtonElement;
const versionEl = document.getElementById("about-version") as HTMLParagraphElement;

let updateCheckInProgress = false;

/// Load and display the current app version
///
/// Fetches the version from the Rust backend (from Cargo.toml)
/// and updates the version display in the UI
async function loadVersion() {
  try {
    const version = await invoke<string>("get_app_version");
    if (versionEl) {
      versionEl.textContent = `v${version}`;
    }
  } catch (error) {
    console.error("Failed to get app version:", error);
    if (versionEl) {
      versionEl.textContent = "v?.?.?";
    }
  }
}

/// Show update check status message
///
/// Parameters:
/// - message: Status text to display
/// - isError: Whether this is an error message (red color)
/// - isSuccess: Whether this is a success message (green color)
function showUpdateStatus(message: string, isError = false, isSuccess = false) {
  if (!updateStatus) return;
  
  updateStatus.textContent = message;
  
  if (isError) {
    updateStatus.className = "update-status update-status-error";
  } else if (isSuccess) {
    updateStatus.className = "update-status update-status-success";
  } else {
    updateStatus.className = "update-status";
  }
}

/// Set the check updates button loading state
///
/// Parameters:
/// - loading: true to show spinner, false to show normal text
function setUpdateButtonLoading(loading: boolean) {
  if (!checkUpdatesBtn) return;
  
  const textEl = checkUpdatesBtn.querySelector(".btn-text");
  const spinnerEl = checkUpdatesBtn.querySelector(".loading");
  
  if (loading) {
    checkUpdatesBtn.disabled = true;
    if (textEl) textEl.textContent = "Checking...";
    if (spinnerEl) spinnerEl.classList.remove("hidden");
  } else {
    checkUpdatesBtn.disabled = false;
    if (textEl) textEl.textContent = "Check for Updates";
    if (spinnerEl) spinnerEl.classList.add("hidden");
  }
}

/// Check for available updates
///
/// Flow:
/// 1. Call Rust backend to query GitHub API
/// 2. If update available: Show update info and open download link
/// 3. If no update: Show success message
/// 4. If error: Show error message
async function checkForUpdates() {
  if (updateCheckInProgress) return;
  
  updateCheckInProgress = true;
  setUpdateButtonLoading(true);
  showUpdateStatus("");
  
  try {
    const updateInfo = await invoke<{
      available: boolean;
      version: string;
      body: string;
      current_version: string;
      release_url: string;
      installer_url: string | null;
    }>("check_for_update");
    
    if (updateInfo.available) {
      showUpdateStatus(
        `Update available: v${updateInfo.version}`,
        false,
        false
      );
      
      // Ask user if they want to download
      const download = confirm(
        `A new version (v${updateInfo.version}) is available!\n\n` +
        `Current: v${updateInfo.current_version}\n` +
        `Latest: v${updateInfo.version}\n\n` +
        `Would you like to download it now?`
      );
      
      if (download) {
        showUpdateStatus("Opening download page...", false, false);
        
        // Use invoke to call the Rust opener plugin
        try {
          await invoke("plugin:opener|open", { path: updateInfo.release_url });
          showUpdateStatus("Download page opened in browser", false, true);
        } catch (error) {
          showUpdateStatus(`Failed to open download page: ${error}`, true, false);
        }
      }
    } else {
      showUpdateStatus("You are running the latest version", false, true);
    }
  } catch (error) {
    console.error("Update check failed:", error);
    showUpdateStatus(`Update check failed: ${error}`, true, false);
  } finally {
    updateCheckInProgress = false;
    setUpdateButtonLoading(false);
  }
}

// Wire up event handlers
checkUpdatesBtn?.addEventListener("click", checkForUpdates);

closeBtn?.addEventListener("click", () => {
  currentWindow.close();
});

// Load version on page load
window.addEventListener("DOMContentLoaded", loadVersion);
