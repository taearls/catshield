//! Close button label view for the shield overlay

use crate::ui::helpers::create_label;
use crate::ui::ptr_helper::with_raw_ptr;
use crate::ui::state::{
    calculate_hold_progress, close_button, shield, IS_MOUSE_INSIDE, MOUSE_DOWN_TIME,
};
use objc2::rc::Retained;
use objc2::{define_class, msg_send};
use objc2_app_kit::{NSBezierPath, NSColor, NSFont, NSTextAlignment, NSTextField, NSView};
use objc2_core_foundation::{CGFloat, CGPoint, CGRect};
use objc2_foundation::{MainThreadMarker, NSString};
use std::ffi::c_void;
use std::sync::atomic::Ordering;

/// Ivars for the CloseButtonLabelView
pub struct CloseButtonLabelViewIvars {}

define_class!(
    #[unsafe(super(NSView))]
    #[name = "CloseButtonLabelView"]
    #[ivars = CloseButtonLabelViewIvars]
    pub struct CloseButtonLabelView;

    impl CloseButtonLabelView {
        #[unsafe(method(drawRect:))]
        unsafe fn draw_rect(&self, _dirty_rect: CGRect) {
            draw_close_button_label(self);
        }
    }
);

impl CloseButtonLabelView {
    pub fn new(mtm: MainThreadMarker, frame: CGRect) -> Retained<Self> {
        let this = mtm.alloc::<CloseButtonLabelView>();
        let this = this.set_ivars(CloseButtonLabelViewIvars {});
        unsafe { msg_send![super(this), initWithFrame: frame] }
    }
}

/// Draw the close button label with hold instructions
fn draw_close_button_label(view: &NSView) {
    let bounds = view.bounds();

    // Get current hold progress
    let progress = MOUSE_DOWN_TIME.with(|time| {
        if let Some(start) = time.get() {
            calculate_hold_progress(
                start.elapsed().as_secs_f64(),
                close_button::HOLD_DURATION_SECS,
            )
        } else {
            0.0
        }
    });
    let is_inside = IS_MOUSE_INSIDE.with(|inside| inside.get());
    let is_holding = progress > 0.0 && is_inside;

    // Background rounded rectangle (fully opaque to block overlay behind)
    let bg_color = if is_holding {
        NSColor::colorWithRed_green_blue_alpha(0.2, 0.2, 0.25, 1.0)
    } else {
        NSColor::colorWithRed_green_blue_alpha(0.1, 0.1, 0.15, 1.0)
    };
    bg_color.set();

    let corner_radius: CGFloat = 8.0;
    let bg_rect = CGRect {
        origin: CGPoint { x: 0.0, y: 0.0 },
        size: bounds.size,
    };
    let bg_path = NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(
        bg_rect,
        corner_radius,
        corner_radius,
    );
    bg_path.fill();

    // Border
    let border_color = if is_holding {
        NSColor::colorWithRed_green_blue_alpha(0.4, 0.9, 0.4, 1.0)
    } else {
        NSColor::colorWithRed_green_blue_alpha(0.5, 0.5, 0.5, 0.8)
    };
    border_color.set();
    bg_path.setLineWidth(1.5);
    bg_path.stroke();

    // Text colors
    let hint_color = NSColor::colorWithRed_green_blue_alpha(0.7, 0.7, 0.7, 1.0);
    let progress_color = NSColor::colorWithRed_green_blue_alpha(0.4, 1.0, 0.4, 1.0);

    // Get main thread marker (we're guaranteed to be on main thread in drawRect)
    let mtm = unsafe { MainThreadMarker::new_unchecked() };

    // Create or update NSTextField label
    let label_ptr = shield::CLOSE_BUTTON_TEXT.load(Ordering::Acquire);

    // Calculate centered frame for the label
    let label_frame = CGRect {
        origin: CGPoint { x: 0.0, y: 0.0 },
        size: bounds.size,
    };

    // Determine text and color based on state
    let (text, color, font_size): (String, &NSColor, CGFloat) = if is_holding {
        let remaining_secs = ((1.0 - progress) * close_button::HOLD_DURATION_SECS).ceil() as u32;
        (format!("{remaining_secs}s..."), &progress_color, 16.0)
    } else {
        ("Hold 3s to exit".to_string(), &hint_color, 12.0)
    };

    if label_ptr.is_null() {
        // Create the label
        let label = create_label(mtm, &text, label_frame, font_size, color, false);
        // Center the text horizontally
        label.setAlignment(NSTextAlignment::Center);
        shield::CLOSE_BUTTON_TEXT.store(Retained::as_ptr(&label) as *mut c_void, Ordering::Release);
        view.addSubview(&label);
        // SAFETY: Transferring ownership to Objective-C runtime and global AtomicPtr.
        // Preventing drop here is correct because:
        // - The view is retained by the parent view hierarchy (addSubview)
        // - The pointer is stored in shield::CLOSE_BUTTON_TEXT for text updates
        // Cleanup: shield::CLOSE_BUTTON_TEXT is set to null in deactivate_shield(),
        // and the parent view releases subviews when the shield window closes.
        std::mem::forget(label);
    } else {
        // Update existing label
        unsafe {
            with_raw_ptr::<NSTextField, _>(label_ptr, |label| {
                label.setStringValue(&NSString::from_str(&text));
                label.setTextColor(Some(color));
                let font = NSFont::systemFontOfSize(font_size);
                label.setFont(Some(&font));
            });
        }
    }
}
