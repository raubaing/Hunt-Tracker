// Évite l'ouverture d'une console Windows en release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    hunt_tracker_lib::run()
}
