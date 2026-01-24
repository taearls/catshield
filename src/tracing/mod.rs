//! Event tracing infrastructure for Cat Shield
//!
//! Provides persistent file logging for debugging intermittent issues.
//! Trace logging is opt-in via `--trace` CLI flag or `enable_trace_logging = true` in config.
//!
//! # Features
//!
//! - Persistent file logging to `~/.config/catshield/logs/`
//! - Daily log rotation with 7-day retention
//! - Structured event format with timestamps and metadata
//! - Conditional logging based on trace enable flag
//!
//! # Example
//!
//! ```ignore
//! use cat_shield::tracing::{trace_event, events::MenuClicked};
//!
//! // Log a menu click event
//! trace_event(&MenuClicked {
//!     item: "Settings",
//!     enabled: true,
//! });
//! ```

pub mod events;

use chrono::{Local, Utc};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

/// Global flag indicating whether trace logging is enabled
static TRACING_ENABLED: AtomicBool = AtomicBool::new(false);

/// Global log file handle (lazily initialized)
static LOG_FILE: Mutex<Option<File>> = Mutex::new(None);

/// Number of days to retain log files
const LOG_RETENTION_DAYS: u64 = 7;

/// Trait for events that can be traced
///
/// Implement this trait for any event type that should be logged
/// when trace logging is enabled.
pub trait TracedEvent {
    /// Event category (e.g., "menu", "window", "settings")
    fn category(&self) -> &'static str;

    /// Event name (e.g., "settings_clicked", "window_closed")
    fn name(&self) -> &'static str;

    /// Event metadata as key-value pairs
    fn metadata(&self) -> Vec<(&'static str, String)>;
}

/// Initialize trace logging
///
/// Sets up the trace logging system if enabled via CLI flag or config.
/// Creates the log directory and opens the log file for the current day.
///
/// # Arguments
///
/// * `enabled` - Whether trace logging should be enabled
///
/// # Returns
///
/// `Ok(())` if initialization succeeded, `Err(String)` otherwise
pub fn init(enabled: bool) -> Result<(), String> {
    if !enabled {
        TRACING_ENABLED.store(false, Ordering::Release);
        return Ok(());
    }

    // Get log directory path
    let log_dir = get_log_dir()?;

    // Create log directory if it doesn't exist
    fs::create_dir_all(&log_dir).map_err(|e| format!("Failed to create log directory: {}", e))?;

    // Clean up old log files
    cleanup_old_logs(&log_dir)?;

    // Open log file for today
    let log_path = get_log_path(&log_dir);
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .map_err(|e| format!("Failed to open log file: {}", e))?;

    // Store file handle
    if let Ok(mut guard) = LOG_FILE.lock() {
        *guard = Some(file);
    }

    // Enable tracing
    TRACING_ENABLED.store(true, Ordering::Release);

    // Log initialization
    log::info!("Trace logging enabled: {}", log_path.display());

    Ok(())
}

/// Check if trace logging is enabled
pub fn is_tracing_enabled() -> bool {
    TRACING_ENABLED.load(Ordering::Acquire)
}

/// Log an event if tracing is enabled
///
/// Writes the event to the trace log file in the format:
/// `TIMESTAMP TRACE [category] event_name | key=value, key=value`
///
/// # Arguments
///
/// * `event` - The event to log
pub fn trace_event<E: TracedEvent>(event: &E) {
    if !is_tracing_enabled() {
        return;
    }

    let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ");
    let category = event.category();
    let name = event.name();

    let metadata = event.metadata();
    let meta_str = if metadata.is_empty() {
        String::new()
    } else {
        metadata
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join(", ")
    };

    let line = if meta_str.is_empty() {
        format!("{} TRACE [{}] {}\n", timestamp, category, name)
    } else {
        format!(
            "{} TRACE [{}] {} | {}\n",
            timestamp, category, name, meta_str
        )
    };

    // Write to log file
    if let Ok(mut guard) = LOG_FILE.lock() {
        if let Some(ref mut file) = *guard {
            let _ = file.write_all(line.as_bytes());
            let _ = file.flush();
        }
    }

    // Also log at trace level for console output when verbose
    log::trace!("[{}] {} | {}", category, name, meta_str);
}

/// Get the log directory path
fn get_log_dir() -> Result<PathBuf, String> {
    dirs::config_dir()
        .map(|p| p.join("catshield").join("logs"))
        .ok_or_else(|| "Could not determine config directory".to_string())
}

/// Get the log file path for today
fn get_log_path(log_dir: &Path) -> PathBuf {
    let date = Local::now().format("%Y-%m-%d");
    log_dir.join(format!("catshield-{}.log", date))
}

/// Clean up log files older than retention period
fn cleanup_old_logs(log_dir: &Path) -> Result<(), String> {
    let cutoff = std::time::SystemTime::now()
        - std::time::Duration::from_secs(LOG_RETENTION_DAYS * 24 * 60 * 60);

    if let Ok(entries) = fs::read_dir(log_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("log") {
                if let Ok(metadata) = path.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        if modified < cutoff {
                            let _ = fs::remove_file(&path);
                            log::debug!("Removed old log file: {}", path.display());
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Shutdown trace logging
///
/// Flushes and closes the log file.
pub fn shutdown() {
    if let Ok(mut guard) = LOG_FILE.lock() {
        if let Some(ref mut file) = *guard {
            let _ = file.flush();
        }
        *guard = None;
    }
    TRACING_ENABLED.store(false, Ordering::Release);
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestEvent {
        action: String,
    }

    impl TracedEvent for TestEvent {
        fn category(&self) -> &'static str {
            "test"
        }

        fn name(&self) -> &'static str {
            "test_action"
        }

        fn metadata(&self) -> Vec<(&'static str, String)> {
            vec![("action", self.action.clone())]
        }
    }

    #[test]
    fn test_tracing_disabled_by_default() {
        assert!(!is_tracing_enabled());
    }

    #[test]
    fn test_init_with_disabled() {
        let result = init(false);
        assert!(result.is_ok());
        assert!(!is_tracing_enabled());
    }

    #[test]
    fn test_traced_event_trait() {
        let event = TestEvent {
            action: "click".to_string(),
        };
        assert_eq!(event.category(), "test");
        assert_eq!(event.name(), "test_action");
        assert_eq!(event.metadata(), vec![("action", "click".to_string())]);
    }

    #[test]
    fn test_trace_event_when_disabled() {
        // Should not panic even when tracing is disabled
        let event = TestEvent {
            action: "test".to_string(),
        };
        trace_event(&event);
    }
}
