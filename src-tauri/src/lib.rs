mod models;
mod utils;
mod systems;
mod brain;
mod scanner; 

use std::sync::Mutex;
// use tauri::Manager;
use crate::models::NetchiState;
use crate::utils::load_state;
use crate::systems::{start_biological_clock, start_network_sniffer};

pub struct AppState(pub Mutex<NetchiState>);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Forces the .toml creation during startup
    utils::load_or_create_config();

    tauri::Builder::default()
        .manage(AppState(Mutex::new(load_state())))
        .setup(|app| {
            start_biological_clock(app.handle().clone());
            start_network_sniffer(app.handle().clone());
            Ok(())
        })
        // <- ADDED INVOKE HANDLER THERE!
        .invoke_handler(tauri::generate_handler![
            scanner::trigger_active_scan, 
            scanner::list_saved_scans, 
            scanner::open_scan,
            scanner::check_dependencies])
        .run(tauri::generate_context!())
        .expect("Errore durante l'avvio di Tauri");
}