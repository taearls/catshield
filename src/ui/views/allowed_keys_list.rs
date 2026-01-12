//! Allowed keys list view for the settings window
//!
//! This view displays a selectable list of allowed key combinations.
//! Clicking a row selects it, clicking again deselects it.
//! Selection state is stored in the settings module state.

use crate::ui::ptr_helper::with_ptr_void;
use crate::ui::state::settings;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{define_class, msg_send};
use objc2_app_kit::{NSBezierPath, NSColor, NSEvent, NSFont, NSStringDrawing, NSView};
use objc2_core_foundation::{CGFloat, CGPoint, CGRect, CGSize};
use objc2_foundation::{MainThreadMarker, NSDictionary, NSString};

/// Row height for each key item
const ROW_HEIGHT: CGFloat = 22.0;
/// Horizontal padding within rows
const ROW_PADDING_X: CGFloat = 8.0;
/// Vertical padding within rows
const ROW_PADDING_Y: CGFloat = 2.0;

/// Ivars for the AllowedKeysListView
pub struct AllowedKeysListViewIvars {}

define_class!(
    #[unsafe(super(NSView))]
    #[name = "AllowedKeysListView"]
    #[ivars = AllowedKeysListViewIvars]
    pub struct AllowedKeysListView;

    impl AllowedKeysListView {
        #[unsafe(method(drawRect:))]
        unsafe fn draw_rect(&self, dirty_rect: CGRect) {
            draw_allowed_keys_list(self, dirty_rect);
        }

        #[unsafe(method(mouseDown:))]
        unsafe fn mouse_down(&self, event: &NSEvent) {
            handle_mouse_click(self, event);
        }

        /// Indicate that this view can accept first responder status
        /// for keyboard events (enables keyboard selection)
        #[unsafe(method(acceptsFirstResponder))]
        fn accepts_first_responder(&self) -> bool {
            true
        }

        /// Handle keyboard events for row deletion
        #[unsafe(method(keyDown:))]
        unsafe fn key_down(&self, event: &NSEvent) {
            let key_code = event.keyCode();
            // Delete key (51) or Backspace (117 on some keyboards)
            if key_code == 51 || key_code == 117 {
                // Delete selected key if there is one
                if let Some(selected_idx) = settings::get_selected_allowed_key_index() {
                    if let Some(mut keys) = settings::get_pending_allowed_keys() {
                        let idx = selected_idx as usize;
                        if idx < keys.len() {
                            keys.remove(idx);
                            settings::set_pending_allowed_keys(Some(keys.clone()));

                            // Clear selection since the item is gone
                            settings::set_selected_allowed_key_index(None);

                            // Update the list view size and redraw
                            update_list_view_size(self);
                            self.setNeedsDisplay(true);

                            // Update Remove button state
                            update_remove_button_enabled();

                            // Update validation label
                            show_removal_feedback();
                        }
                    }
                }
            } else {
                // Pass other key events to super
                let _: () = msg_send![super(self), keyDown: event];
            }
        }

        /// Make the view flipped (origin at top-left) for easier row calculations
        #[unsafe(method(isFlipped))]
        fn is_flipped(&self) -> bool {
            true
        }
    }
);

impl AllowedKeysListView {
    pub fn new(mtm: MainThreadMarker, frame: CGRect) -> Retained<Self> {
        let this = mtm.alloc::<AllowedKeysListView>();
        let this = this.set_ivars(AllowedKeysListViewIvars {});
        unsafe { msg_send![super(this), initWithFrame: frame] }
    }

    /// Get the required height for the current number of keys
    pub fn required_height(&self) -> CGFloat {
        let key_count = settings::get_pending_allowed_keys()
            .map(|k| k.len())
            .unwrap_or(0);
        if key_count == 0 {
            ROW_HEIGHT // At least one row height for "No allowed keys" message
        } else {
            key_count as CGFloat * ROW_HEIGHT
        }
    }
}

/// Draw the list of allowed keys with selection highlighting
fn draw_allowed_keys_list(view: &NSView, _dirty_rect: CGRect) {
    let bounds = view.bounds();

    // Draw background
    NSColor::textBackgroundColor().set();
    NSBezierPath::fillRect(bounds);

    // Get pending keys from state
    let keys = settings::get_pending_allowed_keys().unwrap_or_default();
    let selected_idx = settings::get_selected_allowed_key_index();

    if keys.is_empty() {
        // Draw placeholder text
        draw_placeholder_text(view);
        return;
    }

    // Draw each row
    for (i, key) in keys.iter().enumerate() {
        let y = i as CGFloat * ROW_HEIGHT;
        let row_rect = CGRect {
            origin: CGPoint { x: 0.0, y },
            size: CGSize {
                width: bounds.size.width,
                height: ROW_HEIGHT,
            },
        };

        // Draw selection highlight if this row is selected
        if Some(i as isize) == selected_idx {
            NSColor::selectedContentBackgroundColor().set();
            NSBezierPath::fillRect(row_rect);
        }

        // Draw row text
        let text_rect = CGRect {
            origin: CGPoint {
                x: ROW_PADDING_X,
                y: y + ROW_PADDING_Y,
            },
            size: CGSize {
                width: bounds.size.width - ROW_PADDING_X * 2.0,
                height: ROW_HEIGHT - ROW_PADDING_Y * 2.0,
            },
        };

        draw_row_text(key, text_rect, Some(i as isize) == selected_idx);
    }
}

/// Draw placeholder text when no keys are configured
fn draw_placeholder_text(_view: &NSView) {
    let text = NSString::from_str("No allowed keys configured");

    let attrs = create_text_attributes(NSColor::placeholderTextColor());

    let text_point = CGPoint {
        x: ROW_PADDING_X,
        y: ROW_PADDING_Y + 3.0, // Slight offset for vertical centering
    };

    unsafe {
        text.drawAtPoint_withAttributes(text_point, Some(&attrs));
    }
}

/// Draw text for a single row
fn draw_row_text(text: &str, rect: CGRect, is_selected: bool) {
    let ns_text = NSString::from_str(text);

    // Use contrasting color when selected
    let color = if is_selected {
        // Use alternateSelectedControlTextColor for contrast on selection
        NSColor::alternateSelectedControlTextColor()
    } else {
        NSColor::labelColor()
    };

    let attrs = create_text_attributes(color);

    let text_point = CGPoint {
        x: rect.origin.x,
        y: rect.origin.y + 3.0, // Slight offset for vertical centering
    };

    unsafe {
        ns_text.drawAtPoint_withAttributes(text_point, Some(&attrs));
    }
}

/// Create text attributes dictionary with font and color
fn create_text_attributes(
    color: Retained<NSColor>,
) -> Retained<NSDictionary<objc2_foundation::NSAttributedStringKey, AnyObject>> {
    use objc2_app_kit::{NSFontAttributeName, NSForegroundColorAttributeName};

    let font = NSFont::systemFontOfSize(12.0);

    // Create dictionary with raw pointers using msg_send
    // This is the objc2 pattern for creating dictionaries with mixed types
    unsafe {
        let keys: &[&NSString] = &[&*NSFontAttributeName, &*NSForegroundColorAttributeName];
        let objects: &[&AnyObject] = &[
            std::mem::transmute::<&NSFont, &AnyObject>(&*font),
            std::mem::transmute::<&NSColor, &AnyObject>(&*color),
        ];

        NSDictionary::from_slices(keys, objects)
    }
}

/// Handle mouse click to select/deselect rows
fn handle_mouse_click(view: &AllowedKeysListView, event: &NSEvent) {
    // Make this view the first responder to receive keyboard events
    if let Some(window) = view.window() {
        let _ = window.makeFirstResponder(Some(view));
    }

    let keys = settings::get_pending_allowed_keys().unwrap_or_default();
    if keys.is_empty() {
        return;
    }

    // Get click location in view coordinates
    let location = event.locationInWindow();
    let local_point = view.convertPoint_fromView(location, None);

    // Calculate which row was clicked
    let clicked_row = (local_point.y / ROW_HEIGHT) as isize;

    // Validate row index
    if clicked_row < 0 || clicked_row >= keys.len() as isize {
        return;
    }

    // Toggle selection
    let current_selection = settings::get_selected_allowed_key_index();
    if current_selection == Some(clicked_row) {
        // Deselect
        settings::set_selected_allowed_key_index(None);
    } else {
        // Select this row
        settings::set_selected_allowed_key_index(Some(clicked_row));
    }

    // Update Remove button enabled state
    update_remove_button_enabled();

    // Redraw
    view.setNeedsDisplay(true);
}

/// Update the enabled state of the Remove button based on selection
pub fn update_remove_button_enabled() {
    use objc2_app_kit::NSButton;

    let has_selection = settings::get_selected_allowed_key_index().is_some();

    unsafe {
        with_ptr_void::<NSButton, _>(&settings::REMOVE_KEY_BUTTON, |button| {
            button.setEnabled(has_selection);
        });
    }
}

/// Update the list view's frame size based on content
fn update_list_view_size(view: &AllowedKeysListView) {
    let key_count = settings::get_pending_allowed_keys()
        .map(|k| k.len())
        .unwrap_or(0);
    let new_height = if key_count == 0 {
        ROW_HEIGHT
    } else {
        key_count as CGFloat * ROW_HEIGHT
    };

    let current_frame = view.frame();
    let new_frame = CGRect {
        origin: current_frame.origin,
        size: CGSize {
            width: current_frame.size.width,
            height: new_height,
        },
    };

    view.setFrame(new_frame);
}

/// Show feedback after removing a key
fn show_removal_feedback() {
    use crate::ui::windows::set_validation_label;

    set_validation_label(
        &settings::ADD_KEY_VALIDATION,
        true,
        "✓ Removed (click Save to persist)",
    );
}

/// Refresh the allowed keys list view display
pub fn refresh_allowed_keys_list() {
    unsafe {
        with_ptr_void::<AllowedKeysListView, _>(&settings::ALLOWED_KEYS_VIEW, |view| {
            // Update size for new content
            let key_count = settings::get_pending_allowed_keys()
                .map(|k| k.len())
                .unwrap_or(0);
            let new_height = if key_count == 0 {
                ROW_HEIGHT
            } else {
                key_count as CGFloat * ROW_HEIGHT
            };

            let current_frame = view.frame();
            let new_frame = CGRect {
                origin: current_frame.origin,
                size: CGSize {
                    width: current_frame.size.width,
                    height: new_height,
                },
            };
            view.setFrame(new_frame);
            view.setNeedsDisplay(true);
        });
    }
}
