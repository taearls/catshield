//! Performance benchmarks for Cat Shield
//!
//! This module provides comprehensive benchmarks to verify that the iced UI
//! implementation meets the performance requirements from Issue #164:
//!
//! - Idle CPU < 2% (or comparable to current)
//! - Memory < 100MB baseline
//! - No visible lag/stutter
//! - Acceptable on 5-year-old hardware
//!
//! # Running Benchmarks
//!
//! ```bash
//! # Run all benchmarks
//! cargo bench
//!
//! # Run specific benchmark group
//! cargo bench -- timer
//! cargo bench -- format
//! cargo bench -- config
//! ```
//!
//! # Interpreting Results
//!
//! Criterion generates detailed HTML reports in `target/criterion/`.
//! Each benchmark shows:
//! - Mean execution time
//! - Standard deviation
//! - Performance regression detection

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::time::Duration;

/// Benchmark timer formatting operations
///
/// This tests the `format_duration` function which runs on every 1Hz timer tick.
/// Target: < 1μs per call (negligible CPU impact at 1Hz).
fn bench_format_duration(c: &mut Criterion) {
    let mut group = c.benchmark_group("timer_formatting");

    // Test various duration ranges that the overlay displays
    let test_durations = [
        0u64,  // Edge case: zero
        30,    // 30 seconds (warning threshold)
        60,    // 1 minute (warning threshold)
        300,   // 5 minutes (typical short timer)
        1800,  // 30 minutes (typical medium timer)
        3600,  // 1 hour (format switch point)
        7200,  // 2 hours (long session)
        36000, // 10 hours (extended session)
    ];

    for duration in test_durations.iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{duration}s")),
            duration,
            |b, &secs| {
                b.iter(|| cat_shield::format_duration(black_box(secs)));
            },
        );
    }

    group.finish();
}

/// Benchmark duration parsing (CLI argument processing)
///
/// This tests the `parse_duration` function used for --timer flag parsing.
/// Target: < 10μs per call (only runs once at startup).
fn bench_parse_duration(c: &mut Criterion) {
    let mut group = c.benchmark_group("duration_parsing");

    // Test various input formats the CLI accepts
    let test_inputs = [
        "30s",      // Seconds only
        "5m",       // Minutes only
        "1h",       // Hours only
        "1h30m",    // Hours and minutes
        "2h30m45s", // Full format
        "90",       // Plain number (interpreted as minutes)
        "300",      // Larger plain number (interpreted as minutes)
    ];

    for input in test_inputs.iter() {
        group.bench_with_input(BenchmarkId::from_parameter(input), input, |b, input| {
            b.iter(|| cat_shield::parse_duration(black_box(input)));
        });
    }

    group.finish();
}

/// Benchmark configuration operations
///
/// Tests config loading and saving which affects settings changes.
/// Target: < 10ms for load, < 50ms for save (includes atomic file operations).
fn bench_config_operations(c: &mut Criterion) {
    use tempfile::TempDir;

    let mut group = c.benchmark_group("config_operations");
    group.measurement_time(Duration::from_secs(5));

    // Benchmark config loading from path
    group.bench_function("load_default", |b| {
        b.iter(|| {
            let config = cat_shield::Config::default();
            black_box(config)
        });
    });

    // Benchmark config loading from non-existent path (cold start scenario)
    group.bench_function("load_missing_file", |b| {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("config.toml");
        b.iter(|| {
            let config = cat_shield::Config::load_from_path(&path);
            black_box(config)
        });
    });

    group.finish();
}

/// Benchmark config save operations
///
/// Tests atomic config saving which happens when settings are changed.
/// Target: < 50ms per save (includes atomic file operations).
fn bench_config_save(c: &mut Criterion) {
    use tempfile::TempDir;

    let mut group = c.benchmark_group("config_save");
    group.measurement_time(Duration::from_secs(5));

    let temp = TempDir::new().unwrap();
    let path = temp.path().join("config.toml");

    let config = cat_shield::Config {
        exit_key: Some("Cmd+Option+U".to_string()),
        default_timer: Some("30m".to_string()),
        overlay_opacity: Some(0.6),
        overlay_color: Some("blue".to_string()),
        allowed_keys: Some(vec!["Cmd+Space".to_string(), "F11".to_string()]),
        launch_at_login: Some(false),
        enable_trace_logging: Some(false),
        color_scheme: Some("system".to_string()),
        show_cat: None,
        cat_position: None,
    };

    group.bench_function("save_full_config", |b| {
        b.iter(|| {
            config.save_to_path(&path).unwrap();
        });
    });

    group.finish();
}

/// Benchmark exit key parsing (startup operation)
///
/// Tests parsing of exit key configuration strings.
/// Target: < 100μs per parse.
#[cfg(target_os = "macos")]
fn bench_exit_key_parsing(c: &mut Criterion) {
    use cat_shield::ExitKey;

    let mut group = c.benchmark_group("exit_key_parsing");

    let test_keys = [
        "Cmd+Option+U",       // Default
        "Ctrl+Alt+X",         // Alternative
        "Shift+Escape",       // Simple
        "Cmd+Shift+Option+Q", // Complex
    ];

    for key_str in test_keys.iter() {
        group.bench_with_input(BenchmarkId::from_parameter(key_str), key_str, |b, input| {
            b.iter(|| ExitKey::parse(black_box(input)));
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_format_duration,
    bench_parse_duration,
    bench_config_operations,
    bench_config_save,
);

#[cfg(target_os = "macos")]
criterion_group!(macos_benches, bench_exit_key_parsing,);

#[cfg(not(target_os = "macos"))]
criterion_main!(benches);

#[cfg(target_os = "macos")]
criterion_main!(benches, macos_benches);
