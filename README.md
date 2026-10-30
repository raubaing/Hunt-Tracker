# Hunt Tracker

Petit widget flottant pour suivre ses sessions de **Hunt: Showdown 1896** (kills, morts, assists, KDA).

> Statut : squelette du projet, aucune fonctionnalité implémentée pour l'instant.

## Stack

- **Tauri 2** (Rust) : fenêtres natives, exécutable léger
- **Svelte 5 + TypeScript + Vite** : interface
- **JSON** : une session = un fichier, dans `%APPDATA%\com.robinnaud.hunttracker\sessions\`

## Structure

```
src/
  lib/          types.ts (modèle), stats.ts (KDA), storage.ts (JSON)
  widget/       widget flottant : sans bordure, toujours au premier plan, déplaçable
  main/         fenêtre principale : tableau de session, historique
src-tauri/
  tauri.conf.json         2 fenêtres : "widget" et "main" (cachée au démarrage)
  capabilities/           permissions des fenêtres
  src/lib.rs              point d'entrée Rust (plugins à venir)
data-example/             exemple de fichier de session
.github/workflows/        build.yml : compile le .exe Windows sur GitHub
```

## Développer (facultatif)

Prérequis : Node.js LTS, Rust (rustup), Visual Studio Build Tools (C++).

```
npm install
npm run tauri dev      # lance l'appli en mode dev
npm run tauri build    # produit l'installeur dans src-tauri/target/release/bundle/nsis/
```

## Compiler via GitHub (sans rien installer)

- Chaque push sur `main` compile l'appli : onglet **Actions** > dernier run > **Artifacts** >
  `HuntTracker-windows` (contient l'installeur `*-setup.exe` et l'exe portable `hunt-tracker.exe`).
- Un tag `v0.1.0` crée en plus une **Release** (brouillon) avec l'installeur, à publier pour la partager.

## Feuille de route

1. Widget : compteurs K / D / A, validation d'une game
2. Sauvegarde JSON + calcul du KDA
3. Raccourcis globaux (F9 kill, F10 mort, F11 assist, F12 valider)
4. Fenêtre principale : tableau de session + historique
5. Systray, verrouillage, opacité, mémorisation de la position
6. V2 : graphiques, filtres, champs propres à Hunt (map, mode, extraction, MMR…)
