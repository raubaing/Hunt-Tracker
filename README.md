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

## Tests

- `cargo test --manifest-path src-tauri/Cargo.toml` : tests de la logique de données (compteurs,
  sessions, corrections, coéquipiers, export CSV, compatibilité des anciens fichiers), dans un dossier
  temporaire (les vraies données ne sont jamais touchées). Nécessite `npm run build` au préalable.
- `npm run check` : vérification des types de l'interface.
- Le workflow `checks.yml` lance les deux à chaque push sur GitHub (onglet Actions, « Vérifications »).

## Compiler via GitHub (sans rien installer)

- **Build de test** (workflow « Build ») : à chaque push sur `main`. Onglet **Actions** > dernier run >
  **Artifacts** > `HuntTracker-windows` (installeur `*-setup.exe` et exe portable).
- **Publier une version** (workflow « Release ») : mettre à jour le numéro de version (tauri.conf.json,
  Cargo.toml, package.json), pousser, puis Actions > **Release** > **Run workflow** (avec les nouveautés
  en option). La Release GitHub `vX.Y.Z` est créée avec l'installeur signé et `latest.json`.

## Mises à jour automatiques

- Au lancement (puis toutes les 6 h), l'appli lit
  `https://github.com/raubaing/Hunt-Tracker/releases/latest/download/latest.json`. Si une version plus
  récente existe : pastille **MAJ** dans le widget, installation depuis Options > Mises à jour.
- Le dépôt doit être **public** pour que l'appli puisse lire les Releases.
- Les mises à jour sont signées : clé publique dans `tauri.conf.json`, clé privée et mot de passe dans les
  secrets GitHub `TAURI_SIGNING_PRIVATE_KEY` et `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`
  (jamais dans le dépôt). Sans ces secrets, les builds échouent.

## Idées pour la suite

- Opacité du widget et clic traversant, mode ultra-compact
- Lancement au démarrage de Windows
- Export / import de sessions (JSON, CSV)
