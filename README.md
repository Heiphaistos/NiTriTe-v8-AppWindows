<div align="center">
  <h1>🔧 NITriTe 2.0</h1>
  <p><strong>Outil de diagnostic, réparation et optimisation Windows ultra-complet avec interface Tauri v2 moderne.</strong></p>

  ![Version](https://img.shields.io/badge/version-8.55.0-blue)
  ![Stack](https://img.shields.io/badge/stack-Tauri%20v2%20%2B%20Rust%20%2B%20Vue%203-purple)
  ![Platform](https://img.shields.io/badge/platform-Windows%2010%2F11-informational)
  ![Language](https://img.shields.io/badge/language-Rust%20%2B%20TypeScript-orange)
  ![License](https://img.shields.io/badge/licence-MIT-green)
</div>

---

## 📋 Description

NITriTe 2.0 est un outil Windows tout-en-un conçu pour les techniciens et utilisateurs avancés. Il regroupe 48 pages et plus de 35 onglets de diagnostic pour couvrir l'intégralité du diagnostic, de la réparation et de l'optimisation d'un poste Windows — le tout via une interface native moderne construite avec Tauri v2.

La version 8.55.0 intègre un mode WinPE (ISO bootable), un désinstalleur silencieux multi-format, un système complet de récupération de données via VSS, un assistant IA local (Ollama / llama.cpp portable) et un moteur de scripts intégré.

---

## 📺 Démonstration

<video src="https://media.heiphaistos.org/videos/nitrite.mp4" controls width="100%" preload="none"></video>

---

## ✨ Fonctionnalités

### 🖥️ Système & Diagnostic
- **Diagnostic complet** : CPU, RAM, disques, GPU, carte réseau, températures
- **Historique système** : suivi des événements et alertes Windows
- **Informations matérielles** : inventaire complet BIOS, pilotes, périphériques

### 🔒 Sécurité
- **Scanner de ports** : détection des ports ouverts et services exposés
- **Pare-feu Windows** : visualisation et gestion des règles
- **Partages réseau** : audit des dossiers partagés
- **Registre** : analyse des clés de persistance suspectes

### 🌐 Réseau
- **Connexions actives** : liste des connexions TCP/UDP en temps réel
- **Interfaces réseau** : configuration, statistiques, Wi-Fi
- **Diagnostic DNS / Ping / Traceroute** intégré

### ⚡ Performances
- **Processus** : liste, kill, priorité, consommation CPU/RAM
- **Services Windows** : démarrage/arrêt, type de démarrage
- **Tâches planifiées** : visualisation et suppression

### 💾 Stockage
- **Clonage disque** : wbadmin + robocopy avec gestion des codes de retour
- **Récupération de données** : VSS (Shadow Copy), Corbeille, fichiers supprimés
- **Analyse disque** : espace, santé SMART, partitions

### 🔧 Maintenance & Réparation
- **Réparation Windows** : SFC (`sfc /scannow`), DISM (`RestoreHealth`), politiques de groupe
- **Désinstalleur silencieux** : détection automatique NSIS (`/S`), Inno Setup (`/VERYSILENT`), winget
- **Export de rapports** : rapports complets en TXT/HTML
- **Mode WinPE** : ISO bootable incluse pour réparation hors-OS (15 commandes PE)

---

## 🛠️ Stack technique

| Couche | Technologies |
|--------|-------------|
| Frontend | Vue 3 + TypeScript + Vite |
| Backend natif | Rust + Windows API (`std::process::Command`) |
| Framework desktop | Tauri v2 |
| IPC | Tauri commands (`invoke`) |
| Installer | NSIS (bundle Tauri) |
| Build | `build.bat` (kill → tsc → tauri build) |

---

## 🚀 Installation

### Prérequis

- Windows 10 / 11 (x64)
- Rust stable (`rustup`) + Node.js 18+
- WebView2 Runtime (inclus dans Windows 11, auto-installé sinon)

### Installer (utilisateur final)

Télécharger et exécuter le setup NSIS :

```
Nitrite_8.55.0_x64-setup.exe
```

### Build depuis les sources

```bat
REM Prérequis : npm install (première fois)
npm install

REM Build complet (kill process existant -> tsc -> tauri build)
build.bat
```

L'exécutable généré : `src-tauri\target\release\nitrite.exe`
L'installeur NSIS : `src-tauri\target\release\bundle\nsis\Nitrite_8.55.0_x64-setup.exe`

### Développement

```bat
npm run tauri dev
```

---

## 📂 Architecture

```
NiTriTe-v8-AppWindows/
├── src/
│   ├── components/diagnostic/     # onglets composants Vue (DiagTab*)
│   ├── pages/
│   │   ├── DiagnosticPage.vue     # Orchestrateur principal
│   │   ├── ClonePage.vue
│   │   ├── DataRecoveryPage.vue
│   │   ├── UninstallerPage.vue
│   │   ├── AiAgentsPage.vue       # Assistant IA (Ollama / llama.cpp)
│   │   └── WinPEModePage.vue      # Mode WinPE
│   └── assets/diagnostic.css     # CSS partagé
├── src-tauri/
│   ├── src/
│   │   ├── system/
│   │   │   ├── clone.rs           # wbadmin + robocopy
│   │   │   ├── data_recovery/     # VSS + Corbeille + dossiers utilisateur
│   │   │   ├── connections.rs
│   │   │   ├── processes.rs
│   │   │   ├── services.rs
│   │   │   ├── security.rs
│   │   │   ├── tasks.rs
│   │   │   └── winpe.rs           # commandes PE
│   │   ├── ai/                    # Ollama / llama.cpp portable
│   │   └── installer/
│   │       └── uninstaller.rs     # Désinstall silencieux
│   └── target/release/bundle/nsis/    # Installeur NSIS final
├── build.bat                      # Script de build Windows
└── package.json
```

---

## 📝 Notes techniques

- **CREATE_NO_WINDOW** (`0x08000000`) appliqué sur tous les modules Rust — pas de flash CMD
- Robocopy : codes de retour `< 8` = succès (comportement normal)
- VSS paths : format `\\?\GLOBALROOT` + device_object
- Désinstall NSIS détecté via metadata `VersionInfo` (`Nullsoft`) → `/S`
- Désinstall Inno Setup détecté via `Inno|Jordan Russell` → `/VERYSILENT`

---

## 📝 Licence

MIT — © 2026 Heiphaistos
