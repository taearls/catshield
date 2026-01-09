//! Close button view for the shield overlay

use crate::ui::state::{calculate_hold_progress, close_button, IS_MOUSE_INSIDE, MOUSE_DOWN_TIME};
use objc2::rc::Retained;
use objc2::{define_class, msg_send};
use objc2_app_kit::{NSBezierPath, NSColor, NSEvent, NSView};
use objc2_core_foundation::{CGFloat, CGPoint, CGRect, CGSize};
use objc2_foundation::MainThreadMarker;

/// Ivars for the CloseButtonView
pub struct CloseButtonViewIvars {}

define_class!(
    #[unsafe(super(NSView))]
    #[name = "CloseButtonView"]
    #[ivars = CloseButtonViewIvars]
    pub struct CloseButtonView;

    impl CloseButtonView {
        #[unsafe(method(drawRect:))]
        unsafe fn draw_rect(&self, _dirty_rect: CGRect) {
            draw_close_button(self);
        }

        #[unsafe(method(mouseDown:))]
        unsafe fn mouse_down(&self, _event: &NSEvent) {
            MOUSE_DOWN_TIME.with(|time| {
                time.set(Some(std::time::Instant::now()));
            });
            IS_MOUSE_INSIDE.with(|inside| inside.set(true));
            self.setNeedsDisplay(true);
        }

        #[unsafe(method(mouseUp:))]
        unsafe fn mouse_up(&self, _event: &NSEvent) {
            MOUSE_DOWN_TIME.with(|time| {
                time.set(None);
            });
            self.setNeedsDisplay(true);
        }

        #[unsafe(method(mouseDragged:))]
        unsafe fn mouse_dragged(&self, event: &NSEvent) {
            // Check if mouse is still inside the button
            let location = event.locationInWindow();
            let bounds = self.bounds();

            // Convert to view coordinates
            let local_point = self.convertPoint_fromView(location, None);

            let is_inside = local_point.x >= 0.0
                && local_point.x <= bounds.size.width
                && local_point.y >= 0.0
                && local_point.y <= bounds.size.height;

            let was_inside = IS_MOUSE_INSIDE.with(|inside| inside.get());

            if is_inside != was_inside {
                IS_MOUSE_INSIDE.with(|inside| inside.set(is_inside));

                // Reset timer if mouse left the button
                if !is_inside {
                    MOUSE_DOWN_TIME.with(|time| {
                        time.set(None);
                    });
                } else {
                    // Restart timer if mouse re-entered
                    MOUSE_DOWN_TIME.with(|time| {
                        time.set(Some(std::time::Instant::now()));
                    });
                }
            }

            self.setNeedsDisplay(true);
        }
    }
);

impl CloseButtonView {
    pub fn new(mtm: MainThreadMarker, frame: CGRect) -> Retained<Self> {
        let this = mtm.alloc::<CloseButtonView>();
        let this = this.set_ivars(CloseButtonViewIvars {});
        unsafe { msg_send![super(this), initWithFrame: frame] }
    }
}

/// Draw the close button with progress indicator
fn draw_close_button(view: &NSView) {
    let bounds = view.bounds();
    let center_x = bounds.size.width / 2.0;
    let center_y = bounds.size.height / 2.0;
    let radius = (bounds.size.width.min(bounds.size.height) / 2.0) - 2.0;

    // Calculate progress (0.0 to 1.0)
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

    // Background circle - bright red for visibility
    let bg_color = if is_inside && progress > 0.0 {
        NSColor::colorWithRed_green_blue_alpha(0.9, 0.2, 0.2, 1.0) // Bright red when pressed
    } else {
        NSColor::colorWithRed_green_blue_alpha(0.8, 0.1, 0.1, 0.95) // Dark red normally
    };

    bg_color.set();

    let bg_path = NSBezierPath::bezierPathWithOvalInRect(CGRect {
        origin: CGPoint {
            x: center_x - radius,
            y: center_y - radius,
        },
        size: CGSize {
            width: radius * 2.0,
            height: radius * 2.0,
        },
    });
    bg_path.fill();

    // White border for extra visibility
    let border_color = NSColor::colorWithRed_green_blue_alpha(1.0, 1.0, 1.0, 0.9);
    border_color.set();
    let border_path = NSBezierPath::bezierPathWithOvalInRect(CGRect {
        origin: CGPoint {
            x: center_x - radius,
            y: center_y - radius,
        },
        size: CGSize {
            width: radius * 2.0,
            height: radius * 2.0,
        },
    });
    border_path.setLineWidth(3.0);
    border_path.stroke();

    // Progress arc (if holding) - bright green
    if progress > 0.0 && is_inside {
        let progress_color = NSColor::colorWithRed_green_blue_alpha(0.2, 1.0, 0.2, 1.0);
        progress_color.set();

        // Draw arc from top, going clockwise
        let start_angle: CGFloat = 90.0; // Top of circle
        let end_angle = 90.0 - (progress * 360.0);

        let arc_path = NSBezierPath::bezierPath();
        arc_path.setLineWidth(6.0); // Thicker progress ring

        arc_path.appendBezierPathWithArcWithCenter_radius_startAngle_endAngle_clockwise(
            CGPoint {
                x: center_x,
                y: center_y,
            },
            radius - 5.0,
            start_angle,
            end_angle,
            true, // clockwise
        );
        arc_path.stroke();
    }

    // Draw X - always white and bold
    let x_color = NSColor::colorWithRed_green_blue_alpha(1.0, 1.0, 1.0, 1.0);
    x_color.set();

    let x_size = radius * 0.4;
    let x_path = NSBezierPath::bezierPath();
    x_path.setLineWidth(5.0); // Thicker X

    // First line of X (top-left to bottom-right)
    x_path.moveToPoint(CGPoint {
        x: center_x - x_size,
        y: center_y + x_size,
    });
    x_path.lineToPoint(CGPoint {
        x: center_x + x_size,
        y: center_y - x_size,
    });

    // Second line of X (top-right to bottom-left)
    x_path.moveToPoint(CGPoint {
        x: center_x + x_size,
        y: center_y + x_size,
    });
    x_path.lineToPoint(CGPoint {
        x: center_x - x_size,
        y: center_y - x_size,
    });

    x_path.stroke();
}
