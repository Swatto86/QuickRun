// runner.rs - Windows PATH resolution and process spawning
//
// This module implements Windows-style command resolution:
// 1. Check if input is an explicit path (absolute or relative with path separators)
// 2. If explicit, verify existence and spawn directly
// 3. Otherwise, search the PATH environment variable
// 4. Respect PATHEXT (.EXE, .CMD, .BAT, etc.) for extensionless commands
// 5. Spawn the process detached (no shell wrapper, direct execution)

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Check if the input looks like an explicit file path (contains \ or / or :)
/// Examples: "C:\Windows\notepad.exe", ".\script.bat", "folder\app.exe"
pub fn is_explicit_path(input: &str) -> bool {
    input.contains('\\') || input.contains('/') || input.contains(':')
}

/// Resolve a command name by searching the PATH environment variable.
/// Respects PATHEXT for extensionless commands (e.g., "notepad" → "notepad.exe").
///
/// Algorithm:
/// - Split PATH by ';' to get directory list
/// - If input already has an extension, try exact match in each PATH directory
/// - If no extension, append each PATHEXT extension and test
/// - Return the first existing file
pub fn resolve_on_path(command: &str) -> Option<PathBuf> {
    // Get PATHEXT (default to common Windows extensions if not set)
    let pathext = env::var("PATHEXT")
        .unwrap_or_else(|_| ".COM;.EXE;.BAT;.CMD".to_string());
    
    let extensions: Vec<&str> = pathext.split(';').collect();
    
    // Get PATH directories
    let path_var = env::var("PATH").ok()?;
    let paths = env::split_paths(&path_var);
    
    // Determine if the command already has an extension
    let has_extension = command.contains('.');
    
    for dir in paths {
        if has_extension {
            // Try exact match first
            let candidate = dir.join(command);
            if candidate.is_file() {
                return Some(candidate);
            }
        } else {
            // Try each PATHEXT extension
            for ext in &extensions {
                let candidate = dir.join(format!("{}{}", command, ext));
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
    }
    
    None
}

/// Spawn a process from the given executable path.
/// Uses std::process::Command to spawn without blocking.
/// Does NOT use cmd.exe or shell interpretation (direct execution for security).
///
/// On Windows, this will:
/// - Spawn the process detached (no console window for GUI apps)
/// - Return immediately (non-blocking)
pub fn spawn_process(path: &Path) -> Result<(), String> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        
        // CREATE_NO_WINDOW flag prevents console window for GUI apps
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        
        Command::new(path)
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(|e| format!("Failed to spawn process: {}", e))?;
    }
    
    #[cfg(not(windows))]
    {
        Command::new(path)
            .spawn()
            .map_err(|e| format!("Failed to spawn process: {}", e))?;
    }
    
    Ok(())
}

/// Main entry point: resolve and run a command from user input
///
/// This mimics the Windows Run dialog (Win+R) behavior:
/// - Recognizes explicit paths: "C:\\Windows\\notepad.exe", ".\\script.bat"
/// - Searches PATH for commands: "notepad", "calc", "code"
/// - Handles extensionless commands via PATHEXT: "notepad" → "notepad.exe"
///
/// Flow:
/// 1. Trim whitespace and check for empty input
/// 2. If input contains path separators (\\ / :) → treat as explicit path
///    a. Verify the file exists
///    b. If not found → return error
/// 3. Otherwise → search PATH environment variable
///    a. Try each directory in PATH
///    b. Try each extension in PATHEXT if command has no extension
///    c. Return first match found
/// 4. Spawn the process detached (CREATE_NO_WINDOW on Windows)
/// 5. Return Ok(()) on success, Err(message) on failure
///
/// Examples:
/// - "notepad" → finds "C:\\Windows\\System32\\notepad.exe"
/// - "calc" → finds "C:\\Windows\\System32\\calc.exe"
/// - "code" → finds VS Code if installed in PATH
/// - "C:\\test.exe" → runs C:\\test.exe directly
/// - ".\\script.bat" → runs script.bat in current directory
pub fn run_command(input: &str) -> Result<(), String> {
    let input = input.trim();
    
    if input.is_empty() {
        return Err("Please enter a command".to_string());
    }
    
    let executable_path = if is_explicit_path(input) {
        // Explicit path: verify it exists
        let path = Path::new(input);
        if path.is_file() {
            path.to_path_buf()
        } else {
            return Err(format!("File not found: {}", input));
        }
    } else {
        // Search PATH
        resolve_on_path(input)
            .ok_or_else(|| format!("'{}' is not recognized as a command or program", input))?
    };
    
    // Spawn the process
    spawn_process(&executable_path)?;
    
    Ok(())
}
