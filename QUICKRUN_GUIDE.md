# QuickRun - Windows Launcher MVP

A Windows-only "Run" dialog replacement built with Tauri v2 + Vite + TypeScript.

## ✨ Features

- **System tray**: Runs in background with tray icon & menu (Settings, Quit)
- **Global hotkey**: Alt+Space toggles the launcher (works app-wide)
- **Borderless window**: Clean, centered, auto-focused launcher UI
- **Windows PATH resolution**: Properly resolves commands via PATH + PATHEXT
- **Inline errors**: Shows friendly error messages without closing the window
- **Keyboard shortcuts**:
  - Enter: Run command
  - Esc: Hide window

## 📦 What Changed

### Rust Backend (`src-tauri/`)

1. **src/runner.rs** (NEW)
   - `is_explicit_path()`: Detects absolute/relative paths
   - `resolve_on_path()`: Searches PATH with PATHEXT support (.EXE, .CMD, .BAT, etc.)
   - `spawn_process()`: Launches processes without shell wrapper (security)
   - `run_command()`: Main entry point for command execution

2. **src/lib.rs** (MODIFIED)
   - System tray setup with menu (Settings, Quit, separator)
   - Global hotkey registration (Alt+Space)
   - Window toggle/centering logic
   - `run_command` Tauri command exposed to frontend

3. **Cargo.toml** (MODIFIED)
   - Added `tauri` with `tray-icon` feature
   - Added `tauri-plugin-global-shortcut@2`
   - Added `tauri-plugin-shell@2`

4. **tauri.conf.json** (MODIFIED)
   - Window config: 500x80, borderless, non-resizable, skip taskbar, always on top
   - Hidden by default (shown via hotkey)

### Frontend (`src/`)

1. **index.html** (MODIFIED)
   - Simple launcher UI: text input + error message div

2. **main.ts** (MODIFIED)
   - Enter key: calls `run_command` → hides window on success OR shows error
   - Escape key: hides window + clears input
   - Listens for `window-show` event from Rust → focuses input

3. **styles.css** (MODIFIED)
   - Minimal, clean styling for borderless window
   - Inline error message styles (hidden/visible)

## 🚀 How to Run

```bash
# Install dependencies (first time only)
npm install

# Run in development mode
npm run tauri dev
```

## 🧪 Manual Test Cases

After running `npm run tauri dev`:

1. **Hotkey toggle**:
   - Press Alt+Space → window appears centered
   - Press Alt+Space again → window hides

2. **Valid commands**:
   - Type `notepad` → Enter → Notepad opens, window hides
   - Type `cmd` → Enter → Command prompt opens
   - Type `powershell` → Enter → PowerShell opens

3. **PATH with extension**:
   - Type `calc.exe` → Enter → Calculator opens

4. **Explicit path**:
   - Type `C:\Windows\System32\mspaint.exe` → Enter → Paint opens

5. **Explicit path with spaces**:
   - Type `C:\Program Files\WindowsApps\...` (if you have a valid path)

6. **Invalid command**:
   - Type `foobar123` → Enter → Error: "'foobar123' is not recognized as a command or program"
   - Window stays open, input is still focused

7. **Escape key**:
   - Type something → Escape → window hides, input cleared

8. **Tray menu**:
   - Right-click tray icon → Settings → placeholder settings window opens
   - Right-click tray icon → Quit → app exits cleanly

9. **Tray icon click** (optional):
   - Left-click tray icon → toggles window (same as Alt+Space)

## 📁 File Structure

```
QuickRun/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs         (Entry point)
│   │   ├── lib.rs          (App setup: tray, hotkey, window mgmt)
│   │   └── runner.rs       (PATH resolution & spawning)
│   ├── Cargo.toml          (Rust dependencies)
│   └── tauri.conf.json     (Tauri config)
├── src/
│   ├── main.ts             (Frontend logic)
│   └── styles.css          (Minimal styles)
├── index.html              (Launcher UI)
└── package.json            (npm dependencies)
```

## 🔧 Architecture Decisions

### Why a separate settings window?
For MVP simplicity. Reusing the main window would require:
- Conditional rendering (launcher vs. settings mode)
- State management to track mode
- Window size/config changes on mode switch

A separate window is cleaner and easier to extend later.

### Why not use `cmd /c` or PowerShell?
Security and performance. Direct process spawning via `std::process::Command`:
- Avoids shell injection risks
- Faster execution (no shell overhead)
- Matches Windows Run dialog behavior

### Why PATHEXT?
Windows uses PATHEXT to resolve extensionless commands:
- "notepad" → tries "notepad.com", "notepad.exe", "notepad.bat", etc.
- Default PATHEXT: `.COM;.EXE;.BAT;.CMD`
- We respect the user's PATHEXT env var if set

### Why Alt+Space?
Classic Windows hotkey convention (used by launchers like PowerToys Run).

## 📝 Known Limitations (MVP)

1. **No argument support**: Commands must be simple (no args like `notepad file.txt`)
2. **No history**: Previous commands are not saved
3. **No autocomplete**: No suggestion dropdown
4. **Settings window is a placeholder**: Doesn't actually do anything yet
5. **No multi-monitor DPI handling**: Centering may be off on mixed-DPI setups
6. **No tray icon customization**: Uses default Tauri icon

## 🔮 Future Enhancements

- Command history (up/down arrow keys)
- Autocomplete/suggestions from PATH
- Argument parsing (`notepad C:\file.txt`)
- Settings UI (hotkey customization, theme, startup behavior)
- Command aliases/shortcuts
- Recent files/folders
- Web search fallback (if command not found, search with default browser)

## 🐛 Troubleshooting

**"Hotkey doesn't work"**
- Check if another app is using Alt+Space (e.g., PowerToys)
- Try running as administrator (some apps capture global hotkeys)

**"Window doesn't center"**
- Known issue with multi-monitor setups
- Try moving window manually once, then it should remember position

**"Tray icon not visible"**
- Check Windows tray overflow (hidden icons)
- Restart the app

**"Command not found but it works in CMD"**
- Check your PATH environment variable
- Verify PATHEXT includes the file extension
- Try the full path with extension

## 📜 License

This is a tutorial/example project. Use however you like!

---

**QuickRun MVP - Built with Tauri v2, Vite, and TypeScript**
