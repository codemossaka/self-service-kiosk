#Requires -RunAsAdministrator
<#
.SYNOPSIS
    Configure Windows en mode kiosque pour l'application Branham.

.DESCRIPTION
    À exécuter UNE SEULE FOIS après l'installation de l'application.
    Lance ce script en tant qu'Administrateur :
        PowerShell → clic droit → "Exécuter en tant qu'administrateur"
        Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass
        .\scripts\kiosque-windows.ps1

.PARAMETER AppPath
    Chemin vers l'exécutable. Détecté automatiquement si omis.

.PARAMETER Username
    Compte Windows pour l'auto-login. Par défaut : compte courant.

.PARAMETER Password
    Mot de passe du compte pour l'auto-login. Laisser vide si aucun.

.PARAMETER ShellReplacement
    Si présent, remplace Explorer par l'application (kiosque total).
    ATTENTION : désactive complètement le bureau Windows.
    Pour revenir à la normale : .\kiosque-windows.ps1 -Restore

.PARAMETER Restore
    Restaure tous les paramètres Windows normaux.
#>
param(
    [string]$AppPath      = "",
    [string]$Username     = $env:USERNAME,
    [string]$Password     = "",
    [switch]$ShellReplacement,
    [switch]$Restore
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# ══════════════════════════════════════════════════════════
#  DÉTECTION DU CHEMIN DE L'APPLICATION
# ══════════════════════════════════════════════════════════

function Find-AppPath {
    $candidates = @(
        "$env:ProgramFiles\Bibliotheque\Bibliotheque.exe",
        "$env:ProgramFiles\branham-messages\branham-messages.exe",
        "$env:LOCALAPPDATA\Bibliotheque\Bibliotheque.exe",
        # Chercher dans Program Files
        (Get-ChildItem "$env:ProgramFiles" -Filter "Bibliotheque.exe" -Recurse -ErrorAction SilentlyContinue |
            Select-Object -First 1 -ExpandProperty FullName),
        (Get-ChildItem "$env:ProgramFiles" -Filter "branham-messages.exe" -Recurse -ErrorAction SilentlyContinue |
            Select-Object -First 1 -ExpandProperty FullName)
    ) | Where-Object { $_ -and (Test-Path $_) }

    return $candidates | Select-Object -First 1
}

if (-not $Restore) {
    if (-not $AppPath) {
        $AppPath = Find-AppPath
    }
    if (-not $AppPath -or -not (Test-Path $AppPath)) {
        Write-Error "Application introuvable. Installez d'abord l'application ou spécifiez -AppPath 'C:\...\app.exe'"
        exit 1
    }
    Write-Host "Application trouvée : $AppPath" -ForegroundColor Green
}

# ══════════════════════════════════════════════════════════
#  RESTAURATION (annuler toutes les modifications)
# ══════════════════════════════════════════════════════════

if ($Restore) {
    Write-Host "`n=== Restauration des paramètres Windows ===" -ForegroundColor Yellow

    # Supprimer l'autostart
    Remove-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run" `
        -Name "BranhamKiosque" -ErrorAction SilentlyContinue
    Remove-ItemProperty -Path "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Run" `
        -Name "BranhamKiosque" -ErrorAction SilentlyContinue

    # Restaurer le shell Explorer
    Set-ItemProperty -Path "HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon" `
        -Name "Shell" -Value "explorer.exe"
    Set-ItemProperty -Path "HKCU:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon" `
        -Name "Shell" -Value "" -ErrorAction SilentlyContinue
    Remove-ItemProperty -Path "HKCU:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon" `
        -Name "Shell" -ErrorAction SilentlyContinue

    # Réactiver le Gestionnaire des tâches
    Remove-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Policies\System" `
        -Name "DisableTaskMgr" -ErrorAction SilentlyContinue

    # Réactiver la barre des tâches
    Set-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\StuckRects3" `
        -Name "Settings" -Type Binary -Value ([byte[]](0x30,0x00,0x00,0x00,0xFF,0xFF,0xFF,0xFF,
            0x02,0x00,0x00,0x00,0x03,0x00,0x00,0x00,0x5E,0x00,0x00,0x00,0x1E,0x00,0x00,0x00,
            0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x80,0x07,0x00,0x00,0x38,0x04,0x00,0x00,
            0x00,0x00,0x00,0x00,0x28,0x00,0x00,0x00)) -ErrorAction SilentlyContinue

    # Désactiver l'auto-login
    Remove-ItemProperty -Path "HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon" `
        -Name "AutoAdminLogon" -ErrorAction SilentlyContinue
    Remove-ItemProperty -Path "HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon" `
        -Name "DefaultUserName" -ErrorAction SilentlyContinue
    Remove-ItemProperty -Path "HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon" `
        -Name "DefaultPassword" -ErrorAction SilentlyContinue

    # Restaurer la gestion de l'alimentation par défaut
    powercfg /change monitor-timeout-ac 10
    powercfg /change standby-timeout-ac 30
    powercfg /change hibernate-timeout-ac 60

    Write-Host "Restauration terminée. Redémarrez Windows." -ForegroundColor Green
    exit 0
}

# ══════════════════════════════════════════════════════════
#  1. DÉMARRAGE AUTOMATIQUE AU BOOT
# ══════════════════════════════════════════════════════════

Write-Host "`n[1/6] Configuration du démarrage automatique..." -ForegroundColor Cyan

# Méthode 1 : Registre Run (tous utilisateurs)
Set-ItemProperty `
    -Path "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Run" `
    -Name "BranhamKiosque" `
    -Value "`"$AppPath`""

# Méthode 2 : Planificateur de tâches (plus fiable, démarre avant la session)
$action  = New-ScheduledTaskAction -Execute $AppPath
$trigger = New-ScheduledTaskTrigger -AtLogOn -User $Username
$settings = New-ScheduledTaskSettingsSet `
    -AllowStartIfOnBatteries `
    -DontStopIfGoingOnBatteries `
    -ExecutionTimeLimit (New-TimeSpan -Hours 0) `
    -RestartCount 3 `
    -RestartInterval (New-TimeSpan -Minutes 1)
$principal = New-ScheduledTaskPrincipal `
    -UserId $Username `
    -LogonType Interactive `
    -RunLevel Highest

Register-ScheduledTask `
    -TaskName  "BranhamKiosque" `
    -TaskPath  "\" `
    -Action    $action `
    -Trigger   $trigger `
    -Settings  $settings `
    -Principal $principal `
    -Force | Out-Null

Write-Host "  ✓ Démarrage automatique configuré (registre + planificateur)" -ForegroundColor Green

# ══════════════════════════════════════════════════════════
#  2. CONNEXION AUTOMATIQUE (AUTO-LOGIN)
# ══════════════════════════════════════════════════════════

Write-Host "`n[2/6] Configuration de la connexion automatique..." -ForegroundColor Cyan

$winlogonPath = "HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon"
Set-ItemProperty -Path $winlogonPath -Name "AutoAdminLogon"  -Value "1"
Set-ItemProperty -Path $winlogonPath -Name "DefaultUserName" -Value $Username
Set-ItemProperty -Path $winlogonPath -Name "DefaultDomainName" -Value $env:COMPUTERNAME

if ($Password) {
    Set-ItemProperty -Path $winlogonPath -Name "DefaultPassword" -Value $Password
} else {
    # Compte sans mot de passe
    Remove-ItemProperty -Path $winlogonPath -Name "DefaultPassword" -ErrorAction SilentlyContinue
}

Write-Host "  ✓ Auto-login activé pour l'utilisateur : $Username" -ForegroundColor Green

# ══════════════════════════════════════════════════════════
#  3. ALIMENTATION — écran toujours allumé, pas de veille
# ══════════════════════════════════════════════════════════

Write-Host "`n[3/6] Configuration de l'alimentation..." -ForegroundColor Cyan

# Créer un plan d'alimentation "Kiosque" (copie de Haute performance)
$plan = powercfg /duplicatescheme 8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c 2>&1
if ($plan -match "\{([0-9a-f-]+)\}") {
    $planGuid = $Matches[1]
    powercfg /changename $planGuid "Kiosque Branham"
    powercfg /setactive $planGuid
} else {
    $planGuid = (powercfg /getactivescheme) -replace '.*GUID: ([0-9a-f-]+).*','$1'
}

powercfg /change monitor-timeout-ac  0   # Écran : jamais
powercfg /change monitor-timeout-dc  0
powercfg /change standby-timeout-ac  0   # Veille : jamais
powercfg /change standby-timeout-dc  0
powercfg /change hibernate-timeout-ac 0  # Hibernation : jamais
powercfg /change hibernate-timeout-dc 0
powercfg /hibernate off

Write-Host "  ✓ Écran et veille : jamais" -ForegroundColor Green

# ══════════════════════════════════════════════════════════
#  4. GESTIONNAIRE DES TÂCHES — laissé actif
#     L'administrateur peut fermer l'application via :
#       - Ctrl+F4 (raccourci intégré dans l'app)
#       - Ctrl+Alt+Suppr → Gestionnaire des tâches → Fin de tâche
# ══════════════════════════════════════════════════════════

Write-Host "`n[4/6] Gestionnaire des tâches : accessible (fermeture via Ctrl+F4 ou TaskMgr)" -ForegroundColor Cyan

# S'assurer qu'il n'est PAS désactivé (au cas où un réglage précédent l'aurait bloqué)
$policyPath = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Policies\System"
Remove-ItemProperty -Path $policyPath -Name "DisableTaskMgr" -ErrorAction SilentlyContinue

Write-Host "  ✓ Gestionnaire des tâches : activé" -ForegroundColor Green

# ══════════════════════════════════════════════════════════
#  5. MASQUER LA BARRE DES TÂCHES
# ══════════════════════════════════════════════════════════

Write-Host "`n[5/6] Masquage de la barre des tâches..." -ForegroundColor Cyan

# Masquage automatique de la barre des tâches
$regPath = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\StuckRects3"
if (Test-Path $regPath) {
    $settings = (Get-ItemProperty -Path $regPath -Name "Settings").Settings
    # Bit 3 de l'octet 8 = auto-hide
    $settings[8] = $settings[8] -bor 0x01
    Set-ItemProperty -Path $regPath -Name "Settings" -Value $settings -Type Binary
}

Write-Host "  ✓ Barre des tâches en masquage automatique" -ForegroundColor Green

# ══════════════════════════════════════════════════════════
#  6. MODE SHELL REPLACEMENT (optionnel, kiosque total)
# ══════════════════════════════════════════════════════════

if ($ShellReplacement) {
    Write-Host "`n[6/6] Remplacement du shell Windows (kiosque total)..." -ForegroundColor Cyan
    Write-Host "  ATTENTION : le bureau Windows sera désactivé." -ForegroundColor Yellow
    Write-Host "  Pour revenir à la normale : .\kiosque-windows.ps1 -Restore" -ForegroundColor Yellow

    # Remplacer Explorer par l'application pour cet utilisateur
    $userWinlogon = "HKCU:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon"
    if (-not (Test-Path $userWinlogon)) { New-Item -Path $userWinlogon -Force | Out-Null }
    Set-ItemProperty -Path $userWinlogon -Name "Shell" -Value "`"$AppPath`""

    Write-Host "  ✓ Shell remplacé par l'application Branham" -ForegroundColor Green
} else {
    Write-Host "`n[6/6] Mode shell replacement : ignoré (utiliser -ShellReplacement pour activer)" -ForegroundColor Gray
}

# ══════════════════════════════════════════════════════════
#  DÉSACTIVER LES NOTIFICATIONS WINDOWS
# ══════════════════════════════════════════════════════════

# Désactiver les notifications toast (mises à jour, alertes…)
$notifPath = "HKCU:\Software\Microsoft\Windows\CurrentVersion\PushNotifications"
if (-not (Test-Path $notifPath)) { New-Item -Path $notifPath -Force | Out-Null }
Set-ItemProperty -Path $notifPath -Name "ToastEnabled" -Value 0 -Type DWord

# Désactiver Windows Update automatique pendant les heures de bureau
$wuPath = "HKLM:\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate\AU"
if (-not (Test-Path $wuPath)) { New-Item -Path $wuPath -Force | Out-Null }
Set-ItemProperty -Path $wuPath -Name "NoAutoRebootWithLoggedOnUsers" -Value 1 -Type DWord

# ══════════════════════════════════════════════════════════
#  RÉSUMÉ
# ══════════════════════════════════════════════════════════

Write-Host @"

╔══════════════════════════════════════════════════════════╗
║           Configuration kiosque terminée ✓              ║
╠══════════════════════════════════════════════════════════╣
║  Application  : $AppPath
║  Utilisateur  : $Username
║  Auto-login   : Oui
║  Autostart    : Registre + Planificateur de tâches
║  Écran        : Ne s'éteint jamais
║  Veille       : Désactivée
║  Fermer app   : Ctrl+F4  ou  TaskMgr → Fin de tâche
║                                                          ║
║  → Redémarrez Windows pour appliquer tous les réglages  ║
║                                                          ║
║  Pour annuler : .\kiosque-windows.ps1 -Restore           ║
╚══════════════════════════════════════════════════════════╝
"@ -ForegroundColor Green

$reboot = Read-Host "`nRedémarrer maintenant ? (O/N)"
if ($reboot -match "^[Oo]") {
    Write-Host "Redémarrage dans 5 secondes..." -ForegroundColor Yellow
    Start-Sleep 5
    Restart-Computer -Force
}
