//! Tests automatiques de la logique de données (cargo test).
//! Chaque test travaille dans son propre dossier temporaire : les vraies données
//! (%APPDATA%) ne sont jamais touchées.

use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use tauri::test::{mock_app, MockRuntime};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Application simulée + dossier de données temporaire (supprimé à la fin du test)
struct TestApp {
    app: tauri::App<MockRuntime>,
    dir: PathBuf,
}

impl TestApp {
    fn new() -> Self {
        let dir = std::env::temp_dir().join(format!(
            "hunt-tracker-test-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        let _ = fs::remove_dir_all(&dir);
        let app = mock_app();
        app.manage(AppState { data_override: Some(dir.clone()), ..Default::default() });
        Self { app, dir }
    }
    fn h(&self) -> &AppHandle<MockRuntime> {
        self.app.handle()
    }
    fn st(&self) -> tauri::State<'_, AppState> {
        self.app.state::<AppState>()
    }
}

impl Drop for TestApp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

fn game(id: u32, kills: u32, deaths: u32, assists: u32) -> Game {
    serde_json::from_value(serde_json::json!({
        "id": id, "kills": kills, "deaths": deaths, "assists": assists, "timestamp": "2026-10-30T21:30:00"
    }))
    .unwrap()
}

fn session(json: serde_json::Value) -> Session {
    serde_json::from_value(json).unwrap()
}

// ---------- Compteurs, validation, game rouverte ----------

#[test]
fn first_kill_creates_a_session_and_counts() {
    let t = TestApp::new();
    let s = add_stat_inner(t.h(), &t.st(), Stat::Kills, 1).unwrap();
    assert!(s.name.starts_with("Session du "));
    assert_eq!(s.current.kills, 1);
    assert_eq!(read_config(t.h()).active_session_id.as_deref(), Some(s.id.as_str()));
}

#[test]
fn counters_never_go_below_zero() {
    let t = TestApp::new();
    let s = add_stat_inner(t.h(), &t.st(), Stat::Deaths, -1).unwrap();
    assert_eq!(s.current.deaths, 0);
}

#[test]
fn validate_then_reopen_last_game() {
    let t = TestApp::new();
    add_stat_inner(t.h(), &t.st(), Stat::Kills, 1).unwrap();
    add_stat_inner(t.h(), &t.st(), Stat::Kills, 1).unwrap();
    let s = validate_game_inner(t.h(), &t.st()).unwrap();
    assert_eq!(s.games.len(), 1);
    assert_eq!(s.games[0].kills, 2);
    assert_eq!(s.current.kills, 0);

    // La game suivante a commencé, puis on rouvre la précédente : les compteurs s'additionnent
    add_stat_inner(t.h(), &t.st(), Stat::Assists, 1).unwrap();
    let s = unvalidate_game_inner(t.h(), &t.st()).unwrap();
    assert!(s.games.is_empty());
    assert_eq!((s.current.kills, s.current.assists), (2, 1));
}

#[test]
fn format_and_stars_are_carried_over() {
    let t = TestApp::new();
    let s = validate_game_inner(t.h(), &t.st()).unwrap();
    update_session(t.h(), &t.st(), &s.id, |s| {
        s.games[0].team_size = Some(3);
        s.games[0].mmr_stars = Some(4);
        s.games[0].map = Some("DeSalle".into());
        Ok(())
    })
    .unwrap();
    let s = validate_game_inner(t.h(), &t.st()).unwrap();
    assert_eq!(s.games[1].team_size, Some(3));
    assert_eq!(s.games[1].mmr_stars, Some(4));
    assert_eq!(s.games[1].map, None, "la map change à chaque game");
}

// ---------- Sessions ----------

#[test]
fn second_session_of_the_day_gets_a_suffix() {
    let t = TestApp::new();
    let st = t.st();
    let _g = st.io.lock().unwrap();
    let a = create_session_locked(t.h()).unwrap();
    let b = create_session_locked(t.h()).unwrap();
    assert_ne!(a.id, b.id);
    assert_eq!(b.name, format!("{} (2)", a.name));
    assert_eq!(read_all_sessions(t.h()).unwrap().len(), 2);
}

#[test]
fn stale_session_detection() {
    let now = NaiveDateTime::parse_from_str("2026-10-31T20:00:00", TIME_FORMAT).unwrap();
    let mut s = session(serde_json::json!({ "id": "a", "name": "x", "startedAt": "2026-10-30T21:00:00" }));
    assert!(is_stale(&s, now), "aucune activité depuis la veille");
    s.last_activity = Some("2026-10-31T16:00:00".into());
    assert!(!is_stale(&s, now), "activité il y a 4 h");
    s.last_activity = None;
    s.games.push(serde_json::from_value(serde_json::json!({
        "id": 1, "kills": 0, "deaths": 0, "assists": 0, "timestamp": "2026-10-31T19:00:00"
    })).unwrap());
    assert!(!is_stale(&s, now), "dernière game il y a 1 h");
}

#[test]
fn stale_active_session_is_replaced_on_next_kill() {
    let t = TestApp::new();
    let first = add_stat_inner(t.h(), &t.st(), Stat::Kills, 1).unwrap();
    // On vieillit artificiellement la session
    update_session(t.h(), &t.st(), &first.id, |s| {
        s.last_activity = Some("2020-01-01T12:00:00".into());
        Ok(())
    })
    .unwrap();
    let second = add_stat_inner(t.h(), &t.st(), Stat::Kills, 1).unwrap();
    assert_ne!(first.id, second.id);
    assert_eq!(second.current.kills, 1);
}

#[test]
fn invalid_ids_are_rejected() {
    assert!(check_id("2026-10-30_210500").is_ok());
    assert!(check_id("../config").is_err());
    assert!(check_id("a/b").is_err());
    assert!(check_id("").is_err());
}

// ---------- Correction des games ----------

#[test]
fn insert_game_restores_position_and_refuses_duplicates() {
    let mut s = session(serde_json::json!({ "id": "a", "name": "x", "startedAt": "2026-10-30T21:00:00" }));
    s.games = vec![game(1, 1, 0, 0), game(3, 3, 0, 0)];
    insert_game(&mut s, game(2, 2, 0, 0), 1).unwrap();
    assert_eq!(s.games.iter().map(|g| g.id).collect::<Vec<_>>(), vec![1, 2, 3]);
    assert!(insert_game(&mut s, game(2, 0, 0, 0), 0).is_err());
    insert_game(&mut s, game(9, 0, 0, 0), 99).unwrap();
    assert_eq!(s.games.last().unwrap().id, 9, "position bornée à la fin");
}

// ---------- Coéquipiers ----------

#[test]
fn teammates_are_normalized() {
    let out = normalize_teammates(vec!["  Max ".into(), "max".into(), "Léo  le   Grand".into(), "".into()]).unwrap();
    assert_eq!(out, vec!["Max".to_string(), "Léo le Grand".to_string()]);
    assert!(normalize_teammates((0..11).map(|i| format!("J{i}")).collect()).is_err());
    assert!(normalize_teammates(vec!["x".repeat(31)]).is_err());
}

// ---------- Export CSV ----------

#[test]
fn csv_export_format() {
    let s = session(serde_json::json!({
        "id": "a", "name": "Session du 30/10/26", "startedAt": "2026-10-30T21:05:00",
        "note": "Trio; \"fun\"", "teammates": ["Max", "Léo"],
        "games": [{ "id": 1, "kills": 3, "deaths": 2, "assists": 1, "map": "Mammon's Gulch",
                    "teamSize": 3, "extracted": true, "mmrStars": 4, "timestamp": "2026-10-30T21:30:00" }]
    }));
    let csv = sessions_to_csv(&[s]);
    assert!(csv.starts_with('\u{feff}'), "BOM UTF-8 pour Excel");
    let lines: Vec<&str> = csv.trim_start_matches('\u{feff}').split("\r\n").collect();
    assert!(lines[0].starts_with("Session;Date;Game;"));
    assert_eq!(
        lines[1],
        "Session du 30/10/26;30/10/2026;1;21:30;3;1;2;2,00;Trio;Mammon's Gulch;Oui;;4;Max, Léo;\"Trio; \"\"fun\"\"\""
    );
}

// ---------- Compatibilité des anciens fichiers ----------

#[test]
fn old_files_still_load() {
    // Session créée par une ancienne version (sans current, teammates, lastActivity…)
    let s = session(serde_json::json!({
        "id": "2026-10-30_210500", "name": "Session du 30/10/26", "startedAt": "2026-10-30T21:05:00",
        "games": [{ "id": 1, "kills": 1, "deaths": 0, "assists": 0, "timestamp": "2026-10-30T21:30:00" }]
    }));
    assert_eq!(s.current.kills, 0);
    assert!(s.teammates.is_empty());

    // Ancienne config sans le raccourci F7 ni les préférences
    let c: Config = serde_json::from_str(
        r#"{ "activeSessionId": "x", "shortcuts": { "kill": "F1", "assist": "F2", "death": "F3", "validate": "F4" } }"#,
    )
    .unwrap();
    assert_eq!(c.shortcuts.kill, "F1");
    assert_eq!(c.shortcuts.toggle, "F7");
    assert!(c.preferences.show_details_prompt);
    assert!(!c.preferences.locked);
}
