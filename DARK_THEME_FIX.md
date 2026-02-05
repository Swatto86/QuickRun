# Tauri v2 - Dark Theme White Flash Fix

## Problem
When creating Tauri windows with dark themes, there's a brief white flash on window load before the CSS loads and applies the dark background.

## Solution
Enable window transparency and set transparent initial background:

### 1. Window Configuration (tauri.conf.json)
```json
{
  "windows": [
    {
      "transparent": true,
      "backgroundColor": "#1e1e1e00"  // Dark color with alpha 0
    }
  ]
}
```

### 2. Programmatically Created Windows (Rust)
```rust
WebviewWindowBuilder::new(app, "window-label", url)
    .transparent(true)
    .build()
```

### 3. CSS (Critical Inline Styles)
```html
<head>
  <style>
    html, body {
      margin: 0;
      padding: 0;
      background-color: #1e1e1e;
      color: #e0e0e0;
    }
  </style>
  <!-- External CSS after inline styles -->
  <link rel="stylesheet" href="/styles.css" />
</head>
```

### How It Works
- Window starts transparent (invisible background)
- Inline CSS in `<head>` applies immediately before external CSS loads
- Dark background renders with no white flash
- External CSS enhances styling

### Key Points
- **Always set `.transparent(true)` on programmatically created windows**
- **Use inline critical CSS in HTML `<head>`**
- **Set backgroundColor with alpha 0 in window config**
- Works for both config-defined and runtime-created windows

---
*QuickRun Project - February 2026*
