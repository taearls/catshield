//! Duration formatting utilities for Cat Shield

use std::cell::RefCell;

// Cached formatted duration string
// Stores (cached_seconds, cached_string) to avoid re-formatting when unchanged
thread_local! {
    static CACHED_DURATION: RefCell<(u64, String)> = const { RefCell::new((u64::MAX, String::new())) };
}

/// Format seconds as a human-readable string with caching
///
/// Since the timer display is redrawn 60 times per second but the seconds value
/// only changes once per second, this caches the formatted string to avoid
/// 59 unnecessary allocations per second.
pub fn format_duration_cached(total_secs: u64) -> String {
    CACHED_DURATION.with(|cache| {
        let cached = cache.borrow();
        if cached.0 == total_secs {
            return cached.1.clone();
        }
        drop(cached);

        let formatted = format_duration(total_secs);
        *cache.borrow_mut() = (total_secs, formatted.clone());
        formatted
    })
}

/// Format seconds as a human-readable string (e.g., "1h 30m 45s")
pub fn format_duration(total_secs: u64) -> String {
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let secs = total_secs % 60;

    if hours > 0 {
        format!("{hours}h {minutes:02}m {secs:02}s")
    } else if minutes > 0 {
        format!("{minutes}m {secs:02}s")
    } else {
        format!("{secs}s")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration_seconds_only() {
        assert_eq!(format_duration(45), "45s");
        assert_eq!(format_duration(1), "1s");
    }

    #[test]
    fn test_format_duration_minutes_and_seconds() {
        assert_eq!(format_duration(90), "1m 30s");
        assert_eq!(format_duration(3599), "59m 59s");
    }

    #[test]
    fn test_format_duration_hours_minutes_seconds() {
        assert_eq!(format_duration(3600), "1h 00m 00s");
        assert_eq!(format_duration(3661), "1h 01m 01s");
        assert_eq!(format_duration(7200 + 1800 + 45), "2h 30m 45s");
    }

    #[test]
    fn test_format_duration_cached_returns_same_result() {
        // Cached version should return identical results to uncached
        assert_eq!(format_duration_cached(45), format_duration(45));
        assert_eq!(format_duration_cached(90), format_duration(90));
        assert_eq!(format_duration_cached(3661), format_duration(3661));
    }

    #[test]
    fn test_format_duration_cached_returns_cached_value() {
        // First call should compute and cache
        let first = format_duration_cached(120);
        assert_eq!(first, "2m 00s");

        // Second call with same value should return cached result
        let second = format_duration_cached(120);
        assert_eq!(second, "2m 00s");
        assert_eq!(first, second);

        // Call with different value should compute new result
        let third = format_duration_cached(121);
        assert_eq!(third, "2m 01s");
    }

    #[test]
    fn test_format_duration_cached_handles_value_changes() {
        // Simulate countdown: 5, 4, 3, 2, 1, 0
        for secs in (0..=5).rev() {
            let cached = format_duration_cached(secs);
            let expected = format_duration(secs);
            assert_eq!(cached, expected, "Mismatch at {secs} seconds");
        }
    }
}
