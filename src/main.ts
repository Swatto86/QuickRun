// main.ts - QuickRun frontend logic
//
// This file handles:
// - Running commands when the user presses Enter
// - Hiding the window when the user presses Escape
// - Focusing and clearing the input when the window is shown
// - Displaying inline error messages

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

const commandInput = document.getElementById("command-input") as HTMLInputElement;
const errorMessage = document.getElementById("error-message") as HTMLDivElement;
const currentWindow = getCurrentWebviewWindow();

/// Load and apply theme on startup
/// 
/// Checks the saved theme preference and applies it immediately.
/// 
/// Flow:
/// 1. Call Rust backend to read theme setting from JSON file
/// 2. Apply the theme by setting data-theme attribute on <html>
/// 3. CSS variables change based on data-theme value
/// 4. If there's an error reading the setting, default to dark theme
/// 
/// This runs once at startup to ensure the launcher uses the user's preferred theme
async function loadTheme() {
  try {
    const isLight = await invoke<boolean>("is_light_mode");
    document.documentElement.setAttribute("data-theme", isLight ? "light" : "dark");
  } catch (err) {
    // Default to dark theme if there's an error
    document.documentElement.setAttribute("data-theme", "dark");
  }
}

loadTheme();

/// Listen for theme changes from settings window
/// 
/// When the user changes theme in settings:
/// 1. Settings window calls set_light_mode() in Rust
/// 2. Settings window emits "theme-changed" event to all windows
/// 3. This listener receives the event and updates the theme
/// 4. Theme changes instantly without restart
/// 
/// This enables live theme switching across all windows
listen<{ theme: string }>("theme-changed", (event) => {
  document.documentElement.setAttribute("data-theme", event.payload.theme);
});

/// Run the command when the user presses Enter
/// 
/// Flow:
/// 1. User types a command and presses Enter
/// 2. Call Rust backend with the command text
/// 3. Rust resolves it via PATH (like Windows Run dialog)
/// 4. Rust spawns the process and hides the window
/// 5. If success: Clear input and error (window already hidden)
/// 6. If error: Show error inline, select text for easy correction
/// 
/// Error handling:
/// - Command not found → "Command not found: xyz"
/// - Empty input → Do nothing (early return)
/// - Permission denied → Show error, keep window open
commandInput.addEventListener("keydown", async (e) => {
  if (e.key === "Enter") {
    e.preventDefault();
    const command = commandInput.value.trim();
    
    if (!command) {
      return;
    }
    
    // Store the command for error recovery (so we can restore it if there's an error)
    const commandToRun = command;
    
    try {
      // Call the Rust command to run the user's input
      // The Rust side will:
      // 1. Resolve the command via PATH (check explicit path first, then search PATH)
      // 2. Spawn the process detached (no blocking)
      // 3. Hide the window immediately on success
      await invoke("run_command", { input: commandToRun });
      
      // Success: Clear the UI (window already hidden by Rust)
      commandInput.value = "";
      hideError();
      
    } catch (error) {
      // Failure: show the error, keep window open and focused
      // User can see what went wrong and try again
      showError(String(error));
      commandInput.value = commandToRun;
      commandInput.focus();
      commandInput.select(); // Select the text so user can easily retype or fix it
    }
  } else if (e.key === "Escape") {
    /// Escape key: hide the window and clear everything
    /// This is the "dismiss" action - user changed their mind
    e.preventDefault();
    commandInput.value = "";
    hideError();
    await currentWindow.hide();
  }
});

/// Listen for the "window-show" event from Rust
/// 
/// This event is emitted by the Rust backend when:
/// - User presses Alt+Space (global hotkey)
/// - User clicks the system tray icon
/// 
/// What we do:
/// - Clear any previous command text
/// - Hide any previous error messages
/// - Focus the input so user can start typing immediately
/// 
/// This ensures a clean slate every time the launcher appears
listen("window-show", () => {
  commandInput.value = "";
  hideError();
  commandInput.focus();
});

/// Helper: show an inline error message
/// 
/// Displays error message in a styled div below the input.
/// Used when command execution fails (not found, permission denied, etc.)
function showError(message: string) {
  errorMessage.textContent = message;
  errorMessage.className = "error-visible";
  errorMessage.style.display = "block";
}

/// Helper: hide the error message
/// 
/// Clears and hides the error div.
/// Called when showing the window, running a successful command, or dismissing via Escape.
function hideError() {
  errorMessage.textContent = "";
  errorMessage.className = "error-hidden";
  errorMessage.style.display = "none";
}

// Focus the input on load (in case the window is already visible at startup)
window.addEventListener("DOMContentLoaded", () => {
  commandInput.focus();
});

