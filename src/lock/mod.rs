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
pub(crate) fn lock_file_path() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join("catshield").join(LOCK_FILE_NAME))
}

/// Check if a process with the given PID is still running
pub(crate) fn is_process_running(pid: u32) -> bool {
    // Use kill with signal 0 to check if process exists
    // This doesn't actually send a signal, just checks if the process exists
    unsafe { kill(pid as i32, 0) == 0 }
}

/// Result of attempting to acquire the single-instance lock
#[must_use]
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    // ============================================================
    // Helper functions for testing with isolated lock files
    // ============================================================

    /// Acquire a lock using a custom path (for isolated testing)
    fn acquire_lock_at_path(lock_path: &PathBuf) -> LockResult {
        // Ensure the parent directory exists
        if let Some(parent) = lock_path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                return LockResult::Error(format!("Failed to create directory: {}", e));
            }
        }

        let current_pid = process::id();

        for attempt in 0..LOCK_RETRY_LIMIT {
            match OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(lock_path)
            {
                Ok(mut file) => {
                    if let Err(e) = write!(file, "{}", current_pid) {
                        let _ = fs::remove_file(lock_path);
                        return LockResult::Error(format!("Failed to write lock file: {}", e));
                    }
                    return LockResult::Acquired;
                }
                Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
                    match fs::read_to_string(lock_path) {
                        Ok(contents) => {
                            if let Ok(existing_pid) = contents.trim().parse::<u32>() {
                                if is_process_running(existing_pid) {
                                    return LockResult::AlreadyRunning(existing_pid);
                                }
                                let _ = fs::remove_file(lock_path);
                            } else {
                                let _ = fs::remove_file(lock_path);
                            }
                        }
                        Err(_) => {
                            let _ = fs::remove_file(lock_path);
                        }
                    }
                }
                Err(e) => {
                    return LockResult::Error(format!("Failed to create lock file: {}", e));
                }
            }

            if attempt > 0 {
                // Suppress retry messages in tests
            }
        }

        LockResult::Error(format!(
            "Failed to acquire lock after {} attempts",
            LOCK_RETRY_LIMIT
        ))
    }

    /// Release a lock at a custom path
    fn release_lock_at_path(lock_path: &PathBuf) {
        if let Ok(contents) = fs::read_to_string(lock_path) {
            if let Ok(file_pid) = contents.trim().parse::<u32>() {
                if file_pid == process::id() {
                    let _ = fs::remove_file(lock_path);
                }
            }
        }
    }

    // ============================================================
    // Tests for lock_file_path()
    // ============================================================

    #[test]
    fn test_lock_file_path_returns_some() {
        // lock_file_path should return a valid path on macOS/Linux
        let path = lock_file_path();
        assert!(
            path.is_some(),
            "lock_file_path() should return Some on macOS"
        );
    }

    #[test]
    fn test_lock_file_path_contains_catshield() {
        // The path should contain "catshield" directory and filename
        let path = lock_file_path().expect("Should return a path");
        let path_str = path.to_string_lossy();
        assert!(
            path_str.contains("catshield"),
            "Path should contain 'catshield': {}",
            path_str
        );
        assert!(
            path_str.ends_with("catshield.lock"),
            "Path should end with 'catshield.lock': {}",
            path_str
        );
    }

    // ============================================================
    // Tests for is_process_running()
    // ============================================================

    #[test]
    fn test_is_process_running_current_process() {
        // The current process should always be detected as running
        let current_pid = process::id();
        assert!(
            is_process_running(current_pid),
            "Current process (PID {}) should be detected as running",
            current_pid
        );
    }

    #[test]
    fn test_is_process_running_nonexistent_pid() {
        // A very high PID is unlikely to exist
        // On macOS, PIDs are typically < 100000
        let nonexistent_pid = 999_999_999;
        assert!(
            !is_process_running(nonexistent_pid),
            "PID {} should not exist",
            nonexistent_pid
        );
    }

    #[test]
    fn test_is_process_running_known_system_process() {
        // Test with current process since it's guaranteed to be accessible
        // Note: On macOS, kill(1, 0) for PID 1 (launchd) requires root privileges
        // so we test with the current process which is always accessible
        let current_pid = process::id();
        assert!(
            is_process_running(current_pid),
            "Current process should be detected as running"
        );

        // Also verify that a non-existent process returns false
        // This ensures the function distinguishes between running and not running
        let nonexistent = 999_999_999u32;
        assert!(
            !is_process_running(nonexistent),
            "Non-existent process should not be detected as running"
        );
    }

    #[test]
    fn test_is_process_running_pid_0() {
        // PID 0 is the kernel process on Unix; kill(0, 0) has special semantics
        // (sends signal to all processes in current process group)
        // This test just ensures we don't crash
        let _ = is_process_running(0);
    }

    // ============================================================
    // Tests for acquire_instance_lock() / release_instance_lock()
    // Using isolated temp directories
    // ============================================================

    #[test]
    fn test_lock_acquired_creates_file() {
        // Acquiring a lock should create the lock file
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let lock_path = temp_dir.path().join("catshield.lock");

        assert!(!lock_path.exists(), "Lock file should not exist initially");

        let result = acquire_lock_at_path(&lock_path);
        assert!(
            matches!(result, LockResult::Acquired),
            "Should acquire lock successfully"
        );
        assert!(
            lock_path.exists(),
            "Lock file should exist after acquisition"
        );

        // Cleanup
        release_lock_at_path(&lock_path);
    }

    #[test]
    fn test_lock_file_contains_current_pid() {
        // The lock file should contain the current process PID
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let lock_path = temp_dir.path().join("catshield.lock");

        let result = acquire_lock_at_path(&lock_path);
        assert!(matches!(result, LockResult::Acquired));

        let contents = fs::read_to_string(&lock_path).expect("Should read lock file");
        let file_pid: u32 = contents.trim().parse().expect("Should parse PID");
        assert_eq!(
            file_pid,
            process::id(),
            "Lock file should contain current PID"
        );

        // Cleanup
        release_lock_at_path(&lock_path);
    }

    #[test]
    fn test_release_removes_lock_file() {
        // Releasing a lock should remove the lock file
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let lock_path = temp_dir.path().join("catshield.lock");

        let result = acquire_lock_at_path(&lock_path);
        assert!(matches!(result, LockResult::Acquired));
        assert!(lock_path.exists(), "Lock file should exist");

        release_lock_at_path(&lock_path);
        assert!(
            !lock_path.exists(),
            "Lock file should be removed after release"
        );
    }

    #[test]
    fn test_release_only_removes_own_lock() {
        // release_lock_at_path should only remove the file if it contains our PID
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let lock_path = temp_dir.path().join("catshield.lock");

        // Create a lock file with a different PID
        let mut file = fs::File::create(&lock_path).expect("Should create file");
        writeln!(file, "12345").expect("Should write PID");
        drop(file);

        assert!(lock_path.exists(), "Lock file should exist");

        // Try to release - should NOT remove because PID doesn't match
        release_lock_at_path(&lock_path);
        assert!(
            lock_path.exists(),
            "Lock file should still exist (different PID)"
        );

        // Cleanup manually
        let _ = fs::remove_file(&lock_path);
    }

    #[test]
    fn test_stale_lock_from_dead_process_cleaned_up() {
        // A lock from a non-existent process should be cleaned up
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let lock_path = temp_dir.path().join("catshield.lock");

        // Create a stale lock file with a non-existent PID
        let stale_pid = 999_999_999u32;
        let mut file = fs::File::create(&lock_path).expect("Should create file");
        write!(file, "{}", stale_pid).expect("Should write stale PID");
        drop(file);

        // Now try to acquire - should clean up stale lock and succeed
        let result = acquire_lock_at_path(&lock_path);
        assert!(
            matches!(result, LockResult::Acquired),
            "Should acquire lock after cleaning stale lock"
        );

        // Verify the file now contains our PID
        let contents = fs::read_to_string(&lock_path).expect("Should read lock file");
        let file_pid: u32 = contents.trim().parse().expect("Should parse PID");
        assert_eq!(file_pid, process::id(), "Lock should now contain our PID");

        // Cleanup
        release_lock_at_path(&lock_path);
    }

    #[test]
    fn test_lock_with_invalid_pid_content() {
        // A lock file with non-numeric content should be treated as stale
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let lock_path = temp_dir.path().join("catshield.lock");

        // Create a lock file with invalid content
        fs::write(&lock_path, "not-a-number").expect("Should write invalid content");

        // Should clean up invalid lock and acquire
        let result = acquire_lock_at_path(&lock_path);
        assert!(
            matches!(result, LockResult::Acquired),
            "Should acquire lock after cleaning invalid lock"
        );

        // Cleanup
        release_lock_at_path(&lock_path);
    }

    #[test]
    fn test_lock_with_empty_file() {
        // An empty lock file should be treated as stale
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let lock_path = temp_dir.path().join("catshield.lock");

        // Create an empty lock file
        fs::write(&lock_path, "").expect("Should create empty file");

        // Should clean up empty lock and acquire
        let result = acquire_lock_at_path(&lock_path);
        assert!(
            matches!(result, LockResult::Acquired),
            "Should acquire lock after cleaning empty lock"
        );

        // Cleanup
        release_lock_at_path(&lock_path);
    }

    #[test]
    fn test_lock_with_whitespace_only() {
        // A lock file with only whitespace should be treated as stale
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let lock_path = temp_dir.path().join("catshield.lock");

        // Create a lock file with only whitespace
        fs::write(&lock_path, "   \n\t  ").expect("Should write whitespace");

        // Should clean up and acquire
        let result = acquire_lock_at_path(&lock_path);
        assert!(
            matches!(result, LockResult::Acquired),
            "Should acquire lock after cleaning whitespace-only lock"
        );

        // Cleanup
        release_lock_at_path(&lock_path);
    }

    #[test]
    fn test_double_acquire_by_running_process_fails() {
        // If a lock exists for a running process, acquisition should fail
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let lock_path = temp_dir.path().join("catshield.lock");

        // Use the current process PID (guaranteed to be running and accessible)
        let current_pid = process::id();
        fs::write(&lock_path, current_pid.to_string()).expect("Should write current PID");

        // Try to acquire - should fail because current process is running
        let result = acquire_lock_at_path(&lock_path);
        assert!(
            matches!(result, LockResult::AlreadyRunning(pid) if pid == current_pid),
            "Should report already running with current PID"
        );

        // Cleanup manually (since it's "another process's" lock)
        let _ = fs::remove_file(&lock_path);
    }

    #[test]
    fn test_lock_result_error_variant() {
        // Test that Error variant properly captures error messages
        let error_msg = "Test error message";
        let result = LockResult::Error(error_msg.to_string());

        match result {
            LockResult::Error(msg) => {
                assert_eq!(msg, error_msg, "Error message should match");
            }
            _ => panic!("Expected Error variant"),
        }
    }

    #[test]
    fn test_lock_result_already_running_variant() {
        // Test that AlreadyRunning variant properly captures PID
        let pid = 12345u32;
        let result = LockResult::AlreadyRunning(pid);

        match result {
            LockResult::AlreadyRunning(p) => {
                assert_eq!(p, pid, "PID should match");
            }
            _ => panic!("Expected AlreadyRunning variant"),
        }
    }

    #[test]
    fn test_lock_creates_parent_directories() {
        // Acquiring a lock should create parent directories if needed
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let lock_path = temp_dir
            .path()
            .join("nested")
            .join("dirs")
            .join("catshield.lock");

        // Parent directories don't exist yet
        assert!(!lock_path.parent().unwrap().exists());

        let result = acquire_lock_at_path(&lock_path);
        assert!(
            matches!(result, LockResult::Acquired),
            "Should acquire lock in nested directory"
        );
        assert!(lock_path.exists(), "Lock file should exist");
        assert!(
            lock_path.parent().unwrap().exists(),
            "Parent dirs should exist"
        );

        // Cleanup
        release_lock_at_path(&lock_path);
    }

    #[test]
    fn test_acquire_release_cycle() {
        // Test a full acquire-release-acquire cycle
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let lock_path = temp_dir.path().join("catshield.lock");

        // First acquisition
        let result1 = acquire_lock_at_path(&lock_path);
        assert!(
            matches!(result1, LockResult::Acquired),
            "First acquire should succeed"
        );

        // Release
        release_lock_at_path(&lock_path);
        assert!(!lock_path.exists(), "Lock should be released");

        // Second acquisition
        let result2 = acquire_lock_at_path(&lock_path);
        assert!(
            matches!(result2, LockResult::Acquired),
            "Second acquire should succeed"
        );

        // Final cleanup
        release_lock_at_path(&lock_path);
    }

    // ============================================================
    // Tests for edge cases
    // ============================================================

    #[test]
    fn test_lock_file_with_negative_pid() {
        // A lock file with a negative number should be treated as invalid
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let lock_path = temp_dir.path().join("catshield.lock");

        // Create a lock file with negative number
        fs::write(&lock_path, "-1234").expect("Should write negative number");

        // Should clean up invalid lock and acquire (parse as u32 will fail)
        let result = acquire_lock_at_path(&lock_path);
        assert!(
            matches!(result, LockResult::Acquired),
            "Should acquire lock after cleaning invalid (negative) lock"
        );

        // Cleanup
        release_lock_at_path(&lock_path);
    }

    #[test]
    fn test_lock_file_with_overflow_pid() {
        // A PID larger than u32::MAX should be treated as invalid
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let lock_path = temp_dir.path().join("catshield.lock");

        // Create a lock file with overflow number
        fs::write(&lock_path, "99999999999999999999").expect("Should write overflow number");

        // Should clean up invalid lock and acquire
        let result = acquire_lock_at_path(&lock_path);
        assert!(
            matches!(result, LockResult::Acquired),
            "Should acquire lock after cleaning overflow lock"
        );

        // Cleanup
        release_lock_at_path(&lock_path);
    }

    #[test]
    fn test_constants_are_reasonable() {
        // Verify our constants have sensible values
        assert_eq!(LOCK_FILE_NAME, "catshield.lock");
        // Use const block for compile-time constant assertion
        const _: () = assert!(LOCK_RETRY_LIMIT >= 1 && LOCK_RETRY_LIMIT <= 10);
    }
}
