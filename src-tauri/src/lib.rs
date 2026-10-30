#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Plugins à ajouter plus tard : global-shortcut, window-state, fs, tray…
        .run(tauri::generate_context!())
        .expect("erreur au lancement de Hunt Tracker");
}
