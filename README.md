# Branham Kiosk — Sermon Library

A fullscreen touchscreen kiosk application for searching and printing William Branham sermon PDFs.  
Built for church self-service printing stations running Windows, macOS, or Linux.

**Stack:** Tauri 2 · Vue 3 · TypeScript · Rust

---

## Table of Contents

1. [Overview](#1-overview)
2. [Application Screens](#2-application-screens)
3. [PDF File Format](#3-pdf-file-format)
4. [Data Sources](#4-data-sources)
5. [Printing](#5-printing)
6. [Kiosk Mode](#6-kiosk-mode)
7. [Development](#7-development)
8. [Building](#8-building)
9. [Windows Kiosk Deployment](#9-windows-kiosk-deployment)
10. [Configuration Storage](#10-configuration-storage)
11. [Project Structure](#11-project-structure)
12. [Tauri Commands Reference](#12-tauri-commands-reference)

---

## 1. Overview

Branham Kiosk is a locked-down desktop application designed to run as a single-purpose printing station at a church. Users can search through a collection of sermon PDFs, preview them, select the number of copies, and send them to the default printer — all without touching a keyboard or mouse, using the built-in on-screen Cyrillic keyboard.

Key characteristics:
- **Fullscreen + always on top** — no desktop visible, no taskbar access by default
- **No window decorations** — no title bar, no resize handles
- **Self-contained** — works with a local folder of PDFs or a remote HTTP server
- **Silent printing** — sends directly to the default printer with no print dialog
- **Touchscreen-ready** — all interactions are large tap targets with on-screen keyboard

---

## 2. Application Screens

The app is a single-page application with six screens managed by an internal router (`Screen` type). There is no URL routing — navigation is state-based.

### Screen: Setup (`s-setup`)

Shown on first launch or when no source is configured. The admin selects where the PDFs come from.

**Local folder tab**
- Opens a native folder picker dialog
- Scans all `.pdf` files in the selected folder
- Files that do not match the Branham filename format are silently ignored
- The selected path is saved to `config.json` for subsequent launches

**Remote server tab**
- Admin enters an HTTPS URL pointing to a folder of PDFs
- The app first attempts to fetch `sermons.json` from that URL (pre-built index)
- If not found, falls back to parsing the HTML directory listing for `.pdf` links
- The URL is saved to `config.json`

On successful load the app navigates directly to the Home screen. On subsequent launches the saved source is loaded automatically — the Setup screen is skipped.

---

### Screen: Home (`s-home`)

The main screen users see. Contains three elements stacked vertically:

**Search bar**
- Large text input (EB Garamond, 28 px) with a gold focus ring
- Supports physical keyboard input and the on-screen keyboard
- `Escape` clears the input

**On-screen Cyrillic keyboard (ЙЦУКЕН layout)**
```
Й Ц У К Е Н Г Ш Щ З Х Ъ
Ф Ы В А П Р О Л Д Ж Э
Я Ч С М И Т Ь Б Ю
[⌫ Erase]  [SPACE]  [Search ›]
```
Each key press appends the character to the search input without stealing focus.

**Browse by year button**
- Shows the total count and year range (e.g. "1947 – 1965 · 1 132 sermons")
- Navigates to the Years screen

---

### Screen: Search Results (`s-results`)

Displays sermons matching the search query, sorted by code.

**Query bar (top)**
- Shows the active search query
- **"✎ Edit"** button expands the bar into an editable input with the on-screen keyboard inline — the user can refine the search without going back to Home
- Results update in place when a new search is submitted
- **"✕ Cancel"** collapses the keyboard and restores the previous query

**Result list**
Each card shows:
- Year badge (left)
- Title (large)
- Date · Location · Filename (small metadata)
- Arrow indicator

Tapping a card opens the Preview screen.

---

### Screen: Browse by Year (`s-years`)

A 5-column grid of year cards. Each card shows the year number and the count of sermons for that year. Tapping a year navigates to the year list.

---

### Screen: Year List (`s-yr-list`)

The full list of sermons for the selected year, same card layout as search results.

---

### Screen: Preview & Print (`s-preview`)

Split layout: PDF viewer on the left, print panel on the right.

**PDF viewer (left)**
- Renders the PDF in a native WebView iframe
- Shows a loading spinner while the file loads
- For local files: uses Tauri's `asset://` protocol (converted via `convertFileSrc`)
- For remote files: uses the direct HTTPS URL

**Print panel (right, 250 px wide)**

*Document info card*
| Field | Value |
|---|---|
| Code | e.g. `63-0318` |
| Date | e.g. `18 мар. 1963` |
| Location | `Jeffersonville, IN` |
| Filename | original filename |

*Copies selector*
- `−` / `+` buttons, range 1–9
- Resets to 1 when a new sermon is opened

*Print button*
- **Idle:** gold, shows "PRINT" + copy count + "Duplex"
- **Printing:** amber, shows "SENDING…"
- **Success:** green, shows "SENT!" for 4 seconds
- **Error:** red, shows the error message for 5 seconds

---

## 3. PDF File Format

PDF files must be named using the Branham date code as a prefix. The parser (implemented identically in both Rust and TypeScript) extracts the code, title, and date from the filename.

### Format

```
AA-MMDD[Letter] Title.pdf
```

| Part | Description | Example |
|---|---|---|
| `AA` | Two-digit year | `63` → 1963 |
| `MM` | Month (01–12) | `03` → March |
| `DD` | Day (01–31) | `18` |
| `Letter` | Optional suffix letter (A–Z) | `E` |
| `Title` | Sermon title (spaces or underscores) | `The First Seal` |

### Valid examples

```
63-0318 The First Seal.pdf
63-0318E The First Seal.pdf
63-0318 - The First Seal.pdf
47-1207_The_Angel_of_God.pdf
```

### Year interpretation

| Code year | Interpreted as |
|---|---|
| `40` – `99` | 1940 – 1999 |
| `00` – `39` | 2000 – 2039 |

Files that do not match the `AA-MMDD` prefix pattern are silently ignored during folder scan.

---

## 4. Data Sources

### Local folder

The Rust backend scans the folder using `std::fs::read_dir`, filters for `.pdf` extensions (case-insensitive), and parses each filename. The resulting list is sorted by code.

Config key: `sourceType = "local"`, `source = "/path/to/folder"`

### Remote server

The TypeScript frontend handles remote loading directly (no Rust involvement):

1. **`sermons.json` manifest** — fetched first. Expected format: an array of `Sermon` objects matching the TypeScript interface. This is the fastest and most reliable method.

2. **HTML directory listing fallback** — if `sermons.json` returns a non-200 or is absent, the app fetches the base URL and parses all `<a href="...">` links ending in `.pdf`. Works with Apache/Nginx autoindex and similar directory listing servers.

Config key: `sourceType = "remote"`, `source = "https://example.com/sermons"`

### `sermons.json` format

```json
[
  {
    "code": "63-0318",
    "title": "The First Seal",
    "date": "18 мар. 1963",
    "year": 1963,
    "filename": "63-0318 The First Seal.pdf",
    "lieu": "Jeffersonville, IN"
  }
]
```

---

## 5. Printing

### Print settings (fixed)

| Setting | Value |
|---|---|
| Orientation | Portrait |
| Duplex | Two-sided, short-edge binding (book style) |
| Copies | Selected by user (1–9) |
| Printer | System default printer |
| Dialog | None — fully silent |

### Windows — printing priority

The Rust backend checks for SumatraPDF in the following locations, in order:

1. `%LOCALAPPDATA%\SumatraPDF\SumatraPDF.exe`
2. `%ProgramFiles%\SumatraPDF\SumatraPDF.exe`
3. `%ProgramFiles(x86)%\SumatraPDF\SumatraPDF.exe`
4. `C:\SumatraPDF\SumatraPDF.exe`

**If SumatraPDF is found:**
```
SumatraPDF.exe -print-to-default -print-settings "duplexshort,Nx" -silent file.pdf
```

**If SumatraPDF is not found (PowerShell/WMI fallback):**
- Reads the default printer via `Win32_Printer`
- Sets `Orientation = 1` (portrait), `Duplex = 3` (short-edge), `Copies = N` via `Win32_PrinterConfiguration`
- Invokes `Shell.Application` → `InvokeVerb("Print")`
- Restores original printer settings after 7 seconds

> SumatraPDF is strongly recommended. The WMI fallback is less reliable and depends on printer driver behavior.

### macOS / Linux — CUPS

```bash
lp -o sides=two-sided-short-edge -o media=A4 -n <copies> <file>
```

### Remote PDF printing

For remote sources, the Rust backend downloads the PDF to a temporary file using `reqwest` (async), writes it to disk with `tempfile`, prints it, then deletes the temp file.

### Setting the default printer

| Platform | How to set |
|---|---|
| Windows | Settings → Bluetooth & devices → Printers & scanners |
| macOS | System Preferences → Printers & Scanners |
| Linux | `lpoptions -d printer-name` or via `http://localhost:631` |

---

## 6. Kiosk Mode

### Window properties (`tauri.conf.json`)

| Property | Value | Effect |
|---|---|---|
| `fullscreen` | `true` | Starts fullscreen |
| `decorations` | `false` | No title bar, no border |
| `alwaysOnTop` | `true` | Stays above all other windows |
| `minimizable` | `false` | Cannot be minimized |
| `maximizable` | `false` | Cannot be maximized |
| `closable` | `true` | Allows closing via shortcuts (see below) |

### Closing the application (admin)

| Method | Action |
|---|---|
| `Ctrl + F4` | Closes immediately (built-in keyboard shortcut) |
| Taskbar → right-click → Close window | Sends WM_CLOSE, closes normally |
| Task Manager → End Task | Always works |

`Ctrl + F4` calls `getCurrentWindow().destroy()` from the Vue layer, which bypasses any close prevention in the Rust layer.

---

## 7. Development

### Prerequisites

- **Node.js** 20+ — https://nodejs.org
- **Rust** stable — https://rustup.rs
- **Yarn** — `npm install -g yarn`
- **Tauri CLI** — installed automatically via `yarn install`

**Windows only:** Visual Studio 2022 Build Tools with the "Desktop development with C++" workload.

### Install dependencies

```bash
yarn install
```

### Start the dev server

```bash
yarn dev
```

This runs `tauri dev`, which:
1. Starts the Vite frontend on `http://localhost:1420` (via `beforeDevCommand: yarn vite`)
2. Compiles the Rust backend
3. Opens the application window with hot-reload for the frontend

Frontend changes (Vue, CSS) reload instantly. Rust changes trigger a recompile.

### Other scripts

```bash
yarn vite          # frontend only (no Tauri window)
yarn tauri:build   # production build
```

---

## 8. Building

> Each platform must be compiled **on** that platform. Tauri does not support cross-compilation.  
> Use [GitHub Actions](#github-actions) to build Windows `.exe` without a Windows machine.

### Windows

**Additional prerequisites:**
- Visual Studio 2022 Build Tools (Desktop C++ workload + Windows SDK)
- WebView2 Runtime (pre-installed on Windows 10 2004+ and Windows 11)
- SumatraPDF on the target kiosk machine

```powershell
yarn install
yarn tauri:build
```

**Output:**
```
src-tauri\target\release\bundle\nsis\Bibliotheque_1.0.0_x64-setup.exe   ← installer
src-tauri\target\release\branham-messages.exe                            ← portable
```

The NSIS installer:
- Installs to `C:\Program Files\Bibliotheque\`
- Creates a Start Menu folder "Branham"
- Installs for all users (`perMachine`)
- Language: Russian

---

### macOS

```bash
yarn install
yarn tauri:build
# or universal binary (Intel + Apple Silicon):
yarn tauri:mac:universal
```

**Output:**
```
src-tauri/target/release/bundle/dmg/Bibliotheque_1.0.0_x64.dmg
src-tauri/target/release/bundle/macos/Bibliotheque.app
```

---

### Linux (Ubuntu 22.04 / Debian 12)

Install system dependencies first:

```bash
sudo apt update && sudo apt install -y \
  libwebkit2gtk-4.1-dev libssl-dev libayatana-appindicator3-dev \
  librsvg2-dev libgtk-3-dev libsoup-3.0-dev \
  libjavascriptcoregtk-4.1-dev build-essential patchelf
```

```bash
yarn install
yarn tauri:build
```

**Output:**
```
src-tauri/target/release/bundle/appimage/Bibliotheque_1.0.0_amd64.AppImage
src-tauri/target/release/bundle/deb/Bibliotheque_1.0.0_amd64.deb
```

---

### GitHub Actions

The workflow at `.github/workflows/build.yml` builds all three platforms automatically on every push to `main`.

**To get a Windows `.exe` without a Windows machine:**

```bash
git add .
git commit -m "build"
git push origin main
```

Then on GitHub → **Actions** tab → latest run → **Artifacts** → download `windows-installer`.

**To publish a release with download links:**

```bash
git tag v1.0.0
git push origin v1.0.0
```

GitHub Actions compiles all platforms and publishes them automatically under **Releases**.

---

## 9. Windows Kiosk Deployment

### Step 1 — Install the app

Run `Bibliotheque_1.0.0_x64-setup.exe` on the kiosk machine.

### Step 2 — Run the kiosk setup script

Open **PowerShell as Administrator** and run:

```powershell
Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass
.\scripts\kiosque-windows.ps1
```

#### What the script configures

| Setting | Result |
|---|---|
| Auto-start | Registry `Run` key + Scheduled Task (dual method, restarts on crash) |
| Auto-login | Logs in automatically without password prompt |
| Screen timeout | Never (monitor stays on) |
| Sleep / Hibernate | Disabled |
| Task Manager | **Enabled** (admin can kill the app) |
| Taskbar | Auto-hide |
| Windows notifications | Disabled |
| Windows Update reboot | Blocked while user is logged in |

#### Parameters

```powershell
# Specify app path manually (auto-detected by default)
.\scripts\kiosque-windows.ps1 -AppPath "C:\Program Files\Bibliotheque\Bibliotheque.exe"

# Specify which user account auto-logs in
.\scripts\kiosque-windows.ps1 -Username "Kiosk" -Password "secret"

# Full kiosk mode: replace Windows Explorer entirely (no desktop at all)
.\scripts\kiosque-windows.ps1 -ShellReplacement

# Undo all changes and restore Windows defaults
.\scripts\kiosque-windows.ps1 -Restore
```

### Step 3 — Configure the PDF source

On first launch the Setup screen appears. The admin selects either:
- A local folder containing the sermon PDFs
- A remote HTTPS URL

The choice is saved permanently. On all subsequent boots the app loads the source automatically and goes straight to the Home screen.

### Step 4 — Configure the printer

In Windows Settings → Printers & Scanners, set the correct printer as default.  
The app always prints to whatever printer is set as default — there is no printer selection in the UI.

---

## 10. Configuration Storage

The app stores a single `config.json` file:

| Platform | Path |
|---|---|
| Windows | `%APPDATA%\com.branham.messages\config.json` |
| macOS | `~/Library/Application Support/com.branham.messages/config.json` |
| Linux | `~/.config/com.branham.messages/config.json` |

### File format

```json
{
  "source": "/path/to/sermons",
  "source_type": "local"
}
```

or for remote:

```json
{
  "source": "https://example.com/sermons",
  "source_type": "remote"
}
```

To reset the app to first-launch state, delete this file.

---

## 11. Project Structure

```
branham-tauri/
│
├── src/                              # Vue 3 frontend
│   ├── App.vue                       # All screens + navigation logic
│   ├── main.ts                       # Vue app entry point
│   ├── style.css                     # Global styles (fonts, resets)
│   ├── types/
│   │   └── index.ts                  # Shared TypeScript interfaces
│   └── composables/
│       ├── useSermons.ts             # Sermon loading, search, year filter
│       └── usePrinter.ts            # PDF URL resolution, print invocation
│
├── src-tauri/                        # Rust backend
│   ├── src/
│   │   ├── main.rs                   # Tauri entry point
│   │   └── lib.rs                    # All Tauri commands + print logic
│   ├── Cargo.toml                    # Rust dependencies
│   ├── tauri.conf.json               # Window config, bundle targets, NSIS settings
│   └── icons/                        # App icons (all sizes)
│
├── scripts/
│   └── kiosque-windows.ps1           # Windows kiosk setup script
│
├── .github/
│   └── workflows/
│       └── build.yml                 # CI: build Windows + macOS + Linux
│
├── package.json                      # Node scripts and JS dependencies
├── vite.config.ts                    # Vite build config
├── tsconfig.json
└── index.html
```

---

## 12. Tauri Commands Reference

These are the Rust functions exposed to the frontend via `invoke()`.

### `get_config() → Config`

Reads `config.json` from the app config directory.  
Returns `{ source: null, source_type: null }` if the file does not exist.

### `save_config(config: Config) → void`

Writes `config.json`. Creates the directory if it does not exist.

### `read_sermons(folder: string) → Sermon[]`

Scans a local directory for PDF files, parses each filename using the Branham date-code format, and returns the list sorted by code. Throws if the directory cannot be read.

### `print_pdf(folder: string, filename: string, copies: number) → PrintResult`

Prints a local PDF file. Constructs the full path as `folder + filename`, then delegates to `do_print()`.

### `print_remote_pdf(url: string, copies: number) → PrintResult`

Downloads the PDF from `url` using `reqwest` (async), writes it to a temp file using `tempfile`, calls `do_print()`, then deletes the temp file.

### `PrintResult`

```typescript
interface PrintResult {
  success: boolean
  reason:  string | null   // error message if success === false
}
```

### Internal print dispatch (`do_print`)

```
do_print(path, copies)
    │
    ├── [Windows] print_windows(path, copies)
    │       ├── SumatraPDF found → SumatraPDF.exe -print-to-default ...
    │       └── not found → PowerShell WMI script
    │
    └── [macOS / Linux] print_unix(path, copies)
            └── lp -o sides=two-sided-short-edge -o media=A4 -n <copies> <path>
```
