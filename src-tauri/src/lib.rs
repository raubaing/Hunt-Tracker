use chrono::Local;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf, sync::Mutex};
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, Runtime, State};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

// ---------- Modèle de données (miroir de src/lib/types.ts) ----------

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    id: u32,
    kills: u32,
    deaths: u32,
    assists: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    team_size: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    map: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    extracted: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    bounty: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    mmr_stars: Option<u8>,
    timestamp: String,
}

/// Compteurs de la game en cours (pas encore validée)
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Counters {
    kills: u32,
    deaths: u32,
    assists: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    id: String,
    name: String,
    started_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    ended_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    note: Option<String>,
    #[serde(default)]
    games: Vec<Game>,
    #[serde(default)]
    current: Counters,
}

/// Raccourcis clavier globaux ("" = désactivé)
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct Shortcuts {
    kill: String,
    assist: String,
    death: String,
    validate: String,
    /// Plier / déplier le panneau
    toggle: String,
}

impl Default for Shortcuts {
    fn default() -> Self {
        Self {
            kill: "F8".into(),
            assist: "F9".into(),
            death: "F10".into(),
            validate: "F11".into(),
            toggle: "F7".into(),
        }
    }
}

/// Position de la barre du widget à l'écran, en pixels physiques
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
struct WidgetPosition {
    x: i32,
    y: i32,
}

/// Préférences diverses
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct Preferences {
    /// Proposer les infos de la game (extraction, map…) juste après la validation
    show_details_prompt: bool,
}

impl Default for Preferences {
    fn default() -> Self {
        Self { show_details_prompt: true }
    }
}

/// Infos Hunt d'une game, renseignées après la validation
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct GameDetails {
    #[serde(default)]
    team_size: Option<u8>,
    #[serde(default)]
    map: Option<String>,
    #[serde(default)]
    extracted: Option<bool>,
    #[serde(default)]
    bounty: Option<bool>,
    #[serde(default)]
    mmr_stars: Option<u8>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
struct Config {
    #[serde(default)]
    active_session_id: Option<String>,
    #[serde(default)]
    shortcuts: Shortcuts,
    #[serde(default)]
    widget_position: Option<WidgetPosition>,
    #[serde(default)]
    preferences: Preferences,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Stat {
    Kills,
    Deaths,
    Assists,
}

#[derive(Clone, Copy, Debug)]
enum Action {
    /// Ajoute (+1) ou retire (-1) à un compteur
    Add(Stat, i32),
    Validate,
    /// Rouvre la dernière game validée
    Unvalidate,
    /// Plie / déplie le panneau (géré par l'interface)
    TogglePanel,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Settings {
    shortcuts: Shortcuts,
    preferences: Preferences,
    /// Erreur d'enregistrement des raccourcis (touche déjà prise par une autre appli…)
    shortcut_error: Option<String>,
}

#[derive(Default)]
struct AppState {
    /// Sérialise les lectures/écritures de fichiers (boutons + raccourcis)
    io: Mutex<()>,
    bindings: Mutex<Vec<(Shortcut, Action)>>,
    shortcut_error: Mutex<Option<String>>,
}

// ---------- Fichiers ----------
// %APPDATA%\com.robinnaud.hunttracker\
//   config.json
//   sessions\<id>.json

const EVT_SESSIONS: &str = "sessions-changed";
const EVT_ACTIVE: &str = "active-session-changed";

fn data_dir<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn sessions_dir<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf, String> {
    let dir = data_dir(app)?.join("sessions");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

/// Un id ne contient que chiffres, lettres, '-' et '_' (évite tout chemin hors du dossier).
fn check_id(id: &str) -> Result<(), String> {
    let ok = !id.is_empty()
        && id.len() <= 64
        && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if ok {
        Ok(())
    } else {
        Err(format!("Identifiant de session invalide : {id}"))
    }
}

fn session_path<R: Runtime>(app: &AppHandle<R>, id: &str) -> Result<PathBuf, String> {
    check_id(id)?;
    Ok(sessions_dir(app)?.join(format!("{id}.json")))
}

/// Écriture atomique : fichier temporaire puis renommage.
fn write_json<T: Serialize>(path: &PathBuf, value: &T) -> Result<(), String> {
    let json = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, json).map_err(|e| e.to_string())?;
    fs::rename(&tmp, path).map_err(|e| e.to_string())
}

fn read_config<R: Runtime>(app: &AppHandle<R>) -> Config {
    data_dir(app)
        .ok()
        .and_then(|d| fs::read_to_string(d.join("config.json")).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn write_config<R: Runtime>(app: &AppHandle<R>, config: &Config) -> Result<(), String> {
    write_json(&data_dir(app)?.join("config.json"), config)
}

fn read_session<R: Runtime>(app: &AppHandle<R>, id: &str) -> Result<Option<Session>, String> {
    let path = session_path(app, id)?;
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&raw).map(Some).map_err(|e| e.to_string())
}

fn read_all_sessions<R: Runtime>(app: &AppHandle<R>) -> Result<Vec<Session>, String> {
    let mut sessions = Vec::new();
    for entry in fs::read_dir(sessions_dir(app)?).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        // Un fichier illisible est ignoré plutôt que de bloquer toute la liste
        match fs::read_to_string(&path)
            .map_err(|e| e.to_string())
            .and_then(|raw| serde_json::from_str::<Session>(&raw).map_err(|e| e.to_string()))
        {
            Ok(s) => sessions.push(s),
            Err(e) => eprintln!("Session ignorée {}: {e}", path.display()),
        }
    }
    // Plus récente en premier
    sessions.sort_by(|a, b| b.started_at.cmp(&a.started_at));
    Ok(sessions)
}

fn set_active_id<R: Runtime>(app: &AppHandle<R>, id: Option<String>) -> Result<(), String> {
    let mut config = read_config(app);
    config.active_session_id = id;
    write_config(app, &config)?;
    let _ = app.emit(EVT_ACTIVE, ());
    Ok(())
}

/// Crée la session du jour ("Session du dd/MM/YY", suffixée (2), (3)… si besoin)
/// et en fait la session en cours. L'appelant doit tenir le verrou `io`.
fn create_session_locked<R: Runtime>(app: &AppHandle<R>) -> Result<Session, String> {
    let now = Local::now();
    let base = now.format("Session du %d/%m/%y").to_string();
    let same_day = read_all_sessions(app)?
        .iter()
        .filter(|s| s.name == base || s.name.starts_with(&format!("{base} (")))
        .count();
    let name = if same_day == 0 { base } else { format!("{base} ({})", same_day + 1) };

    let mut id = now.format("%Y-%m-%d_%H%M%S").to_string();
    let mut n = 2;
    while session_path(app, &id)?.exists() {
        id = format!("{}-{n}", now.format("%Y-%m-%d_%H%M%S"));
        n += 1;
    }

    let session = Session {
        id,
        name,
        started_at: now.format("%Y-%m-%dT%H:%M:%S").to_string(),
        ended_at: None,
        note: None,
        games: Vec::new(),
        current: Counters::default(),
    };
    write_json(&session_path(app, &session.id)?, &session)?;
    set_active_id(app, Some(session.id.clone()))?;
    let _ = app.emit(EVT_SESSIONS, ());
    Ok(session)
}

/// Modifie la session en cours (créée automatiquement s'il n'y en a pas), sauvegarde et notifie.
fn update_active<R: Runtime>(
    app: &AppHandle<R>,
    state: &AppState,
    change: impl FnOnce(&mut Session),
) -> Result<Session, String> {
    let _guard = state.io.lock().map_err(|e| e.to_string())?;
    let existing = match read_config(app).active_session_id {
        Some(id) => read_session(app, &id)?,
        None => None,
    };
    let mut session = match existing {
        Some(s) => s,
        None => create_session_locked(app)?,
    };
    change(&mut session);
    write_json(&session_path(app, &session.id)?, &session)?;
    let _ = app.emit(EVT_ACTIVE, ());
    let _ = app.emit(EVT_SESSIONS, ());
    Ok(session)
}

fn add_stat_inner<R: Runtime>(app: &AppHandle<R>, state: &AppState, stat: Stat, delta: i32) -> Result<Session, String> {
    update_active(app, state, |s| {
        let field = match stat {
            Stat::Kills => &mut s.current.kills,
            Stat::Deaths => &mut s.current.deaths,
            Stat::Assists => &mut s.current.assists,
        };
        *field = (*field as i64 + delta as i64).clamp(0, u32::MAX as i64) as u32;
    })
}

fn validate_game_inner<R: Runtime>(app: &AppHandle<R>, state: &AppState) -> Result<Session, String> {
    update_active(app, state, |s| {
        let id = s.games.iter().map(|g| g.id).max().unwrap_or(0) + 1;
        let c = std::mem::take(&mut s.current);
        // Le format (solo/duo/trio) et les étoiles changent rarement : on reprend ceux de la game précédente
        let previous = s.games.last();
        s.games.push(Game {
            id,
            kills: c.kills,
            deaths: c.deaths,
            assists: c.assists,
            mode: None,
            team_size: previous.and_then(|g| g.team_size),
            map: None,
            extracted: None,
            bounty: None,
            mmr_stars: previous.and_then(|g| g.mmr_stars),
            timestamp: Local::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
        });
    })
}

/// Annule la dernière validation : la dernière game redevient la game en cours.
/// Si la game suivante avait déjà commencé, ses compteurs s'ajoutent à ceux de la game rouverte.
fn unvalidate_game_inner<R: Runtime>(app: &AppHandle<R>, state: &AppState) -> Result<Session, String> {
    update_active(app, state, |s| {
        if let Some(last) = s.games.pop() {
            s.current.kills += last.kills;
            s.current.deaths += last.deaths;
            s.current.assists += last.assists;
        }
    })
}

/// Modifie n'importe quelle session (pas forcément celle en cours), sauvegarde et notifie.
fn update_session<R: Runtime>(
    app: &AppHandle<R>,
    state: &AppState,
    id: &str,
    change: impl FnOnce(&mut Session) -> Result<(), String>,
) -> Result<Session, String> {
    let _guard = state.io.lock().map_err(|e| e.to_string())?;
    let mut session = read_session(app, id)?.ok_or_else(|| format!("Session introuvable : {id}"))?;
    change(&mut session)?;
    write_json(&session_path(app, &session.id)?, &session)?;
    let _ = app.emit(EVT_SESSIONS, ());
    if read_config(app).active_session_id.as_deref() == Some(id) {
        let _ = app.emit(EVT_ACTIVE, ());
    }
    Ok(session)
}

// ---------- Raccourcis globaux ----------

/// (Ré)enregistre les raccourcis auprès de Windows.
fn apply_shortcuts<R: Runtime>(app: &AppHandle<R>, state: &AppState, shortcuts: &Shortcuts) -> Result<(), String> {
    let gs = app.global_shortcut();
    let _ = gs.unregister_all();
    state.bindings.lock().map_err(|e| e.to_string())?.clear();

    let wanted = [
        ("Kill", &shortcuts.kill, Action::Add(Stat::Kills, 1)),
        ("Assist", &shortcuts.assist, Action::Add(Stat::Assists, 1)),
        ("Mort", &shortcuts.death, Action::Add(Stat::Deaths, 1)),
        ("Valider la game", &shortcuts.validate, Action::Validate),
        ("Plier / déplier", &shortcuts.toggle, Action::TogglePanel),
    ];

    let mut bindings: Vec<(Shortcut, Action)> = Vec::new();
    let mut labels: Vec<String> = Vec::new();
    for (label, accel, action) in wanted {
        if accel.trim().is_empty() {
            continue;
        }
        let shortcut: Shortcut = accel
            .parse()
            .map_err(|e| format!("{label} : raccourci « {accel} » invalide ({e})"))?;
        if let Some(i) = bindings.iter().position(|(s, _)| *s == shortcut) {
            return Err(format!("{label} : « {accel} » est déjà utilisé par « {} »", labels[i]));
        }
        bindings.push((shortcut, action));
        labels.push(label.to_string());
    }

    // Ctrl + le raccourci fait l'inverse : retirer 1 à un compteur, ou rouvrir la dernière game.
    // (Rien si le raccourci contient déjà Ctrl.)
    let primaries = bindings.clone();
    for (i, (shortcut, action)) in primaries.iter().enumerate() {
        if shortcut.mods.contains(Modifiers::CONTROL) {
            continue;
        }
        let (inverse, what) = match action {
            Action::Add(stat, _) => (Action::Add(*stat, -1), "retirer 1"),
            Action::Validate => (Action::Unvalidate, "rouvrir la dernière game"),
            Action::Unvalidate | Action::TogglePanel => continue,
        };
        let ctrl = Shortcut::new(Some(shortcut.mods | Modifiers::CONTROL), shortcut.key);
        if let Some(j) = bindings.iter().position(|(s, _)| *s == ctrl) {
            return Err(format!(
                "{} : le raccourci pour {what} (Ctrl + la même touche) est déjà utilisé par « {} »",
                labels[i], labels[j]
            ));
        }
        bindings.push((ctrl, inverse));
        labels.push(format!("{} (Ctrl)", labels[i]));
    }

    for (shortcut, _) in &bindings {
        if let Err(e) = gs.register(*shortcut) {
            let _ = gs.unregister_all();
            return Err(format!(
                "Impossible d'enregistrer « {} » : la touche est peut-être déjà utilisée par une autre application ({e})",
                shortcut.into_string()
            ));
        }
    }
    *state.bindings.lock().map_err(|e| e.to_string())? = bindings;
    Ok(())
}

fn on_shortcut<R: Runtime>(app: &AppHandle<R>, shortcut: &Shortcut) {
    let state = app.state::<AppState>();
    let action = state
        .bindings
        .lock()
        .ok()
        .and_then(|b| b.iter().find(|(s, _)| s == shortcut).map(|(_, a)| *a));
    let result = match action {
        Some(Action::Add(stat, delta)) => add_stat_inner(app, &state, stat, delta).map(|_| ()),
        Some(Action::Validate) => validate_game_inner(app, &state).map(|_| ()),
        Some(Action::Unvalidate) => unvalidate_game_inner(app, &state).map(|_| ()),
        Some(Action::TogglePanel) => app.emit("toggle-panel", ()).map_err(|e| e.to_string()),
        None => Ok(()),
    };
    if let Err(e) = result {
        eprintln!("Raccourci : {e}");
    }
}

// ---------- Commandes appelées depuis l'interface ----------

#[tauri::command]
fn list_sessions(app: AppHandle) -> Result<Vec<Session>, String> {
    read_all_sessions(&app)
}

#[tauri::command]
fn create_session(app: AppHandle, state: State<'_, AppState>) -> Result<Session, String> {
    let _guard = state.io.lock().map_err(|e| e.to_string())?;
    create_session_locked(&app)
}

#[tauri::command]
fn save_session(app: AppHandle, state: State<'_, AppState>, session: Session) -> Result<(), String> {
    let _guard = state.io.lock().map_err(|e| e.to_string())?;
    write_json(&session_path(&app, &session.id)?, &session)?;
    let _ = app.emit(EVT_SESSIONS, ());
    if read_config(&app).active_session_id.as_deref() == Some(session.id.as_str()) {
        let _ = app.emit(EVT_ACTIVE, ());
    }
    Ok(())
}

#[tauri::command]
fn delete_session(app: AppHandle, state: State<'_, AppState>, id: String) -> Result<(), String> {
    let _guard = state.io.lock().map_err(|e| e.to_string())?;
    let path = session_path(&app, &id)?;
    if path.exists() {
        fs::remove_file(&path).map_err(|e| e.to_string())?;
    }
    if read_config(&app).active_session_id.as_deref() == Some(id.as_str()) {
        set_active_id(&app, None)?;
    }
    let _ = app.emit(EVT_SESSIONS, ());
    Ok(())
}

#[tauri::command]
fn get_active_session(app: AppHandle) -> Result<Option<Session>, String> {
    match read_config(&app).active_session_id {
        Some(id) => read_session(&app, &id),
        None => Ok(None),
    }
}

#[tauri::command]
fn set_active_session(app: AppHandle, state: State<'_, AppState>, id: Option<String>) -> Result<(), String> {
    let _guard = state.io.lock().map_err(|e| e.to_string())?;
    if let Some(id) = &id {
        if !session_path(&app, id)?.exists() {
            return Err(format!("Session introuvable : {id}"));
        }
    }
    set_active_id(&app, id)
}

#[tauri::command]
fn add_stat(app: AppHandle, state: State<'_, AppState>, stat: Stat, delta: i32) -> Result<Session, String> {
    add_stat_inner(&app, &state, stat, delta)
}

#[tauri::command]
fn validate_game(app: AppHandle, state: State<'_, AppState>) -> Result<Session, String> {
    validate_game_inner(&app, &state)
}

#[tauri::command]
fn unvalidate_game(app: AppHandle, state: State<'_, AppState>) -> Result<Session, String> {
    unvalidate_game_inner(&app, &state)
}

/// Corrige les compteurs d'une game déjà validée
#[tauri::command]
fn update_game(
    app: AppHandle,
    state: State<'_, AppState>,
    session_id: String,
    game_id: u32,
    kills: u32,
    deaths: u32,
    assists: u32,
) -> Result<Session, String> {
    update_session(&app, &state, &session_id, |s| {
        let game = s
            .games
            .iter_mut()
            .find(|g| g.id == game_id)
            .ok_or_else(|| format!("Game introuvable : {game_id}"))?;
        game.kills = kills;
        game.deaths = deaths;
        game.assists = assists;
        Ok(())
    })
}

/// Renseigne les infos Hunt d'une game (remplace les valeurs existantes)
#[tauri::command]
fn set_game_details(
    app: AppHandle,
    state: State<'_, AppState>,
    session_id: String,
    game_id: u32,
    details: GameDetails,
) -> Result<Session, String> {
    if details.team_size.is_some_and(|t| !(1..=3).contains(&t)) {
        return Err("Format invalide (1 à 3 joueurs)".into());
    }
    if details.mmr_stars.is_some_and(|t| !(1..=6).contains(&t)) {
        return Err("Étoiles invalides (1 à 6)".into());
    }
    if details.map.as_ref().is_some_and(|m| m.len() > 60) {
        return Err("Nom de map trop long".into());
    }
    update_session(&app, &state, &session_id, |s| {
        let game = s
            .games
            .iter_mut()
            .find(|g| g.id == game_id)
            .ok_or_else(|| format!("Game introuvable : {game_id}"))?;
        game.team_size = details.team_size;
        game.map = details.map.filter(|m| !m.trim().is_empty());
        game.extracted = details.extracted;
        game.bounty = details.bounty;
        game.mmr_stars = details.mmr_stars;
        Ok(())
    })
}

#[tauri::command]
fn set_preferences(app: AppHandle, state: State<'_, AppState>, preferences: Preferences) -> Result<(), String> {
    let _guard = state.io.lock().map_err(|e| e.to_string())?;
    let mut config = read_config(&app);
    config.preferences = preferences;
    write_config(&app, &config)?;
    let _ = app.emit("preferences-changed", ());
    Ok(())
}

#[tauri::command]
fn delete_game(app: AppHandle, state: State<'_, AppState>, session_id: String, game_id: u32) -> Result<Session, String> {
    update_session(&app, &state, &session_id, |s| {
        s.games.retain(|g| g.id != game_id);
        Ok(())
    })
}

#[tauri::command]
fn get_settings(app: AppHandle, state: State<'_, AppState>) -> Settings {
    Settings {
        shortcuts: read_config(&app).shortcuts,
        preferences: read_config(&app).preferences,
        shortcut_error: state.shortcut_error.lock().ok().and_then(|e| e.clone()),
    }
}

/// Enregistre de nouveaux raccourcis. En cas d'échec, les anciens sont rétablis.
#[tauri::command]
fn set_shortcuts(app: AppHandle, state: State<'_, AppState>, shortcuts: Shortcuts) -> Result<(), String> {
    let mut config = read_config(&app);
    if let Err(e) = apply_shortcuts(&app, &state, &shortcuts) {
        let _ = apply_shortcuts(&app, &state, &config.shortcuts);
        return Err(e);
    }
    config.shortcuts = shortcuts;
    write_config(&app, &config)?;
    if let Ok(mut err) = state.shortcut_error.lock() {
        *err = None;
    }
    Ok(())
}

/// Désactive temporairement les raccourcis (pendant la saisie d'une touche dans les options).
#[tauri::command]
fn pause_shortcuts(app: AppHandle, state: State<'_, AppState>, paused: bool) -> Result<(), String> {
    if paused {
        let _ = app.global_shortcut().unregister_all();
        state.bindings.lock().map_err(|e| e.to_string())?.clear();
        Ok(())
    } else {
        apply_shortcuts(&app, &state, &read_config(&app).shortcuts)
    }
}

/// Mémorise la position de la barre du widget (appelé après un déplacement).
#[tauri::command]
fn save_widget_position(app: AppHandle, state: State<'_, AppState>, x: i32, y: i32) -> Result<(), String> {
    let _guard = state.io.lock().map_err(|e| e.to_string())?;
    let mut config = read_config(&app);
    config.widget_position = Some(WidgetPosition { x, y });
    write_config(&app, &config)
}

/// Replace le widget à sa dernière position, si elle est toujours sur un écran branché,
/// puis l'affiche (il est masqué au lancement pour éviter qu'il « saute »).
fn restore_widget_position<R: Runtime>(app: &AppHandle<R>) {
    let Some(window) = app.get_webview_window("widget") else { return };
    if let Some(pos) = read_config(app).widget_position {
        let monitors = window.available_monitors().unwrap_or_default();
        // On vérifie qu'un petit carré en haut à gauche de la barre est visible sur un écran
        let visible = monitors.iter().any(|m| {
            let (mx, my) = (m.position().x, m.position().y);
            let (mw, mh) = (m.size().width as i32, m.size().height as i32);
            pos.x + 40 > mx && pos.x + 40 < mx + mw && pos.y + 20 > my && pos.y + 20 < my + mh
        });
        if visible {
            let _ = window.set_position(PhysicalPosition::new(pos.x, pos.y));
        }
    }
    let _ = window.show();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        on_shortcut(app, shortcut);
                    }
                })
                .build(),
        )
        .setup(|app| {
            let handle = app.handle();
            let state = handle.state::<AppState>();
            if let Err(e) = apply_shortcuts(handle, &state, &read_config(handle).shortcuts) {
                eprintln!("Raccourcis : {e}");
                if let Ok(mut err) = state.shortcut_error.lock() {
                    *err = Some(e);
                }
            }
            restore_widget_position(handle);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_sessions,
            create_session,
            save_session,
            delete_session,
            get_active_session,
            set_active_session,
            add_stat,
            validate_game,
            unvalidate_game,
            update_game,
            delete_game,
            set_game_details,
            set_preferences,
            get_settings,
            set_shortcuts,
            pause_shortcuts,
            save_widget_position
        ])
        .run(tauri::generate_context!())
        .expect("erreur au lancement de Hunt Tracker");
}

