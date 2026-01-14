//! Config file I/O and global state management for Cat Shield

use super::types::Config;
use std::sync::Mutex;

/// Global reference to the current loaded config
static CURRENT_CONFIG: std::sync::OnceLock<Mutex<Config>> = std::sync::OnceLock::new();

/// Get a reference to the current config.
///
/// If the mutex is poisoned (a thread panicked while holding the lock),
/// this will recover the inner value and continue. This is safe because
/// Config is a simple data struct with no complex invariants.
pub fn get_current_config() -> Config {
    CURRENT_CONFIG
        .get_or_init(|| Mutex::new(Config::load()))
        .lock()
        .unwrap_or_else(|poisoned| {
            // Recover from poisoned mutex - the Config struct has no complex
            // invariants that could be violated by a panic, so we can safely
            // recover the inner value.
            log::warn!("Config mutex was poisoned, recovering...");
            poisoned.into_inner()
        })
        .clone()
}

/// Update the current config.
///
/// If the mutex is poisoned (a thread panicked while holding the lock),
/// this will recover and update the value. This is safe because
/// Config is a simple data struct with no complex invariants.
pub fn set_current_config(config: Config) {
    let mutex = CURRENT_CONFIG.get_or_init(|| Mutex::new(Config::load()));
    let mut guard = mutex.lock().unwrap_or_else(|poisoned| {
        // Recover from poisoned mutex - safe for Config struct
        log::warn!("Config mutex was poisoned, recovering...");
        poisoned.into_inner()
    });
    *guard = config;
}
