//! Timer display view for the shield overlay

use crate::timer::{
    format_duration_cached, get_remaining_seconds, AUTO_EXIT_DURATION_SECS, WARNING_SECONDS,
};
use crate::ui::helpers::create_label;
use crate::ui::ptr_helper::with_raw_ptr;
use crate::ui::state::shield;
use objc2::rc::Retained;
use objc2::{define_class, msg_send};
use objc2_app_kit::{NSBezierPath, NSColor, NSTextField, NSView};
use objc2_core_foundation::{CGFloat, CGPoint, CGRect, CGSize};
use objc2_foundation::{MainThreadMarker, NSString};
use std::ffi::c_void;
use std::sync::atomic::Ordering;

/// Ivars for the TimerDisplayView
pub struct TimerDisplayViewIvars {}

define_class!(
    #[unsafe(super(NSView))]
    #[name = "TimerDisplayView"]
    #[ivars = TimerDisplayViewIvars]
    pub struct TimerDisplayView;

    impl TimerDisplayView {
        #[unsafe(method(drawRect:))]
        unsafe fn draw_rect(&self, _dirty_rect: CGRect) {
            draw_timer_display(self);
        }
    }
);

impl TimerDisplayView {
    pub fn new(mtm: MainThreadMarker, frame: CGRect) -> Retained<Self> {
        let this = mtm.alloc::<TimerDisplayView>();
        let this = this.set_ivars(TimerDisplayViewIvars {});
        unsafe { msg_send![super(this), initWithFrame: frame] }
    }
}

/// Draw the timer countdown display
fn draw_timer_display(view: &NSView) {
    let bounds = view.bounds();
    let remaining = get_remaining_seconds();
    let is_warning = remaining <= WARNING_SECONDS;

    // Background rounded rectangle (fully opaque to block overlay behind)
    let bg_color = if is_warning {
        // Red/orange warning color
        NSColor::colorWithRed_green_blue_alpha(0.8, 0.3, 0.1, 1.0)
    } else {
        // Dark opaque background
        NSColor::colorWithRed_green_blue_alpha(0.1, 0.1, 0.15, 1.0)
    };
    bg_color.set();

    let corner_radius: CGFloat = 10.0;
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
    let border_color = if is_warning {
        NSColor::colorWithRed_green_blue_alpha(1.0, 0.5, 0.2, 1.0)
    } else {
        NSColor::colorWithRed_green_blue_alpha(0.5, 0.5, 0.5, 0.8)
    };
    border_color.set();
    bg_path.setLineWidth(2.0);
    bg_path.stroke();

    // Format time string (cached to avoid 59 redundant allocations per second)
    let time_str = format_duration_cached(remaining);

    // Text colors
    let text_color = NSColor::colorWithRed_green_blue_alpha(1.0, 1.0, 1.0, 1.0);
    let label_color = NSColor::colorWithRed_green_blue_alpha(0.8, 0.8, 0.8, 1.0);
    let warning_color = NSColor::colorWithRed_green_blue_alpha(1.0, 1.0, 0.0, 1.0);

    // Get main thread marker (we're guaranteed to be on main thread in drawRect)
    let mtm = unsafe { MainThreadMarker::new_unchecked() };

    // Create or update NSTextField labels
    let header_ptr = shield::TIMER_HEADER.load(Ordering::Acquire);
    let time_ptr = shield::TIMER_TIME.load(Ordering::Acquire);
    let warning_ptr = shield::TIMER_WARNING.load(Ordering::Acquire);

    // Header label "Time Remaining:"
    let header_y = bounds.size.height - 22.0;
    let header_frame = CGRect {
        origin: CGPoint {
            x: 12.0,
            y: header_y,
        },
        size: CGSize {
            width: 120.0,
            height: 16.0,
        },
    };

    if header_ptr.is_null() {
        // Create the header label
        let header_label = create_label(
            mtm,
            "Time Remaining:",
            header_frame,
            12.0,
            &label_color,
            false,
        );
        shield::TIMER_HEADER.store(
            Retained::as_ptr(&header_label) as *mut c_void,
            Ordering::Release,
        );
        view.addSubview(&header_label);
        // SAFETY: Transferring ownership to Objective-C runtime and global AtomicPtr.
        // Preventing drop here is correct because:
        // - The view is retained by the parent view hierarchy (addSubview)
        // - The pointer is stored in shield::TIMER_HEADER for potential updates
        // Cleanup: shield::TIMER_HEADER is set to null in deactivate_shield(),
        // and the parent view releases subviews when the shield window closes.
        std::mem::forget(header_label);
    }

    // Time label (large bold countdown)
    let time_y = bounds.size.height - 48.0;
    let time_frame = CGRect {
        origin: CGPoint { x: 12.0, y: time_y },
        size: CGSize {
            width: 130.0,
            height: 26.0,
        },
    };

    if time_ptr.is_null() {
        // Create the time label
        let time_label = create_label(mtm, &time_str, time_frame, 20.0, &text_color, true);
        shield::TIMER_TIME.store(
            Retained::as_ptr(&time_label) as *mut c_void,
            Ordering::Release,
        );
        view.addSubview(&time_label);
        // SAFETY: Transferring ownership to Objective-C runtime and global AtomicPtr.
        // Preventing drop here is correct because:
        // - The view is retained by the parent view hierarchy (addSubview)
        // - The pointer is stored in shield::TIMER_TIME for countdown updates
        // Cleanup: shield::TIMER_TIME is set to null in deactivate_shield(),
        // and the parent view releases subviews when the shield window closes.
        std::mem::forget(time_label);
    } else {
        // Update existing time label
        unsafe {
            with_raw_ptr::<NSTextField, _>(time_ptr, |time_label| {
                time_label.setStringValue(&NSString::from_str(&time_str));
                let color = if is_warning {
                    &warning_color
                } else {
                    &text_color
                };
                time_label.setTextColor(Some(color));
            });
        }
    }

    // Warning label "Exiting soon!"
    let warning_frame = CGRect {
        origin: CGPoint {
            x: 140.0,
            y: time_y + 4.0,
        },
        size: CGSize {
            width: 100.0,
            height: 18.0,
        },
    };

    if warning_ptr.is_null() {
        // Create the warning label (initially hidden)
        let warning_label = create_label(
            mtm,
            "Exiting soon!",
            warning_frame,
            14.0,
            &warning_color,
            false,
        );
        warning_label.setHidden(!is_warning);
        shield::TIMER_WARNING.store(
            Retained::as_ptr(&warning_label) as *mut c_void,
            Ordering::Release,
        );
        view.addSubview(&warning_label);
        // SAFETY: Transferring ownership to Objective-C runtime and global AtomicPtr.
        // Preventing drop here is correct because:
        // - The view is retained by the parent view hierarchy (addSubview)
        // - The pointer is stored in shield::TIMER_WARNING for visibility updates
        // Cleanup: shield::TIMER_WARNING is set to null in deactivate_shield(),
        // and the parent view releases subviews when the shield window closes.
        std::mem::forget(warning_label);
    } else {
        // Update existing warning label visibility
        unsafe {
            with_raw_ptr::<NSTextField, _>(warning_ptr, |warning_label| {
                warning_label.setHidden(!is_warning);
            });
        }
    }

    // Draw a progress bar showing remaining time
    let duration = AUTO_EXIT_DURATION_SECS.load(Ordering::Acquire);
    let progress = if duration > 0 {
        remaining as f64 / duration as f64
    } else {
        0.0
    };

    // Progress bar background
    let bar_margin: CGFloat = 12.0;
    let bar_height: CGFloat = 8.0;
    let bar_y: CGFloat = 10.0;
    let bar_width = bounds.size.width - (bar_margin * 2.0);

    let bar_bg_color = NSColor::colorWithRed_green_blue_alpha(0.2, 0.2, 0.2, 1.0);
    bar_bg_color.set();

    let bar_bg_rect = CGRect {
        origin: CGPoint {
            x: bar_margin,
            y: bar_y,
        },
        size: CGSize {
            width: bar_width,
            height: bar_height,
        },
    };
    let bar_bg_path =
        NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(bar_bg_rect, 4.0, 4.0);
    bar_bg_path.fill();

    // Progress bar fill
    let bar_fill_color = if is_warning {
        NSColor::colorWithRed_green_blue_alpha(1.0, 0.3, 0.1, 1.0)
    } else {
        NSColor::colorWithRed_green_blue_alpha(0.2, 0.8, 0.3, 1.0)
    };
    bar_fill_color.set();

    let fill_width = bar_width * progress;
    if fill_width > 0.0 {
        let bar_fill_rect = CGRect {
            origin: CGPoint {
                x: bar_margin,
                y: bar_y,
            },
            size: CGSize {
                width: fill_width,
                height: bar_height,
            },
        };
        let bar_fill_path =
            NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(bar_fill_rect, 4.0, 4.0);
        bar_fill_path.fill();
    }
}
