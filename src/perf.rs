//! Performance profiling and monitoring for Cat Shield
//!
//! This module provides runtime performance measurement capabilities to verify
//! that Cat Shield meets its performance requirements:
//!
//! - Idle CPU < 2%
//! - Memory < 100MB baseline
//! - No visible lag/stutter
//! - Acceptable on 5-year-old hardware
//!
//! # Usage
//!
//! ```rust,ignore
//! use cat_shield::perf::{PerformanceMonitor, PerformanceSnapshot};
//!
//! // Create a monitor for the current process
//! let mut monitor = PerformanceMonitor::new();
//!
//! // Take snapshots at intervals
//! let snapshot = monitor.snapshot();
//! println!("CPU: {:.1}%, Memory: {:.1}MB", snapshot.cpu_percent, snapshot.memory_mb);
//!
//! // Collect statistics over time
//! for _ in 0..10 {
//!     std::thread::sleep(std::time::Duration::from_secs(1));
//!     monitor.snapshot();
//! }
//! let stats = monitor.statistics();
//! println!("{}", stats.summary());
//! ```
//!
//! # Benchmarks
//!
//! Run performance benchmarks with:
//! ```bash
//! cargo bench --bench performance
//! ```

use std::time::{Duration, Instant};

#[cfg(test)]
use sysinfo::{Pid, ProcessesToUpdate, System};

/// Performance requirements from Issue #164
pub mod requirements {
    /// Maximum acceptable CPU usage when idle (percentage)
    pub const MAX_IDLE_CPU_PERCENT: f64 = 2.0;

    /// Maximum acceptable baseline memory usage (MB)
    pub const MAX_MEMORY_MB: f64 = 100.0;

    /// Target frame time for smooth rendering (16.67ms = 60 FPS)
    pub const TARGET_FRAME_TIME_MS: f64 = 16.67;

    /// Maximum acceptable event processing latency (ms)
    pub const MAX_EVENT_LATENCY_MS: f64 = 10.0;
}

/// A snapshot of current performance metrics
#[derive(Debug, Clone)]
pub struct PerformanceSnapshot {
    /// CPU usage as a percentage (0-100)
    pub cpu_percent: f64,
    /// Memory usage in megabytes
    pub memory_mb: f64,
    /// Resident set size in bytes
    pub rss_bytes: u64,
    /// Virtual memory size in bytes
    pub virtual_memory_bytes: u64,
    /// Timestamp when the snapshot was taken
    pub timestamp: Instant,
}

impl Default for PerformanceSnapshot {
    fn default() -> Self {
        Self {
            cpu_percent: 0.0,
            memory_mb: 0.0,
            rss_bytes: 0,
            virtual_memory_bytes: 0,
            timestamp: Instant::now(),
        }
    }
}

impl PerformanceSnapshot {
    /// Check if this snapshot meets the idle CPU requirement
    pub fn meets_cpu_requirement(&self) -> bool {
        self.cpu_percent <= requirements::MAX_IDLE_CPU_PERCENT
    }

    /// Check if this snapshot meets the memory requirement
    pub fn meets_memory_requirement(&self) -> bool {
        self.memory_mb <= requirements::MAX_MEMORY_MB
    }

    /// Check if this snapshot meets all performance requirements
    pub fn meets_all_requirements(&self) -> bool {
        self.meets_cpu_requirement() && self.meets_memory_requirement()
    }
}

/// Performance statistics collected over a sampling period
#[derive(Debug, Clone)]
pub struct PerformanceStats {
    /// Average CPU usage over the sampling period
    pub avg_cpu_percent: f64,
    /// Peak CPU usage observed
    pub peak_cpu_percent: f64,
    /// Average memory usage in MB
    pub avg_memory_mb: f64,
    /// Peak memory usage in MB
    pub peak_memory_mb: f64,
    /// Number of samples collected
    pub sample_count: usize,
    /// Total sampling duration
    pub duration: Duration,
}

impl PerformanceStats {
    /// Check if the average metrics meet performance requirements
    pub fn meets_requirements(&self) -> bool {
        self.avg_cpu_percent <= requirements::MAX_IDLE_CPU_PERCENT
            && self.avg_memory_mb <= requirements::MAX_MEMORY_MB
    }

    /// Generate a summary report of the performance statistics
    pub fn summary(&self) -> String {
        let cpu_status = if self.avg_cpu_percent <= requirements::MAX_IDLE_CPU_PERCENT {
            "PASS"
        } else {
            "FAIL"
        };
        let mem_status = if self.avg_memory_mb <= requirements::MAX_MEMORY_MB {
            "PASS"
        } else {
            "FAIL"
        };

        format!(
            "Performance Summary ({:.1}s, {} samples)\n\
             ----------------------------------------\n\
             CPU:    Avg {:.2}%, Peak {:.2}% (target <{:.1}%) [{cpu_status}]\n\
             Memory: Avg {:.1}MB, Peak {:.1}MB (target <{:.1}MB) [{mem_status}]",
            self.duration.as_secs_f64(),
            self.sample_count,
            self.avg_cpu_percent,
            self.peak_cpu_percent,
            requirements::MAX_IDLE_CPU_PERCENT,
            self.avg_memory_mb,
            self.peak_memory_mb,
            requirements::MAX_MEMORY_MB,
        )
    }
}

/// Timer for measuring operation latency
#[derive(Debug)]
pub struct LatencyTimer {
    start: Instant,
    name: &'static str,
}

impl LatencyTimer {
    /// Start a new latency timer with a descriptive name
    #[inline]
    pub fn start(name: &'static str) -> Self {
        Self {
            start: Instant::now(),
            name,
        }
    }

    /// Stop the timer and return the elapsed duration
    #[inline]
    pub fn stop(self) -> Duration {
        self.start.elapsed()
    }

    /// Stop the timer and log if it exceeds the target latency
    #[inline]
    pub fn stop_and_log(self, target_ms: f64) {
        let elapsed = self.start.elapsed();
        let elapsed_ms = elapsed.as_secs_f64() * 1000.0;
        if elapsed_ms > target_ms {
            log::warn!(
                "{} took {:.2}ms (target: {:.2}ms)",
                self.name,
                elapsed_ms,
                target_ms
            );
        }
    }
}

/// A guard that logs when dropped if the operation took too long
pub struct LatencyGuard {
    timer: LatencyTimer,
    target_ms: f64,
}

impl LatencyGuard {
    /// Create a new latency guard
    #[inline]
    pub fn new(name: &'static str, target_ms: f64) -> Self {
        Self {
            timer: LatencyTimer::start(name),
            target_ms,
        }
    }
}

impl Drop for LatencyGuard {
    fn drop(&mut self) {
        let elapsed = self.timer.start.elapsed();
        let elapsed_ms = elapsed.as_secs_f64() * 1000.0;
        if elapsed_ms > self.target_ms {
            log::warn!(
                "{} took {:.2}ms (target: {:.2}ms)",
                self.timer.name,
                elapsed_ms,
                self.target_ms
            );
        }
    }
}

/// Macro to measure the latency of a block of code
///
/// # Example
///
/// ```rust,ignore
/// use cat_shield::measure_latency;
///
/// measure_latency!("timer_tick", 1.0, {
///     // Update timer display
///     format_duration(remaining_seconds);
/// });
/// ```
#[macro_export]
macro_rules! measure_latency {
    ($name:expr, $target_ms:expr, $block:expr) => {{
        let _guard = $crate::perf::LatencyGuard::new($name, $target_ms);
        $block
    }};
}

/// Frame timing tracker for detecting stutters
#[derive(Debug)]
pub struct FrameTimer {
    last_frame: Instant,
    target_frame_time: Duration,
    frame_count: u64,
    total_time: Duration,
    missed_frames: u64,
}

impl FrameTimer {
    /// Create a new frame timer with a target FPS
    pub fn new(target_fps: f64) -> Self {
        Self {
            last_frame: Instant::now(),
            target_frame_time: Duration::from_secs_f64(1.0 / target_fps),
            frame_count: 0,
            total_time: Duration::ZERO,
            missed_frames: 0,
        }
    }

    /// Mark the end of a frame and return the frame time
    pub fn end_frame(&mut self) -> Duration {
        let now = Instant::now();
        let frame_time = now.duration_since(self.last_frame);
        self.last_frame = now;
        self.frame_count += 1;
        self.total_time += frame_time;

        // Count missed frame targets
        if frame_time > self.target_frame_time * 2 {
            self.missed_frames += 1;
        }

        frame_time
    }

    /// Get the average frame time
    pub fn average_frame_time(&self) -> Duration {
        if self.frame_count == 0 {
            Duration::ZERO
        } else {
            // Use nanos to avoid truncation for very long sessions (frame_count > u32::MAX)
            Duration::from_nanos(self.total_time.as_nanos() as u64 / self.frame_count)
        }
    }

    /// Get the effective FPS based on average frame time
    pub fn effective_fps(&self) -> f64 {
        let avg = self.average_frame_time();
        if avg.is_zero() {
            0.0
        } else {
            1.0 / avg.as_secs_f64()
        }
    }

    /// Get the percentage of frames that missed their target
    pub fn missed_frame_percent(&self) -> f64 {
        if self.frame_count == 0 {
            0.0
        } else {
            (self.missed_frames as f64 / self.frame_count as f64) * 100.0
        }
    }
}

impl Default for FrameTimer {
    fn default() -> Self {
        // Default to 60 FPS target
        Self::new(60.0)
    }
}

/// Performance monitor for tracking CPU and memory usage
///
/// Uses the `sysinfo` crate to measure process-level metrics.
/// This is only available in test/benchmark configurations to avoid
/// adding runtime dependencies to the production binary.
#[cfg(test)]
pub struct PerformanceMonitor {
    system: System,
    pid: Pid,
    snapshots: Vec<PerformanceSnapshot>,
    start_time: Instant,
}

#[cfg(test)]
impl PerformanceMonitor {
    /// Create a new performance monitor for the current process
    pub fn new() -> Self {
        let pid = Pid::from_u32(std::process::id());
        let mut system = System::new();
        system.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);

        Self {
            system,
            pid,
            snapshots: Vec::new(),
            start_time: Instant::now(),
        }
    }

    /// Take a performance snapshot and record it
    pub fn snapshot(&mut self) -> PerformanceSnapshot {
        self.system
            .refresh_processes(ProcessesToUpdate::Some(&[self.pid]), true);

        let snapshot = if let Some(process) = self.system.process(self.pid) {
            PerformanceSnapshot {
                cpu_percent: process.cpu_usage() as f64,
                memory_mb: process.memory() as f64 / 1024.0 / 1024.0,
                rss_bytes: process.memory(),
                virtual_memory_bytes: process.virtual_memory(),
                timestamp: Instant::now(),
            }
        } else {
            PerformanceSnapshot::default()
        };

        self.snapshots.push(snapshot.clone());
        snapshot
    }

    /// Get statistics from all collected snapshots
    pub fn statistics(&self) -> PerformanceStats {
        if self.snapshots.is_empty() {
            return PerformanceStats {
                avg_cpu_percent: 0.0,
                peak_cpu_percent: 0.0,
                avg_memory_mb: 0.0,
                peak_memory_mb: 0.0,
                sample_count: 0,
                duration: Duration::ZERO,
            };
        }

        let total_cpu: f64 = self.snapshots.iter().map(|s| s.cpu_percent).sum();
        let total_mem: f64 = self.snapshots.iter().map(|s| s.memory_mb).sum();
        let count = self.snapshots.len() as f64;

        let peak_cpu = self
            .snapshots
            .iter()
            .map(|s| s.cpu_percent)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);

        let peak_mem = self
            .snapshots
            .iter()
            .map(|s| s.memory_mb)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);

        PerformanceStats {
            avg_cpu_percent: total_cpu / count,
            peak_cpu_percent: peak_cpu,
            avg_memory_mb: total_mem / count,
            peak_memory_mb: peak_mem,
            sample_count: self.snapshots.len(),
            duration: self.start_time.elapsed(),
        }
    }

    /// Reset the monitor, clearing all snapshots
    pub fn reset(&mut self) {
        self.snapshots.clear();
        self.start_time = Instant::now();
    }
}

#[cfg(test)]
impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_performance_snapshot_default() {
        let snapshot = PerformanceSnapshot::default();
        assert_eq!(snapshot.cpu_percent, 0.0);
        assert_eq!(snapshot.memory_mb, 0.0);
        assert!(snapshot.meets_all_requirements());
    }

    #[test]
    fn test_performance_snapshot_requirements() {
        let passing = PerformanceSnapshot {
            cpu_percent: 1.5,
            memory_mb: 80.0,
            ..Default::default()
        };
        assert!(passing.meets_cpu_requirement());
        assert!(passing.meets_memory_requirement());
        assert!(passing.meets_all_requirements());

        let failing_cpu = PerformanceSnapshot {
            cpu_percent: 5.0,
            memory_mb: 80.0,
            ..Default::default()
        };
        assert!(!failing_cpu.meets_cpu_requirement());
        assert!(failing_cpu.meets_memory_requirement());
        assert!(!failing_cpu.meets_all_requirements());

        let failing_memory = PerformanceSnapshot {
            cpu_percent: 1.0,
            memory_mb: 150.0,
            ..Default::default()
        };
        assert!(failing_memory.meets_cpu_requirement());
        assert!(!failing_memory.meets_memory_requirement());
        assert!(!failing_memory.meets_all_requirements());
    }

    #[test]
    fn test_performance_stats_summary() {
        let stats = PerformanceStats {
            avg_cpu_percent: 1.5,
            peak_cpu_percent: 3.0,
            avg_memory_mb: 75.0,
            peak_memory_mb: 90.0,
            sample_count: 100,
            duration: Duration::from_secs(10),
        };
        let summary = stats.summary();
        assert!(summary.contains("PASS"));
        assert!(summary.contains("1.5"));
        assert!(summary.contains("75.0"));
    }

    #[test]
    fn test_latency_timer() {
        let timer = LatencyTimer::start("test_operation");
        thread::sleep(Duration::from_millis(5));
        let elapsed = timer.stop();
        assert!(elapsed >= Duration::from_millis(5));
    }

    #[test]
    fn test_frame_timer() {
        let mut timer = FrameTimer::new(60.0);

        // Simulate a few frames with minimal sleep to avoid timing issues on CI
        for _ in 0..10 {
            thread::sleep(Duration::from_millis(1));
            timer.end_frame();
        }

        assert_eq!(timer.frame_count, 10);
        // Just verify FPS calculation works (not timing-dependent)
        let fps = timer.effective_fps();
        assert!(fps > 0.0, "FPS should be positive, was {fps}");
        // Verify average frame time is calculated
        assert!(!timer.average_frame_time().is_zero());
    }

    #[test]
    fn test_frame_timer_missed_frames() {
        let mut timer = FrameTimer::new(60.0);

        // Simulate some normal frames
        for _ in 0..5 {
            thread::sleep(Duration::from_millis(16));
            timer.end_frame();
        }

        // Simulate a long frame (stutter)
        thread::sleep(Duration::from_millis(50));
        timer.end_frame();

        assert!(timer.missed_frame_percent() > 0.0);
    }

    #[test]
    fn test_performance_monitor_new() {
        let monitor = super::PerformanceMonitor::new();
        assert!(monitor.snapshots.is_empty());
    }

    #[test]
    fn test_performance_monitor_snapshot() {
        let mut monitor = super::PerformanceMonitor::new();
        let snapshot = monitor.snapshot();

        // Memory should be non-zero for a running process
        assert!(snapshot.memory_mb > 0.0);
        assert!(snapshot.rss_bytes > 0);
    }

    #[test]
    fn test_performance_monitor_statistics() {
        let mut monitor = super::PerformanceMonitor::new();

        // Take a few snapshots
        for _ in 0..3 {
            monitor.snapshot();
            thread::sleep(Duration::from_millis(10));
        }

        let stats = monitor.statistics();
        assert_eq!(stats.sample_count, 3);
        assert!(stats.avg_memory_mb > 0.0);
        assert!(stats.peak_memory_mb >= stats.avg_memory_mb);
    }

    #[test]
    fn test_performance_monitor_reset() {
        let mut monitor = super::PerformanceMonitor::new();
        monitor.snapshot();
        monitor.snapshot();
        assert_eq!(monitor.snapshots.len(), 2);

        monitor.reset();
        assert!(monitor.snapshots.is_empty());
    }

    #[test]
    fn test_performance_stats_empty() {
        let monitor = super::PerformanceMonitor::new();
        let stats = monitor.statistics();
        assert_eq!(stats.sample_count, 0);
        assert_eq!(stats.avg_cpu_percent, 0.0);
        assert_eq!(stats.avg_memory_mb, 0.0);
    }
}
