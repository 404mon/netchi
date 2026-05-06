use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub enum LifeStage { Egg, Baby, Adult }

#[derive(Clone, Serialize, Deserialize)]
pub struct NetworkNode {
    pub mac: String,
    pub vendor: String,
    pub ip: Option<String>,
    pub open_ports: Vec<u16>,
    pub first_seen: u64,
    pub last_seen: u64,
}

// --- TOML CONFIG ---
#[derive(Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub allow_active_scans: bool,
    pub target_subnet: String, // "auto" to dynamically detect the NET
    pub scan_cooldown_minutes: u32, // IA "patience"
    pub noise_threshold: u32, // Noise tolerance threshold
    pub nmap_arguments: Vec<String>, // NMAP detail level
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            allow_active_scans: true,
            target_subnet: "auto".to_string(),
            scan_cooldown_minutes: 5,
            noise_threshold: 800, // Di default tollera 800 pacchetti prima di sentirsi "affollato"
            nmap_arguments: vec!["-F".to_string(), "-T4".to_string()],
        }
    }
}

// --- NETCHI DNA ---
#[derive(Clone, Serialize, Deserialize)]
pub struct NetchiState {
    pub name: String,             
    pub skin: String,
    pub hunger: u8,
    // No more energy
    pub happiness: u8,
    pub stage: LifeStage,
    pub age: u32,
    pub networks_eaten: u32,
    pub netdex: HashMap<String, NetworkNode>, 
    pub current_location: String,
    pub packet_buffer: u32,
    pub current_action: String,

    // REINFORCEMENT LEARNING
    pub q_table: HashMap<String, HashMap<String, f32>>, 
    pub exploration_rate: f32, 
    pub last_state: String,    
    pub last_action: String,   
    pub last_hosts_count: u32,
    pub last_scan_age: u32,
}

impl Default for NetchiState {
    fn default() -> Self {
        Self {
            name: "Subject-0".to_string(),
            skin: "base".to_string(),
            hunger: 100,
            happiness: 100,
            stage: LifeStage::Egg,
            age: 0,
            networks_eaten: 0,
            netdex: HashMap::new(),
            current_location: "Unknown Sector".to_string(),
            packet_buffer: 0,
            current_action: "idle".to_string(),
            q_table: HashMap::new(),
            exploration_rate: 1.0, 
            last_state: String::new(),
            last_action: String::new(),
            last_hosts_count: 0,
            last_scan_age: 0,
        }
    }
}

// LIGHT PAYLOAD fot the UI
#[derive(Clone, Serialize)]
pub struct NetchiUIState {
    pub name: String,
    pub skin: String,
    pub hunger: u8,
    pub happiness: u8,
    pub stage: LifeStage,
    pub age: u32,
    pub networks_eaten: u32,
    pub current_location: String,
    pub current_action: String,
}

impl NetchiState {
    pub fn to_ui_state(&self) -> NetchiUIState {
        NetchiUIState {
            name: self.name.clone(),
            skin: self.skin.clone(),
            hunger: self.hunger,
            happiness: self.happiness,
            stage: self.stage.clone(),
            age: self.age,
            networks_eaten: self.networks_eaten,
            current_location: self.current_location.clone(),
            current_action: self.current_action.clone(),
        }
    }
}