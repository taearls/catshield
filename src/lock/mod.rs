//! Single-instance lock mechanism for Cat Shield
//!
//! Ensures only one instance of Cat Shield can run at a time.

use crate::platform::kill;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;
use std::process;

/// Lock file name for single-instance enforcement
const LOCK_FILE_NAME: &str = "catshield.lock";

/// Maximum number of retry attempts for lock acquisition
const LOCK_RETRY_LIMIT: u32 = 3;

/// Get the path to the lock file
/// On macOS: ~/Library/Application Support/catshield/catshield.lock
/// On Linux: ~/.config/catshield/catshield.lock
fn lock_file_path() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join("catshield").join(LOCK_FILE_NAME))
}

/// Check if a process with the given PID is still running
fn is_process_running(pid: u32) -> bool {
    // Use kill with signal 0 to check if process exists
    // This doesn't actually send a signal, just checks if the process exists
    unsafe { kill(pid as i32, 0) == 0 }
}

/// Result of attempting to acquire the single-instance lock
pub enum LockResult {
    /// Successfully acquired the lock
    Acquired,
    /// Another instance is already running with the given PID
    AlreadyRunning(u32),
    /// Failed to acquire lock due to an error
    Error(String),
}

/// Attempt to acquire the single-instance lock
/// Returns LockResult indicating success, existing instance, or error
///
/// Uses atomic file creation (O_CREAT | O_EXCL) to prevent TOCTOU race conditions
/// where two instances could simultaneously detect a stale lock and both acquire it.
pub fn acquire_instance_lock() -> LockResult {
    let Some(lock_path) = lock_file_path() else {
        return LockResult::Error("Could not determine lock file path".to_string());
    };

    // Ensure the config directory exists
    if let Some(parent) = lock_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            return LockResult::Error(format!("Failed to create config directory: {}", e));
        }
    }

    let current_pid = process::id();

    // Use a loop with retry limit to handle stale lock cleanup
    for attempt in 0..LOCK_RETRY_LIMIT {
        // Try to atomically create the lock file (fails if it already exists)
        match OpenOptions::new()
            .write(true)
            .create_new(true) // Atomic: fails with AlreadyExists if file exists
            .open(&lock_path)
        {
            Ok(mut file) => {
                // Successfully created new lock file atomically
                if let Err(e) = write!(file, "{}", current_pid) {
                    // Write failed - clean up the file we created
                    let _ = fs::remove_file(&lock_path);
                    return LockResult::Error(format!("Failed to write lock file: {}", e));
                }
                return LockResult::Acquired;
            }
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
                // Lock file exists - check if it's from a running process
                match fs::read_to_string(&lock_path) {
                    Ok(contents) => {
                        if let Ok(existing_pid) = contents.trim().parse::<u32>() {
                            if is_process_running(existing_pid) {
                                // Another instance is actually running
                                return LockResult::AlreadyRunning(existing_pid);
                            }
                            // Stale lock from a dead process - try to remove and retry
                            let _ = fs::remove_file(&lock_path);
                            // Continue to next iteration to retry atomic creation
                        } else {
                            // Invalid PID in lock file - try to remove and retry
                            let _ = fs::remove_file(&lock_path);
                        }
                    }
                    Err(_) => {
                        // Can't read lock file - try to remove and retry
                        let _ = fs::remove_file(&lock_path);
                    }
                }
            }
            Err(e) => {
                // Other error (permissions, etc.)
                return LockResult::Error(format!("Failed to create lock file: {}", e));
            }
        }

        // If we're here, we removed a stale lock and will retry
        // Log only on later attempts to avoid noise
        if attempt > 0 {
            eprintln!(
                "  ⚠️  Retrying lock acquisition (attempt {}/{})",
                attempt + 1,
                LOCK_RETRY_LIMIT
            );
        }
    }

    // Exhausted retries - this shouldn't normally happen
    LockResult::Error(format!(
        "Failed to acquire lock after {} attempts",
        LOCK_RETRY_LIMIT
    ))
}

/// Release the single-instance lock by removing the lock file
pub fn release_instance_lock() {
    if let Some(lock_path) = lock_file_path() {
        // Only remove the lock file if it contains our PID
        if let Ok(contents) = fs::read_to_string(&lock_path) {
            if let Ok(file_pid) = contents.trim().parse::<u32>() {
                if file_pid == process::id() {
                    let _ = fs::remove_file(&lock_path);
                }
            }
        }
    }
}
