use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use pcap::{Device, Capture};
use rand::Rng;
use crate::models::LifeStage;
use crate::utils::{save_state, write_log};
use crate::AppState;

fn sys_log(app: &AppHandle, msg: &str) {
    write_log(msg);
    let _ = app.emit("sys-log", msg);
}

pub fn start_biological_clock(app_handle: AppHandle) {
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(5));
            let mut trigger_autonomous_scan = false; 

            if let Ok(mut state) = app_handle.state::<AppState>().0.lock() {
                if state.stage == LifeStage::Egg {
                    state.age += 1;
                    if state.age >= 60 { // 5 minutes
                        let skins = ["ghost", "duck", "beagle"];
                        let mut rng = rand::thread_rng();
                        state.skin = skins[rng.gen_range(0..skins.len())].to_string();
                        state.stage = LifeStage::Baby;
                        state.current_action = "idle".to_string();
                        sys_log(&app_handle, &format!("[SYS] HATCH_EVENT. CLASS_ALLOCATED: '{}'", state.skin.to_uppercase()));
                    }
                } else {
                    state.age += 1;
                    let is_baby = state.stage == LifeStage::Baby;

                    let config = crate::utils::load_or_create_config();
                    let noise_threshold = config.noise_threshold;

                    if !is_baby && state.age % 6 == 0 {
                        if state.age.saturating_sub(state.last_scan_age) > 180 {
                            state.last_scan_age = state.age;
                            trigger_autonomous_scan = true;
                            sys_log(&app_handle, "[AI] IDLE_TIMEOUT. FORCING_ACTIVE_SCAN.");
                        } else {
                            let decision = crate::brain::decide_action(&mut state, noise_threshold);
                            let is_crowded = state.packet_buffer > noise_threshold;
                            
                            match decision.as_str() {
                                "Rest" => {
                                    let reward = if !is_crowded { 5.0 } else { -2.0 };
                                    crate::brain::learn_from_action(&mut state, reward);
                                    state.current_action = "sleeping".to_string();
                                    sys_log(&app_handle, "[AI] ENV_QUIET. INITIATING_SLEEP_MODE.");
                                },
                                "Passive" => {
                                    let reward = if is_crowded { 3.0 } else { -1.0 };
                                    crate::brain::learn_from_action(&mut state, reward);
                                    state.current_action = "idle".to_string(); 
                                    
                                    let env_desc = if is_crowded { "CROWDED" } else { "QUIET" };
                                    sys_log(&app_handle, &format!("[AI] PASSIVE_SNIFF. ENV_STATUS: {}.", env_desc));
                                },
                                "ScanFast" => {
                                    let cycles_since_last = state.age.saturating_sub(state.last_scan_age);
                                    
                                    if cycles_since_last < 60 { 
                                        crate::brain::learn_from_action(&mut state, -20.0);
                                        sys_log(&app_handle, "[AI] RATE_LIMIT_EXCEEDED. ACTION_REJECTED (-20). FORCING_SLEEP.");
                                        state.current_action = "sleeping".to_string();
                                    } else {
                                        state.last_scan_age = state.age;
                                        trigger_autonomous_scan = true;
                                        sys_log(&app_handle, "[AI] ACTIVE_RECON_INITIATED.");
                                    }
                                },
                                _ => {}
                            }
                        }
                    }

                    let hunger_drain = if is_baby { 2 } else { 1 };
                    state.hunger = state.hunger.saturating_sub(hunger_drain);

                    if state.hunger == 0 {
                        state.happiness = state.happiness.saturating_sub(5);
                        state.current_action = "dying".to_string();
                    } else if state.current_action == "sleeping" {
                        // Sleep overrides
                    } else if state.hunger < 50 {
                        state.current_action = "hungry".to_string();
                    } else {
                        state.current_action = "idle".to_string();
                    }

                    if is_baby && state.age >= 3600 && state.networks_eaten >= 1000 {
                        state.stage = LifeStage::Adult;
                        sys_log(&app_handle, "[SYS] MATURATION_COMPLETE. STAGE: ADULT.");
                    }
                }

                save_state(&state);
                app_handle.emit("state-update", state.to_ui_state()).unwrap();
            }

            if trigger_autonomous_scan {
                let _ = crate::scanner::trigger_active_scan(app_handle.clone());
            }
        }
    });
}

pub fn start_network_sniffer(app_handle: AppHandle) {
    thread::spawn(move || {
        let device = match Device::lookup() { Ok(Some(d)) => d, _ => return };
        
        // PCAP PERMISSIONS SECURE MANAGEMENT
        let cap_result = Capture::from_device(device).unwrap().promisc(false).snaplen(64).timeout(1000).open();
        
        let mut cap = match cap_result {
            Ok(c) => c,
            Err(_) => {
                sys_log(&app_handle, "[ERR] PCAP_INIT_FAILED: MISSING_CAPABILITIES.");
                sys_log(&app_handle, "[ERR] RUN: sudo setcap cap_net_raw,cap_net_admin=eip /path/to/netchi");
                return; // Elegantly exits avoiding Rust crash!
            }
        };

        let _ = cap.filter("ether broadcast", true);

        loop {
            if let Ok(packet) = cap.next_packet() {
                if packet.data.len() >= 12 {
                    let src_mac = format!("{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}", packet.data[6], packet.data[7], packet.data[8], packet.data[9], packet.data[10], packet.data[11]);

                    if let Ok(mut state) = app_handle.state::<AppState>().0.lock() {
                        if state.stage != LifeStage::Egg {
                            state.packet_buffer += 1;

                            let mac_upper = src_mac.to_uppercase();
                            
                            if let Some(brain_msg) = crate::brain::process_experience(&mut state, &mac_upper) {
                                sys_log(&app_handle, &brain_msg);
                            }
                            
                            let is_random = crate::brain::is_mac_randomized(&mac_upper);
                            let is_new_mac = !state.netdex.contains_key(&mac_upper) && !is_random;

                            if is_new_mac {
                                let new_node = crate::brain::create_netdex_entry(&mac_upper);
                                state.netdex.insert(mac_upper.clone(), new_node);
                                state.happiness = state.happiness.saturating_add(2).min(100);
                                
                                sys_log(&app_handle, &format!("[NET] NEW_BSSID_DETECTED: {}", mac_upper));

                                if state.stage == LifeStage::Adult {
                                    app_handle.emit("intruder-alert", &mac_upper).unwrap();
                                }
                            }

                            let mut trigger_ui = is_new_mac || state.packet_buffer % 10 == 0;
                            let is_sleeping = state.current_action == "sleeping";
                            let is_starving = state.hunger < 30 && state.packet_buffer > 0;
                            let is_hungry_and_has_meal = state.hunger <= 70 && state.packet_buffer >= 10;

                            if is_starving || (!is_sleeping && is_hungry_and_has_meal) {
                                let nutrition = (state.packet_buffer * 2) as u8;
                                state.hunger = state.hunger.saturating_add(nutrition).min(100);
                                state.networks_eaten += state.packet_buffer;
                                
                                sys_log(&app_handle, &format!("[SYS] BUFFER_FLUSH. EXTRACTED_PAYLOADS: {}", state.packet_buffer));
                                
                                state.packet_buffer = 0;
                                state.current_action = "eating".to_string(); 
                                trigger_ui = true;
                            }

                            if trigger_ui {
                                crate::utils::save_state(&state);
                                app_handle.emit("state-update", state.to_ui_state()).unwrap();
                            }
                        }
                    }
                }
            }
        }
    });
}