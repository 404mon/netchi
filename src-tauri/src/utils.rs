use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use chrono::Local;
use crate::models::{NetchiState, AppConfig};

// --- DIRECTORY TRUTH SOURCE ---
pub fn get_app_dir() -> PathBuf {
    if let Some(base) = directories::BaseDirs::new() {
        // Linux Folder
        let path = base.data_dir().join("netchi");
        let _ = fs::create_dir_all(&path);
        return path;
    }
    PathBuf::from(".")
}

pub fn load_or_create_config() -> AppConfig {
    let app_dir = get_app_dir();
    let config_path = app_dir.join("config.toml");

    if config_path.exists() {
        if let Ok(config_str) = fs::read_to_string(&config_path) {
            if let Ok(config) = toml::from_str(&config_str) {
                return config;
            }
        }
    }

    let default_config = AppConfig::default();
    
    // RUST MAGIC: Reads the file during the compilation injecting it as a static string
    let toml_string = include_str!("default_config.toml");

    let _ = fs::write(&config_path, toml_string);
    default_config
}

pub fn write_log(message: &str) {
    let path = get_app_dir().join("netchi_sniff.log");
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    // Removed [NETCHI] PREFIX, messages have their tag
    let log_entry = format!("[{}] {}\n", timestamp, message);

    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = file.write_all(log_entry.as_bytes());
    }
    println!("{}", log_entry.trim());
}

pub fn load_state() -> NetchiState {
    let path = get_app_dir().join("netchi_memory.json");
    if let Ok(data) = fs::read_to_string(path) {
        if let Ok(state) = serde_json::from_str(&data) {
            return state;
        }
    }
    NetchiState::default()
}

pub fn save_state(state: &NetchiState) {
    let path = get_app_dir().join("netchi_memory.json");
    if let Ok(json) = serde_json::to_string_pretty(state) {
        let _ = fs::write(path, json);
    }
}