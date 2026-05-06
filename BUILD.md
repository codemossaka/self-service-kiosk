# Guide de compilation — Branham Kiosque

## Table des matières
1. [Compilation Windows (cible principale)](#windows)
2. [Compilation macOS](#macos)
3. [Compilation Linux](#linux)
4. [Installation kiosque Windows](#kiosque)

---

## <a name="windows"></a>1. Compilation Windows

> Compiler **sur une machine Windows**. La cross-compilation depuis macOS/Linux n'est pas supportée par Tauri pour Windows.

### 1.1 Prérequis

Installer dans cet ordre :

#### a) Rust
```powershell
# Télécharger et lancer l'installeur
Invoke-WebRequest https://win.rustup.rs -OutFile rustup-init.exe
.\rustup-init.exe -y
# Redémarrer le terminal après installation
```

Vérification :
```powershell
rustc --version   # rustc 1.78+ attendu
cargo --version
```

#### b) Node.js LTS
Télécharger depuis https://nodejs.org (version LTS, 20.x ou 22.x)

Vérification :
```powershell
node --version    # v20+ attendu
npm --version
```

#### c) Visual Studio Build Tools 2022
Télécharger "Build Tools for Visual Studio 2022" depuis :
https://visualstudio.microsoft.com/visual-cpp-build-tools/

Dans l'installeur, cocher :
- ✅ **Développement Desktop en C++**
  - MSVC v143 (ou plus récent)
  - Windows 11 SDK (ou Windows 10 SDK)

> WebView2 est déjà inclus dans Windows 10 (mise à jour 2004+) et Windows 11.
> Si manquant : https://developer.microsoft.com/microsoft-edge/webview2/

#### d) SumatraPDF (sur la machine kiosque uniquement)
SumatraPDF est utilisé pour l'impression PDF silencieuse.
```powershell
winget install SumatraPDF.SumatraPDF
```
Ou télécharger depuis : https://www.sumatrapdfreader.org/download-free-pdf-viewer

---

### 1.2 Compilation

```powershell
# Cloner / récupérer le projet
cd branham-tauri

# Installer les dépendances JavaScript
npm install

# Compiler en mode release → génère l'installeur .exe
npm run tauri build
```

**Durée** : 5–15 minutes (première compilation, Rust compile tout)

**Fichiers générés** :
```
src-tauri/target/release/
├── branham-messages.exe          ← exécutable seul (portable)
└── bundle/
    └── nsis/
        └── Bibliotheque_1.0.0_x64-setup.exe  ← installeur complet
```

---

### 1.3 Options de déploiement

#### Option A — Installeur (recommandé pour le kiosque)
Copier `Bibliotheque_1.0.0_x64-setup.exe` sur la machine kiosque et l'exécuter.
L'application s'installe dans `C:\Program Files\Bibliotheque\`.

#### Option B — Portable (clé USB)
Copier uniquement `branham-messages.exe` et le lancer directement.
*(L'impression nécessite quand même SumatraPDF installé.)*

---

## <a name="kiosque"></a>2. Installation kiosque Windows

Après avoir installé l'application, exécuter le script de configuration kiosque **en tant qu'Administrateur** :

```powershell
# Dans PowerShell (clic droit → "Exécuter en tant qu'administrateur")
Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass
.\scripts\kiosque-windows.ps1
```

Le script configure automatiquement :
- ✅ Démarrage automatique de l'application au boot
- ✅ Connexion automatique sans mot de passe (auto-login)
- ✅ L'écran ne s'éteint jamais
- ✅ L'ordinateur ne se met jamais en veille
- ✅ Désactivation du Gestionnaire des tâches
- ✅ Masquage de la barre des tâches

Voir le script `scripts/kiosque-windows.ps1` pour les détails et options avancées.

---

## <a name="macos"></a>3. Compilation macOS

### Prérequis

```bash
# Xcode Command Line Tools
xcode-select --install

# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# Node.js (via Homebrew)
brew install node
```

### Compilation

```bash
cd branham-tauri
npm install
npm run tauri build
```

**Fichiers générés** :
```
src-tauri/target/release/bundle/
├── macos/Bibliotheque.app    ← application macOS
└── dmg/Bibliotheque_1.0.0_x64.dmg  ← image disque installable
```

### Note impression macOS
Sur macOS, l'impression utilise `lp` (CUPS), préinstallé.
Configurer l'imprimante dans **Préférences Système → Imprimantes**.

---

## <a name="linux"></a>4. Compilation Linux

### Prérequis (Ubuntu / Debian)

```bash
# Dépendances système
sudo apt update
sudo apt install -y \
  libwebkit2gtk-4.1-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  build-essential \
  curl \
  wget \
  libgtk-3-dev

# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# Node.js
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt install -y nodejs
```

### Compilation

```bash
cd branham-tauri
npm install
npm run tauri build
```

**Fichiers générés** :
```
src-tauri/target/release/bundle/
├── appimage/Bibliotheque_1.0.0_amd64.AppImage  ← portable Linux
└── deb/Bibliotheque_1.0.0_amd64.deb            ← paquet Debian/Ubuntu
```

### Note impression Linux
L'impression utilise `lp` (CUPS).
```bash
sudo apt install cups
sudo systemctl enable --now cups
# Ajouter l'imprimante via http://localhost:631
```

---

## Résumé rapide

| Plateforme | Commande principale | Fichier de sortie |
|---|---|---|
| Windows | `npm run tauri build` | `bundle/nsis/*-setup.exe` |
| macOS | `npm run tauri build` | `bundle/dmg/*.dmg` |
| Linux | `npm run tauri build` | `bundle/appimage/*.AppImage` |

> **Important** : toujours compiler sur la plateforme cible.
> Un `.exe` Windows ne peut pas être produit depuis macOS ou Linux.
