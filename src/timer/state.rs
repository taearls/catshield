//! Auto-exit timer state management for Cat Shield

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// Global timer state for auto-exit feature
pub static AUTO_EXIT_ENABLED: AtomicBool = AtomicBool::new(false);
pub static AUTO_EXIT_START_TIME: AtomicU64 = AtomicU64::new(0);
pub static AUTO_EXIT_DURATION_SECS: AtomicU64 = AtomicU64::new(0);
pub static WARNING_SHOWN: AtomicBool = AtomicBool::new(false);

/// Initialize the auto-exit timer with the specified duration in seconds
pub fn init_auto_exit_timer(duration_secs: u64) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    AUTO_EXIT_START_TIME.store(now, Ordering::SeqCst);
    AUTO_EXIT_DURATION_SECS.store(duration_secs, Ordering::SeqCst);
    AUTO_EXIT_ENABLED.store(true, Ordering::SeqCst);
}

/// Get the remaining seconds until auto-exit, or 0 if expired
pub fn get_remaining_seconds() -> u64 {
    if !AUTO_EXIT_ENABLED.load(Ordering::SeqCst) {
        return u64::MAX;
    }

    let start = AUTO_EXIT_START_TIME.load(Ordering::SeqCst);
    let duration = AUTO_EXIT_DURATION_SECS.load(Ordering::SeqCst);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let elapsed = now.saturating_sub(start);
    duration.saturating_sub(elapsed)
}
