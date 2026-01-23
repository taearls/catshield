//! Cat Shield - A cat-proof screen overlay for macOS, Windows, and Linux
//!
//! Creates a semi-transparent overlay that:
//! - Blocks all keyboard and mouse input
//! - Keeps the machine awake
//! - Click and hold close button (3 seconds) to exit
//! - Or unlock with configurable keyboard shortcut (default: Cmd+Option+U on macOS, Win+Alt+U on Windows)
//! - Optional timer-based auto-exit
//!
//! Usage: Run the application, and it will immediately activate the shield.
//! Click and hold the X button in the top-right corner for 3 seconds to exit.
//!
//! Timer: Use --timer or -t to set auto-exit timer:
//!   cat_shield --timer 30m      # Exit after 30 minutes
//!   cat_shield --timer 2h       # Exit after 2 hours
//!   cat_shield -t 45m           # Short form
//!
//! Exit Key: Use --exit-key or -e to set custom exit shortcut:
//!   cat_shield --exit-key "Cmd+Shift+Q"
//!   cat_shield --exit-key "Ctrl+Option+Escape"
//!   cat_shield -e "Cmd+Shift+X"
//!
//! Config File: Persistent settings can be stored in ~/.config/catshield/config.toml:
//!   exit_key = "Cmd+Option+U"
//!
//! Note: Keyboard shortcuts require Accessibility permissions on macOS.
//! Go to System Preferences → Security & Privacy → Privacy → Accessibility
//! and add this application.

// =============================================================================
// Linux Entry Point and Event Loop
// =============================================================================

#[cfg(target_os = "linux")]
use cat_shield::config::{has_immediate_start_args_linux, LinuxArgs};
#[cfg(target_os = "linux")]
use cat_shield::lock::{acquire_instance_lock, release_instance_lock, LockResult};
#[cfg(target_os = "linux")]
use cat_shield::platform::{
    detect_display_server, display_info_summary, should_proceed, DisplayServer,
    LinuxOverlayWindow, LinuxPowerManager, LinuxSystemTray, MenuCallback, TrayMenuAction,
    WaylandOverlayWindow,
};
#[cfg(target_os = "linux")]
use cat_shield::platform::traits::{OverlayWindow, PowerManager, SystemTray};
#[cfg(target_os = "linux")]
use cat_shield::Config;
#[cfg(target_os = "linux")]
use clap::Parser;
#[cfg(target_os = "linux")]
use std::process;
#[cfg(target_os = "linux")]
use std::sync::atomic::{AtomicBool, Ordering};

/// Global flag to signal application shutdown on Linux
#[cfg(target_os = "linux")]
static LINUX_SHOULD_QUIT: AtomicBool = AtomicBool::new(false);

/// Global flag indicating shield is active on Linux
#[cfg(target_os = "linux")]
static LINUX_SHIELD_ACTIVE: AtomicBool = AtomicBool::new(false);

/// Linux main entry point
#[cfg(target_os = "linux")]
fn main() {
    // Parse CLI arguments
    let args = LinuxArgs::parse();

    // Initialize logging with verbosity level from CLI args
    cat_shield::logging::init(args.verbose);

    // Check for existing instance (single-instance enforcement)
    match acquire_instance_lock() {
        LockResult::Acquired => {
            // Successfully acquired lock, continue startup
        }
        LockResult::AlreadyRunning(pid) => {
            log::warn!("Cat Shield is already running (PID: {})", pid);
            log::warn!("Look for the Cat Shield icon in your system tray.");
            process::exit(0);
        }
        LockResult::Error(e) => {
            log::warn!("Could not check for existing instance: {}", e);
            // Continue anyway - lock check is best-effort
        }
    }

    // Detect display server and check compatibility
    let display_info = detect_display_server();
    log::info!("{}", display_info_summary(&display_info));

    // Check if we should proceed based on display server and user preferences
    let effective_display = match should_proceed(&display_info, args.wayland_mode) {
        Ok(server) => server,
        Err(msg) => {
            log::error!("{}", msg);
            release_instance_lock();
            process::exit(1);
        }
    };

    log::info!("Using display server: {}", effective_display);

    // Load config file
    let config = Config::load();

    log::info!("🐱 CAT SHIELD 🛡️");
    log::info!("════════════════════════════════════════");

    // Initialize Linux components based on detected display server
    let mut tray = LinuxSystemTray::new();
    let power_manager = LinuxPowerManager::new();

    // Set up system tray
    if let Err(e) = tray.setup() {
        log::error!("Failed to set up system tray: {}", e);
        release_instance_lock();
        process::exit(1);
    }
    log::info!("✓ System tray active");

    // Set up tray menu callback
    let menu_callback: MenuCallback = Box::new(move |action| {
        match action {
            TrayMenuAction::StartProtection => {
                log::info!("Starting protection...");
                if !LINUX_SHIELD_ACTIVE.load(Ordering::SeqCst) {
                    LINUX_SHIELD_ACTIVE.store(true, Ordering::SeqCst);
                }
            }
            TrayMenuAction::StopProtection => {
                log::info!("Stopping protection...");
                LINUX_SHIELD_ACTIVE.store(false, Ordering::SeqCst);
            }
            TrayMenuAction::OpenSettings => {
                log::info!("Settings requested (not yet implemented on Linux)");
            }
            TrayMenuAction::ShowAbout => {
                log::info!("Cat Shield - A cat-proof screen overlay");
                log::info!("Display server: {}", effective_display);
            }
            TrayMenuAction::Quit => {
                log::info!("Quit requested");
                LINUX_SHOULD_QUIT.store(true, Ordering::SeqCst);
            }
        }
    });
    LinuxSystemTray::set_callback(menu_callback);

    log::info!("════════════════════════════════════════");

    // Check if we should start shield immediately (CLI args provided)
    if has_immediate_start_args_linux(&args) {
        log::info!("Immediate start mode active");
        LINUX_SHIELD_ACTIVE.store(true, Ordering::SeqCst);
    } else {
        log::info!("System tray mode active");
        log::info!("Right-click the Cat Shield icon in your system tray to access options.");
        log::info!("Use 'Start Protection' to activate the shield.");
    }

    // Run the event loop based on display server
    match effective_display {
        DisplayServer::X11 | DisplayServer::XWayland => {
            run_linux_x11_event_loop(&mut tray, &power_manager, config.opacity());
        }
        DisplayServer::WaylandNative | DisplayServer::WaylandLimited => {
            run_linux_wayland_event_loop(&mut tray, &power_manager, config.opacity());
        }
        DisplayServer::Unknown => {
            log::error!("Unknown display server, cannot proceed");
            release_instance_lock();
            process::exit(1);
        }
    }

    // Cleanup
    log::info!("Shutting down...");

    // Remove tray icon
    LinuxSystemTray::clear_callback();
    tray.remove();

    // Release single-instance lock
    release_instance_lock();

    log::info!("👋 Cat Shield closed. Goodbye!");
}

/// Run the event loop for X11/XWayland
#[cfg(target_os = "linux")]
fn run_linux_x11_event_loop(
    tray: &mut LinuxSystemTray,
    power_manager: &LinuxPowerManager,
    opacity: f64,
) {
    use cat_shield::platform::types::SleepAssertion;

    let mut overlay = LinuxOverlayWindow::new();
    let mut sleep_assertion: Option<SleepAssertion> = None;
    let mut shield_was_active = false;

    // Create overlay window (but don't show yet)
    if let Err(e) = overlay.create() {
        log::error!("Failed to create X11 overlay window: {}", e);
        return;
    }
    log::info!("✓ X11 overlay window created");
    overlay.set_opacity(opacity);

    // Main event loop
    loop {
        // Check if we should quit
        if LINUX_SHOULD_QUIT.load(Ordering::SeqCst) {
            break;
        }

        // Handle shield activation/deactivation
        let shield_active = LINUX_SHIELD_ACTIVE.load(Ordering::SeqCst);

        if shield_active && !shield_was_active {
            // Activating shield
            log::info!("Activating shield (X11)...");

            // Prevent sleep
            match power_manager.prevent_sleep() {
                Ok(assertion) => {
                    sleep_assertion = Some(assertion);
                }
                Err(e) => {
                    log::warn!("Failed to prevent sleep: {}", e);
                }
            }

            // Show overlay
            overlay.show();
            log::info!("✓ Overlay visible");

            // Update tray state
            tray.set_state(true);

            log::info!("════════════════════════════════════════");
            log::info!("🛡️  SHIELD ACTIVE (X11)");
            log::info!("════════════════════════════════════════");

            shield_was_active = true;
        } else if !shield_active && shield_was_active {
            // Deactivating shield
            log::info!("Deactivating shield...");

            // Hide overlay
            overlay.hide();

            // Allow sleep
            if let Some(assertion) = sleep_assertion.take() {
                if let Err(e) = power_manager.allow_sleep(assertion) {
                    log::warn!("Failed to release sleep assertion: {}", e);
                }
            }

            // Update tray state
            tray.set_state(false);

            log::info!("════════════════════════════════════════");
            log::info!("Shield deactivated");
            log::info!("════════════════════════════════════════");

            shield_was_active = false;
        }

        // Process X11 events
        match overlay.process_events() {
            Ok(should_close) => {
                if should_close {
                    log::info!("Exit key detected, deactivating shield");
                    LINUX_SHIELD_ACTIVE.store(false, Ordering::SeqCst);
                }
            }
            Err(e) => {
                log::debug!("Error processing X11 events: {}", e);
            }
        }

        // Small sleep to prevent busy-waiting
        std::thread::sleep(std::time::Duration::from_millis(16));
    }

    // Cleanup
    if let Some(assertion) = sleep_assertion.take() {
        if let Err(e) = power_manager.allow_sleep(assertion) {
            log::warn!("Failed to release sleep assertion: {}", e);
        }
    }
    overlay.close();
}

/// Run the event loop for native Wayland
#[cfg(target_os = "linux")]
fn run_linux_wayland_event_loop(
    tray: &mut LinuxSystemTray,
    power_manager: &LinuxPowerManager,
    opacity: f64,
) {
    use cat_shield::platform::types::SleepAssertion;

    let mut overlay = WaylandOverlayWindow::new();
    let mut sleep_assertion: Option<SleepAssertion> = None;
    let mut shield_was_active = false;

    // Create overlay window (but don't show yet)
    if let Err(e) = overlay.create() {
        log::error!("Failed to create Wayland overlay window: {}", e);
        return;
    }
    log::info!("✓ Wayland overlay window created");
    overlay.set_opacity(opacity);

    // Main event loop
    loop {
        // Check if we should quit
        if LINUX_SHOULD_QUIT.load(Ordering::SeqCst) {
            break;
        }

        // Handle shield activation/deactivation
        let shield_active = LINUX_SHIELD_ACTIVE.load(Ordering::SeqCst);

        if shield_active && !shield_was_active {
            // Activating shield
            log::info!("Activating shield (Wayland)...");

            // Prevent sleep
            match power_manager.prevent_sleep() {
                Ok(assertion) => {
                    sleep_assertion = Some(assertion);
                }
                Err(e) => {
                    log::warn!("Failed to prevent sleep: {}", e);
                }
            }

            // Show overlay
            overlay.show();
            log::info!("✓ Overlay visible");

            // Update tray state
            tray.set_state(true);

            log::info!("════════════════════════════════════════");
            log::info!("🛡️  SHIELD ACTIVE (Wayland)");
            log::info!("Note: Pointer clicks can bypass the overlay");
            log::info!("════════════════════════════════════════");

            shield_was_active = true;
        } else if !shield_active && shield_was_active {
            // Deactivating shield
            log::info!("Deactivating shield...");

            // Hide overlay
            overlay.hide();

            // Allow sleep
            if let Some(assertion) = sleep_assertion.take() {
                if let Err(e) = power_manager.allow_sleep(assertion) {
                    log::warn!("Failed to release sleep assertion: {}", e);
                }
            }

            // Update tray state
            tray.set_state(false);

            log::info!("════════════════════════════════════════");
            log::info!("Shield deactivated");
            log::info!("════════════════════════════════════════");

            shield_was_active = false;
        }

        // Process Wayland events
        match overlay.process_events() {
            Ok(should_close) => {
                if should_close {
                    log::info!("Exit key detected, deactivating shield");
                    LINUX_SHIELD_ACTIVE.store(false, Ordering::SeqCst);
                }
            }
            Err(e) => {
                log::debug!("Error processing Wayland events: {}", e);
            }
        }

        // Small sleep to prevent busy-waiting
        std::thread::sleep(std::time::Duration::from_millis(16));
    }

    // Cleanup
    if let Some(assertion) = sleep_assertion.take() {
        if let Err(e) = power_manager.allow_sleep(assertion) {
            log::warn!("Failed to release sleep assertion: {}", e);
        }
    }
    overlay.close();
}

// =============================================================================
// Windows Entry Point and Event Loop
// =============================================================================

#[cfg(target_os = "windows")]
use cat_shield::config::Config;
#[cfg(target_os = "windows")]
use cat_shield::input::keycodes::{key_from_name, key_to_keycode};
#[cfg(target_os = "windows")]
use cat_shield::lock::{acquire_instance_lock, release_instance_lock, LockResult};
#[cfg(target_os = "windows")]
use cat_shield::platform::traits::{InputBlocker, OverlayWindow, PowerManager, SystemTray};
#[cfg(target_os = "windows")]
use cat_shield::platform::{
    set_exit_key_config, MenuCallback, TrayMenuAction, WindowsInputBlocker, WindowsOverlayWindow,
    WindowsPowerManager, WindowsSystemTray,
};
// Note: format_duration will be used when timer display is added to Windows overlay
#[cfg(target_os = "windows")]
#[allow(unused_imports)]
use cat_shield::timer::format_duration;
#[cfg(target_os = "windows")]
use std::process;
#[cfg(target_os = "windows")]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(target_os = "windows")]
use std::sync::Arc;

/// Global flag to signal application shutdown on Windows
#[cfg(target_os = "windows")]
static WINDOWS_SHOULD_QUIT: AtomicBool = AtomicBool::new(false);

/// Global flag indicating shield is active on Windows
#[cfg(target_os = "windows")]
static WINDOWS_SHIELD_ACTIVE: AtomicBool = AtomicBool::new(false);

/// Default exit key for Windows: Win+Alt+U (similar to macOS Cmd+Option+U)
#[cfg(target_os = "windows")]
const WINDOWS_DEFAULT_EXIT_KEY: &str = "Win+Alt+U";

/// Parse a Windows exit key string and configure the keyboard hook.
///
/// Supports modifier names: Win/Windows, Alt/Option, Shift, Ctrl/Control
/// Key names: Same as macOS (A-Z, 0-9, F1-F12, Escape, etc.)
#[cfg(target_os = "windows")]
fn parse_and_set_windows_exit_key(input: &str) -> Result<String, String> {
    let input = input.trim();
    if input.is_empty() {
        return Err("Exit key cannot be empty".to_string());
    }

    let parts: Vec<&str> = input.split('+').map(|s| s.trim()).collect();
    if parts.is_empty() {
        return Err("Invalid key combination format".to_string());
    }

    let mut requires_win = false;
    let mut requires_alt = false;
    let mut requires_shift = false;
    let mut requires_ctrl = false;
    let mut key_name: Option<&str> = None;

    for part in &parts {
        let lower = part.to_lowercase();
        match lower.as_str() {
            "win" | "windows" | "super" | "cmd" | "command" | "⊞" => requires_win = true,
            "alt" | "opt" | "option" | "⌥" => requires_alt = true,
            "shift" | "⇧" => requires_shift = true,
            "ctrl" | "control" | "⌃" => requires_ctrl = true,
            _ => {
                if key_name.is_some() {
                    return Err(format!("Multiple keys specified in '{}'", input));
                }
                key_name = Some(part);
            }
        }
    }

    let key_name = key_name.ok_or("No key specified in combination")?;

    // Convert key name to platform-agnostic Key, then to Windows virtual key code
    let key = key_from_name(key_name).ok_or_else(|| {
        format!(
            "Unknown key: '{}'. Valid keys: A-Z, 0-9, F1-F12, Escape, Return, Tab, Space, Delete, arrow keys",
            key_name
        )
    })?;

    let vk_code = key_to_keycode(key)
        .ok_or_else(|| format!("Key '{}' has no Windows keycode mapping", key_name))?;

    // Require at least one modifier
    if !requires_win && !requires_alt && !requires_shift && !requires_ctrl {
        return Err("At least one modifier required (Win, Alt, Shift, or Ctrl)".to_string());
    }

    // Configure the keyboard hook with the exit key
    set_exit_key_config(
        vk_code,
        requires_win,
        requires_alt,
        requires_shift,
        requires_ctrl,
    );

    Ok(input.to_string())
}

/// Windows main entry point
#[cfg(target_os = "windows")]
fn main() {
    // Initialize logger with default verbosity (warn level)
    cat_shield::logging::init(0);

    // Check for existing instance (single-instance enforcement)
    match acquire_instance_lock() {
        LockResult::Acquired => {
            // Successfully acquired lock, continue startup
        }
        LockResult::AlreadyRunning(pid) => {
            log::warn!("Cat Shield is already running (PID: {})", pid);
            log::warn!("Look for the Cat Shield icon in your system tray.");
            process::exit(0);
        }
        LockResult::Error(e) => {
            log::warn!("Could not check for existing instance: {}", e);
            // Continue anyway - lock check is best-effort
        }
    }

    // Load config file
    let config = Config::load();

    // Determine exit key: config file > default
    let exit_key_str = config
        .exit_key
        .as_deref()
        .unwrap_or(WINDOWS_DEFAULT_EXIT_KEY);

    let exit_key_display = match parse_and_set_windows_exit_key(exit_key_str) {
        Ok(display) => display,
        Err(e) => {
            log::warn!("Invalid exit_key in config: {}", e);
            log::warn!("Using default: {}", WINDOWS_DEFAULT_EXIT_KEY);
            parse_and_set_windows_exit_key(WINDOWS_DEFAULT_EXIT_KEY)
                .expect("Default exit key should be valid")
        }
    };

    log::info!("🐱 CAT SHIELD 🛡️");
    log::info!("════════════════════════════════════════");

    // Initialize Windows components
    let mut tray = WindowsSystemTray::new();
    let mut overlay = WindowsOverlayWindow::new();
    let mut input_blocker = WindowsInputBlocker::new();
    let power_manager = WindowsPowerManager::new();

    // Set up system tray
    if let Err(e) = tray.setup() {
        log::error!("Failed to set up system tray: {}", e);
        release_instance_lock();
        process::exit(1);
    }
    log::info!("✓ System tray active");

    // Set up tray menu callback
    let exit_key_for_callback = exit_key_display.clone();
    let menu_callback: MenuCallback = Box::new(move |action| {
        match action {
            TrayMenuAction::StartProtection => {
                log::info!("Starting protection...");
                // Signal that we want to activate the shield
                // The main loop will handle the actual activation
                if !WINDOWS_SHIELD_ACTIVE.load(Ordering::SeqCst) {
                    WINDOWS_SHIELD_ACTIVE.store(true, Ordering::SeqCst);
                }
            }
            TrayMenuAction::StopProtection => {
                log::info!("Stopping protection...");
                WINDOWS_SHIELD_ACTIVE.store(false, Ordering::SeqCst);
            }
            TrayMenuAction::OpenSettings => {
                log::info!("Settings requested (not yet implemented on Windows)");
                // TODO: Implement settings window for Windows
            }
            TrayMenuAction::ShowAbout => {
                log::info!("Cat Shield - A cat-proof screen overlay");
                log::info!("Exit key: {}", exit_key_for_callback);
                // TODO: Show About dialog on Windows
            }
            TrayMenuAction::Quit => {
                log::info!("Quit requested");
                WINDOWS_SHOULD_QUIT.store(true, Ordering::SeqCst);
                // Post WM_QUIT to wake the message loop immediately
                // Without this, GetMessageW may block until the next Windows message arrives
                unsafe {
                    windows::Win32::UI::WindowsAndMessaging::PostQuitMessage(0);
                }
            }
        }
    });
    WindowsSystemTray::set_callback(menu_callback);

    // Create overlay window (but don't show yet)
    if let Err(e) = overlay.create() {
        log::error!("Failed to create overlay window: {}", e);
        tray.remove();
        release_instance_lock();
        process::exit(1);
    }
    log::info!("✓ Overlay window created");

    // Set overlay opacity from config
    overlay.set_opacity(config.opacity());

    // Set up close button callback for overlay
    let close_callback = Arc::new(|| {
        log::info!("Close button clicked - deactivating shield");
        WINDOWS_SHIELD_ACTIVE.store(false, Ordering::SeqCst);
    });
    WindowsOverlayWindow::set_close_callback(close_callback);

    // Track power assertion
    let mut sleep_assertion = None;

    log::info!("✓ Exit key: {}", exit_key_display);
    log::info!("════════════════════════════════════════");
    log::info!("System tray mode active");
    log::info!("Right-click the Cat Shield icon in your system tray to access options.");
    log::info!("Use 'Start Protection' to activate the shield.");

    // Windows message loop
    run_windows_message_loop(
        &mut tray,
        &mut overlay,
        &mut input_blocker,
        &power_manager,
        &mut sleep_assertion,
    );

    // Cleanup
    log::info!("Shutting down...");

    // Disable input blocking if still active
    if input_blocker.is_active() {
        input_blocker.disable();
    }

    // Release sleep assertion if held
    if let Some(assertion) = sleep_assertion.take() {
        if let Err(e) = power_manager.allow_sleep(assertion) {
            log::warn!("Failed to release sleep assertion: {}", e);
        }
    }

    // Close overlay
    overlay.close();
    WindowsOverlayWindow::clear_close_callback();

    // Remove tray icon
    WindowsSystemTray::clear_callback();
    tray.remove();

    // Release single-instance lock
    release_instance_lock();

    log::info!("👋 Cat Shield closed. Goodbye!");
}

/// Run the Windows message loop.
///
/// This is the core event loop that processes Windows messages, handles
/// shield activation/deactivation, and responds to user input.
#[cfg(target_os = "windows")]
fn run_windows_message_loop(
    tray: &mut WindowsSystemTray,
    overlay: &mut WindowsOverlayWindow,
    input_blocker: &mut WindowsInputBlocker,
    power_manager: &WindowsPowerManager,
    sleep_assertion: &mut Option<cat_shield::platform::types::SleepAssertion>,
) {
    use windows::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, GetMessageW, TranslateMessage, MSG,
    };

    let mut msg = MSG::default();
    let mut shield_was_active = false;

    // Message loop: GetMessage returns false when WM_QUIT is received
    loop {
        // Check if we should quit
        if WINDOWS_SHOULD_QUIT.load(Ordering::SeqCst) {
            break;
        }

        // Handle shield activation/deactivation state changes
        let shield_active = WINDOWS_SHIELD_ACTIVE.load(Ordering::SeqCst);

        if shield_active && !shield_was_active {
            // Activating shield
            log::info!("Activating shield...");

            // Set up input blocking
            if let Err(e) = input_blocker.setup() {
                log::error!("Failed to set up input blocking: {}", e);
                WINDOWS_SHIELD_ACTIVE.store(false, Ordering::SeqCst);
            } else {
                log::info!("✓ Input blocking active");

                // Prevent sleep
                match power_manager.prevent_sleep() {
                    Ok(assertion) => {
                        *sleep_assertion = Some(assertion);
                    }
                    Err(e) => {
                        log::warn!("Failed to prevent sleep: {}", e);
                    }
                }

                // Show overlay
                overlay.show();
                log::info!("✓ Overlay visible");

                // Update tray state
                tray.set_state(true);

                log::info!("════════════════════════════════════════");
                log::info!("🛡️  SHIELD ACTIVE");
                log::info!("════════════════════════════════════════");

                shield_was_active = true;
            }
        } else if !shield_active && shield_was_active {
            // Deactivating shield
            log::info!("Deactivating shield...");

            // Hide overlay
            overlay.hide();

            // Disable input blocking
            input_blocker.disable();
            log::info!("✓ Input blocking disabled");

            // Allow sleep
            if let Some(assertion) = sleep_assertion.take() {
                if let Err(e) = power_manager.allow_sleep(assertion) {
                    log::warn!("Failed to release sleep assertion: {}", e);
                }
            }

            // Update tray state
            tray.set_state(false);

            log::info!("════════════════════════════════════════");
            log::info!("Shield deactivated");
            log::info!("════════════════════════════════════════");

            shield_was_active = false;
        }

        // Process Windows messages
        // Use GetMessageW which blocks until a message is available
        // Return value: -1 = error, 0 = WM_QUIT, positive = message available
        let result = unsafe { GetMessageW(&mut msg, None, 0, 0) };

        if result.0 == 0 {
            // WM_QUIT received
            break;
        } else if result.0 == -1 {
            // Error occurred
            let err = unsafe { windows::Win32::Foundation::GetLastError() };
            log::error!("GetMessageW error: {:?}", err);
            break;
        }

        // Translate and dispatch the message
        unsafe {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

// =============================================================================
// macOS Entry Point
// =============================================================================

#[cfg(target_os = "macos")]
use cat_shield::config::{has_immediate_start_args, Args, Config};
#[cfg(target_os = "macos")]
use cat_shield::input::{set_exit_key, ExitKey, DEFAULT_EXIT_KEY};
#[cfg(target_os = "macos")]
use cat_shield::lock::{acquire_instance_lock, release_instance_lock, LockResult};
#[cfg(target_os = "macos")]
use cat_shield::platform::{allow_sleep, prevent_sleep, setup_event_tap};
#[cfg(target_os = "macos")]
use cat_shield::shield_core::{
    create_shield_window, ensure_accessibility, print_activation_banner, print_shield_active,
    setup_close_button, setup_timer_display,
};
#[cfg(target_os = "macos")]
use cat_shield::timer::{format_duration, get_remaining_seconds};
#[cfg(target_os = "macos")]
use cat_shield::ui::menu_bar::setup_menu_bar;
#[cfg(target_os = "macos")]
use cat_shield::ui::shield::{stop_close_button_timer, timer_callback};
#[cfg(target_os = "macos")]
use cat_shield::ui::state::{animation, shield};

#[cfg(target_os = "macos")]
use clap::Parser;
#[cfg(target_os = "macos")]
use objc2::rc::Retained;
#[cfg(target_os = "macos")]
use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy, NSScreen};
#[cfg(target_os = "macos")]
use objc2_core_foundation::{kCFRunLoopCommonModes, CFString};
#[cfg(target_os = "macos")]
use objc2_foundation::MainThreadMarker;
#[cfg(target_os = "macos")]
use std::ffi::c_void;
#[cfg(target_os = "macos")]
use std::process;
#[cfg(target_os = "macos")]
use std::sync::atomic::Ordering;

#[cfg(target_os = "macos")]
use cat_shield::platform::{
    CFAbsoluteTimeGetCurrent, CFRunLoopAddTimer, CFRunLoopGetCurrent, CFRunLoopTimerCreate,
};

/// Start the animation timer for immediate mode
#[cfg(target_os = "macos")]
fn start_close_button_timer() {
    // SAFETY: CFRunLoopTimerCreate and related calls are safe because:
    // - All arguments are valid (null allocator uses default, valid time values)
    // - timer_callback is a valid extern "C" function
    // - We check for null return before using the timer
    // - CFRunLoopAddTimer is called on the current thread's run loop
    // - The timer pointer is stored in shield::TIMER_REF for later cleanup
    unsafe {
        let timer = CFRunLoopTimerCreate(
            std::ptr::null(),
            CFAbsoluteTimeGetCurrent() + animation::INTERVAL_SECS,
            animation::INTERVAL_SECS,
            0,
            0,
            timer_callback,
            std::ptr::null(),
        );

        if !timer.is_null() {
            let run_loop = CFRunLoopGetCurrent();
            let mode = kCFRunLoopCommonModes.expect("kCFRunLoopCommonModes should exist");
            CFRunLoopAddTimer(run_loop, timer, (mode as *const CFString) as *const c_void);
            shield::TIMER_REF.store(timer, Ordering::Release);
        }
    }
}

#[cfg(target_os = "macos")]
fn main() {
    // Parse command line arguments
    let args = Args::parse();

    // Initialize logging with verbosity level from CLI args
    cat_shield::logging::init(args.verbose);

    // Check for existing instance (single-instance enforcement)
    match acquire_instance_lock() {
        LockResult::Acquired => {
            // Successfully acquired lock, continue startup
        }
        LockResult::AlreadyRunning(pid) => {
            log::warn!("Cat Shield is already running (PID: {})", pid);
            log::warn!("Look for the 🐱 icon in your menu bar.");
            process::exit(0);
        }
        LockResult::Error(e) => {
            log::warn!("Could not check for existing instance: {}", e);
            // Continue anyway - lock check is best-effort
        }
    }

    // Load config file
    let config = Config::load();

    // Determine exit key: CLI arg > config file > default
    let exit_key = if let Some(ref key) = args.exit_key {
        key.clone()
    } else if let Some(ref key_str) = config.exit_key {
        match ExitKey::parse(key_str) {
            Ok(key) => key,
            Err(e) => {
                log::warn!("Invalid exit_key in config file: {}", e);
                log::warn!("Using default: {}", DEFAULT_EXIT_KEY);
                ExitKey::default()
            }
        }
    } else {
        ExitKey::default()
    };

    // Set the global exit key configuration
    set_exit_key(&exit_key);

    // Get main thread marker - required for AppKit operations
    let mtm = MainThreadMarker::new().expect("Must run on main thread");

    // Initialize the application
    let app = NSApplication::sharedApplication(mtm);
    app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);

    // Always set up menu bar mode - shield deactivation returns here
    shield::MODE_MENU_BAR.store(true, Ordering::Release);

    log::info!("🐱 CAT SHIELD 🛡️");
    log::info!("════════════════════════════════════════");

    // Set up menu bar icon (always, so we can return here after shield deactivation)
    let _status_item = setup_menu_bar(mtm);

    // Check if we should start shield immediately (CLI args provided)
    if has_immediate_start_args(&args) {
        // Immediate shield mode: CLI args provided, start protection now
        // Ensure accessibility permissions (uses shared logic from shield_core)
        ensure_accessibility(&exit_key);

        // Print activation banner (shared)
        print_activation_banner();

        // Get the main screen dimensions
        let screen = NSScreen::mainScreen(mtm);
        let screen = match screen {
            Some(s) => s,
            None => {
                log::error!("Failed to get main screen");
                process::exit(1);
            }
        };
        let screen_frame = screen.frame();

        // Create the shield window using shared logic
        let window = create_shield_window(mtm, screen_frame);

        // Store window reference for cleanup (needed for deactivate_shield)
        shield::WINDOW.store(Retained::as_ptr(&window) as *mut c_void, Ordering::Release);

        // Show the window
        window.makeKeyAndOrderFront(None);

        log::info!("✓ Overlay window active");

        // Create close button and label using shared logic
        let (close_button, close_button_label) = setup_close_button(mtm, screen_frame);

        // Store view references for timer callback.
        // Safety: The view remains valid because contentView retains it and
        // app.run() blocks until we're ready to exit. The timer is stopped
        // before cleanup begins.
        shield::CLOSE_BUTTON.store(
            Retained::as_ptr(&close_button) as *mut c_void,
            Ordering::Release,
        );
        shield::CLOSE_BUTTON_LABEL.store(
            Retained::as_ptr(&close_button_label) as *mut c_void,
            Ordering::Release,
        );

        // Add close button and label to the window's content view
        if let Some(content_view) = window.contentView() {
            content_view.addSubview(&close_button);
            content_view.addSubview(&close_button_label);
        }

        // Start the animation timer
        start_close_button_timer();

        log::info!("✓ Close button active (hold 3s to exit)");
        log::info!("✓ Exit key: {}", exit_key.display_name);

        // Set up auto-exit timer if specified (uses shared helper)
        if let Some(duration_secs) = args.timer {
            if !args.hide_timer {
                setup_timer_display(mtm, &window, screen_frame, duration_secs);
            } else {
                // Timer without display - just initialize the timer
                cat_shield::timer::init_auto_exit_timer(duration_secs);
                log::info!("✓ Auto-exit timer set: {}", format_duration(duration_secs));
            }
        }

        // Prevent sleep
        if let Some(assertion_id) = prevent_sleep() {
            shield::SLEEP_ASSERTION_ID.store(assertion_id as u64, Ordering::Release);
            shield::HAS_SLEEP_ASSERTION.store(true, Ordering::Release);
        }

        // Set up event tap - this is the core security feature
        // Without input blocking, the shield is just a visual overlay
        if !setup_event_tap() {
            log::error!("Failed to create event tap");
            log::error!("════════════════════════════════════════");
            log::error!("SHIELD ACTIVATION FAILED");
            log::error!("════════════════════════════════════════");
            log::error!("Input blocking could not be enabled.");
            log::error!("The shield cannot protect without this feature.");
            log::error!("Please check:");
            log::error!("  - Accessibility permissions are granted");
            log::error!("  - No other apps are blocking event taps");
            std::process::exit(1);
        }
        log::info!("✓ Input blocking active");

        // Mark shield as active
        shield::IS_ACTIVE.store(true, Ordering::Release);

        // Print shield active status (shared)
        let timer_info = if args.timer.is_some() {
            Some(format!(
                "{} remaining",
                format_duration(get_remaining_seconds())
            ))
        } else {
            None
        };
        print_shield_active(&exit_key, timer_info.as_deref());

        // Transfer ownership to prevent deallocation while shield is active.
        // The raw pointers stored in global state are reclaimed in deactivate_shield()
        // using Retained::from_raw, which properly releases the objects.
        std::mem::forget(window);
        std::mem::forget(close_button);
        std::mem::forget(close_button_label);
    } else {
        // Menu bar only mode (no CLI args)
        log::info!("Menu bar mode active");
        log::info!("Click the 🐱 icon in your menu bar to access Cat Shield.");
        log::info!("Use 'Start Protection' to activate the shield.");
        log::info!("Or run with --timer or --exit-key to start immediately.");
    }

    // Finish launching the application (required for menu bar apps)
    app.finishLaunching();

    // Run the NSApplication event loop
    // The status item keeps the app alive in the menu bar
    app.run();

    // Cleanup (when app.terminate() is called)
    stop_close_button_timer();

    if shield::HAS_SLEEP_ASSERTION.load(Ordering::Acquire) {
        let id = shield::SLEEP_ASSERTION_ID.load(Ordering::Acquire) as u32;
        allow_sleep(id);
    }

    // Release single-instance lock before exiting
    release_instance_lock();

    log::info!("👋 Cat Shield closed. Goodbye!");
}
