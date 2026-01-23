//! Procedural cat drawing utilities
//!
//! This module provides geometric primitives for drawing the cat character.
//! Each platform uses these coordinates to render the cat using their native
//! drawing APIs (NSBezierPath on macOS, GDI on Windows, X11/Cairo on Linux).
//!
//! The drawing commands are resolution-independent and scale based on the
//! provided size parameters.

use super::{CatDirection, CatFrame, CatState};

/// Color represented as RGBA (0.0-1.0 range)
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

impl Color {
    pub const fn new(r: f64, g: f64, b: f64, a: f64) -> Self {
        Self { r, g, b, a }
    }

    /// Convert to ARGB u32 (for X11/Windows)
    pub fn to_argb_u32(&self) -> u32 {
        let a = (self.a * 255.0) as u32;
        let r = (self.r * 255.0) as u32;
        let g = (self.g * 255.0) as u32;
        let b = (self.b * 255.0) as u32;
        (a << 24) | (r << 16) | (g << 8) | b
    }

    /// Convert to BGR u32 (for Windows GDI)
    pub fn to_bgr_u32(&self) -> u32 {
        let r = (self.r * 255.0) as u32;
        let g = (self.g * 255.0) as u32;
        let b = (self.b * 255.0) as u32;
        (b << 16) | (g << 8) | r
    }
}

// Cat colors
pub const CAT_BODY_COLOR: Color = Color::new(0.4, 0.35, 0.3, 1.0); // Brown/gray
pub const CAT_LIGHT_COLOR: Color = Color::new(0.85, 0.8, 0.75, 1.0); // Light cream for belly/face
pub const CAT_DARK_COLOR: Color = Color::new(0.25, 0.2, 0.15, 1.0); // Dark for stripes/features
pub const CAT_PINK_COLOR: Color = Color::new(0.9, 0.6, 0.6, 1.0); // Pink for nose/ears
pub const CAT_EYE_COLOR: Color = Color::new(0.3, 0.7, 0.3, 1.0); // Green eyes

/// A point in 2D space
#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

/// An ellipse for drawing
#[derive(Debug, Clone, Copy)]
pub struct Ellipse {
    pub center: Point,
    pub width: f64,
    pub height: f64,
    pub color: Color,
}

/// A triangle for drawing (ears, etc.)
#[derive(Debug, Clone, Copy)]
pub struct Triangle {
    pub p1: Point,
    pub p2: Point,
    pub p3: Point,
    pub color: Color,
}

/// A line segment
#[derive(Debug, Clone, Copy)]
pub struct Line {
    pub start: Point,
    pub end: Point,
    pub color: Color,
    pub width: f64,
}

/// Drawing commands for rendering the cat
#[derive(Debug, Clone)]
pub struct CatDrawCommands {
    /// Filled ellipses (body, head, etc.)
    pub ellipses: Vec<Ellipse>,
    /// Filled triangles (ears)
    pub triangles: Vec<Triangle>,
    /// Lines (whiskers, stripes)
    pub lines: Vec<Line>,
}

impl CatDrawCommands {
    fn new() -> Self {
        Self {
            ellipses: Vec::new(),
            triangles: Vec::new(),
            lines: Vec::new(),
        }
    }
}

/// Generate drawing commands for the cat at the given frame
///
/// The cat is drawn centered at (x, y) where y is the bottom of the cat.
pub fn generate_cat_draw_commands(frame: &CatFrame) -> CatDrawCommands {
    if !frame.enabled {
        return CatDrawCommands::new();
    }

    let mut cmds = CatDrawCommands::new();

    // Flip factor for direction (-1.0 for left, 1.0 for right)
    let flip = match frame.direction {
        CatDirection::Left => -1.0,
        CatDirection::Right => 1.0,
    };

    let x = frame.x;
    let y = frame.y;

    match frame.state {
        CatState::Walking => draw_walking_cat(&mut cmds, x, y, flip, frame.frame_number),
        CatState::Sitting => draw_sitting_cat(&mut cmds, x, y, flip, frame.frame_number),
        CatState::Sleeping => draw_sleeping_cat(&mut cmds, x, y, flip, frame.frame_number),
        CatState::Stretching => draw_stretching_cat(&mut cmds, x, y, flip, frame.frame_number),
        CatState::Grooming => draw_grooming_cat(&mut cmds, x, y, flip, frame.frame_number),
        CatState::Playing => draw_playing_cat(&mut cmds, x, y, flip, frame.frame_number),
    }

    cmds
}

/// Draw cat in walking pose
fn draw_walking_cat(cmds: &mut CatDrawCommands, x: f64, y: f64, flip: f64, frame: u32) {
    // Leg animation offsets based on frame
    let leg_offset = match frame % 4 {
        0 => (5.0, -5.0, -5.0, 5.0),  // Front-right forward
        1 => (0.0, 0.0, 0.0, 0.0),    // Neutral
        2 => (-5.0, 5.0, 5.0, -5.0),  // Front-left forward
        _ => (0.0, 0.0, 0.0, 0.0),    // Neutral
    };

    // Tail bob
    let tail_offset = match frame % 4 {
        0 | 2 => 3.0,
        _ => -3.0,
    };

    // Body
    cmds.ellipses.push(Ellipse {
        center: Point::new(x, y - 20.0),
        width: 50.0,
        height: 25.0,
        color: CAT_BODY_COLOR,
    });

    // Legs (front and back)
    let leg_y = y - 8.0;
    let leg_width = 8.0;
    let leg_height = 16.0;

    // Front legs
    cmds.ellipses.push(Ellipse {
        center: Point::new(x + flip * 15.0 + leg_offset.0 * flip, leg_y),
        width: leg_width,
        height: leg_height,
        color: CAT_BODY_COLOR,
    });
    cmds.ellipses.push(Ellipse {
        center: Point::new(x + flip * 8.0 + leg_offset.1 * flip, leg_y),
        width: leg_width,
        height: leg_height,
        color: CAT_BODY_COLOR,
    });

    // Back legs
    cmds.ellipses.push(Ellipse {
        center: Point::new(x - flip * 15.0 + leg_offset.2 * flip, leg_y),
        width: leg_width,
        height: leg_height,
        color: CAT_BODY_COLOR,
    });
    cmds.ellipses.push(Ellipse {
        center: Point::new(x - flip * 8.0 + leg_offset.3 * flip, leg_y),
        width: leg_width,
        height: leg_height,
        color: CAT_BODY_COLOR,
    });

    // Head
    let head_x = x + flip * 25.0;
    let head_y = y - 28.0;
    cmds.ellipses.push(Ellipse {
        center: Point::new(head_x, head_y),
        width: 22.0,
        height: 18.0,
        color: CAT_BODY_COLOR,
    });

    // Ears
    draw_ears(cmds, head_x, head_y - 12.0, flip);

    // Face
    draw_face(cmds, head_x, head_y, flip, false);

    // Tail (curved upward)
    let tail_x = x - flip * 35.0;
    let tail_y = y - 25.0 + tail_offset;
    cmds.ellipses.push(Ellipse {
        center: Point::new(tail_x, tail_y),
        width: 20.0,
        height: 8.0,
        color: CAT_BODY_COLOR,
    });
}

/// Draw cat in sitting pose
fn draw_sitting_cat(cmds: &mut CatDrawCommands, x: f64, y: f64, flip: f64, frame: u32) {
    // Body (more upright when sitting)
    cmds.ellipses.push(Ellipse {
        center: Point::new(x, y - 25.0),
        width: 35.0,
        height: 35.0,
        color: CAT_BODY_COLOR,
    });

    // Front paws
    cmds.ellipses.push(Ellipse {
        center: Point::new(x + flip * 10.0, y - 5.0),
        width: 12.0,
        height: 8.0,
        color: CAT_BODY_COLOR,
    });
    cmds.ellipses.push(Ellipse {
        center: Point::new(x + flip * 2.0, y - 5.0),
        width: 12.0,
        height: 8.0,
        color: CAT_BODY_COLOR,
    });

    // Head (higher up when sitting)
    let head_x = x + flip * 5.0;
    let head_y = y - 48.0;

    // Head bob animation
    let head_offset = match frame % 2 {
        0 => 0.0,
        _ => 2.0,
    };

    cmds.ellipses.push(Ellipse {
        center: Point::new(head_x, head_y + head_offset),
        width: 24.0,
        height: 20.0,
        color: CAT_BODY_COLOR,
    });

    // Ears
    draw_ears(cmds, head_x, head_y - 12.0 + head_offset, flip);

    // Face (looking around based on frame)
    let look_direction = if frame % 2 == 0 { flip } else { -flip };
    draw_face(cmds, head_x + look_direction * 2.0, head_y + head_offset, flip, false);

    // Tail wrapped around
    cmds.ellipses.push(Ellipse {
        center: Point::new(x - flip * 20.0, y - 8.0),
        width: 25.0,
        height: 10.0,
        color: CAT_BODY_COLOR,
    });
}

/// Draw cat sleeping
fn draw_sleeping_cat(cmds: &mut CatDrawCommands, x: f64, y: f64, flip: f64, frame: u32) {
    // Curled up body
    cmds.ellipses.push(Ellipse {
        center: Point::new(x, y - 15.0),
        width: 45.0,
        height: 25.0,
        color: CAT_BODY_COLOR,
    });

    // Head resting on paws
    let head_x = x + flip * 15.0;
    let head_y = y - 20.0;

    // Breathing animation
    let breath_offset = match frame % 2 {
        0 => 1.0,
        _ => -1.0,
    };

    cmds.ellipses.push(Ellipse {
        center: Point::new(head_x, head_y),
        width: 20.0 + breath_offset,
        height: 16.0 + breath_offset * 0.5,
        color: CAT_BODY_COLOR,
    });

    // Ears (folded down slightly)
    cmds.triangles.push(Triangle {
        p1: Point::new(head_x + 6.0, head_y - 6.0),
        p2: Point::new(head_x + 12.0, head_y - 2.0),
        p3: Point::new(head_x + 4.0, head_y + 2.0),
        color: CAT_BODY_COLOR,
    });
    cmds.triangles.push(Triangle {
        p1: Point::new(head_x - 6.0, head_y - 6.0),
        p2: Point::new(head_x - 12.0, head_y - 2.0),
        p3: Point::new(head_x - 4.0, head_y + 2.0),
        color: CAT_BODY_COLOR,
    });

    // Closed eyes (simple lines)
    cmds.lines.push(Line {
        start: Point::new(head_x + flip * 5.0 - 3.0, head_y),
        end: Point::new(head_x + flip * 5.0 + 3.0, head_y),
        color: CAT_DARK_COLOR,
        width: 2.0,
    });
    cmds.lines.push(Line {
        start: Point::new(head_x - flip * 2.0 - 3.0, head_y),
        end: Point::new(head_x - flip * 2.0 + 3.0, head_y),
        color: CAT_DARK_COLOR,
        width: 2.0,
    });

    // Tail curled around body
    cmds.ellipses.push(Ellipse {
        center: Point::new(x - flip * 25.0, y - 10.0),
        width: 20.0,
        height: 8.0,
        color: CAT_BODY_COLOR,
    });

    // "Z" for sleeping (only on some frames)
    if frame % 2 == 0 {
        let z_x = head_x + flip * 20.0;
        let z_y = head_y - 15.0;
        cmds.lines.push(Line {
            start: Point::new(z_x, z_y),
            end: Point::new(z_x + 8.0, z_y),
            color: CAT_LIGHT_COLOR,
            width: 2.0,
        });
        cmds.lines.push(Line {
            start: Point::new(z_x + 8.0, z_y),
            end: Point::new(z_x, z_y + 10.0),
            color: CAT_LIGHT_COLOR,
            width: 2.0,
        });
        cmds.lines.push(Line {
            start: Point::new(z_x, z_y + 10.0),
            end: Point::new(z_x + 8.0, z_y + 10.0),
            color: CAT_LIGHT_COLOR,
            width: 2.0,
        });
    }
}

/// Draw cat stretching
fn draw_stretching_cat(cmds: &mut CatDrawCommands, x: f64, y: f64, flip: f64, frame: u32) {
    // Stretch animation phases
    let stretch_phase = frame % 4;

    let body_width = match stretch_phase {
        0 => 45.0,
        1 => 55.0,
        2 => 60.0,
        _ => 50.0,
    };

    let back_arch = match stretch_phase {
        1 | 2 => 5.0,
        _ => 0.0,
    };

    // Body (stretched)
    cmds.ellipses.push(Ellipse {
        center: Point::new(x, y - 20.0 - back_arch),
        width: body_width,
        height: 22.0,
        color: CAT_BODY_COLOR,
    });

    // Front legs (stretched forward)
    let front_stretch = match stretch_phase {
        1 | 2 => 10.0,
        _ => 0.0,
    };
    cmds.ellipses.push(Ellipse {
        center: Point::new(x + flip * (20.0 + front_stretch), y - 6.0),
        width: 10.0,
        height: 14.0,
        color: CAT_BODY_COLOR,
    });

    // Back legs
    cmds.ellipses.push(Ellipse {
        center: Point::new(x - flip * 20.0, y - 8.0),
        width: 10.0,
        height: 14.0,
        color: CAT_BODY_COLOR,
    });

    // Head
    let head_x = x + flip * (25.0 + front_stretch * 0.5);
    let head_y = y - 28.0 - back_arch;
    cmds.ellipses.push(Ellipse {
        center: Point::new(head_x, head_y),
        width: 22.0,
        height: 18.0,
        color: CAT_BODY_COLOR,
    });

    // Ears
    draw_ears(cmds, head_x, head_y - 12.0, flip);

    // Face (eyes closed or squinting during stretch)
    if stretch_phase == 2 {
        // Eyes closed during peak stretch
        cmds.lines.push(Line {
            start: Point::new(head_x + flip * 5.0 - 3.0, head_y),
            end: Point::new(head_x + flip * 5.0 + 3.0, head_y),
            color: CAT_DARK_COLOR,
            width: 2.0,
        });
    } else {
        draw_face(cmds, head_x, head_y, flip, false);
    }

    // Tail up
    let tail_angle = match stretch_phase {
        2 => -20.0,
        _ => -10.0,
    };
    cmds.ellipses.push(Ellipse {
        center: Point::new(x - flip * 35.0, y - 30.0 + tail_angle),
        width: 20.0,
        height: 8.0,
        color: CAT_BODY_COLOR,
    });
}

/// Draw cat grooming
fn draw_grooming_cat(cmds: &mut CatDrawCommands, x: f64, y: f64, flip: f64, frame: u32) {
    // Sitting body
    cmds.ellipses.push(Ellipse {
        center: Point::new(x, y - 22.0),
        width: 35.0,
        height: 30.0,
        color: CAT_BODY_COLOR,
    });

    // Grooming animation - licking paw or body
    let groom_phase = frame % 3;

    // Head position changes based on what's being groomed
    let (head_x, head_y) = match groom_phase {
        0 => (x + flip * 15.0, y - 35.0), // Looking at paw
        1 => (x, y - 38.0),                // Looking at body
        _ => (x - flip * 5.0, y - 40.0),   // Looking back
    };

    cmds.ellipses.push(Ellipse {
        center: Point::new(head_x, head_y),
        width: 22.0,
        height: 18.0,
        color: CAT_BODY_COLOR,
    });

    // Ears
    draw_ears(cmds, head_x, head_y - 12.0, flip);

    // Paw raised for grooming
    if groom_phase == 0 {
        cmds.ellipses.push(Ellipse {
            center: Point::new(x + flip * 18.0, y - 30.0),
            width: 10.0,
            height: 10.0,
            color: CAT_BODY_COLOR,
        });
    }

    // Face with tongue out
    draw_face(cmds, head_x, head_y, flip, true);

    // Front paw (not raised)
    cmds.ellipses.push(Ellipse {
        center: Point::new(x + flip * 5.0, y - 5.0),
        width: 12.0,
        height: 8.0,
        color: CAT_BODY_COLOR,
    });

    // Tail
    cmds.ellipses.push(Ellipse {
        center: Point::new(x - flip * 22.0, y - 8.0),
        width: 22.0,
        height: 8.0,
        color: CAT_BODY_COLOR,
    });
}

/// Draw cat playing
fn draw_playing_cat(cmds: &mut CatDrawCommands, x: f64, y: f64, flip: f64, frame: u32) {
    let play_phase = frame % 4;

    // Playful crouching body
    let crouch_offset = match play_phase {
        0 | 2 => 3.0,
        _ => 0.0,
    };

    cmds.ellipses.push(Ellipse {
        center: Point::new(x, y - 18.0 + crouch_offset),
        width: 45.0,
        height: 22.0,
        color: CAT_BODY_COLOR,
    });

    // Wiggling butt for pounce
    let butt_wiggle = match play_phase {
        0 => 3.0,
        2 => -3.0,
        _ => 0.0,
    };

    // Back legs
    cmds.ellipses.push(Ellipse {
        center: Point::new(x - flip * 15.0 + butt_wiggle, y - 6.0),
        width: 10.0,
        height: 14.0,
        color: CAT_BODY_COLOR,
    });

    // Front legs (ready to pounce)
    cmds.ellipses.push(Ellipse {
        center: Point::new(x + flip * 18.0, y - 6.0),
        width: 10.0,
        height: 14.0,
        color: CAT_BODY_COLOR,
    });

    // Head (focused on "prey")
    let head_x = x + flip * 22.0;
    let head_y = y - 25.0 + crouch_offset;
    cmds.ellipses.push(Ellipse {
        center: Point::new(head_x, head_y),
        width: 22.0,
        height: 18.0,
        color: CAT_BODY_COLOR,
    });

    // Ears (alert, forward)
    draw_ears(cmds, head_x, head_y - 14.0, flip);

    // Face with dilated pupils (excited)
    draw_face(cmds, head_x, head_y, flip, false);

    // Twitching tail
    let tail_twitch = match play_phase {
        0 | 2 => 5.0,
        _ => -5.0,
    };
    cmds.ellipses.push(Ellipse {
        center: Point::new(x - flip * 35.0, y - 15.0 + tail_twitch),
        width: 22.0,
        height: 8.0,
        color: CAT_BODY_COLOR,
    });
}

/// Draw cat ears
fn draw_ears(cmds: &mut CatDrawCommands, head_x: f64, head_y: f64, _flip: f64) {
    // Outer ears (main color)
    cmds.triangles.push(Triangle {
        p1: Point::new(head_x + 8.0, head_y),
        p2: Point::new(head_x + 14.0, head_y - 12.0),
        p3: Point::new(head_x + 2.0, head_y - 4.0),
        color: CAT_BODY_COLOR,
    });
    cmds.triangles.push(Triangle {
        p1: Point::new(head_x - 8.0, head_y),
        p2: Point::new(head_x - 14.0, head_y - 12.0),
        p3: Point::new(head_x - 2.0, head_y - 4.0),
        color: CAT_BODY_COLOR,
    });

    // Inner ears (pink)
    cmds.triangles.push(Triangle {
        p1: Point::new(head_x + 8.0, head_y + 1.0),
        p2: Point::new(head_x + 12.0, head_y - 8.0),
        p3: Point::new(head_x + 4.0, head_y - 2.0),
        color: CAT_PINK_COLOR,
    });
    cmds.triangles.push(Triangle {
        p1: Point::new(head_x - 8.0, head_y + 1.0),
        p2: Point::new(head_x - 12.0, head_y - 8.0),
        p3: Point::new(head_x - 4.0, head_y - 2.0),
        color: CAT_PINK_COLOR,
    });
}

/// Draw cat face features
fn draw_face(cmds: &mut CatDrawCommands, head_x: f64, head_y: f64, flip: f64, tongue_out: bool) {
    // Eyes (white sclera)
    cmds.ellipses.push(Ellipse {
        center: Point::new(head_x + flip * 5.0, head_y - 2.0),
        width: 6.0,
        height: 5.0,
        color: CAT_LIGHT_COLOR,
    });
    cmds.ellipses.push(Ellipse {
        center: Point::new(head_x - flip * 2.0, head_y - 2.0),
        width: 6.0,
        height: 5.0,
        color: CAT_LIGHT_COLOR,
    });

    // Pupils (looking forward)
    cmds.ellipses.push(Ellipse {
        center: Point::new(head_x + flip * 5.5, head_y - 2.0),
        width: 3.0,
        height: 4.0,
        color: CAT_EYE_COLOR,
    });
    cmds.ellipses.push(Ellipse {
        center: Point::new(head_x - flip * 1.5, head_y - 2.0),
        width: 3.0,
        height: 4.0,
        color: CAT_EYE_COLOR,
    });

    // Nose
    cmds.ellipses.push(Ellipse {
        center: Point::new(head_x + flip * 2.0, head_y + 4.0),
        width: 4.0,
        height: 3.0,
        color: CAT_PINK_COLOR,
    });

    // Whiskers
    let whisker_y = head_y + 3.0;
    let whisker_x = head_x + flip * 5.0;
    cmds.lines.push(Line {
        start: Point::new(whisker_x, whisker_y),
        end: Point::new(whisker_x + flip * 15.0, whisker_y - 3.0),
        color: CAT_LIGHT_COLOR,
        width: 1.0,
    });
    cmds.lines.push(Line {
        start: Point::new(whisker_x, whisker_y),
        end: Point::new(whisker_x + flip * 15.0, whisker_y),
        color: CAT_LIGHT_COLOR,
        width: 1.0,
    });
    cmds.lines.push(Line {
        start: Point::new(whisker_x, whisker_y),
        end: Point::new(whisker_x + flip * 15.0, whisker_y + 3.0),
        color: CAT_LIGHT_COLOR,
        width: 1.0,
    });

    // Tongue (if grooming)
    if tongue_out {
        cmds.ellipses.push(Ellipse {
            center: Point::new(head_x + flip * 2.0, head_y + 8.0),
            width: 4.0,
            height: 5.0,
            color: CAT_PINK_COLOR,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_to_argb() {
        let color = Color::new(1.0, 0.5, 0.0, 0.8);
        let argb = color.to_argb_u32();
        // Alpha: 0xCC (204), Red: 0xFF (255), Green: 0x7F (127), Blue: 0x00 (0)
        assert_eq!(argb & 0xFF000000, 0xCC000000); // Alpha
        assert_eq!(argb & 0x00FF0000, 0x00FF0000); // Red
    }

    #[test]
    fn test_color_to_bgr() {
        let color = Color::new(1.0, 0.0, 0.0, 1.0); // Pure red
        let bgr = color.to_bgr_u32();
        // In BGR: Blue=0, Green=0, Red=255
        assert_eq!(bgr, 0x000000FF);
    }

    #[test]
    fn test_generate_commands_disabled() {
        let frame = CatFrame {
            enabled: false,
            ..Default::default()
        };
        let cmds = generate_cat_draw_commands(&frame);
        assert!(cmds.ellipses.is_empty());
        assert!(cmds.triangles.is_empty());
        assert!(cmds.lines.is_empty());
    }

    #[test]
    fn test_generate_commands_walking() {
        let frame = CatFrame {
            state: CatState::Walking,
            enabled: true,
            ..Default::default()
        };
        let cmds = generate_cat_draw_commands(&frame);
        // Should have body, legs, head, ears, eyes, nose, whiskers
        assert!(!cmds.ellipses.is_empty());
    }

    #[test]
    fn test_generate_commands_all_states() {
        for state in [
            CatState::Walking,
            CatState::Sitting,
            CatState::Sleeping,
            CatState::Stretching,
            CatState::Grooming,
            CatState::Playing,
        ] {
            let frame = CatFrame {
                state,
                enabled: true,
                ..Default::default()
            };
            let cmds = generate_cat_draw_commands(&frame);
            // All states should produce some drawing commands
            assert!(!cmds.ellipses.is_empty(), "State {:?} should have ellipses", state);
        }
    }

    #[test]
    fn test_direction_flip() {
        let frame_left = CatFrame {
            state: CatState::Walking,
            direction: CatDirection::Left,
            enabled: true,
            ..Default::default()
        };
        let frame_right = CatFrame {
            state: CatState::Walking,
            direction: CatDirection::Right,
            enabled: true,
            ..Default::default()
        };

        let cmds_left = generate_cat_draw_commands(&frame_left);
        let cmds_right = generate_cat_draw_commands(&frame_right);

        // Both should have same number of drawing commands
        assert_eq!(cmds_left.ellipses.len(), cmds_right.ellipses.len());
    }
}
