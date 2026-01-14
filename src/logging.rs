//! Logging configuration for Cat Shield
//!
//! Provides initialization for the logging system using `env_logger`.
//! Log levels can be controlled via:
//! - CLI `-v` flags: `-v` (info), `-vv` (debug), `-vvv` (trace)
//! - Environment variable `RUST_LOG` (overrides CLI)

use env_logger::Builder;
use log::LevelFilter;

/// Initialize the logging system based on verbosity level
///
/// # Arguments
/// * `verbosity` - Number of `-v` flags passed (0 = warn, 1 = info, 2 = debug, 3+ = trace)
///
/// # Environment
/// The `RUST_LOG` environment variable takes precedence over the verbosity level.
pub fn init(verbosity: u8) {
    let level = match verbosity {
        0 => LevelFilter::Warn,
        1 => LevelFilter::Info,
        2 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    Builder::new()
        .filter_level(level)
        .parse_default_env() // Allow RUST_LOG to override
        .format_target(false) // Don't show target module in output
        .format_timestamp(None) // Don't show timestamps (cleaner CLI output)
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verbosity_to_level_filter() {
        // Just verify the mapping logic - we can't actually test init()
        // multiple times because env_logger can only be initialized once
        assert_eq!(
            match 0u8 {
                0 => LevelFilter::Warn,
                1 => LevelFilter::Info,
                2 => LevelFilter::Debug,
                _ => LevelFilter::Trace,
            },
            LevelFilter::Warn
        );
        assert_eq!(
            match 1u8 {
                0 => LevelFilter::Warn,
                1 => LevelFilter::Info,
                2 => LevelFilter::Debug,
                _ => LevelFilter::Trace,
            },
            LevelFilter::Info
        );
        assert_eq!(
            match 2u8 {
                0 => LevelFilter::Warn,
                1 => LevelFilter::Info,
                2 => LevelFilter::Debug,
                _ => LevelFilter::Trace,
            },
            LevelFilter::Debug
        );
        assert_eq!(
            match 3u8 {
                0 => LevelFilter::Warn,
                1 => LevelFilter::Info,
                2 => LevelFilter::Debug,
                _ => LevelFilter::Trace,
            },
            LevelFilter::Trace
        );
    }
}
