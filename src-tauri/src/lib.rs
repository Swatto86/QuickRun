// lib.rs - QuickRun main application setup
//
// This is the heart of the Tauri application. It configures and launches:
// - System tray (icon + menu with Settings and Quit)
// - Global hotkey (Alt+Space) to toggle the launcher window
// - Window management (show/hide, center on active monitor, focus)
// - Command execution (via the runner module)
// - Settings persistence (Windows registry for startup, JSON for theme)
//
// Architecture:
// - Tauri is a framework that combines a Rust backend with a web frontend
// - This file contains the Rust backend logic
// - The frontend is in src/main.ts and src/settings.ts
// - Communication happens via Tauri "commands" (Rust functions callable from JS)

mod runner;

use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager, Runtime, WebviewWindow, WebviewWindowBuilder};
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::TrayIconBuilder;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

// Windows-specific imports for registry access (startup settings)
#[cfg(windows)]
use winreg::enums::*;
#[cfg(windows)]
use winreg::RegKey;

/// Get the path to the settings file
/// 
/// Settings are stored as JSON in the user's config directory:
/// - Windows: C:\Users\<username>\AppData\Roaming\QuickRun\settings.json
/// - Creates the directory if it doesn't exist
/// 
/// This approach is platform-agnostic (uses dirs crate to find the right location)
fn get_settings_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("QuickRun");
    std::fs::create_dir_all(&path).ok();
    path.push("settings.json");
    path
}

/// Load a setting from the settings file
/// 
/// Parameters:
/// - key: The setting name (e.g., "light_mode")
/// 
/// Returns:
/// - true if the setting exists and is true
/// - false if the setting doesn't exist, is false, or file can't be read
/// 
/// This is used to persist user preferences across app restarts
fn load_setting(key: &str) -> bool {
    let path = get_settings_path();
    if let Ok(contents) = std::fs::read_to_string(&path) {
        if let Ok(settings) = serde_json::from_str::<serde_json::Value>(&contents) {
            return settings.get(key).and_then(|v| v.as_bool()).unwrap_or(false);
        }
    }
    false
}

/// Save a setting to the settings file
/// 
/// Parameters:
/// - key: The setting name (e.g., "light_mode")
/// - value: The boolean value to save
/// 
/// How it works:
/// 1. Load existing settings from file (or create empty object)
/// 2. Update the specified key with the new value
/// 3. Write the entire settings object back to file as pretty-printed JSON
/// 
/// This preserves other settings while updating just one
fn save_setting(key: &str, value: bool) -> Result<(), String> {
    let path = get_settings_path();
    
    let mut settings = if let Ok(contents) = std::fs::read_to_string(&path) {
        serde_json::from_str(&contents).unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };
    
    settings[key] = serde_json::json!(value);
    
    std::fs::write(&path, serde_json::to_string_pretty(&settings).unwrap())
        .map_err(|e| format!("Failed to save settings: {}", e))
}

/// Check if startup is enabled in Windows registry
/// 
/// Windows loads applications at startup from:
/// HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run
/// 
/// This function checks if QuickRun has an entry there.
/// The #[tauri::command] attribute makes this callable from JavaScript.
/// The #[cfg(windows)] ensures it only compiles on Windows.
#[tauri::command]
#[cfg(windows)]
fn is_startup_enabled() -> Result<bool, String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run_key = hkcu
        .open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run")
        .map_err(|e| format!("Failed to open registry: {}", e))?;
    
    Ok(run_key.get_value::<String, _>("QuickRun").is_ok())
}

#[tauri::command]
#[cfg(not(windows))]
fn is_startup_enabled() -> Result<bool, String> {
    Ok(false)
}

/// Set startup enabled/disabled in Windows registry
/// 
/// Parameters:
/// - enabled: true to add QuickRun to startup, false to remove it
/// 
/// How it works:
/// - If enabled: Adds registry value "QuickRun" = path to this exe
/// - If disabled: Deletes the "QuickRun" registry value
/// 
/// Windows will automatically launch the exe at login if the value exists
#[tauri::command]
#[cfg(windows)]
fn set_startup_enabled(enabled: bool) -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run_key = hkcu
        .open_subkey_with_flags("Software\\Microsoft\\Windows\\CurrentVersion\\Run", KEY_WRITE)
        .map_err(|e| format!("Failed to open registry: {}", e))?;
    
    if enabled {
        let exe_path = std::env::current_exe()
            .map_err(|e| format!("Failed to get exe path: {}", e))?;
        let exe_str = exe_path.to_string_lossy().to_string();
        
        run_key
            .set_value("QuickRun", &exe_str)
            .map_err(|e| format!("Failed to set registry value: {}", e))?;
    } else {
        run_key
            .delete_value("QuickRun")
            .map_err(|e| format!("Failed to delete registry value: {}", e))?;
    }
    
    Ok(())
}

#[tauri::command]
#[cfg(not(windows))]
fn set_startup_enabled(_enabled: bool) -> Result<(), String> {
    Err("Startup settings are only supported on Windows".to_string())
}

/// Check if light mode is enabled
/// 
/// Returns the saved theme preference from settings.json.
/// Defaults to false (dark mode) if not set.
/// 
/// Called from frontend on app startup to apply the correct theme
#[tauri::command]
fn is_light_mode() -> Result<bool, String> {
    Ok(load_setting("light_mode"))
}

/// Set light mode enabled/disabled
/// 
/// Parameters:
/// - enabled: true for light mode, false for dark mode
/// 
/// Saves the preference to settings.json for persistence across restarts.
/// The frontend applies the theme immediately without requiring a restart.
#[tauri::command]
fn set_light_mode(enabled: bool) -> Result<(), String> {
    save_setting("light_mode", enabled)
}

/// Tauri command: run a command from user input
/// 
/// This is the core function that executes user commands.
/// 
/// Flow:
/// 1. Frontend calls this when user presses Enter
/// 2. Delegates to runner::run_command() for PATH resolution and execution
/// 3. On success: Hides the launcher window immediately
/// 4. On error: Returns error message to display inline in the UI
/// 
/// Why hide on Rust side?
/// - More reliable than frontend async calls
/// - Window hides instantly before the app even starts launching
/// - User sees immediate feedback
#[tauri::command]
fn run_command(app: AppHandle, input: String) -> Result<(), String> {
    // Run the command via the runner module
    runner::run_command(&input)?;
    
    // Success! Hide the main window immediately
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
    
    Ok(())
}

/// Toggle the main launcher window: show+center+focus if hidden, hide if visible
/// 
/// This is the "heartbeat" of QuickRun - called whenever:
/// - User presses Alt+Space (global hotkey)
/// - User clicks the system tray icon
/// 
/// Behavior:
/// - If window is visible: Hide it (dismiss the launcher)
/// - If window is hidden: Show it, center it on current monitor, and focus input
/// 
/// Why center every time?
/// - User might have moved to a different monitor
/// - Ensures launcher always appears where the user is working
fn toggle_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            // Already visible → hide it
            let _ = window.hide();
        } else {
            // Hidden → show, center, and focus
            show_and_center_window(&window);
        }
    }
}

/// Show the window, center it on the active monitor, and focus the input field
/// 
/// Multi-monitor support:
/// 1. Get the monitor the window is currently on
/// 2. Calculate the center position of that monitor
/// 3. Move window to center position
/// 4. Show the window
/// 5. Give it keyboard focus
/// 6. Emit "window-show" event so frontend can clear input and focus it
/// 
/// This ensures the launcher appears on whichever monitor the user is working on
fn show_and_center_window<R: Runtime>(window: &WebviewWindow<R>) {
    // Center the window on the current monitor
    if let Ok(monitor) = window.current_monitor() {
        if let Some(monitor) = monitor {
            let monitor_size = monitor.size();
            let monitor_pos = monitor.position();
            
            // Window size is defined in tauri.conf.json (500x80)
            let window_size = window.outer_size().unwrap_or_default();
            
            // Calculate centered position
            let x = monitor_pos.x + (monitor_size.width as i32 - window_size.width as i32) / 2;
            let y = monitor_pos.y + (monitor_size.height as i32 - window_size.height as i32) / 2;
            
            let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }));
        }
    }
    
    // Show and focus the window
    let _ = window.show();
    let _ = window.set_focus();
    
    // Emit an event to the frontend so it can clear input and focus the textbox
    let _ = window.emit("window-show", ());
}

/// Open the settings window (or show it if already open)
/// 
/// Settings window features:
/// - Separate window from main launcher (cleaner UX)
/// - Loads settings.html with checkboxes for startup and theme
/// - Transparent background (consistent with main window)
/// - Singleton pattern: only one settings window at a time
/// 
/// Called when:
/// - User clicks \"Settings\" in system tray menu
fn open_settings<R: Runtime>(app: &AppHandle<R>) {
    // Check if settings window already exists (singleton pattern)
    // If it does, just show and focus it instead of creating a new one
    if let Some(settings_window) = app.get_webview_window("settings") {
        let _ = settings_window.show();
        let _ = settings_window.set_focus();
        return;
    }
    
    // Create a new settings window
    let _settings_window = WebviewWindowBuilder::new(
        app,
        "settings",
        tauri::WebviewUrl::App("settings.html".into()),
    )
    .title("QuickRun Settings")
    .inner_size(500.0, 320.0)
    .resizable(false)
    .transparent(true)
    .center()
    .build();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Build the system tray menu
            let settings_item = MenuItemBuilder::with_id("settings", "Settings").build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
            
            let menu = MenuBuilder::new(app)
                .item(&settings_item)
                .separator()
                .item(&quit_item)
                .build()?;
            
            // Create the tray icon
            // Load the icon from the generated icon files
            let icon = app.default_window_icon().unwrap().clone();
            
            let _tray = TrayIconBuilder::new()
                .icon(icon)
                .tooltip("QuickRun - Press Alt+Space")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| {
                    match event.id().as_ref() {
                        "settings" => open_settings(app),
                        "quit" => app.exit(0),
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    // Optional: clicking the tray icon toggles the window
                    // Check if it's a left click
                    if let tauri::tray::TrayIconEvent::Click {
                        button: tauri::tray::MouseButton::Left,
                        ..
                    } = event
                    {
                        toggle_window(tray.app_handle());
                    }
                })
                .build(app)?;
            
            // Register the global hotkey: Alt+Space
            // This works even when the app is not focused.
            // Note: If this fails, another app (like PowerToys) might be using Alt+Space.
            let shortcut = "Alt+Space".parse::<Shortcut>().unwrap();
            
            let app_handle = app.handle().clone();
            
            // on_shortcut() automatically registers the hotkey
            // We wrap it in a match to gracefully handle conflicts
            if let Err(e) = app.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    toggle_window(&app_handle);
                }
            }) {
                eprintln!("Warning: Could not register Alt+Space hotkey: {}", e);
                eprintln!("The app will still work via the tray icon (click to toggle).");
            }
            
            // Start with the window hidden (user must press Alt+Space to show it)
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.hide();
            }
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            run_command,
            is_startup_enabled,
            set_startup_enabled,
            is_light_mode,
            set_light_mode
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
