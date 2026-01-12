//! Error types for platform-specific operations
//!
//! Each error type corresponds to a platform abstraction trait and provides
//! detailed error information for debugging and user feedback.

use std::error::Error;
use std::fmt;

/// Error type for input blocking operations.
#[derive(Debug)]
pub enum InputBlockError {
    /// Failed to create the input blocker (e.g., event tap, keyboard hook)
    CreationFailed(String),
    /// Insufficient permissions to block input
    PermissionDenied(String),
    /// The input blocker was disabled by the system
    DisabledBySystem(String),
    /// Platform-specific error
    Platform(String),
}

impl fmt::Display for InputBlockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CreationFailed(msg) => write!(f, "Failed to create input blocker: {msg}"),
            Self::PermissionDenied(msg) => write!(f, "Permission denied for input blocking: {msg}"),
            Self::DisabledBySystem(msg) => write!(f, "Input blocker disabled by system: {msg}"),
            Self::Platform(msg) => write!(f, "Platform error: {msg}"),
        }
    }
}

impl Error for InputBlockError {}

/// Error type for power management operations.
#[derive(Debug)]
pub enum PowerError {
    /// Failed to create a sleep prevention assertion
    AssertionFailed(String),
    /// Failed to release a sleep prevention assertion
    ReleaseFailed(String),
    /// The assertion ID is invalid
    InvalidAssertion(String),
    /// Platform-specific error
    Platform(String),
}

impl fmt::Display for PowerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AssertionFailed(msg) => write!(f, "Failed to create power assertion: {msg}"),
            Self::ReleaseFailed(msg) => write!(f, "Failed to release power assertion: {msg}"),
            Self::InvalidAssertion(msg) => write!(f, "Invalid power assertion: {msg}"),
            Self::Platform(msg) => write!(f, "Power management error: {msg}"),
        }
    }
}

impl Error for PowerError {}

/// Error type for system tray operations.
#[derive(Debug)]
pub enum TrayError {
    /// Failed to create the system tray icon
    CreationFailed(String),
    /// Failed to set the tray icon image
    IconFailed(String),
    /// Failed to create or update the tray menu
    MenuFailed(String),
    /// The system tray is not available on this platform
    NotAvailable(String),
    /// Platform-specific error
    Platform(String),
}

impl fmt::Display for TrayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CreationFailed(msg) => write!(f, "Failed to create system tray: {msg}"),
            Self::IconFailed(msg) => write!(f, "Failed to set tray icon: {msg}"),
            Self::MenuFailed(msg) => write!(f, "Failed to create tray menu: {msg}"),
            Self::NotAvailable(msg) => write!(f, "System tray not available: {msg}"),
            Self::Platform(msg) => write!(f, "System tray error: {msg}"),
        }
    }
}

impl Error for TrayError {}

/// Error type for overlay window operations.
#[derive(Debug)]
pub enum WindowError {
    /// Failed to create the overlay window
    CreationFailed(String),
    /// Failed to configure window properties (level, style, etc.)
    ConfigurationFailed(String),
    /// Failed to show or hide the window
    VisibilityFailed(String),
    /// The display/screen is not available
    DisplayNotAvailable(String),
    /// Platform-specific error
    Platform(String),
}

impl fmt::Display for WindowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CreationFailed(msg) => write!(f, "Failed to create overlay window: {msg}"),
            Self::ConfigurationFailed(msg) => write!(f, "Failed to configure window: {msg}"),
            Self::VisibilityFailed(msg) => write!(f, "Failed to change window visibility: {msg}"),
            Self::DisplayNotAvailable(msg) => write!(f, "Display not available: {msg}"),
            Self::Platform(msg) => write!(f, "Window error: {msg}"),
        }
    }
}

impl Error for WindowError {}

/// Error type for permission checking operations.
#[derive(Debug)]
pub enum PermissionError {
    /// The required permission was not granted
    NotGranted(String),
    /// Failed to check permission status
    CheckFailed(String),
    /// Failed to prompt the user for permissions
    PromptFailed(String),
    /// Failed to open system settings
    SettingsFailed(String),
    /// Platform-specific error
    Platform(String),
}

impl fmt::Display for PermissionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotGranted(msg) => write!(f, "Permission not granted: {msg}"),
            Self::CheckFailed(msg) => write!(f, "Failed to check permissions: {msg}"),
            Self::PromptFailed(msg) => write!(f, "Failed to prompt for permissions: {msg}"),
            Self::SettingsFailed(msg) => write!(f, "Failed to open settings: {msg}"),
            Self::Platform(msg) => write!(f, "Permission error: {msg}"),
        }
    }
}

impl Error for PermissionError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_block_error_display() {
        let err = InputBlockError::CreationFailed("test".to_string());
        assert!(err.to_string().contains("Failed to create input blocker"));
        assert!(err.to_string().contains("test"));
    }

    #[test]
    fn test_input_block_error_variants() {
        let cases = vec![
            InputBlockError::CreationFailed("a".into()),
            InputBlockError::PermissionDenied("b".into()),
            InputBlockError::DisabledBySystem("c".into()),
            InputBlockError::Platform("d".into()),
        ];
        for err in cases {
            // Ensure all variants can be displayed and debugged
            let _ = format!("{err}");
            let _ = format!("{err:?}");
        }
    }

    #[test]
    fn test_power_error_display() {
        let err = PowerError::AssertionFailed("IOKit error".to_string());
        assert!(err.to_string().contains("Failed to create power assertion"));
        assert!(err.to_string().contains("IOKit error"));
    }

    #[test]
    fn test_power_error_variants() {
        let cases = vec![
            PowerError::AssertionFailed("a".into()),
            PowerError::ReleaseFailed("b".into()),
            PowerError::InvalidAssertion("c".into()),
            PowerError::Platform("d".into()),
        ];
        for err in cases {
            let _ = format!("{err}");
            let _ = format!("{err:?}");
        }
    }

    #[test]
    fn test_tray_error_display() {
        let err = TrayError::NotAvailable("headless mode".to_string());
        assert!(err.to_string().contains("System tray not available"));
        assert!(err.to_string().contains("headless mode"));
    }

    #[test]
    fn test_tray_error_variants() {
        let cases = vec![
            TrayError::CreationFailed("a".into()),
            TrayError::IconFailed("b".into()),
            TrayError::MenuFailed("c".into()),
            TrayError::NotAvailable("d".into()),
            TrayError::Platform("e".into()),
        ];
        for err in cases {
            let _ = format!("{err}");
            let _ = format!("{err:?}");
        }
    }

    #[test]
    fn test_window_error_display() {
        let err = WindowError::DisplayNotAvailable("no monitor".to_string());
        assert!(err.to_string().contains("Display not available"));
        assert!(err.to_string().contains("no monitor"));
    }

    #[test]
    fn test_window_error_variants() {
        let cases = vec![
            WindowError::CreationFailed("a".into()),
            WindowError::ConfigurationFailed("b".into()),
            WindowError::VisibilityFailed("c".into()),
            WindowError::DisplayNotAvailable("d".into()),
            WindowError::Platform("e".into()),
        ];
        for err in cases {
            let _ = format!("{err}");
            let _ = format!("{err:?}");
        }
    }

    #[test]
    fn test_permission_error_display() {
        let err = PermissionError::NotGranted("accessibility".to_string());
        assert!(err.to_string().contains("Permission not granted"));
        assert!(err.to_string().contains("accessibility"));
    }

    #[test]
    fn test_permission_error_variants() {
        let cases = vec![
            PermissionError::NotGranted("a".into()),
            PermissionError::CheckFailed("b".into()),
            PermissionError::PromptFailed("c".into()),
            PermissionError::SettingsFailed("d".into()),
            PermissionError::Platform("e".into()),
        ];
        for err in cases {
            let _ = format!("{err}");
            let _ = format!("{err:?}");
        }
    }

    #[test]
    fn test_errors_implement_error_trait() {
        // Verify that all error types implement std::error::Error
        fn assert_error<E: Error>() {}

        assert_error::<InputBlockError>();
        assert_error::<PowerError>();
        assert_error::<TrayError>();
        assert_error::<WindowError>();
        assert_error::<PermissionError>();
    }
}
