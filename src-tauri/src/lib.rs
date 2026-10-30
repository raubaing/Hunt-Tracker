use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use tauri::{AppHandle, Emitter, Manager, WindowEvent};

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
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
struct Config {
    #[serde(default)]
    active_session_id: Option<String>,
}

// ---------- Fichiers ----------
// %APPDATA%\com.robinnaud.hunttracker\
//   config.json
//   sessions\<id>.json

const EVT_SESSIONS: &str = "sessions-changed";
const EVT_ACTIVE: &str = "active-session-changed";

fn data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn sessions_dir(app: &AppHandle) -> Result<PathBuf, String> {
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

fn session_path(app: &AppHandle, id: &str) -> Result<PathBuf, String> {
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

fn read_config(app: &AppHandle) -> Config {
    data_dir(app)
        .ok()
        .and_then(|d| fs::read_to_string(d.join("config.json")).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn write_config(app: &AppHandle, config: &Config) -> Result<(), String> {
    write_json(&data_dir(app)?.join("config.json"), config)
}

fn read_session(app: &AppHandle, id: &str) -> Result<Option<Session>, String> {
    let path = session_path(app, id)?;
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&raw).map(Some).map_err(|e| e.to_string())
}

// ---------- Commandes appelées depuis l'interface ----------

#[tauri::command]
fn list_sessions(app: AppHandle) -> Result<Vec<Session>, String> {
    let mut sessions = Vec::new();
    for entry in fs::read_dir(sessions_dir(&app)?).map_err(|e| e.to_string())? {
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

#[tauri::command]
fn save_session(app: AppHandle, session: Session) -> Result<(), String> {
    write_json(&session_path(&app, &session.id)?, &session)?;
    let _ = app.emit(EVT_SESSIONS, ());
    if read_config(&app).active_session_id.as_deref() == Some(session.id.as_str()) {
        let _ = app.emit(EVT_ACTIVE, ());
    }
    Ok(())
}

#[tauri::command]
fn delete_session(app: AppHandle, id: String) -> Result<(), String> {
    let path = session_path(&app, &id)?;
    if path.exists() {
        fs::remove_file(&path).map_err(|e| e.to_string())?;
    }
    let mut config = read_config(&app);
    if config.active_session_id.as_deref() == Some(id.as_str()) {
        config.active_session_id = None;
        write_config(&app, &config)?;
        let _ = app.emit(EVT_ACTIVE, ());
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
fn set_active_session(app: AppHandle, id: Option<String>) -> Result<(), String> {
    if let Some(id) = &id {
        if !session_path(&app, id)?.exists() {
            return Err(format!("Session introuvable : {id}"));
        }
    }
    let mut config = read_config(&app);
    config.active_session_id = id;
    write_config(&app, &config)?;
    let _ = app.emit(EVT_ACTIVE, ());
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            list_sessions,
            save_session,
            delete_session,
            get_active_session,
            set_active_session
        ])
        // Fermer la fenêtre principale la cache seulement : le widget reste actif
        .on_window_event(|window, event| {
            if window.label() == "main" {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("erreur au lancement de Hunt Tracker");
}
