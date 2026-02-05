// settings.ts - QuickRun settings window logic
//
// This file manages the settings UI that appears when the user
// clicks "Settings" in the system tray menu.
//
// Features:
// - Startup with Windows toggle (modifies Windows registry)
// - Light/Dark theme toggle (saves to JSON, applies immediately)
// - Cross-window communication (theme changes apply to launcher window too)
//
// Architecture:
// - Calls Rust backend via Tauri commands for settings persistence
// - Uses event system to notify main window of theme changes
// - Changes apply immediately without restart

import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { getAllWebviewWindows } from "@tauri-apps/api/webviewWindow";

// Get references to UI elements
const startupCheckbox = document.getElementById("startup-checkbox") as HTMLInputElement;
const lightModeCheckbox = document.getElementById("light-mode-checkbox") as HTMLInputElement;
const closeButton = document.getElementById("close-button") as HTMLButtonElement;
const currentWindow = getCurrentWebviewWindow();

/// Apply theme to all open windows
/// 
/// This function:
/// 1. Updates the theme on this settings window
/// 2. Finds all other windows (specifically the main launcher window)
/// 3. Emits a "theme-changed" event to each window
/// 4. Those windows listen for this event and update their theme
/// 
/// Why emit events instead of directly manipulating windows?
/// - Cleaner separation of concerns
/// - Each window manages its own DOM
/// - Works even if windows are on different monitors
async function applyTheme(isLight: boolean) {
  const theme = isLight ? "light" : "dark";
  document.documentElement.setAttribute("data-theme", theme);
  
  // Also update the main window's theme if it exists
  const windows = await getAllWebviewWindows();
  windows.forEach(window => {
    if (window.label !== "settings") {
      window.emit("theme-changed", { theme });
    }
  });
}

/// Load current settings from backend and update UI
/// 
/// Called when settings window opens.
/// 
/// Flow:
/// 1. Call Rust backend to check if startup is enabled (reads Windows registry)
/// 2. Update startup checkbox to match
/// 3. Call Rust backend to get theme preference (reads JSON file)
/// 4. Update light mode checkbox to match
/// 5. Apply the theme immediately (affects both settings and launcher windows)
async function loadSettings() {
  try {
    const startupEnabled = await invoke<boolean>("is_startup_enabled");
    startupCheckbox.checked = startupEnabled;

    const lightMode = await invoke<boolean>("is_light_mode");
    lightModeCheckbox.checked = lightMode;
    await applyTheme(lightMode);
  } catch (error) {
    console.error("Failed to load settings:", error);
  }
}

/// Handle startup checkbox change
/// 
/// When user toggles "Start with Windows":
/// 1. Call Rust backend to modify Windows registry
/// 2. If success: Checkbox stays in new state
/// 3. If error: Revert checkbox and show error alert
/// 
/// Registry location: HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Run
startupCheckbox.addEventListener("change", async () => {
  try {
    await invoke("set_startup_enabled", { enabled: startupCheckbox.checked });
  } catch (error) {
    console.error("Failed to set startup:", error);
    // Revert checkbox on error - give user feedback that it didn't work
    startupCheckbox.checked = !startupCheckbox.checked;
    alert("Failed to update startup setting: " + error);
  }
});

/// Handle light mode checkbox change
/// 
/// When user toggles theme:
/// 1. Call Rust backend to save preference to JSON file
/// 2. Apply theme immediately to all windows (no restart needed!)
/// 3. If error: Revert checkbox and show error alert
/// 
/// The theme change is instant - user sees it happen in real-time
lightModeCheckbox.addEventListener("change", async () => {
  try {
    await invoke("set_light_mode", { enabled: lightModeCheckbox.checked });
    // Apply theme immediately - this updates both windows instantly
    await applyTheme(lightModeCheckbox.checked);
  } catch (error) {
    console.error("Failed to set light mode:", error);
    // Revert checkbox on error
    lightModeCheckbox.checked = !lightModeCheckbox.checked;
    alert("Failed to update theme setting: " + error);
  }
});

// Close button - simply closes the settings window
closeButton.addEventListener("click", () => {
  currentWindow.close();
});

// Load settings when page loads
window.addEventListener("DOMContentLoaded", loadSettings);
