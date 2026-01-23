//! Cat animation view for macOS overlay
//!
//! This view renders the animated cat companion on the shield overlay.

use crate::cat_animation::{self, drawing};
use objc2::rc::Retained;
use objc2::{define_class, msg_send};
use objc2_app_kit::{NSBezierPath, NSColor, NSView};
use objc2_core_foundation::{CGPoint, CGRect, CGSize};
use objc2_foundation::MainThreadMarker;

/// Ivars for the CatAnimationView
pub struct CatAnimationViewIvars {}

define_class!(
    #[unsafe(super(NSView))]
    #[name = "CatAnimationView"]
    #[ivars = CatAnimationViewIvars]
    pub struct CatAnimationView;

    impl CatAnimationView {
        #[unsafe(method(drawRect:))]
        unsafe fn draw_rect(&self, _dirty_rect: CGRect) {
            draw_cat(self);
        }
    }
);

impl CatAnimationView {
    pub fn new(mtm: MainThreadMarker, frame: CGRect) -> Retained<Self> {
        let this = mtm.alloc::<CatAnimationView>();
        let this = this.set_ivars(CatAnimationViewIvars {});
        unsafe { msg_send![super(this), initWithFrame: frame] }
    }
}

/// Get the current time in seconds (for animation updates)
fn get_current_time() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

/// Draw the cat using the drawing commands
fn draw_cat(_view: &NSView) {
    // Update the animation state and get the current frame
    let frame = cat_animation::update(get_current_time());

    if !frame.enabled {
        return;
    }

    // Generate drawing commands
    let commands = drawing::generate_cat_draw_commands(&frame);

    // Draw all ellipses
    for ellipse in &commands.ellipses {
        let color = NSColor::colorWithRed_green_blue_alpha(
            ellipse.color.r,
            ellipse.color.g,
            ellipse.color.b,
            ellipse.color.a,
        );
        color.set();

        let path = NSBezierPath::bezierPathWithOvalInRect(CGRect {
            origin: CGPoint {
                x: ellipse.center.x - ellipse.width / 2.0,
                y: ellipse.center.y - ellipse.height / 2.0,
            },
            size: CGSize {
                width: ellipse.width,
                height: ellipse.height,
            },
        });
        path.fill();
    }

    // Draw all triangles
    for triangle in &commands.triangles {
        let color = NSColor::colorWithRed_green_blue_alpha(
            triangle.color.r,
            triangle.color.g,
            triangle.color.b,
            triangle.color.a,
        );
        color.set();

        let path = NSBezierPath::bezierPath();
        path.moveToPoint(CGPoint {
            x: triangle.p1.x,
            y: triangle.p1.y,
        });
        path.lineToPoint(CGPoint {
            x: triangle.p2.x,
            y: triangle.p2.y,
        });
        path.lineToPoint(CGPoint {
            x: triangle.p3.x,
            y: triangle.p3.y,
        });
        path.closePath();
        path.fill();
    }

    // Draw all lines
    for line in &commands.lines {
        let color = NSColor::colorWithRed_green_blue_alpha(
            line.color.r,
            line.color.g,
            line.color.b,
            line.color.a,
        );
        color.set();

        let path = NSBezierPath::bezierPath();
        path.setLineWidth(line.width);
        path.moveToPoint(CGPoint {
            x: line.start.x,
            y: line.start.y,
        });
        path.lineToPoint(CGPoint {
            x: line.end.x,
            y: line.end.y,
        });
        path.stroke();
    }
}
