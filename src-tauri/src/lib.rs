use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
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
    pub folder: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PrintResult {
    pub success: bool,
    pub reason:  Option<String>,
}

// ═══════════════════════════════════════════════════════
//  PARSING DES NOMS DE FICHIERS
//  Formats acceptés :
//    63-0318E The First Seal.pdf
//    63-0318 - The First Seal.pdf
//    63-0318_The_First_Seal.pdf
// ═══════════════════════════════════════════════════════

const MONTHS_RU: [&str; 12] = [
    "янв.", "фев.", "мар.", "апр.", "мая", "июн.",
    "июл.", "авг.", "сен.", "окт.", "ноя.", "дек.",
];

fn parse_sermon_filename(filename: &str) -> Option<Sermon> {
    // Retirer l'extension .pdf (insensible à la casse)
    let base = filename
        .strip_suffix(".pdf")
        .or_else(|| filename.strip_suffix(".PDF"))
        .unwrap_or(filename);

    let chars: Vec<char> = base.chars().collect();

    // Vérifier le format : NN-NNNN
    if chars.len() < 7 { return None; }
    if !chars[0].is_ascii_digit() || !chars[1].is_ascii_digit()
        || chars[2] != '-'
        || !chars[3].is_ascii_digit() || !chars[4].is_ascii_digit()
        || !chars[5].is_ascii_digit() || !chars[6].is_ascii_digit()
    {
        return None;
    }

    // Longueur du code (lettre optionnelle après les chiffres)
    let code_end = if chars.len() > 7 && chars[7].is_ascii_alphabetic() { 8 } else { 7 };
    let code = base[..code_end].to_uppercase();

    // Titre = reste après séparateurs
    let rest = base[code_end..].trim_start_matches([' ', '-', '–', '_']);
    let title = rest.replace('_', " ").trim().to_string();
    if title.is_empty() { return None; }

    // Date depuis le code
    let yy: u32 = base[..2].parse().ok()?;
    let year = if yy >= 40 { 1900 + yy } else { 2000 + yy };
    let mm: u32 = base[3..5].parse().ok()?;
    let dd: u32 = base[5..7].parse().ok()?;
    let month   = MONTHS_RU.get(mm.saturating_sub(1) as usize).unwrap_or(&"?");
    let date    = format!("{} {} {}", dd, month, year);

    Some(Sermon {
        code,
        title,
        date,
        year,
        filename: filename.to_string(),
        lieu: "Jeffersonville, IN".to_string(),
    })
}

// ═══════════════════════════════════════════════════════
//  CONFIG
// ═══════════════════════════════════════════════════════

fn config_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_config_dir()
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
        Ok(Config { folder: None })
    }
}

#[tauri::command]
fn save_config(app: tauri::AppHandle, config: Config) -> Result<(), String> {
    let path = config_path(&app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())
}

// ═══════════════════════════════════════════════════════
//  LECTURE DU DOSSIER PDF
// ═══════════════════════════════════════════════════════

#[tauri::command]
fn read_sermons(folder: String) -> Result<Vec<Sermon>, String> {
    let entries = fs::read_dir(&folder).map_err(|e| {
        format!("Невозможно открыть папку «{}»: {}", folder, e)
    })?;

    let mut sermons: Vec<Sermon> = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|x| x.to_str())
                .map(|x| x.eq_ignore_ascii_case("pdf"))
                .unwrap_or(false)
        })
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            parse_sermon_filename(&name)
        })
        .collect();

    sermons.sort_by(|a, b| a.code.cmp(&b.code));
    Ok(sermons)
}

// ═══════════════════════════════════════════════════════
//  IMPRESSION
//  Configuration fixe : Paysage (Album) + Recto-Verso
// ═══════════════════════════════════════════════════════

#[tauri::command]
async fn print_pdf(folder: String, filename: String, copies: u32) -> PrintResult {
    let path = PathBuf::from(&folder).join(&filename);
    let path_str = path.to_string_lossy().to_string();

    let result = do_print(&path_str, copies);

    match result {
        Ok(()) => PrintResult { success: true, reason: None },
        Err(e)  => PrintResult { success: false, reason: Some(e) },
    }
}

fn do_print(path: &str, copies: u32) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    return print_windows(path, copies);

    #[cfg(target_os = "macos")]
    return print_unix(path, copies);

    #[cfg(target_os = "linux")]
    return print_unix(path, copies);

    #[allow(unreachable_code)]
    Err("Plateforme non supportée".to_string())
}

// ── Windows ──────────────────────────────────────────
// Stratégie :
//   1. SumatraPDF  →  -print-settings "landscape,duplexlong,Nx"
//   2. Fallback PowerShell via WMI (configure l'imprimante puis imprime)
#[cfg(target_os = "windows")]
fn print_windows(path: &str, copies: u32) -> Result<(), String> {
    let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_default();
    let program_files  = std::env::var("ProgramFiles").unwrap_or_else(|_| "C:\\Program Files".into());
    let program_files_x86 = std::env::var("ProgramFiles(x86)").unwrap_or_else(|_| "C:\\Program Files (x86)".into());

    let candidates = [
        format!("{}\\SumatraPDF\\SumatraPDF.exe", local_app_data),
        format!("{}\\SumatraPDF\\SumatraPDF.exe", program_files),
        format!("{}\\SumatraPDF\\SumatraPDF.exe", program_files_x86),
        "C:\\SumatraPDF\\SumatraPDF.exe".to_string(),
    ];

    if let Some(sumatra) = candidates.iter().find(|p| std::path::Path::new(*p).exists()) {
        // -print-settings : landscape = format paysage
        //                   duplexlong = recto-verso bord long
        //                   Nx = N copies
        let settings = format!("landscape,duplexlong,{}x", copies);
        let status = std::process::Command::new(sumatra)
            .args([
                "-print-to-default",
                "-print-settings", &settings,
                "-silent",
                path,
            ])
            .status()
            .map_err(|e| format!("SumatraPDF launch error: {}", e))?;

        return if status.success() {
            Ok(())
        } else {
            Err(format!("SumatraPDF exit code: {:?}", status.code()))
        };
    }

    // ── Fallback : PowerShell + WMI ─────────────────
    // Configure l'imprimante par défaut (paysage + recto-verso)
    // puis imprime, puis restaure les paramètres d'origine.
    let path_safe = path.replace('"', "'");
    let script = format!(r#"
$ErrorActionPreference = 'Stop'
try {{
    # Récupérer l'imprimante par défaut
    $printerName = (Get-WmiObject -Query "SELECT * FROM Win32_Printer WHERE Default=$true").Name
    if (-not $printerName) {{ throw "Aucune imprimante par défaut" }}

    $cfg = Get-WmiObject Win32_PrinterConfiguration -Filter "Name='$printerName'"

    # Sauvegarder les paramètres originaux
    $origOrientation = $cfg.Orientation
    $origDuplex      = $cfg.Duplex
    $origCopies      = $cfg.Copies

    # Appliquer : 2=Paysage, 2=Recto-Verso bord long
    $cfg.Orientation = 2
    $cfg.Duplex      = 2
    $cfg.Copies      = {copies}
    $cfg.Put() | Out-Null

    Start-Sleep -Milliseconds 400

    # Imprimer
    $shell  = New-Object -ComObject Shell.Application
    $dir    = $shell.Namespace([System.IO.Path]::GetDirectoryName("{path_safe}"))
    $file   = $dir.ParseName([System.IO.Path]::GetFileName("{path_safe}"))
    $file.InvokeVerb("Print")

    Start-Sleep -Seconds 6

    # Restaurer les paramètres
    $cfg.Orientation = $origOrientation
    $cfg.Duplex      = $origDuplex
    $cfg.Copies      = $origCopies
    $cfg.Put() | Out-Null

}} catch {{
    Write-Error $_.Exception.Message
    exit 1
}}
"#, copies = copies, path_safe = path_safe);

    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-WindowStyle", "Hidden", "-Command", &script])
        .output()
        .map_err(|e| format!("PowerShell error: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("PowerShell: {}", stderr.trim()))
    }
}

// ── Linux / macOS ─────────────────────────────────────
// Utilise la commande `lp` (CUPS)
//   -o landscape              → format paysage (album)
//   -o sides=two-sided-long-edge → recto-verso bord long
//   -o media=A4               → format papier A4
//   -n N                      → nombre de copies
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn print_unix(path: &str, copies: u32) -> Result<(), String> {
    let output = std::process::Command::new("lp")
        .args([
            "-o", "landscape",
            "-o", "sides=two-sided-long-edge",
            "-o", "media=A4",
            "-n", &copies.to_string(),
            path,
        ])
        .output()
        .map_err(|e| format!("lp command error: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("lp: {}", stderr.trim()))
    }
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
