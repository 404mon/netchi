use crate::models::{NetchiState, NetworkNode};
use rand::Rng;
use std::collections::HashMap;

const PREFIXES: &[&str] = &[
    "Abibliophobia", "Absquatulate", "Allegator", "Anencephalous", "Argle-bargle",
    "Batrachomyomachy", "Billingsgate", "Bloviate", "Blunderbuss", "Borborygm",
    "Boustrophedon", "Bowyang", "Brouhaha", "Bumbershoot", "Callipygian",
    "Canoodle", "Cantankerous", "Catercornered", "Cockalorum", "Cockamamie",
    "Codswallop", "Collop", "Collywobbles", "Comeuppance", "Crapulence",
    "Crudivore", "Discombobulate", "Donnybrook", "Doozy", "Dudgeon",
    "Ecdysiast", "Eructation", "Fard", "Fartlek", "Fatuous",
    "Filibuster", "Firkin", "Flibbertigibbet", "Flummox", "Folderol",
    "Formication", "Fuddy-duddy", "Furbelow", "Furphy", "Gaberlunzie",
    "Gardyloo!", "Gastromancy", "Gazump", "Gobbledygook", "Gobemouche",
    "Godwottery", "Gongoozle", "Gonzo", "Goombah", "Hemidemisemiquaver",
    "Hobbledehoy", "Hocus-pocus", "Hoosegow", "Hootenanny", "Jackanapes",
    "Kerfuffle", "Klutz", "La-di-da", "Lagopodous", "Lickety-split",
    "Lickspittle", "Logorrhea", "Lollygag", "Malarkey", "Maverick",
    "Mollycoddle", "Mugwump", "Mumpsimus", "Namby-pamby", "Nincompoop",
    "Oocephalus", "Ornery", "Pandiculation", "Panjandrum", "Pettifogger",
    "Pratfall", "Quean", "Rambunctious", "Ranivorous", "Rigmarole",
    "Shenanigan", "Sialoquent", "Skedaddle", "Skullduggery", "Slangwhanger",
    "Smellfungus", "Snickersnee", "Snollygoster", "Snool", "Tatterdemalion",
    "Troglodyte", "Turdiform", "Unremacadamized", "Vomitory", "Wabbit",
    "Widdershins", "Yahoo",
];

const SUFFIXES: &[&str] = &[
    "Lodge", "Niche", "Park", "Berth", "Range", "Cluster", "File", "Anchor", 
    "Compartment", "Vault", "Hub", "Sector", "Gateway", "Basement", "Grid", 
    "Haven", "Den", "Burrow", "Shack", "Manor", "Bastion", "Outpost", 
    "Grotto", "Nest", "Spire", "Sanctum", "Ward", "Quarter", "Hollow", 
    "Cove", "Abode", "Pocket", "Landing", "Dock", "Station", "Point", 
    "Rise", "Manse", "Hearth", "Bunker", "Flat", "Chateau", "Annex",
    "Nexus", "Core", "Module", "Pod", "Void", "Array", "Matrix", "Foundry", 
    "Relay", "Silo", "Pillar", "Terminal", "Deck", "Hangar", "Orbit", 
    "Monolith", "Sprawl", "Aperture", "Biolab", "Citadel", "Gantry", 
    "Habitation", "Junction", "Platform", "Quadrant", "Synapse", "Vortex", 
    "Zenith", "Uplink", "Data-Bank", "Biosphere", "Reactor", "Node",
    "Crib", "Lair", "Nook", "Cranny", "Unit", "Archive", "Basin", "Coffer", 
    "Dome", "Expanse", "Fragment", "Gulch", "Keep", "Lattice", "Mezzanine", 
    "Nucleus", "Outlier", "Prism", "Quarry", "Rift", "Trench", "Vantage", 
    "Wharf", "Yard", "Zone", "Cubicle", "Hideout", "Shelter", "Joint",
];

pub fn generate_location_name(_mac: &str) -> String {
    let mut rng = rand::thread_rng();
    let prefix = PREFIXES[rng.gen_range(0..PREFIXES.len())];
    let suffix = SUFFIXES[rng.gen_range(0..SUFFIXES.len())];
    format!("{} {}", prefix, suffix)
}

pub fn is_mac_randomized(mac_upper: &str) -> bool {
    let second_char = mac_upper.chars().nth(1).unwrap_or('0');
    matches!(second_char, '2' | '6' | 'A' | 'E')
}

pub fn identify_vendor(mac_upper: &str) -> String {
    if mac_upper.starts_with("00:03:93") || mac_upper.starts_with("F0:98:9D") { "Apple".to_string() } 
    else if mac_upper.starts_with("00:15:99") || mac_upper.starts_with("24:4B:03") { "Samsung".to_string() } 
    else { "Unknown".to_string() }
}


// LOGGING
pub fn process_experience(state: &mut NetchiState, mac: &str) -> Option<String> {
    let netdex_size = state.netdex.len() as f32;
    let familiarity_ratio = if state.networks_eaten > 0 { netdex_size / state.networks_eaten as f32 } else { 1.0 };

    if (familiarity_ratio < 0.5 || state.networks_eaten > 50) && state.current_location == "Unknown Sector" {
        state.current_location = generate_location_name(mac);
        return Some(format!("[SYS] SECTOR_LOCKED: {}", state.current_location.to_uppercase()));
    }
    
    None
}

pub fn create_netdex_entry(mac: &str) -> NetworkNode {
    let mac_upper = mac.to_uppercase();
    NetworkNode { mac: mac_upper.clone(), vendor: identify_vendor(&mac_upper), ip: None, open_ports: Vec::new(), first_seen: 0, last_seen: 0 }
}

pub fn get_environment_state(state: &NetchiState, noise_threshold: u32) -> String {
    // Usa la soglia configurabile dal TOML per capire se c'è folla
    let crowd_level = if state.packet_buffer > noise_threshold { "Crowded" } else { "Quiet" };
    let hunger_level = if state.hunger > 60 { "Full" } else { "Hungry" };
    
    format!("{}_{}", hunger_level, crowd_level)
}

pub fn decide_action(state: &mut NetchiState, noise_threshold: u32) -> String {
    let current_state = get_environment_state(state, noise_threshold);
    let actions = vec!["Rest", "Passive", "ScanFast"];
    
    let mut rng = rand::thread_rng();
    let action = if rng.gen::<f32>() < state.exploration_rate {
        actions[rng.gen_range(0..actions.len())].to_string()
    } else {
        let state_actions = state.q_table.entry(current_state.clone()).or_insert_with(HashMap::new);
        let mut best_action = "Passive".to_string();
        let mut max_q = f32::MIN;
        for a in actions {
            let q_val = *state_actions.get(a).unwrap_or(&0.0);
            if q_val > max_q { max_q = q_val; best_action = a.to_string(); }
        }
        best_action
    };

    state.last_state = current_state;
    state.last_action = action.clone();
    if state.exploration_rate > 0.1 { state.exploration_rate *= 0.99; }
    action
}

pub fn learn_from_action(state: &mut NetchiState, reward: f32) {
    if state.last_state.is_empty() || state.last_action.is_empty() { return; }
    let alpha = 0.1;
    let state_actions = state.q_table.entry(state.last_state.clone()).or_insert_with(HashMap::new);
    let old_q = *state_actions.get(&state.last_action).unwrap_or(&0.0);
    let new_q = old_q + alpha * (reward - old_q);
    state_actions.insert(state.last_action.clone(), new_q);
}