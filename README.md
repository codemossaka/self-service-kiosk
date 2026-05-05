# Проповеди Брэнхэма — Tauri 2 + Vue 3 + TypeScript

Application kiosque tactile pour rechercher et imprimer des prédications PDF.  
**Impression fixe : format Album (paysage) + Recto-Verso bord long.**

---

## 📁 Format des fichiers PDF

Les PDFs doivent être nommés avec le code Branham en tête :

```
63-0318  The First Seal.pdf
63-0318E The First Seal.pdf
63-0318 - The First Seal.pdf
47-1207_The_Angel_of_God.pdf
```

Code format : `AA-MMJJ[Lettre]`  
→ `63-0318` = 18 mars 1963

---

## 🚀 Installation

### Prérequis

| Outil | Version | Lien |
|-------|---------|------|
| Node.js | 18+ | https://nodejs.org |
| Rust | stable | https://rustup.rs |
| Tauri CLI | 2.x | `npm install` l'installe |

**Windows uniquement — pour l'impression avancée :**  
Installer **SumatraPDF** → https://www.sumatrapdfreader.org/free-pdf-reader  
*(Sans SumatraPDF, le fallback PowerShell/WMI est utilisé)*

### Commandes

```bash
# Installer les dépendances
npm install

# Développement (hot-reload)
npm run tauri dev

# Build production (.exe)
npm run tauri build
```

---

## 🖨️ Configuration impression

L'impression est **toujours** en :
- **Format : Album (Paysage)**
- **Recto-Verso : Bord long**
- **Imprimante : Imprimante par défaut du système**

Pour changer l'imprimante par défaut → Paramètres Windows → Bluetooth et appareils → Imprimantes.

### Priorité Windows
1. SumatraPDF (recommandé — silencieux, précis)
2. PowerShell + WMI (fallback automatique)

### Linux / macOS
Utilise la commande `lp` (CUPS). L'imprimante par défaut doit être configurée via `lpoptions`.

---

## ⚙️ Mode kiosque (plein écran bloqué)

Dans `src-tauri/tauri.conf.json`, modifier la fenêtre :

```json
"windows": [{
  "title": "Проповеди Брэнхэма",
  "width": 1440,
  "height": 900,
  "fullscreen": true,
  "decorations": false
}]
```

---

## 📂 Données persistées

Config sauvegardée dans :
- **Windows :** `%APPDATA%\com.branham.messages\config.json`
- **macOS :** `~/Library/Application Support/com.branham.messages/config.json`
- **Linux :** `~/.config/com.branham.messages/config.json`

---

## 🗂️ Structure du projet

```
branham-tauri/
├── src/
│   ├── main.ts                   # Entrée Vue
│   ├── App.vue                   # UI complète (tous les écrans)
│   ├── style.css                 # Styles globaux
│   ├── types/
│   │   └── index.ts              # Types TypeScript
│   └── composables/
│       ├── useSermons.ts         # Lecture dossier + recherche
│       └── usePrinter.ts        # Prévisualisation + impression
├── src-tauri/
│   ├── src/
│   │   ├── main.rs               # Entry Rust
│   │   └── lib.rs                # Commands Tauri (parsing, config, print)
│   ├── capabilities/
│   │   └── default.json          # Permissions Tauri 2
│   ├── Cargo.toml
│   └── tauri.conf.json
├── index.html
├── vite.config.ts
├── tsconfig.json
└── package.json
```
