use std::process::Command;
use std::thread;
use std::fs;
use tauri::{AppHandle, Emitter, Manager};
use chrono::Local;
use crate::AppState;
use crate::utils::{save_state, write_log, load_or_create_config, get_app_dir}; 

// CHECK DEPENDANCIES
#[tauri::command]
pub fn check_dependencies() -> bool {
    // NMAP checker
    std::process::Command::new("nmap").arg("--version").output().is_ok()
}


// SUBNET GREP
fn get_local_subnet() -> Option<String> {
    let output = Command::new("ip").args(["-o", "-f", "inet", "addr", "show"]).output().ok()?;
    let output_str = String::from_utf8_lossy(&output.stdout);
    
    for line in output_str.lines() {
        if line.contains("scope global") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let ip_cidr = parts[3]; 
                if let Some((ip, cidr)) = ip_cidr.split_once('/') {
                    let octets: Vec<&str> = ip.split('.').collect();
                    if octets.len() == 4 {
                        return Some(format!("{}.{}.{}.0/{}", octets[0], octets[1], octets[2], cidr));
                    }
                }
                return Some(ip_cidr.to_string());
            }
        }
    }
    None
}

// SHOW SCANS
#[tauri::command]
pub fn list_saved_scans(_app_handle: AppHandle) -> Result<Vec<String>, String> {
    let scan_dir = get_app_dir().join("scans");
    if !scan_dir.exists() { return Ok(Vec::new()); }

    let mut scan_files = Vec::new();
    if let Ok(entries) = fs::read_dir(scan_dir) {
        for entry in entries.flatten() {
            if let Ok(file_name) = entry.file_name().into_string() {
                if file_name.ends_with(".txt") { scan_files.push(file_name); }
            }
        }
    }
    scan_files.sort_by(|a, b| b.cmp(a));
    Ok(scan_files)
}

// OPEN SCANS
#[tauri::command]
pub fn open_scan(_app_handle: AppHandle, file_name: String) -> Result<(), String> {
    let file_path = get_app_dir().join("scans").join(&file_name);
    
    #[cfg(target_os = "linux")]
    let _ = Command::new("xdg-open").arg(&file_path).spawn();
    
    Ok(())
}

// SCAN PROCEDURE LOGIC
#[tauri::command]
pub fn trigger_active_scan(app_handle: AppHandle) -> Result<(), String> {
    let config = load_or_create_config();

    let subnet = if config.target_subnet.to_lowercase() == "auto" {
        match get_local_subnet() {
            Some(s) => s,
            None => {
                write_log("[ERR] SUBNET_DISCOVERY_FAILED.");
                return Err("Subnet not found".to_string());
            }
        }
    } else {
        config.target_subnet.clone()
    };

    let scan_dir = get_app_dir().join("scans");
    if let Err(e) = fs::create_dir_all(&scan_dir) {
        write_log(&format!("[ERR] DIR_CREATION_FAILED (scans) - {}", e));
    }

    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    let file_name = format!("scan_{}.txt", timestamp);
    let file_path = scan_dir.join(&file_name);
    let file_path_str = file_path.to_str().unwrap().to_string();
    
    let nmap_custom_args = config.nmap_arguments.clone();

    thread::spawn(move || {
        app_handle.emit("netchi-thought", "INITIATING NMAP SCAN...").unwrap();
        write_log(&format!("[NET] ACTIVE_SCAN_RUNNING: {}", subnet));

        let mut cmd = Command::new("nmap");
        cmd.args(&nmap_custom_args); 
        cmd.args(["-oG", &file_path_str, &subnet]); 

        match cmd.output() {
            Ok(_) => {
                let result_str = fs::read_to_string(&file_path_str).unwrap_or_default();
                let mut hosts_found = 0;
                let mut open_ports = 0;

                for line in result_str.lines() {
                    if line.starts_with("Host: ") {
                        hosts_found += 1;
                        open_ports += line.matches("/open/").count() as u32;
                    }
                }

                write_log(&format!("[NET] SCAN_COMPLETE: {} HOSTS, {} PORTS. DUMP: {}", hosts_found, open_ports, file_name));

                if let Ok(mut state) = app_handle.state::<AppState>().0.lock() {
                    if hosts_found > 0 {
                        let data_payloads = (hosts_found * 5) + (open_ports * 2);
                        state.packet_buffer += data_payloads;
                        state.happiness = state.happiness.saturating_add(20).min(100);
                        state.current_action = "surprised".to_string();
                        
                        crate::brain::learn_from_action(&mut state, 10.0);
                        
                        let intel_msg = format!("[NET] RECON_SUCCESS: {} HOSTS FOUND.", hosts_found);
                        app_handle.emit("netchi-thought", &intel_msg).unwrap();

                        state.last_hosts_count = hosts_found;
                        save_state(&state);
                        
                        app_handle.emit("scan-completed", file_name).unwrap();
                        app_handle.emit("state-update", state.to_ui_state()).unwrap();
                    } else {
                        crate::brain::learn_from_action(&mut state, -10.0);
                        app_handle.emit("netchi-thought", "TARGETS UNREACHABLE...").unwrap();
                        let _ = fs::remove_file(&file_path_str);
                    }
                }
            }
            Err(e) => {
                write_log(&format!("[ERR] NMAP_EXECUTION_FAILED - {}", e));
                app_handle.emit("netchi-thought", "NMAP MODULE CORRUPT!").unwrap();
            }
        }
    });

    Ok(())
}

