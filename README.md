# Библиотека — Kiosque Branham

Application kiosque tactile (plein écran, bloqué) pour rechercher et imprimer des prédications PDF.  
Construite avec **Tauri 2 + Vue 3 + TypeScript + Rust**.

---

## Table des matières

1. [Compilation Windows](#1-compilation-windows)  
2. [Compilation macOS](#2-compilation-macos)  
3. [Compilation Linux](#3-compilation-linux)  
4. [Build automatique via GitHub Actions](#4-build-automatique-github-actions)  
5. [Déploiement kiosque Windows](#5-déploiement-kiosque-windows)  
6. [Configuration de l'impression](#6-configuration-de-limpression)  
7. [Format des fichiers PDF](#7-format-des-fichiers-pdf)  
8. [Structure du projet](#8-structure-du-projet)

---

## 1. Compilation Windows

> **La compilation Windows doit se faire sur une machine Windows.**  
> Tauri ne supporte pas la cross-compilation vers Windows depuis macOS ou Linux.  
> Si vous n'avez pas de PC Windows, utilisez [GitHub Actions](#4-build-automatique-github-actions).

### 1.1 Prérequis (à installer une seule fois)

#### Rust
```powershell
winget install Rustlang.Rustup
# — OU — télécharger https://win.rustup.rs et exécuter rustup-init.exe
# Choisir l'option 1 (installation par défaut)
# Fermer et rouvrir le terminal après installation
```
```powershell
rustc --version    # doit afficher rustc 1.78.0 ou plus récent
```

#### Node.js LTS
```powershell
winget install OpenJS.NodeJS.LTS
# — OU — télécharger https://nodejs.org (version LTS 20.x ou 22.x)
```
```powershell
node --version     # doit afficher v20.x ou v22.x
```

#### Visual Studio Build Tools 2022
Télécharger depuis : https://visualstudio.microsoft.com/visual-cpp-build-tools/

Dans l'installeur, cocher obligatoirement :
- ✅ **Développement Desktop en C++**
  - ✅ MSVC v143 — Outils de build C++ x64/x86
  - ✅ Windows 11 SDK (ou Windows 10 SDK)

> **WebView2** est déjà inclus dans Windows 10 (mise à jour 2004+) et Windows 11.  
> Si absent : https://developer.microsoft.com/microsoft-edge/webview2/

#### SumatraPDF *(pour l'impression silencieuse sur la machine kiosque)*
```powershell
winget install SumatraPDF.SumatraPDF
```
Sans SumatraPDF, l'impression fonctionne via PowerShell/WMI (fallback automatique).

---

### 1.2 Cloner et compiler

```powershell
git clone <url-du-depot>
cd branham-tauri

npm install

npm run tauri build
```

Durée : **10–20 minutes** à la première compilation (Rust compile toutes les dépendances).  
Les compilations suivantes : 1–3 minutes.

---

### 1.3 Fichiers produits

```
src-tauri\target\release\
│
├── branham-messages.exe                          ← exécutable portable (pas d'installation)
│
└── bundle\
    └── nsis\
        └── Bibliotheque_1.0.0_x64-setup.exe     ← installeur Windows (recommandé)
```

**Installeur** — installe l'app dans `C:\Program Files\Bibliotheque\`, crée un raccourci bureau et menu Démarrer.  
**Portable** — copier `branham-messages.exe` sur une clé USB et l'exécuter directement.

---

## 2. Compilation macOS

### 2.1 Prérequis

```bash
# Xcode Command Line Tools (outils de compilation Apple)
xcode-select --install

# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# Node.js (via Homebrew)
brew install node
# — OU — télécharger https://nodejs.org
```

### 2.2 Compiler

```bash
cd branham-tauri
npm install
npm run tauri build
```

### 2.3 Fichiers produits

```
src-tauri/target/release/bundle/
├── macos/
│   └── Bibliotheque.app          ← application macOS (glisser dans /Applications)
└── dmg/
    └── Bibliotheque_1.0.0_x64.dmg  ← image disque distribuable
```

> Sur Apple Silicon (M1/M2/M3), la cible par défaut est `aarch64-apple-darwin`.  
> Pour un binaire universel (Intel + Apple Silicon) :
> ```bash
> rustup target add aarch64-apple-darwin x86_64-apple-darwin
> npm run tauri build -- --target universal-apple-darwin
> ```

---

## 3. Compilation Linux

### 3.1 Prérequis (Ubuntu 22.04 / Debian 12)

```bash
sudo apt update && sudo apt install -y \
  libwebkit2gtk-4.1-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  build-essential \
  curl \
  libgtk-3-dev \
  libsoup-3.0-dev \
  libjavascriptcoregtk-4.1-dev

# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# Node.js 20
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt install -y nodejs
```

### 3.2 Compiler

```bash
cd branham-tauri
npm install
npm run tauri build
```

### 3.3 Fichiers produits

```
src-tauri/target/release/bundle/
├── appimage/
│   └── Bibliotheque_1.0.0_amd64.AppImage   ← portable, aucune installation requise
└── deb/
    └── Bibliotheque_1.0.0_amd64.deb        ← paquet Debian/Ubuntu
```

```bash
# Lancer l'AppImage
chmod +x Bibliotheque_1.0.0_amd64.AppImage
./Bibliotheque_1.0.0_amd64.AppImage

# Installer le .deb
sudo dpkg -i Bibliotheque_1.0.0_amd64.deb
```

---

## 4. Build automatique GitHub Actions

> **Si vous n'avez pas de PC Windows**, cette méthode compile le `.exe` automatiquement  
> sur des serveurs GitHub (gratuit pour les dépôts publics, 2 000 min/mois pour les privés).

### 4.1 Activer le workflow

Le fichier `.github/workflows/build.yml` est déjà présent dans le projet.  
Il se déclenche à chaque `git push` sur la branche `main`.

### 4.2 Récupérer les fichiers compilés

1. Pousser le code sur GitHub :
   ```bash
   git add .
   git commit -m "build"
   git push origin main
   ```
2. Sur GitHub → onglet **Actions** → cliquer sur le dernier workflow
3. En bas de page → section **Artifacts** → télécharger :
   - `windows-installer` → contient le `.exe` Windows
   - `macos-dmg` → contient le `.dmg` macOS
   - `linux-appimage` → contient le `.AppImage` Linux

### 4.3 Créer une Release officielle

Pousser un tag versionné pour créer une Release avec les fichiers en téléchargement direct :

```bash
git tag v1.0.0
git push origin v1.0.0
```

GitHub Actions compile automatiquement et publie les fichiers dans **Releases**.

---

## 5. Déploiement kiosque Windows

### 5.1 Installer l'application

Exécuter l'installeur sur la machine kiosque :
```
Bibliotheque_1.0.0_x64-setup.exe
```

### 5.2 Configurer le kiosque (démarrage auto + plein écran bloqué)

Ouvrir **PowerShell en tant qu'Administrateur** et exécuter :

```powershell
Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass
.\scripts\kiosque-windows.ps1
```

Ce script configure automatiquement :

| Paramètre | Valeur |
|---|---|
| Démarrage automatique | ✅ Au boot Windows (registre + planificateur de tâches) |
| Connexion automatique | ✅ Sans saisir de mot de passe |
| Écran | ✅ Ne s'éteint jamais |
| Veille / Hibernation | ✅ Désactivées |
| Gestionnaire des tâches | ✅ Désactivé (Ctrl+Alt+Suppr bloqué) |
| Barre des tâches | ✅ Masquée automatiquement |
| Notifications Windows | ✅ Désactivées |

#### Option kiosque total (remplace Explorer)

Pour que **seule l'application apparaisse** au démarrage (sans bureau Windows) :

```powershell
.\scripts\kiosque-windows.ps1 -ShellReplacement
```

⚠️ Le bureau Windows est entièrement désactivé. Pour revenir à la normale :

```powershell
.\scripts\kiosque-windows.ps1 -Restore
```

### 5.3 Résultat attendu

```
Allumer le PC
    ↓
Windows démarre (connexion automatique)
    ↓
L'application Branham s'ouvre en plein écran
    ↓
Rien d'autre n'est accessible
```

---

## 6. Configuration de l'impression

L'impression est toujours en :
- **Orientation : Portrait (Книжная)**
- **Recto-Verso : Oui**
- **Imprimante : Imprimante par défaut du système**

Configurer l'imprimante par défaut dans :  
**Paramètres Windows → Bluetooth et appareils → Imprimantes et scanners**

### Priorité des méthodes d'impression (Windows)

| Priorité | Méthode | Condition |
|---|---|---|
| 1 | SumatraPDF | Installé dans `Program Files` ou `%LOCALAPPDATA%` |
| 2 | PowerShell + WMI | Fallback automatique si SumatraPDF absent |

### Impression Linux / macOS

Utilise `lp` (CUPS). Configurer l'imprimante par défaut :
```bash
# Voir les imprimantes disponibles
lpstat -p

# Définir l'imprimante par défaut
lpoptions -d nom-imprimante

# Ou via interface web CUPS
xdg-open http://localhost:631
```

---

## 7. Format des fichiers PDF

Les PDFs doivent être nommés avec le code Branham en préfixe :

```
63-0318 The First Seal.pdf
63-0318E The First Seal.pdf
63-0318 - The First Seal.pdf
47-1207_The_Angel_of_God.pdf
```

Format du code : `AA-MMJJ[Lettre]`  
→ `63-0318` = 18 mars 1963

Les fichiers ne respectant pas ce format sont ignorés.

---

## 8. Structure du projet

```
branham-tauri/
│
├── src/                          # Interface Vue 3
│   ├── App.vue                   # Tous les écrans (recherche, résultats, impression)
│   ├── main.ts
│   ├── style.css
│   ├── types/index.ts
│   └── composables/
│       ├── useSermons.ts         # Lecture dossier local ou serveur distant
│       └── usePrinter.ts        # Prévisualisation PDF + envoi à l'imprimante
│
├── src-tauri/                    # Backend Rust
│   ├── src/lib.rs                # Commandes Tauri : config, parsing PDF, impression
│   ├── Cargo.toml
│   └── tauri.conf.json           # Config fenêtre, installeur NSIS
│
├── scripts/
│   └── kiosque-windows.ps1       # Script de configuration kiosque Windows
│
├── .github/
│   └── workflows/
│       └── build.yml             # Build automatique Windows + macOS + Linux
│
├── BUILD.md                      # Ce fichier (instructions complètes)
├── package.json
├── vite.config.ts
└── index.html
```

---

## Données de configuration

La source des PDFs est sauvegardée dans :

| Plateforme | Chemin |
|---|---|
| Windows | `%APPDATA%\com.branham.messages\config.json` |
| macOS | `~/Library/Application Support/com.branham.messages/config.json` |
| Linux | `~/.config/com.branham.messages/config.json` |
