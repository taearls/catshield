//! Config file I/O and global state management for Cat Shield

use super::types::Config;
use std::sync::Mutex;

/// Global reference to the current loaded config
static CURRENT_CONFIG: std::sync::OnceLock<Mutex<Config>> = std::sync::OnceLock::new();

/// Get a reference to the current config
pub fn get_current_config() -> Config {
    CURRENT_CONFIG
        .get_or_init(|| Mutex::new(Config::load()))
        .lock()
        .unwrap()
        .clone()
}

/// Update the current config
pub fn set_current_config(config: Config) {
    let mutex = CURRENT_CONFIG.get_or_init(|| Mutex::new(Config::load()));
    *mutex.lock().unwrap() = config;
}
