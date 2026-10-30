# Hunt Tracker

Petit widget flottant pour suivre ses sessions de **Hunt: Showdown 1896** (kills, morts, assists, KDA).

> Fonctionnalités : sessions, compteurs K / A / D (boutons + raccourcis globaux), infos Hunt par game,
> correction des games, notes, statistiques et graphiques, widget dépliable et animé.

## Stack

- **Tauri 2** (Rust) : fenêtres natives, exécutable léger
- **Svelte 5 + TypeScript + Vite** : interface
- **JSON** : une session = un fichier, dans `%APPDATA%\com.robinnaud.hunttracker\sessions\`

## Structure

```
src/
  lib/          types.ts (modèle), stats.ts (KDA), storage.ts (JSON)
  widget/       fenêtre unique : barre du widget (compteurs) + panneau dépliable (⤢)
  main/         contenu du panneau : onglets Sessions et Options
src-tauri/
  tauri.conf.json         une fenêtre "widget" : sans bordure, transparente, au premier plan
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

## Idées pour la suite

- Icône dans la barre des tâches système (systray), verrouillage et opacité du widget
- Lancement au démarrage de Windows
- Mises à jour automatiques (plugin updater de Tauri)
- Export / import de sessions (JSON, CSV)
