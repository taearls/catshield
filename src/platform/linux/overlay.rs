//! Linux overlay window implementation for Cat Shield (X11)
//!
//! This module implements the [`OverlayWindow`] trait for Linux using X11.
//! For native Wayland support, see the [`wayland_overlay`](super::wayland_overlay) module.
//!
//! # X11 Implementation
//!
//! The X11 implementation uses:
//! - `_NET_WM_WINDOW_TYPE_SPLASH` or `_NET_WM_STATE_ABOVE` for topmost
//! - `_NET_WM_STATE_FULLSCREEN` for fullscreen coverage
//! - Override-redirect option for true topmost behavior
//! - ARGB visual for transparency (when available)
//!
//! # XWayland Compatibility
//!
//! This implementation works on most Wayland compositors via XWayland.
//! If you need native Wayland support (better performance, no XWayland),
//! use [`WaylandOverlayWindow`](super::WaylandOverlayWindow) instead.
//!
//! # Choosing Between X11 and Wayland
//!
//! ```ignore
//! use cat_shield::platform::linux::{
//!     LinuxOverlayWindow, WaylandOverlayWindow, is_native_wayland_overlay_available
//! };
//!
//! // Prefer native Wayland when available
//! if is_native_wayland_overlay_available() {
//!     let mut overlay = WaylandOverlayWindow::new();
//!     overlay.create()?;
//! } else {
//!     // Fall back to X11 (works on X11 and XWayland)
//!     let mut overlay = LinuxOverlayWindow::new();
//!     overlay.create()?;
//! }
//! ```
//!
//! # Implementation Notes
//!
//! The close button and timer display are rendered using simple X11 drawing
//! primitives. The window captures mouse events for the close button interaction.

use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Arc, Mutex, RwLock};

use log::{debug, info, warn};
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{
    Atom, AtomEnum, ColormapAlloc, ConfigureWindowAux, ConnectionExt, CreateGCAux, CreateWindowAux,
    EventMask, PropMode, Screen, VisualClass, Visualid, Window, WindowClass,
};
use x11rb::protocol::Event;
use x11rb::rust_connection::RustConnection;
use x11rb::wrapper::ConnectionExt as WrapperConnectionExt;

use crate::platform::errors::WindowError;
use crate::platform::traits::OverlayWindow;
use crate::platform::types::Rect;

/// Background color (dark semi-transparent) - ARGB format
const BG_COLOR_ARGB: u32 = 0xCC202020; // 80% opacity, dark gray RGB(32, 32, 32)

/// Close button color (red) - ARGB format
const CLOSE_BUTTON_COLOR: u32 = 0xFFFF4040; // Opaque red RGB(255, 64, 64)

/// Close button hover color (lighter red) - ARGB format
const CLOSE_BUTTON_HOVER_COLOR: u32 = 0xFFFF6060; // Lighter red

/// Text color (white) - ARGB format
const TEXT_COLOR: u32 = 0xFFFFFFFF; // White

/// Close button radius in pixels
const CLOSE_BUTTON_RADIUS: i32 = 30;

/// Close button margin from bottom-right corner
const CLOSE_BUTTON_MARGIN: i32 = 50;

/// Global window ID
static OVERLAY_WINDOW_ID: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// Global visibility state
static OVERLAY_VISIBLE: AtomicBool = AtomicBool::new(false);

/// Global opacity (0-255)
static OVERLAY_OPACITY: AtomicU8 = AtomicU8::new(200);

/// Mouse hover state for close button
static CLOSE_BUTTON_HOVERED: AtomicBool = AtomicBool::new(false);

/// Callback for close button click (protected by mutex for thread safety)
static CLOSE_CALLBACK: Mutex<Option<Arc<dyn Fn() + Send + Sync>>> = Mutex::new(None);

/// Timer text to display
static TIMER_TEXT: RwLock<Option<String>> = RwLock::new(None);

/// Callback type for close button click
pub type CloseCallback = Arc<dyn Fn() + Send + Sync>;

/// Detects whether the current session is running on Wayland
fn is_wayland_session() -> bool {
    std::env::var("WAYLAND_DISPLAY").is_ok()
        || std::env::var("XDG_SESSION_TYPE")
            .map(|v| v.to_lowercase() == "wayland")
            .unwrap_or(false)
}

/// Detects whether the current session is running on X11
fn is_x11_session() -> bool {
    std::env::var("DISPLAY").is_ok()
}

/// Linux overlay window implementation.
///
/// This struct manages the fullscreen overlay window on Linux.
/// It implements the [`OverlayWindow`] trait for platform abstraction.
///
/// # Example
///
/// ```ignore
/// let mut overlay = LinuxOverlayWindow::new();
/// overlay.create()?;
/// overlay.set_opacity(0.8);
/// overlay.show();
/// // ... later ...
/// overlay.close();
/// ```
pub struct LinuxOverlayWindow {
    /// X11 connection
    connection: Option<RustConnection>,
    /// The overlay window ID
    window_id: Window,
    /// Graphics context for drawing
    gc: Option<u32>,
    /// Whether the window is currently visible
    visible: bool,
    /// Current opacity (0.0 to 1.0)
    opacity: f64,
    /// Cached window bounds
    bounds: Rect,
    /// Screen number
    screen_num: usize,
    /// Atoms for EWMH properties
    atoms: Option<OverlayAtoms>,
    /// Whether we're running under Wayland (XWayland fallback)
    is_wayland: bool,
}

/// EWMH atoms used for window management
#[derive(Debug)]
struct OverlayAtoms {
    wm_state: Atom,
    wm_state_above: Atom,
    wm_state_fullscreen: Atom,
    wm_state_skip_taskbar: Atom,
    wm_state_skip_pager: Atom,
    wm_window_type: Atom,
    wm_window_type_splash: Atom,
    net_wm_opacity: Atom,
}

impl LinuxOverlayWindow {
    /// Creates a new Linux overlay window instance.
    ///
    /// The window is not created until [`create()`](Self::create) is called.
    pub fn new() -> Self {
        let is_wayland = is_wayland_session();
        if is_wayland {
            debug!("Detected Wayland session (will use XWayland if available)");
        } else if is_x11_session() {
            debug!("Detected X11 session");
        }

        Self {
            connection: None,
            window_id: 0,
            gc: None,
            visible: false,
            opacity: 0.8,
            bounds: Rect::zero(),
            screen_num: 0,
            atoms: None,
            is_wayland,
        }
    }

    /// Sets the callback function for close button clicks.
    ///
    /// The callback will be invoked when the user clicks the close button.
    /// This method is thread-safe.
    ///
    /// # Arguments
    ///
    /// * `callback` - A function that is called when the close button is clicked
    pub fn set_close_callback(callback: CloseCallback) {
        if let Ok(mut guard) = CLOSE_CALLBACK.lock() {
            *guard = Some(callback);
        }
    }

    /// Clears the close callback.
    pub fn clear_close_callback() {
        if let Ok(mut guard) = CLOSE_CALLBACK.lock() {
            *guard = None;
        }
    }

    /// Sets the timer display text.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to display, or None to hide the timer
    pub fn set_timer_text(text: Option<&str>) {
        if let Ok(mut timer_text) = TIMER_TEXT.write() {
            *timer_text = text.map(|s| s.to_string());
        }
    }

    /// Interns an atom, returning an error if it fails
    fn intern_atom(&self, name: &[u8]) -> Result<Atom, WindowError> {
        let conn = self
            .connection
            .as_ref()
            .ok_or_else(|| WindowError::Platform("No X11 connection".into()))?;

        conn.intern_atom(false, name)
            .map_err(|e| WindowError::Platform(format!("Failed to intern atom: {e}")))?
            .reply()
            .map(|r| r.atom)
            .map_err(|e| WindowError::Platform(format!("Failed to get atom reply: {e}")))
    }

    /// Initialize EWMH atoms for window management
    fn init_atoms(&mut self) -> Result<(), WindowError> {
        let atoms = OverlayAtoms {
            wm_state: self.intern_atom(b"_NET_WM_STATE")?,
            wm_state_above: self.intern_atom(b"_NET_WM_STATE_ABOVE")?,
            wm_state_fullscreen: self.intern_atom(b"_NET_WM_STATE_FULLSCREEN")?,
            wm_state_skip_taskbar: self.intern_atom(b"_NET_WM_STATE_SKIP_TASKBAR")?,
            wm_state_skip_pager: self.intern_atom(b"_NET_WM_STATE_SKIP_PAGER")?,
            wm_window_type: self.intern_atom(b"_NET_WM_WINDOW_TYPE")?,
            wm_window_type_splash: self.intern_atom(b"_NET_WM_WINDOW_TYPE_SPLASH")?,
            net_wm_opacity: self.intern_atom(b"_NET_WM_WINDOW_OPACITY")?,
        };
        self.atoms = Some(atoms);
        Ok(())
    }

    /// Gets the screen info for the specified screen
    fn get_screen(&self) -> Result<&Screen, WindowError> {
        let conn = self
            .connection
            .as_ref()
            .ok_or_else(|| WindowError::Platform("No X11 connection".into()))?;

        conn.setup()
            .roots
            .get(self.screen_num)
            .ok_or_else(|| WindowError::DisplayNotAvailable("Screen not found".into()))
    }

    /// Finds a 32-bit ARGB visual for transparency support
    fn find_argb_visual(&self, screen: &Screen) -> Option<(Visualid, u8)> {
        for depth_info in &screen.allowed_depths {
            if depth_info.depth == 32 {
                for visual in &depth_info.visuals {
                    if visual.class == VisualClass::TRUE_COLOR {
                        return Some((visual.visual_id, 32));
                    }
                }
            }
        }
        None
    }

    /// Sets EWMH window properties for topmost fullscreen behavior
    fn set_window_properties(&self) -> Result<(), WindowError> {
        let conn = self
            .connection
            .as_ref()
            .ok_or_else(|| WindowError::Platform("No X11 connection".into()))?;
        let atoms = self
            .atoms
            .as_ref()
            .ok_or_else(|| WindowError::Platform("Atoms not initialized".into()))?;

        // Set window type to splash (doesn't require decoration)
        conn.change_property32(
            PropMode::REPLACE,
            self.window_id,
            atoms.wm_window_type,
            AtomEnum::ATOM,
            &[atoms.wm_window_type_splash],
        )
        .map_err(|e| WindowError::ConfigurationFailed(format!("Failed to set window type: {e}")))?;

        // Set window states: above, fullscreen, skip taskbar/pager
        conn.change_property32(
            PropMode::REPLACE,
            self.window_id,
            atoms.wm_state,
            AtomEnum::ATOM,
            &[
                atoms.wm_state_above,
                atoms.wm_state_fullscreen,
                atoms.wm_state_skip_taskbar,
                atoms.wm_state_skip_pager,
            ],
        )
        .map_err(|e| {
            WindowError::ConfigurationFailed(format!("Failed to set window state: {e}"))
        })?;

        // Set window opacity using _NET_WM_WINDOW_OPACITY
        let opacity_value = ((self.opacity * u32::MAX as f64) as u32).to_ne_bytes();
        conn.change_property8(
            PropMode::REPLACE,
            self.window_id,
            atoms.net_wm_opacity,
            AtomEnum::CARDINAL,
            &opacity_value,
        )
        .map_err(|e| WindowError::ConfigurationFailed(format!("Failed to set opacity: {e}")))?;

        conn.flush()
            .map_err(|e| WindowError::ConfigurationFailed(format!("Failed to flush: {e}")))?;

        Ok(())
    }

    /// Creates a graphics context for drawing
    fn create_graphics_context(&mut self) -> Result<(), WindowError> {
        let conn = self
            .connection
            .as_ref()
            .ok_or_else(|| WindowError::Platform("No X11 connection".into()))?;

        let gc_id = conn
            .generate_id()
            .map_err(|e| WindowError::CreationFailed(format!("Failed to generate GC ID: {e}")))?;

        let gc_aux = CreateGCAux::new()
            .foreground(TEXT_COLOR)
            .background(BG_COLOR_ARGB);

        conn.create_gc(gc_id, self.window_id, &gc_aux)
            .map_err(|e| WindowError::CreationFailed(format!("Failed to create GC: {e}")))?;

        self.gc = Some(gc_id);
        Ok(())
    }

    /// Draws the overlay content (background, close button, timer)
    pub fn draw(&self) -> Result<(), WindowError> {
        let conn = self
            .connection
            .as_ref()
            .ok_or_else(|| WindowError::Platform("No X11 connection".into()))?;

        // Verify graphics context was created (we use temporary GCs for drawing operations)
        let _gc = self
            .gc
            .ok_or_else(|| WindowError::Platform("Graphics context not created".into()))?;

        let width = self.bounds.width as u16;
        let height = self.bounds.height as u16;

        // Fill background
        let bg_gc = conn
            .generate_id()
            .map_err(|e| WindowError::Platform(format!("Failed to generate ID: {e}")))?;
        let bg_aux = CreateGCAux::new().foreground(BG_COLOR_ARGB & 0x00FFFFFF);
        conn.create_gc(bg_gc, self.window_id, &bg_aux)
            .map_err(|e| WindowError::Platform(format!("Failed to create background GC: {e}")))?;

        conn.poly_fill_rectangle(
            self.window_id,
            bg_gc,
            &[x11rb::protocol::xproto::Rectangle {
                x: 0,
                y: 0,
                width,
                height,
            }],
        )
        .map_err(|e| WindowError::Platform(format!("Failed to fill background: {e}")))?;

        conn.free_gc(bg_gc)
            .map_err(|e| WindowError::Platform(format!("Failed to free background GC: {e}")))?;

        // Draw close button
        let button_x = width as i16 - CLOSE_BUTTON_MARGIN as i16 - CLOSE_BUTTON_RADIUS as i16;
        let button_y = height as i16 - CLOSE_BUTTON_MARGIN as i16 - CLOSE_BUTTON_RADIUS as i16;

        let is_hovered = CLOSE_BUTTON_HOVERED.load(Ordering::SeqCst);
        let button_color = if is_hovered {
            CLOSE_BUTTON_HOVER_COLOR
        } else {
            CLOSE_BUTTON_COLOR
        };

        // Create GC for close button
        let btn_gc = conn
            .generate_id()
            .map_err(|e| WindowError::Platform(format!("Failed to generate ID: {e}")))?;
        let btn_aux = CreateGCAux::new().foreground(button_color & 0x00FFFFFF);
        conn.create_gc(btn_gc, self.window_id, &btn_aux)
            .map_err(|e| WindowError::Platform(format!("Failed to create button GC: {e}")))?;

        // Draw filled circle for close button (approximated with arc)
        let diameter = (CLOSE_BUTTON_RADIUS * 2) as u16;
        conn.poly_fill_arc(
            self.window_id,
            btn_gc,
            &[x11rb::protocol::xproto::Arc {
                x: button_x - CLOSE_BUTTON_RADIUS as i16,
                y: button_y - CLOSE_BUTTON_RADIUS as i16,
                width: diameter,
                height: diameter,
                angle1: 0,
                angle2: 360 * 64, // Full circle in 1/64th degree units
            }],
        )
        .map_err(|e| WindowError::Platform(format!("Failed to draw close button: {e}")))?;

        conn.free_gc(btn_gc)
            .map_err(|e| WindowError::Platform(format!("Failed to free button GC: {e}")))?;

        // Draw "X" on the close button
        let x_gc = conn
            .generate_id()
            .map_err(|e| WindowError::Platform(format!("Failed to generate ID: {e}")))?;
        let x_aux = CreateGCAux::new()
            .foreground(TEXT_COLOR & 0x00FFFFFF)
            .line_width(3);
        conn.create_gc(x_gc, self.window_id, &x_aux)
            .map_err(|e| WindowError::Platform(format!("Failed to create X GC: {e}")))?;

        // Draw X using two lines
        let x_offset = (CLOSE_BUTTON_RADIUS as f64 * 0.5) as i16;
        conn.poly_line(
            x11rb::protocol::xproto::CoordMode::ORIGIN,
            self.window_id,
            x_gc,
            &[
                x11rb::protocol::xproto::Point {
                    x: button_x - x_offset,
                    y: button_y - x_offset,
                },
                x11rb::protocol::xproto::Point {
                    x: button_x + x_offset,
                    y: button_y + x_offset,
                },
            ],
        )
        .map_err(|e| WindowError::Platform(format!("Failed to draw X line 1: {e}")))?;

        conn.poly_line(
            x11rb::protocol::xproto::CoordMode::ORIGIN,
            self.window_id,
            x_gc,
            &[
                x11rb::protocol::xproto::Point {
                    x: button_x + x_offset,
                    y: button_y - x_offset,
                },
                x11rb::protocol::xproto::Point {
                    x: button_x - x_offset,
                    y: button_y + x_offset,
                },
            ],
        )
        .map_err(|e| WindowError::Platform(format!("Failed to draw X line 2: {e}")))?;

        conn.free_gc(x_gc)
            .map_err(|e| WindowError::Platform(format!("Failed to free X GC: {e}")))?;

        // Draw timer text if set
        if let Ok(timer_text) = TIMER_TEXT.read() {
            if let Some(ref text) = *timer_text {
                // Create text GC
                let text_gc = conn
                    .generate_id()
                    .map_err(|e| WindowError::Platform(format!("Failed to generate ID: {e}")))?;
                let text_aux = CreateGCAux::new().foreground(TEXT_COLOR & 0x00FFFFFF);
                conn.create_gc(text_gc, self.window_id, &text_aux)
                    .map_err(|e| WindowError::Platform(format!("Failed to create text GC: {e}")))?;

                // Draw text centered (approximate centering)
                let text_x = (width / 2).saturating_sub((text.len() as u16 * 4));
                let text_y = height / 2;

                conn.image_text8(
                    self.window_id,
                    text_gc,
                    text_x as i16,
                    text_y as i16,
                    text.as_bytes(),
                )
                .map_err(|e| WindowError::Platform(format!("Failed to draw timer text: {e}")))?;

                conn.free_gc(text_gc)
                    .map_err(|e| WindowError::Platform(format!("Failed to free text GC: {e}")))?;
            }
        }

        conn.flush()
            .map_err(|e| WindowError::Platform(format!("Failed to flush: {e}")))?;

        Ok(())
    }

    /// Processes pending X11 events (should be called in event loop)
    pub fn process_events(&mut self) -> Result<bool, WindowError> {
        let conn = self
            .connection
            .as_ref()
            .ok_or_else(|| WindowError::Platform("No X11 connection".into()))?;

        // Non-blocking poll for events
        match conn.poll_for_event() {
            Ok(Some(event)) => {
                match event {
                    Event::Expose(ev) => {
                        if ev.window == self.window_id && ev.count == 0 {
                            self.draw()?;
                        }
                    }
                    Event::MotionNotify(ev) => {
                        if ev.event == self.window_id {
                            let was_hovered = CLOSE_BUTTON_HOVERED.load(Ordering::SeqCst);
                            let is_hovered = self.is_point_in_close_button(ev.event_x, ev.event_y);

                            if was_hovered != is_hovered {
                                CLOSE_BUTTON_HOVERED.store(is_hovered, Ordering::SeqCst);
                                self.draw()?;
                            }
                        }
                    }
                    Event::ButtonPress(ev) => {
                        if ev.event == self.window_id && ev.detail == 1 {
                            // Left mouse button
                            if self.is_point_in_close_button(ev.event_x, ev.event_y) {
                                debug!("Close button clicked");
                                let callback =
                                    CLOSE_CALLBACK.lock().ok().and_then(|guard| guard.clone());
                                if let Some(cb) = callback {
                                    cb();
                                    return Ok(true); // Signal that close was requested
                                }
                            }
                        }
                    }
                    Event::ClientMessage(ev) => {
                        // Handle WM_DELETE_WINDOW if needed
                        debug!("Received client message: {:?}", ev);
                    }
                    _ => {}
                }
            }
            Ok(None) => {}
            Err(e) => {
                warn!("Error polling for events: {}", e);
            }
        }

        Ok(false)
    }

    /// Checks if a point is within the close button area
    fn is_point_in_close_button(&self, x: i16, y: i16) -> bool {
        let width = self.bounds.width as i16;
        let height = self.bounds.height as i16;

        let button_x = width - CLOSE_BUTTON_MARGIN as i16 - CLOSE_BUTTON_RADIUS as i16;
        let button_y = height - CLOSE_BUTTON_MARGIN as i16 - CLOSE_BUTTON_RADIUS as i16;

        let dx = x - button_x;
        let dy = y - button_y;
        let distance_squared = (dx as i32) * (dx as i32) + (dy as i32) * (dy as i32);

        distance_squared <= (CLOSE_BUTTON_RADIUS * CLOSE_BUTTON_RADIUS)
    }
}

impl Default for LinuxOverlayWindow {
    fn default() -> Self {
        Self::new()
    }
}

impl OverlayWindow for LinuxOverlayWindow {
    fn create(&mut self) -> Result<(), WindowError> {
        // Connect to X11 (works with XWayland too)
        let (conn, screen_num) = RustConnection::connect(None).map_err(|e| {
            if self.is_wayland {
                WindowError::CreationFailed(format!(
                    "Failed to connect to X11/XWayland: {e}. \
                    For native Wayland support, use WaylandOverlayWindow instead \
                    (requires wlr-layer-shell protocol support)."
                ))
            } else {
                WindowError::CreationFailed(format!("Failed to connect to X11: {e}"))
            }
        })?;

        self.connection = Some(conn);
        self.screen_num = screen_num;

        // Initialize EWMH atoms
        self.init_atoms()?;

        // Get screen info
        let screen = self.get_screen()?.clone();
        let screen_width = screen.width_in_pixels;
        let screen_height = screen.height_in_pixels;

        if screen_width == 0 || screen_height == 0 {
            return Err(WindowError::DisplayNotAvailable(
                "Screen has zero dimensions".into(),
            ));
        }

        debug!(
            "Creating overlay window: {}x{}",
            screen_width, screen_height
        );

        let conn = self.connection.as_ref().unwrap();

        // Try to find an ARGB visual for transparency
        let (visual_id, depth) = self
            .find_argb_visual(&screen)
            .unwrap_or((screen.root_visual, screen.root_depth));

        debug!("Using visual {:?} with depth {}", visual_id, depth);

        // Generate window ID
        let window_id = conn.generate_id().map_err(|e| {
            WindowError::CreationFailed(format!("Failed to generate window ID: {e}"))
        })?;

        // Create colormap for 32-bit visual if needed
        let colormap = if depth == 32 {
            let colormap_id = conn.generate_id().map_err(|e| {
                WindowError::CreationFailed(format!("Failed to generate colormap ID: {e}"))
            })?;
            conn.create_colormap(ColormapAlloc::NONE, colormap_id, screen.root, visual_id)
                .map_err(|e| {
                    WindowError::CreationFailed(format!("Failed to create colormap: {e}"))
                })?;
            Some(colormap_id)
        } else {
            None
        };

        // Set up window attributes
        let mut win_aux = CreateWindowAux::new()
            .event_mask(
                EventMask::EXPOSURE
                    | EventMask::BUTTON_PRESS
                    | EventMask::BUTTON_RELEASE
                    | EventMask::POINTER_MOTION
                    | EventMask::STRUCTURE_NOTIFY,
            )
            .background_pixel(BG_COLOR_ARGB & 0x00FFFFFF)
            .border_pixel(0)
            .override_redirect(1); // Bypass window manager for true topmost

        if let Some(cmap) = colormap {
            win_aux = win_aux.colormap(cmap);
        }

        // Create the window
        conn.create_window(
            depth,
            window_id,
            screen.root,
            0,
            0,
            screen_width,
            screen_height,
            0,
            WindowClass::INPUT_OUTPUT,
            visual_id,
            &win_aux,
        )
        .map_err(|e| WindowError::CreationFailed(format!("Failed to create window: {e}")))?;

        self.window_id = window_id;
        self.bounds = Rect::new(0.0, 0.0, screen_width as f64, screen_height as f64);

        // Set EWMH properties for topmost/fullscreen
        self.set_window_properties()?;

        // Create graphics context
        self.create_graphics_context()?;

        // Store global state
        OVERLAY_WINDOW_ID.store(window_id, Ordering::SeqCst);
        OVERLAY_OPACITY.store((self.opacity * 255.0) as u8, Ordering::SeqCst);

        info!("Created Linux overlay window: {}", window_id);
        Ok(())
    }

    fn show(&mut self) {
        if self.window_id == 0 {
            warn!("Cannot show overlay: window not created");
            return;
        }

        let conn = match &self.connection {
            Some(c) => c,
            None => {
                warn!("Cannot show overlay: no connection");
                return;
            }
        };

        // Map (show) the window
        if let Err(e) = conn.map_window(self.window_id) {
            warn!("Failed to map window: {}", e);
            return;
        }

        // Raise the window to ensure it's on top
        let configure_aux =
            ConfigureWindowAux::new().stack_mode(x11rb::protocol::xproto::StackMode::ABOVE);
        if let Err(e) = conn.configure_window(self.window_id, &configure_aux) {
            warn!("Failed to raise window: {}", e);
        }

        if let Err(e) = conn.flush() {
            warn!("Failed to flush: {}", e);
            return;
        }

        self.visible = true;
        OVERLAY_VISIBLE.store(true, Ordering::SeqCst);
        info!("Showing overlay window");
    }

    fn hide(&mut self) {
        if self.window_id == 0 {
            return;
        }

        let conn = match &self.connection {
            Some(c) => c,
            None => return,
        };

        if let Err(e) = conn.unmap_window(self.window_id) {
            warn!("Failed to unmap window: {}", e);
        }

        let _ = conn.flush();

        self.visible = false;
        OVERLAY_VISIBLE.store(false, Ordering::SeqCst);
        info!("Hiding overlay window");
    }

    fn set_opacity(&mut self, opacity: f64) {
        let clamped = opacity.clamp(0.0, 1.0);
        self.opacity = clamped;

        let opacity_byte = (clamped * 255.0) as u8;
        OVERLAY_OPACITY.store(opacity_byte, Ordering::SeqCst);

        // Update the _NET_WM_WINDOW_OPACITY property if window exists
        if self.window_id != 0 {
            if let (Some(conn), Some(atoms)) = (&self.connection, &self.atoms) {
                let opacity_value = ((clamped * u32::MAX as f64) as u32).to_ne_bytes();
                let _ = conn.change_property8(
                    PropMode::REPLACE,
                    self.window_id,
                    atoms.net_wm_opacity,
                    AtomEnum::CARDINAL,
                    &opacity_value,
                );
                let _ = conn.flush();
            }
        }

        debug!("Set overlay opacity to {:.2}", clamped);
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn close(&mut self) {
        if self.window_id != 0 {
            // Clear global state first
            OVERLAY_WINDOW_ID.store(0, Ordering::SeqCst);
            OVERLAY_VISIBLE.store(false, Ordering::SeqCst);

            if let Some(conn) = &self.connection {
                // Free graphics context
                if let Some(gc) = self.gc {
                    let _ = conn.free_gc(gc);
                }

                // Destroy window
                let _ = conn.destroy_window(self.window_id);
                let _ = conn.flush();
            }

            self.window_id = 0;
            self.gc = None;
            info!("Closed overlay window");
        }

        self.visible = false;
        self.bounds = Rect::zero();

        // Clear timer text
        if let Ok(mut timer_text) = TIMER_TEXT.write() {
            *timer_text = None;
        }
    }

    fn is_visible(&self) -> bool {
        self.visible
    }
}

impl Drop for LinuxOverlayWindow {
    fn drop(&mut self) {
        Self::clear_close_callback();
        self.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overlay_window_new() {
        let overlay = LinuxOverlayWindow::new();
        assert!(!overlay.visible);
        assert!(overlay.connection.is_none());
        assert!((overlay.opacity - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_overlay_window_default() {
        let overlay = LinuxOverlayWindow::default();
        assert!(!overlay.visible);
        assert!(overlay.connection.is_none());
    }

    #[test]
    fn test_opacity_clamping() {
        let mut overlay = LinuxOverlayWindow::new();

        overlay.set_opacity(0.5);
        assert!((overlay.opacity - 0.5).abs() < 0.001);

        overlay.set_opacity(1.5);
        assert!((overlay.opacity - 1.0).abs() < 0.001);

        overlay.set_opacity(-0.5);
        assert!(overlay.opacity.abs() < 0.001);
    }

    #[test]
    fn test_bounds_initially_zero() {
        let overlay = LinuxOverlayWindow::new();
        let bounds = overlay.bounds();
        assert!(bounds.is_empty());
    }

    #[test]
    fn test_is_wayland_session_detection() {
        // This test just ensures the function doesn't panic
        let _ = is_wayland_session();
        let _ = is_x11_session();
    }

    #[test]
    fn test_is_point_in_close_button() {
        let mut overlay = LinuxOverlayWindow::new();
        overlay.bounds = Rect::new(0.0, 0.0, 1920.0, 1080.0);

        // Point at center of close button should be inside
        let width = 1920i16;
        let height = 1080i16;
        let center_x = width - CLOSE_BUTTON_MARGIN as i16 - CLOSE_BUTTON_RADIUS as i16;
        let center_y = height - CLOSE_BUTTON_MARGIN as i16 - CLOSE_BUTTON_RADIUS as i16;

        assert!(overlay.is_point_in_close_button(center_x, center_y));

        // Point far away should be outside
        assert!(!overlay.is_point_in_close_button(0, 0));
        assert!(!overlay.is_point_in_close_button(width / 2, height / 2));
    }

    #[test]
    fn test_global_atomics_default_state() {
        CLOSE_BUTTON_HOVERED.store(false, Ordering::SeqCst);
        assert!(!CLOSE_BUTTON_HOVERED.load(Ordering::SeqCst));
    }

    #[test]
    fn test_timer_text_lock() {
        LinuxOverlayWindow::set_timer_text(Some("Test"));
        {
            let timer_text = TIMER_TEXT.read().unwrap();
            assert!(timer_text.is_some());
            assert_eq!(timer_text.as_ref().unwrap(), "Test");
        }

        LinuxOverlayWindow::set_timer_text(None);
        {
            let timer_text = TIMER_TEXT.read().unwrap();
            assert!(timer_text.is_none());
        }
    }
}
