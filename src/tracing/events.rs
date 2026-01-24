//! Event type definitions for trace logging
//!
//! This module defines all the event types that can be traced.
//! Each event implements the `TracedEvent` trait.

use super::TracedEvent;

// =============================================================================
// Menu Events
// =============================================================================

/// Menu item was clicked
pub struct MenuClicked {
    pub item: &'static str,
    pub enabled: bool,
}

impl TracedEvent for MenuClicked {
    fn category(&self) -> &'static str {
        "menu"
    }

    fn name(&self) -> &'static str {
        "clicked"
    }

    fn metadata(&self) -> Vec<(&'static str, String)> {
        vec![
            ("item", self.item.to_string()),
            ("enabled", self.enabled.to_string()),
        ]
    }
}

/// Menu item enabled state changed
pub struct MenuItemStateChanged {
    pub item: &'static str,
    pub enabled: bool,
    pub reason: &'static str,
}

impl TracedEvent for MenuItemStateChanged {
    fn category(&self) -> &'static str {
        "menu"
    }

    fn name(&self) -> &'static str {
        "item_state_changed"
    }

    fn metadata(&self) -> Vec<(&'static str, String)> {
        vec![
            ("item", self.item.to_string()),
            ("enabled", self.enabled.to_string()),
            ("reason", self.reason.to_string()),
        ]
    }
}

// =============================================================================
// Window Events
// =============================================================================

/// Window was opened
pub struct WindowOpened {
    pub window: &'static str,
}

impl TracedEvent for WindowOpened {
    fn category(&self) -> &'static str {
        "window"
    }

    fn name(&self) -> &'static str {
        "opened"
    }

    fn metadata(&self) -> Vec<(&'static str, String)> {
        vec![("window", self.window.to_string())]
    }
}

/// Window was closed
pub struct WindowClosed {
    pub window: &'static str,
    pub reason: &'static str,
}

impl TracedEvent for WindowClosed {
    fn category(&self) -> &'static str {
        "window"
    }

    fn name(&self) -> &'static str {
        "closed"
    }

    fn metadata(&self) -> Vec<(&'static str, String)> {
        vec![
            ("window", self.window.to_string()),
            ("reason", self.reason.to_string()),
        ]
    }
}

/// Window focus changed
pub struct WindowFocusChanged {
    pub window: &'static str,
    pub focused: bool,
}

impl TracedEvent for WindowFocusChanged {
    fn category(&self) -> &'static str {
        "window"
    }

    fn name(&self) -> &'static str {
        "focus_changed"
    }

    fn metadata(&self) -> Vec<(&'static str, String)> {
        vec![
            ("window", self.window.to_string()),
            ("focused", self.focused.to_string()),
        ]
    }
}

// =============================================================================
// Shield Events
// =============================================================================

/// Shield was activated
pub struct ShieldActivated {
    pub mode: &'static str,
}

impl TracedEvent for ShieldActivated {
    fn category(&self) -> &'static str {
        "shield"
    }

    fn name(&self) -> &'static str {
        "activated"
    }

    fn metadata(&self) -> Vec<(&'static str, String)> {
        vec![("mode", self.mode.to_string())]
    }
}

/// Shield was deactivated
pub struct ShieldDeactivated {
    pub reason: &'static str,
}

impl TracedEvent for ShieldDeactivated {
    fn category(&self) -> &'static str {
        "shield"
    }

    fn name(&self) -> &'static str {
        "deactivated"
    }

    fn metadata(&self) -> Vec<(&'static str, String)> {
        vec![("reason", self.reason.to_string())]
    }
}

/// Shield state transition
pub struct ShieldStateTransition {
    pub from: &'static str,
    pub to: &'static str,
    pub trigger: &'static str,
}

impl TracedEvent for ShieldStateTransition {
    fn category(&self) -> &'static str {
        "shield"
    }

    fn name(&self) -> &'static str {
        "state_transition"
    }

    fn metadata(&self) -> Vec<(&'static str, String)> {
        vec![
            ("from", self.from.to_string()),
            ("to", self.to.to_string()),
            ("trigger", self.trigger.to_string()),
        ]
    }
}

// =============================================================================
// Settings Events
// =============================================================================

/// Setting was changed
pub struct SettingChanged {
    pub setting: &'static str,
    pub old_value: String,
    pub new_value: String,
}

impl TracedEvent for SettingChanged {
    fn category(&self) -> &'static str {
        "settings"
    }

    fn name(&self) -> &'static str {
        "changed"
    }

    fn metadata(&self) -> Vec<(&'static str, String)> {
        vec![
            ("setting", self.setting.to_string()),
            ("old_value", self.old_value.clone()),
            ("new_value", self.new_value.clone()),
        ]
    }
}

/// Settings were saved
pub struct SettingsSaved;

impl TracedEvent for SettingsSaved {
    fn category(&self) -> &'static str {
        "settings"
    }

    fn name(&self) -> &'static str {
        "saved"
    }

    fn metadata(&self) -> Vec<(&'static str, String)> {
        vec![]
    }
}

/// Settings were cancelled
pub struct SettingsCancelled;

impl TracedEvent for SettingsCancelled {
    fn category(&self) -> &'static str {
        "settings"
    }

    fn name(&self) -> &'static str {
        "cancelled"
    }

    fn metadata(&self) -> Vec<(&'static str, String)> {
        vec![]
    }
}

/// Settings were reset to defaults
pub struct SettingsReset;

impl TracedEvent for SettingsReset {
    fn category(&self) -> &'static str {
        "settings"
    }

    fn name(&self) -> &'static str {
        "reset_to_defaults"
    }

    fn metadata(&self) -> Vec<(&'static str, String)> {
        vec![]
    }
}

// =============================================================================
// Timer Events
// =============================================================================

/// Timer was started
pub struct TimerStarted {
    pub duration_secs: u64,
}

impl TracedEvent for TimerStarted {
    fn category(&self) -> &'static str {
        "timer"
    }

    fn name(&self) -> &'static str {
        "started"
    }

    fn metadata(&self) -> Vec<(&'static str, String)> {
        vec![("duration_secs", self.duration_secs.to_string())]
    }
}

/// Timer was stopped
pub struct TimerStopped {
    pub reason: &'static str,
}

impl TracedEvent for TimerStopped {
    fn category(&self) -> &'static str {
        "timer"
    }

    fn name(&self) -> &'static str {
        "stopped"
    }

    fn metadata(&self) -> Vec<(&'static str, String)> {
        vec![("reason", self.reason.to_string())]
    }
}

/// Timer expired
pub struct TimerExpired;

impl TracedEvent for TimerExpired {
    fn category(&self) -> &'static str {
        "timer"
    }

    fn name(&self) -> &'static str {
        "expired"
    }

    fn metadata(&self) -> Vec<(&'static str, String)> {
        vec![]
    }
}

/// Timer warning threshold reached
pub struct TimerWarning {
    pub remaining_secs: u64,
}

impl TracedEvent for TimerWarning {
    fn category(&self) -> &'static str {
        "timer"
    }

    fn name(&self) -> &'static str {
        "warning"
    }

    fn metadata(&self) -> Vec<(&'static str, String)> {
        vec![("remaining_secs", self.remaining_secs.to_string())]
    }
}

// =============================================================================
// UI Element Events
// =============================================================================

/// UI element state changed
pub struct UIElementStateChanged {
    pub element: &'static str,
    pub state: &'static str,
    pub reason: &'static str,
}

impl TracedEvent for UIElementStateChanged {
    fn category(&self) -> &'static str {
        "ui"
    }

    fn name(&self) -> &'static str {
        "element_state_changed"
    }

    fn metadata(&self) -> Vec<(&'static str, String)> {
        vec![
            ("element", self.element.to_string()),
            ("state", self.state.to_string()),
            ("reason", self.reason.to_string()),
        ]
    }
}

/// Close button interaction
pub struct CloseButtonInteraction {
    pub action: &'static str,
    pub hold_duration_ms: Option<u64>,
}

impl TracedEvent for CloseButtonInteraction {
    fn category(&self) -> &'static str {
        "ui"
    }

    fn name(&self) -> &'static str {
        "close_button"
    }

    fn metadata(&self) -> Vec<(&'static str, String)> {
        let mut meta = vec![("action", self.action.to_string())];
        if let Some(duration) = self.hold_duration_ms {
            meta.push(("hold_duration_ms", duration.to_string()));
        }
        meta
    }
}

// =============================================================================
// Error Events
// =============================================================================

/// An error occurred
pub struct ErrorOccurred {
    pub context: &'static str,
    pub error: String,
}

impl TracedEvent for ErrorOccurred {
    fn category(&self) -> &'static str {
        "error"
    }

    fn name(&self) -> &'static str {
        "occurred"
    }

    fn metadata(&self) -> Vec<(&'static str, String)> {
        vec![
            ("context", self.context.to_string()),
            ("error", self.error.clone()),
        ]
    }
}

// =============================================================================
// Input Events
// =============================================================================

/// Exit key was pressed
pub struct ExitKeyPressed {
    pub key: String,
}

impl TracedEvent for ExitKeyPressed {
    fn category(&self) -> &'static str {
        "input"
    }

    fn name(&self) -> &'static str {
        "exit_key_pressed"
    }

    fn metadata(&self) -> Vec<(&'static str, String)> {
        vec![("key", self.key.clone())]
    }
}

/// Allowed key was pressed (passed through)
pub struct AllowedKeyPressed {
    pub key: String,
}

impl TracedEvent for AllowedKeyPressed {
    fn category(&self) -> &'static str {
        "input"
    }

    fn name(&self) -> &'static str {
        "allowed_key_pressed"
    }

    fn metadata(&self) -> Vec<(&'static str, String)> {
        vec![("key", self.key.clone())]
    }
}

// =============================================================================
// Application Events
// =============================================================================

/// Application started
pub struct AppStarted {
    pub mode: &'static str,
    pub trace_logging: bool,
}

impl TracedEvent for AppStarted {
    fn category(&self) -> &'static str {
        "app"
    }

    fn name(&self) -> &'static str {
        "started"
    }

    fn metadata(&self) -> Vec<(&'static str, String)> {
        vec![
            ("mode", self.mode.to_string()),
            ("trace_logging", self.trace_logging.to_string()),
        ]
    }
}

/// Application shutting down
pub struct AppShuttingDown {
    pub reason: &'static str,
}

impl TracedEvent for AppShuttingDown {
    fn category(&self) -> &'static str {
        "app"
    }

    fn name(&self) -> &'static str {
        "shutting_down"
    }

    fn metadata(&self) -> Vec<(&'static str, String)> {
        vec![("reason", self.reason.to_string())]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_clicked_event() {
        let event = MenuClicked {
            item: "Settings",
            enabled: true,
        };
        assert_eq!(event.category(), "menu");
        assert_eq!(event.name(), "clicked");
        let meta = event.metadata();
        assert_eq!(meta.len(), 2);
        assert_eq!(meta[0], ("item", "Settings".to_string()));
        assert_eq!(meta[1], ("enabled", "true".to_string()));
    }

    #[test]
    fn test_shield_activated_event() {
        let event = ShieldActivated { mode: "menu_bar" };
        assert_eq!(event.category(), "shield");
        assert_eq!(event.name(), "activated");
        let meta = event.metadata();
        assert_eq!(meta.len(), 1);
        assert_eq!(meta[0], ("mode", "menu_bar".to_string()));
    }

    #[test]
    fn test_setting_changed_event() {
        let event = SettingChanged {
            setting: "exit_key",
            old_value: "Cmd+Q".to_string(),
            new_value: "Cmd+Shift+Q".to_string(),
        };
        assert_eq!(event.category(), "settings");
        assert_eq!(event.name(), "changed");
        let meta = event.metadata();
        assert_eq!(meta.len(), 3);
    }

    #[test]
    fn test_timer_started_event() {
        let event = TimerStarted {
            duration_secs: 1800,
        };
        assert_eq!(event.category(), "timer");
        assert_eq!(event.name(), "started");
        let meta = event.metadata();
        assert_eq!(meta[0], ("duration_secs", "1800".to_string()));
    }

    #[test]
    fn test_close_button_interaction_event() {
        let event = CloseButtonInteraction {
            action: "mouse_down",
            hold_duration_ms: None,
        };
        assert_eq!(event.category(), "ui");
        assert_eq!(event.name(), "close_button");
        let meta = event.metadata();
        assert_eq!(meta.len(), 1);

        let event_with_duration = CloseButtonInteraction {
            action: "released",
            hold_duration_ms: Some(2500),
        };
        let meta = event_with_duration.metadata();
        assert_eq!(meta.len(), 2);
    }

    #[test]
    fn test_error_occurred_event() {
        let event = ErrorOccurred {
            context: "config_load",
            error: "File not found".to_string(),
        };
        assert_eq!(event.category(), "error");
        assert_eq!(event.name(), "occurred");
    }

    #[test]
    fn test_app_started_event() {
        let event = AppStarted {
            mode: "menu_bar",
            trace_logging: true,
        };
        assert_eq!(event.category(), "app");
        assert_eq!(event.name(), "started");
        let meta = event.metadata();
        assert_eq!(meta.len(), 2);
    }
}
