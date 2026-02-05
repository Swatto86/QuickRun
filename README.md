# QuickRun

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Built with Tauri](https://img.shields.io/badge/Built%20with-Tauri-24C8D8.svg)](https://tauri.app/)

A lightweight Windows Run-style launcher that lives in your system tray and provides instant command execution via a global Alt+Space hotkey.

![QuickRun Screenshot](screenshot.png)

## Features

- **🚀 Global Hotkey**: Toggle the launcher instantly with Alt+Space from anywhere
- **🔍 PATH Resolution**: Automatically resolves commands using Windows PATH and PATHEXT
- **🎨 Themes**: Choose between Light and Dark themes
- **💾 System Integration**: 
  - Lives in system tray
  - Start with Windows option
  - Transparent, borderless window
- **🔄 Auto-Updates**: Automatic update checking via GitHub releases

## Installation

### Download

Download the latest installer from the [Releases](https://github.com/Swatto86/QuickRun/releases) page.

### Build from Source

#### Prerequisites

- [Node.js](https://nodejs.org/) (v18 or later)
- [Rust](https://www.rust-lang.org/) (latest stable)
- [pnpm](https://pnpm.io/) or npm

#### Steps

```bash
# Clone the repository
git clone https://github.com/Swatto86/QuickRun.git
cd QuickRun

# Install dependencies
npm install

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
```

The installer will be created in `src-tauri/target/release/bundle/nsis/`.

## Usage

### Basic Commands

1. Press **Alt+Space** to open the launcher
2. Type your command (e.g., `notepad`, `calc`, `cmd`)
3. Press **Enter** to execute
4. The launcher will automatically hide after execution

### Examples

- `notepad` - Opens Notepad
- `calc` - Opens Calculator  
- `cmd` - Opens Command Prompt
- `chrome` - Opens Google Chrome (if installed)
- `code` - Opens VS Code (if in PATH)

### Settings

Right-click the system tray icon and select **Settings** to access:

- **Start with Windows**: Launch QuickRun automatically on system startup
- **Light Mode**: Toggle between dark and light themes

### About

Right-click the system tray icon and select **About** to:

- View current version
- Check for updates
- Access GitHub repository

## Development

### Project Structure

```
QuickRun/
├── src/                    # Frontend TypeScript/HTML/CSS
│   ├── main.ts            # Launcher window logic
│   ├── settings.ts        # Settings window logic
│   ├── about.ts           # About window logic
│   └── styles.css         # Global styles
├── src-tauri/             # Rust backend
│   ├── src/
│   │   ├── lib.rs         # Main application setup
│   │   ├── runner.rs      # Command execution logic
│   │   └── updater.rs     # Update checking logic
│   ├── icons/             # Application icons
│   └── Cargo.toml         # Rust dependencies
└── update-application.ps1  # Release automation script
```

### Key Technologies

- **Frontend**: Vanilla TypeScript, HTML5, CSS3
- **Backend**: Rust with Tauri v2
- **Platform APIs**: 
  - Windows Registry (startup settings)
  - Global shortcuts (Alt+Space)
  - System tray integration

### Making Changes

The application uses Tauri's hot-reload during development:

```bash
npm run tauri dev
```

Changes to TypeScript/HTML/CSS will reload automatically. Rust changes require recompilation.

## Releasing

Use the included PowerShell script to create a new release:

```powershell
.\update-application.ps1 -Version "1.0.0" -Notes "Release notes here"
```

This will:
1. Update version numbers in all config files
2. Create a Git tag
3. Push to GitHub (triggering CI/CD if configured)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Author

**Swatto**

- GitHub: [@Swatto86](https://github.com/Swatto86)
- Repository: [QuickRun](https://github.com/Swatto86/QuickRun)

## Acknowledgments

- Built with [Tauri](https://tauri.app/)
- Inspired by Windows Run dialog and other launcher applications

---

**Note**: This application is currently Windows-only due to its reliance on Windows-specific APIs (PATH resolution, registry, etc.)
