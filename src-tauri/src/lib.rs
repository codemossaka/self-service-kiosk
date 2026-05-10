use serde::{Deserialize, Serialize};
use std::{fs, io::Write, path::PathBuf};
use tauri::Manager;

// ═══════════════════════════════════════════════════════
//  TYPES
// ═══════════════════════════════════════════════════════

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Sermon {
    pub code:     String,
    pub title:    String,
    pub date:     String,
    pub year:     u32,
    pub filename: String,
    pub lieu:     String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub source:      Option<String>,
    pub source_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PrintResult {
    pub success: bool,
    pub reason:  Option<String>,
}

// ═══════════════════════════════════════════════════════
//  PARSING NOMS DE FICHIERS PDF
// ═══════════════════════════════════════════════════════

const MONTHS_RU: [&str; 12] = [
    "янв.", "фев.", "мар.", "апр.", "мая", "июн.",
    "июл.", "авг.", "сен.", "окт.", "ноя.", "дек.",
];

fn parse_sermon_filename(filename: &str) -> Option<Sermon> {
    let base = filename
        .strip_suffix(".pdf")
        .or_else(|| filename.strip_suffix(".PDF"))
        .unwrap_or(filename);

    let chars: Vec<char> = base.chars().collect();
    if chars.len() < 7 { return None; }

    if !chars[0].is_ascii_digit() || !chars[1].is_ascii_digit()
        || chars[2] != '-'
        || !chars[3].is_ascii_digit() || !chars[4].is_ascii_digit()
        || !chars[5].is_ascii_digit() || !chars[6].is_ascii_digit()
    { return None; }

    let code_end = if chars.len() > 7 && chars[7].is_ascii_alphabetic() { 8 } else { 7 };
    let code  = base[..code_end].to_uppercase();
    let rest  = base[code_end..].trim_start_matches([' ', '-', '–', '_']);
    let title = rest.replace('_', " ").trim().to_string();
    if title.is_empty() { return None; }

    let yy: u32 = base[..2].parse().ok()?;
    let year    = if yy >= 40 { 1900 + yy } else { 2000 + yy };
    let mm: u32 = base[3..5].parse().ok()?;
    let dd: u32 = base[5..7].parse().ok()?;
    let month   = MONTHS_RU.get(mm.saturating_sub(1) as usize).unwrap_or(&"?");
    let date    = format!("{} {} {}", dd, month, year);

    Some(Sermon {
        code, title, date, year,
        filename: filename.to_string(),
        lieu: "Jeffersonville, IN".to_string(),
    })
}

// ═══════════════════════════════════════════════════════
//  CONFIG
// ═══════════════════════════════════════════════════════

fn config_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path().app_config_dir()
        .map(|d| d.join("config.json"))
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_config(app: tauri::AppHandle) -> Result<Config, String> {
    let path = config_path(&app)?;
    if path.exists() {
        let raw = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        serde_json::from_str(&raw).map_err(|e| e.to_string())
    } else {
        Ok(Config { source: None, source_type: None })
    }
}

#[tauri::command]
fn save_config(app: tauri::AppHandle, config: Config) -> Result<(), String> {
    let path = config_path(&app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(&path, serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}

// ═══════════════════════════════════════════════════════
//  LECTURE DOSSIER LOCAL
// ═══════════════════════════════════════════════════════

#[tauri::command]
fn read_sermons(folder: String) -> Result<Vec<Sermon>, String> {
    let entries = fs::read_dir(&folder)
        .map_err(|e| format!("Невозможно открыть папку «{}»: {}", folder, e))?;

    let mut sermons: Vec<Sermon> = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension()
                .and_then(|x| x.to_str())
                .map(|x| x.eq_ignore_ascii_case("pdf"))
                .unwrap_or(false)
        })
        .filter_map(|e| parse_sermon_filename(&e.file_name().to_string_lossy()))
        .collect();

    sermons.sort_by(|a, b| a.code.cmp(&b.code));
    Ok(sermons)
}

// ═══════════════════════════════════════════════════════
//  IMPRESSION — toujours Paysage + Recto-Verso bord long
// ═══════════════════════════════════════════════════════

#[tauri::command]
async fn print_pdf(folder: String, filename: String, copies: u32) -> PrintResult {
    let path = PathBuf::from(&folder).join(&filename);
    match do_print(&path.to_string_lossy(), copies) {
        Ok(())  => PrintResult { success: true,  reason: None },
        Err(e)  => PrintResult { success: false, reason: Some(e) },
    }
}

#[tauri::command]
async fn print_remote_pdf(url: String, copies: u32) -> PrintResult {
    match download_and_print(&url, copies).await {
        Ok(())  => PrintResult { success: true,  reason: None },
        Err(e)  => PrintResult { success: false, reason: Some(e) },
    }
}

async fn download_and_print(url: &str, copies: u32) -> Result<(), String> {
    let bytes = reqwest::get(url)
        .await
        .map_err(|e| format!("Ошибка загрузки: {}", e))?
        .bytes()
        .await
        .map_err(|e| format!("Ошибка чтения: {}", e))?;

    let mut tmp = tempfile::Builder::new()
        .suffix(".pdf")
        .tempfile()
        .map_err(|e| format!("Ошибка временного файла: {}", e))?;

    tmp.write_all(&bytes).map_err(|e| format!("Ошибка записи: {}", e))?;
    tmp.flush().map_err(|e| e.to_string())?;

    let path = tmp.path().to_string_lossy().to_string();
    let result = do_print(&path, copies);
    drop(tmp);
    result
}

fn do_print(path: &str, copies: u32) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    return print_windows(path, copies);

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    return print_unix(path, copies);

    #[allow(unreachable_code)]
    Err("Plateforme non supportée".to_string())
}

#[cfg(target_os = "windows")]
fn print_windows(path: &str, copies: u32) -> Result<(), String> {
    if let Some(sumatra) = find_sumatra_pdf() {
        let settings = format!("duplexshort,{}x", copies);
        let status = std::process::Command::new(&sumatra)
            .args(["-print-to-default", "-print-settings", &settings, "-silent", path])
            .status()
            .map_err(|e| format!("SumatraPDF: {}", e))?;
        return if status.success() { Ok(()) }
               else { Err(format!("SumatraPDF code: {:?}", status.code())) };
    }

    print_windows_shell_fallback(path, copies)
}

#[cfg(target_os = "windows")]
fn find_sumatra_pdf() -> Option<PathBuf> {
    let local = std::env::var("LOCALAPPDATA").unwrap_or_default();
    let pf    = std::env::var("ProgramFiles").unwrap_or_else(|_| "C:\\Program Files".into());
    let pf86  = std::env::var("ProgramFiles(x86)").unwrap_or_else(|_| "C:\\Program Files (x86)".into());

    let mut candidates = vec![
        PathBuf::from(format!("{}\\SumatraPDF\\SumatraPDF.exe", local)),
        PathBuf::from(format!("{}\\Microsoft\\WinGet\\Links\\SumatraPDF.exe", local)),
        PathBuf::from(format!("{}\\SumatraPDF\\SumatraPDF.exe", pf)),
        PathBuf::from(format!("{}\\SumatraPDF\\SumatraPDF.exe", pf86)),
        PathBuf::from("C:\\SumatraPDF\\SumatraPDF.exe"),
    ];

    if let Some(path) = find_sumatra_in_winget_packages(&local) {
        candidates.push(path);
    }

    if let Some(path) = find_sumatra_with_where() {
        candidates.push(path);
    }

    candidates.into_iter().find(|p| p.exists())
}

#[cfg(target_os = "windows")]
fn find_sumatra_in_winget_packages(local: &str) -> Option<PathBuf> {
    let packages = PathBuf::from(local).join("Microsoft\\WinGet\\Packages");
    let entries = fs::read_dir(packages).ok()?;

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_lowercase();
        if name.contains("sumatrapdf") {
            if let Some(exe) = find_sumatra_under(&entry.path(), 3) {
                return Some(exe);
            }
        }
    }

    None
}

#[cfg(target_os = "windows")]
fn find_sumatra_under(dir: &PathBuf, max_depth: usize) -> Option<PathBuf> {
    if max_depth == 0 {
        return None;
    }

    let entries = fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_lowercase();

        if path.is_file() && name.starts_with("sumatrapdf") && name.ends_with(".exe") {
            return Some(path);
        }

        if path.is_dir() {
            if let Some(found) = find_sumatra_under(&path, max_depth - 1) {
                return Some(found);
            }
        }
    }

    None
}

#[cfg(target_os = "windows")]
fn find_sumatra_with_where() -> Option<PathBuf> {
    let output = std::process::Command::new("where")
        .arg("SumatraPDF.exe")
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(PathBuf::from)
}

#[cfg(target_os = "windows")]
fn print_windows_shell_fallback(path: &str, copies: u32) -> Result<(), String> {
    let attempted_path = PathBuf::from(path);
    let tmp_path = tempfile::Builder::new()
        .prefix("branham-print-")
        .suffix(".pdf")
        .tempfile()
        .map_err(|e| format!("Ошибка временного файла: {}", e))?
        .into_temp_path();

    fs::copy(path, &tmp_path)
        .map_err(|e| format!("Ошибка подготовки печати: {}", e))?;

    let print_path = tmp_path.to_string_lossy().replace('"', "'");
    let script = format!(r#"
$ErrorActionPreference = 'Stop'
try {{
    $name = (Get-WmiObject -Query "SELECT * FROM Win32_Printer WHERE Default=$true").Name
    if (-not $name) {{ throw "Нет принтера по умолчанию" }}
    $cfg = Get-WmiObject Win32_PrinterConfiguration | Where-Object {{ $_.Name -eq $name }} | Select-Object -First 1
    if (-not $cfg) {{ throw "Не удалось открыть настройки принтера" }}
    $o0 = $cfg.Orientation ; $d0 = $cfg.Duplex ; $c0 = $cfg.Copies
    $cfg.Orientation = 1 ; $cfg.Duplex = 3 ; $cfg.Copies = {copies}
    $cfg.Put() | Out-Null
    Start-Sleep -Milliseconds 400
    $sh   = New-Object -ComObject Shell.Application
    $dir  = $sh.Namespace([IO.Path]::GetDirectoryName("{print_path}"))
    if (-not $dir) {{ throw "Не удалось открыть папку PDF" }}
    $file = $dir.ParseName([IO.Path]::GetFileName("{print_path}"))
    if (-not $file) {{ throw "Не удалось открыть PDF для печати" }}
    $file.InvokeVerb("Print")
    Start-Sleep -Seconds 7
    $cfg.Orientation = $o0 ; $cfg.Duplex = $d0 ; $cfg.Copies = $c0
    $cfg.Put() | Out-Null
}} catch {{ Write-Error $_.Exception.Message ; exit 1 }}
"#, copies = copies, print_path = print_path);

    let out = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-WindowStyle", "Hidden", "-Command", &script])
        .output()
        .map_err(|e| format!("PowerShell: {}", e))?;

    if out.status.success() { Ok(()) }
    else {
        Err(format!(
            "Не удалось напечатать PDF через Windows. Установите SumatraPDF и попробуйте снова. Файл: {}",
            attempted_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("PDF")
        ))
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn print_unix(path: &str, copies: u32) -> Result<(), String> {
    let out = std::process::Command::new("lp")
        .args(["-o", "sides=two-sided-short-edge",
               "-o", "media=A4", "-n", &copies.to_string(), path])
        .output()
        .map_err(|e| format!("lp: {}", e))?;

    if out.status.success() { Ok(()) }
    else { Err(format!("lp: {}", String::from_utf8_lossy(&out.stderr).trim())) }
}

// ═══════════════════════════════════════════════════════
//  ENTRY POINT
// ═══════════════════════════════════════════════════════

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_config,
            read_sermons,
            print_pdf,
            print_remote_pdf,
        ])
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();

            window.set_decorations(true).unwrap();
            window.maximize().unwrap();

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
