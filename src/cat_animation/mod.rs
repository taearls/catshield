//! Animated cat companion for the shield overlay
//!
//! This module provides an animated cat character that walks around, naps, stretches,
//! and performs various idle animations on the shield overlay. The cat is rendered
//! using procedural drawing (geometric primitives) for cross-platform compatibility.
//!
//! # Features
//!
//! - Multiple animation states: walking, sitting, sleeping, stretching, grooming, playing
//! - Natural movement patterns with random direction changes
//! - Configurable animation speed
//! - Can be enabled/disabled via configuration
//!
//! # Architecture
//!
//! The animation system is platform-agnostic. Each platform's overlay implementation
//! calls into this module to get the current cat state and then renders it using
//! their native drawing APIs.
//!
//! # Usage
//!
//! ```ignore
//! use cat_shield::cat_animation::{self, drawing};
//!
//! // Initialize with screen dimensions
//! cat_animation::init(1920.0, 1080.0, &config);
//!
//! // In render loop:
//! let frame = cat_animation::update(current_time);
//! let commands = drawing::generate_cat_draw_commands(&frame);
//!
//! // Render commands using platform-specific drawing API
//! for ellipse in &commands.ellipses {
//!     draw_ellipse(ellipse);
//! }
//! ```

pub mod drawing;

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::RwLock;

use crate::config::Config;

// Re-export commonly used types
pub use drawing::{generate_cat_draw_commands, CatDrawCommands, Color, Ellipse, Line, Point, Triangle};

/// Cat animation frame rate (updates per second)
const ANIMATION_FPS: f64 = 10.0;

/// Time between animation frame updates (in seconds)
pub const FRAME_UPDATE_INTERVAL: f64 = 1.0 / ANIMATION_FPS;

/// Cat body width in pixels
pub const CAT_BODY_WIDTH: f64 = 60.0;

/// Cat body height in pixels
pub const CAT_BODY_HEIGHT: f64 = 35.0;

/// Cat total width including tail
pub const CAT_TOTAL_WIDTH: f64 = 90.0;

/// Cat total height including ears
pub const CAT_TOTAL_HEIGHT: f64 = 55.0;

/// Walking speed in pixels per second (at 1.0x speed)
const BASE_WALK_SPEED: f64 = 60.0;

/// Minimum time in an animation state (seconds)
const MIN_STATE_DURATION: f64 = 3.0;

/// Maximum time in an animation state (seconds)
const MAX_STATE_DURATION: f64 = 12.0;

/// Margin from screen edges
const SCREEN_MARGIN: f64 = 50.0;

/// The different animation states the cat can be in
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatState {
    /// Cat is walking in a direction
    Walking,
    /// Cat is sitting and looking around
    Sitting,
    /// Cat is sleeping/napping
    Sleeping,
    /// Cat is stretching
    Stretching,
    /// Cat is grooming itself
    Grooming,
    /// Cat is playing with invisible objects
    Playing,
}

/// Direction the cat is facing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatDirection {
    /// Facing left
    Left,
    /// Facing right
    Right,
}

/// Current animation frame data for rendering
#[derive(Debug, Clone)]
pub struct CatFrame {
    /// Current animation state
    pub state: CatState,
    /// Direction the cat is facing
    pub direction: CatDirection,
    /// X position on screen (center of cat)
    pub x: f64,
    /// Y position on screen (bottom of cat)
    pub y: f64,
    /// Current frame number within the animation (0-based)
    pub frame_number: u32,
    /// Total frames in current animation cycle
    pub total_frames: u32,
    /// Whether the cat animation is enabled
    pub enabled: bool,
}

impl Default for CatFrame {
    fn default() -> Self {
        Self {
            state: CatState::Sitting,
            direction: CatDirection::Right,
            x: 200.0,
            y: 200.0,
            frame_number: 0,
            total_frames: 4,
            enabled: true,
        }
    }
}

/// Internal state for the cat animation system
struct CatAnimationState {
    /// Current cat state
    state: CatState,
    /// Direction facing
    direction: CatDirection,
    /// Position X
    x: f64,
    /// Position Y
    y: f64,
    /// Current frame within animation
    frame: u32,
    /// Time spent in current state
    state_time: f64,
    /// How long to stay in current state
    state_duration: f64,
    /// Screen width for bounds checking
    screen_width: f64,
    /// Screen height for bounds checking
    screen_height: f64,
    /// Animation speed multiplier
    speed: f64,
    /// Whether animations are enabled
    enabled: bool,
    /// Simple RNG state
    rng_state: u64,
}

impl CatAnimationState {
    fn new() -> Self {
        Self {
            state: CatState::Sitting,
            direction: CatDirection::Right,
            x: 200.0,
            y: 200.0,
            frame: 0,
            state_time: 0.0,
            state_duration: 5.0,
            screen_width: 1920.0,
            screen_height: 1080.0,
            speed: 1.0,
            enabled: true,
            rng_state: 12345, // Will be re-seeded on first update
        }
    }

    /// Simple xorshift PRNG
    fn next_random(&mut self) -> u64 {
        let mut x = self.rng_state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.rng_state = x;
        x
    }

    /// Get a random f64 in range [0, 1)
    fn random_f64(&mut self) -> f64 {
        (self.next_random() as f64) / (u64::MAX as f64)
    }

    /// Get a random f64 in range [min, max)
    fn random_range(&mut self, min: f64, max: f64) -> f64 {
        min + self.random_f64() * (max - min)
    }

    /// Pick a random animation state
    fn random_state(&mut self) -> CatState {
        match self.next_random() % 6 {
            0 => CatState::Walking,
            1 => CatState::Sitting,
            2 => CatState::Sleeping,
            3 => CatState::Stretching,
            4 => CatState::Grooming,
            _ => CatState::Playing,
        }
    }

    /// Get number of frames for current state
    fn frames_for_state(&self) -> u32 {
        match self.state {
            CatState::Walking => 4,
            CatState::Sitting => 2,
            CatState::Sleeping => 2,
            CatState::Stretching => 4,
            CatState::Grooming => 3,
            CatState::Playing => 4,
        }
    }

    /// Transition to a new state
    fn transition_to(&mut self, new_state: CatState) {
        self.state = new_state;
        self.frame = 0;
        self.state_time = 0.0;
        self.state_duration = self.random_range(MIN_STATE_DURATION, MAX_STATE_DURATION);

        // Walking gets longer duration since it takes time to move
        if new_state == CatState::Walking {
            self.state_duration *= 1.5;
            // 50% chance to change direction when starting to walk
            if self.random_f64() > 0.5 {
                self.direction = if self.direction == CatDirection::Left {
                    CatDirection::Right
                } else {
                    CatDirection::Left
                };
            }
        }
    }

    /// Update animation state by elapsed time
    fn update(&mut self, delta_time: f64) {
        if !self.enabled {
            return;
        }

        let effective_delta = delta_time * self.speed;
        self.state_time += effective_delta;

        // Update frame
        let frame_time = 1.0 / ANIMATION_FPS;
        let total_frames = self.frames_for_state();
        self.frame = ((self.state_time / frame_time) as u32) % total_frames;

        // Handle state-specific logic
        match self.state {
            CatState::Walking => {
                // Move the cat
                let move_amount = BASE_WALK_SPEED * effective_delta;
                match self.direction {
                    CatDirection::Left => self.x -= move_amount,
                    CatDirection::Right => self.x += move_amount,
                }

                // Bounds checking - turn around at edges
                let min_x = SCREEN_MARGIN + CAT_TOTAL_WIDTH / 2.0;
                let max_x = self.screen_width - SCREEN_MARGIN - CAT_TOTAL_WIDTH / 2.0;

                if self.x < min_x {
                    self.x = min_x;
                    self.direction = CatDirection::Right;
                } else if self.x > max_x {
                    self.x = max_x;
                    self.direction = CatDirection::Left;
                }
            }
            _ => {}
        }

        // Check if it's time to transition to a new state
        if self.state_time >= self.state_duration {
            let new_state = self.random_state();
            self.transition_to(new_state);
        }
    }

    /// Get the current frame for rendering
    fn get_frame(&self) -> CatFrame {
        CatFrame {
            state: self.state,
            direction: self.direction,
            x: self.x,
            y: self.y,
            frame_number: self.frame,
            total_frames: self.frames_for_state(),
            enabled: self.enabled,
        }
    }
}

// Global state for the cat animation
static CAT_STATE: RwLock<Option<CatAnimationState>> = RwLock::new(None);
static CAT_INITIALIZED: AtomicBool = AtomicBool::new(false);
static LAST_UPDATE_TIME: AtomicU64 = AtomicU64::new(0);

/// Initialize the cat animation system with screen dimensions and config
pub fn init(screen_width: f64, screen_height: f64, config: &Config) {
    let mut state = CatAnimationState::new();
    state.screen_width = screen_width;
    state.screen_height = screen_height;
    state.enabled = config.show_cat_animation();
    state.speed = config.cat_animation_speed();

    // Position cat near the bottom of the screen
    state.y = screen_height - SCREEN_MARGIN - CAT_TOTAL_HEIGHT;

    // Start at a random X position
    let time_seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(12345);
    state.rng_state = time_seed;

    let min_x = SCREEN_MARGIN + CAT_TOTAL_WIDTH / 2.0;
    let max_x = screen_width - SCREEN_MARGIN - CAT_TOTAL_WIDTH / 2.0;
    state.x = state.random_range(min_x, max_x);

    // Start with a random state
    let initial_state = state.random_state();
    state.transition_to(initial_state);

    if let Ok(mut guard) = CAT_STATE.write() {
        *guard = Some(state);
    }
    CAT_INITIALIZED.store(true, Ordering::SeqCst);
    LAST_UPDATE_TIME.store(0, Ordering::SeqCst);
}

/// Update the cat animation state
///
/// Call this each frame with the current timestamp in seconds.
/// Returns the current frame data for rendering.
pub fn update(current_time_secs: f64) -> CatFrame {
    if !CAT_INITIALIZED.load(Ordering::SeqCst) {
        return CatFrame::default();
    }

    let last_time_bits = LAST_UPDATE_TIME.load(Ordering::SeqCst);
    let last_time = f64::from_bits(last_time_bits);

    let delta_time = if last_time_bits == 0 {
        FRAME_UPDATE_INTERVAL // First update
    } else {
        (current_time_secs - last_time).max(0.0).min(1.0) // Clamp to prevent huge jumps
    };

    LAST_UPDATE_TIME.store(current_time_secs.to_bits(), Ordering::SeqCst);

    if let Ok(mut guard) = CAT_STATE.write() {
        if let Some(state) = guard.as_mut() {
            state.update(delta_time);
            return state.get_frame();
        }
    }

    CatFrame::default()
}

/// Get the current cat frame without updating
pub fn get_current_frame() -> CatFrame {
    if let Ok(guard) = CAT_STATE.read() {
        if let Some(state) = guard.as_ref() {
            return state.get_frame();
        }
    }
    CatFrame::default()
}

/// Enable or disable the cat animation
pub fn set_enabled(enabled: bool) {
    if let Ok(mut guard) = CAT_STATE.write() {
        if let Some(state) = guard.as_mut() {
            state.enabled = enabled;
        }
    }
}

/// Set the animation speed multiplier
pub fn set_speed(speed: f64) {
    let clamped = speed.clamp(
        crate::config::MIN_CAT_ANIMATION_SPEED,
        crate::config::MAX_CAT_ANIMATION_SPEED,
    );
    if let Ok(mut guard) = CAT_STATE.write() {
        if let Some(state) = guard.as_mut() {
            state.speed = clamped;
        }
    }
}

/// Check if cat animation is enabled
pub fn is_enabled() -> bool {
    if let Ok(guard) = CAT_STATE.read() {
        if let Some(state) = guard.as_ref() {
            return state.enabled;
        }
    }
    false
}

/// Clean up the cat animation state
pub fn cleanup() {
    if let Ok(mut guard) = CAT_STATE.write() {
        *guard = None;
    }
    CAT_INITIALIZED.store(false, Ordering::SeqCst);
    LAST_UPDATE_TIME.store(0, Ordering::SeqCst);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cat_frame_default() {
        let frame = CatFrame::default();
        assert_eq!(frame.state, CatState::Sitting);
        assert_eq!(frame.direction, CatDirection::Right);
        assert!(frame.enabled);
    }

    #[test]
    fn test_cat_state_transitions() {
        let mut state = CatAnimationState::new();
        state.enabled = true;

        // Transition to walking
        state.transition_to(CatState::Walking);
        assert_eq!(state.state, CatState::Walking);
        assert_eq!(state.frame, 0);
        assert_eq!(state.state_time, 0.0);

        // Transition to sleeping
        state.transition_to(CatState::Sleeping);
        assert_eq!(state.state, CatState::Sleeping);
    }

    #[test]
    fn test_cat_walking_movement() {
        let mut state = CatAnimationState::new();
        state.enabled = true;
        state.screen_width = 1920.0;
        state.screen_height = 1080.0;
        state.x = 500.0;
        state.direction = CatDirection::Right;
        // Manually set walking state without transition_to to avoid random direction change
        state.state = CatState::Walking;
        state.frame = 0;
        state.state_time = 0.0;
        state.state_duration = 10.0;

        let initial_x = state.x;
        state.update(1.0); // 1 second

        // Cat should have moved right
        assert!(state.x > initial_x);
    }

    #[test]
    fn test_cat_bounds_checking() {
        let mut state = CatAnimationState::new();
        state.enabled = true;
        state.screen_width = 1920.0;
        state.x = 50.0; // Near left edge
        state.direction = CatDirection::Left;
        state.transition_to(CatState::Walking);

        // Update multiple times to force boundary
        for _ in 0..10 {
            state.update(0.5);
        }

        // Should have turned around
        let min_x = SCREEN_MARGIN + CAT_TOTAL_WIDTH / 2.0;
        assert!(state.x >= min_x);
    }

    #[test]
    fn test_frames_for_state() {
        // Each state should have at least 2 frames
        for cat_state in [
            CatState::Walking,
            CatState::Sitting,
            CatState::Sleeping,
            CatState::Stretching,
            CatState::Grooming,
            CatState::Playing,
        ] {
            let mut test_state = CatAnimationState::new();
            test_state.state = cat_state;
            assert!(test_state.frames_for_state() >= 2);
        }
    }

    #[test]
    fn test_random_range() {
        let mut state = CatAnimationState::new();
        for _ in 0..100 {
            let val = state.random_range(10.0, 20.0);
            assert!(val >= 10.0 && val < 20.0);
        }
    }

    #[test]
    fn test_disabled_cat_no_updates() {
        let mut state = CatAnimationState::new();
        state.enabled = false;
        let initial_time = state.state_time;

        state.update(1.0);

        // State time should not change when disabled
        assert_eq!(state.state_time, initial_time);
    }

    #[test]
    fn test_speed_multiplier() {
        let mut state1 = CatAnimationState::new();
        state1.enabled = true;
        state1.speed = 1.0;
        state1.transition_to(CatState::Walking);

        let mut state2 = CatAnimationState::new();
        state2.enabled = true;
        state2.speed = 2.0;
        state2.x = state1.x;
        state2.direction = state1.direction;
        state2.transition_to(CatState::Walking);

        let x1_before = state1.x;
        let x2_before = state2.x;

        state1.update(1.0);
        state2.update(1.0);

        let x1_delta = (state1.x - x1_before).abs();
        let x2_delta = (state2.x - x2_before).abs();

        // State2 should move roughly twice as far
        assert!((x2_delta - x1_delta * 2.0).abs() < 1.0);
    }
}
