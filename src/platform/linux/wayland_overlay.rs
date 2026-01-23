//! Wayland overlay window implementation using wlr-layer-shell protocol
//!
//! This module implements a fullscreen overlay for Wayland compositors that support
//! the `wlr-layer-shell` protocol (wlroots-based compositors like Sway, Wayfire, etc.).
//!
//! # Protocol Support
//!
//! - **wlr-layer-shell**: Used for creating overlay layer surfaces
//! - Layer: OVERLAY (topmost layer)
//! - Anchored to all edges for fullscreen coverage
//!
//! # Compositor Compatibility
//!
//! - **Sway**: Full support via wlr-layer-shell
//! - **Wayfire**: Full support via wlr-layer-shell
//! - **Hyprland**: Full support via wlr-layer-shell
//! - **GNOME**: Not supported (uses different protocol)
//! - **KDE Plasma**: Partial support (may work with XWayland fallback)
//!
//! # Implementation Notes
//!
//! The overlay uses software rendering with shared memory buffers (wl_shm).
//! This is simpler and more portable than GPU-based rendering for our use case.
//!
//! # Current Limitations
//!
//! - **Timer text rendering**: Currently renders placeholder dots instead of actual
//!   characters. Proper text rendering would require a font library (e.g., freetype).
//!   The X11 implementation uses `image_text8` for basic text rendering.
//! - **Close button interaction**: The close button is drawn but pointer input is not
//!   yet wired up (no `wl_seat`/`wl_pointer` binding). The overlay can only be closed
//!   via compositor-initiated close events.

use std::os::unix::io::AsFd;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Arc, Mutex, RwLock};

use log::{debug, error, info, warn};
use wayland_client::protocol::{
    wl_buffer, wl_callback, wl_compositor, wl_output, wl_registry, wl_shm, wl_shm_pool, wl_surface,
};
use wayland_client::{Connection, Dispatch, QueueHandle, WEnum};
use wayland_protocols_wlr::layer_shell::v1::client::{zwlr_layer_shell_v1, zwlr_layer_surface_v1};

use crate::platform::errors::WindowError;
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

/// Global visibility state
static WAYLAND_OVERLAY_VISIBLE: AtomicBool = AtomicBool::new(false);

/// Global opacity (0-255)
static WAYLAND_OVERLAY_OPACITY: AtomicU8 = AtomicU8::new(200);

/// Mouse hover state for close button
static WAYLAND_CLOSE_BUTTON_HOVERED: AtomicBool = AtomicBool::new(false);

/// Callback for close button click (protected by mutex for thread safety)
static WAYLAND_CLOSE_CALLBACK: Mutex<Option<Arc<dyn Fn() + Send + Sync>>> = Mutex::new(None);

/// Timer text to display
static WAYLAND_TIMER_TEXT: RwLock<Option<String>> = RwLock::new(None);

/// Callback type for close button click
pub type WaylandCloseCallback = Arc<dyn Fn() + Send + Sync>;

/// State for Wayland event dispatching
struct WaylandState {
    /// Whether the layer shell protocol is available
    layer_shell_available: bool,
    /// The layer shell global
    layer_shell: Option<zwlr_layer_shell_v1::ZwlrLayerShellV1>,
    /// The compositor global
    compositor: Option<wl_compositor::WlCompositor>,
    /// The shared memory global
    shm: Option<wl_shm::WlShm>,
    /// Available shm formats
    shm_formats: Vec<wl_shm::Format>,
    /// The output (screen) to use
    output: Option<wl_output::WlOutput>,
    /// Output dimensions
    output_width: i32,
    output_height: i32,
    /// The surface
    surface: Option<wl_surface::WlSurface>,
    /// The layer surface
    layer_surface: Option<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1>,
    /// Whether we've received a configure event
    configured: bool,
    /// Configured width
    configured_width: u32,
    /// Configured height
    configured_height: u32,
    /// Whether the surface should close
    should_close: bool,
}

impl WaylandState {
    fn new() -> Self {
        Self {
            layer_shell_available: false,
            layer_shell: None,
            compositor: None,
            shm: None,
            shm_formats: Vec::new(),
            output: None,
            output_width: 0,
            output_height: 0,
            surface: None,
            layer_surface: None,
            configured: false,
            configured_width: 0,
            configured_height: 0,
            should_close: false,
        }
    }
}

// Dispatch implementation for registry events
impl Dispatch<wl_registry::WlRegistry, ()> for WaylandState {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<WaylandState>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            match interface.as_str() {
                "wl_compositor" => {
                    let compositor = registry.bind::<wl_compositor::WlCompositor, _, _>(
                        name,
                        version.min(4),
                        qh,
                        (),
                    );
                    state.compositor = Some(compositor);
                    debug!("Bound wl_compositor v{}", version.min(4));
                }
                "wl_shm" => {
                    let shm = registry.bind::<wl_shm::WlShm, _, _>(name, version.min(1), qh, ());
                    state.shm = Some(shm);
                    debug!("Bound wl_shm v{}", version.min(1));
                }
                "wl_output" => {
                    // Only bind the first output
                    if state.output.is_none() {
                        let output = registry.bind::<wl_output::WlOutput, _, _>(
                            name,
                            version.min(4),
                            qh,
                            (),
                        );
                        state.output = Some(output);
                        debug!("Bound wl_output v{}", version.min(4));
                    }
                }
                "zwlr_layer_shell_v1" => {
                    let layer_shell = registry.bind::<zwlr_layer_shell_v1::ZwlrLayerShellV1, _, _>(
                        name,
                        version.min(4),
                        qh,
                        (),
                    );
                    state.layer_shell = Some(layer_shell);
                    state.layer_shell_available = true;
                    debug!("Bound zwlr_layer_shell_v1 v{}", version.min(4));
                }
                _ => {}
            }
        }
    }
}

// Dispatch for compositor
impl Dispatch<wl_compositor::WlCompositor, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _: &wl_compositor::WlCompositor,
        _event: wl_compositor::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<WaylandState>,
    ) {
        // wl_compositor has no events
    }
}

// Dispatch for shm
impl Dispatch<wl_shm::WlShm, ()> for WaylandState {
    fn event(
        state: &mut Self,
        _: &wl_shm::WlShm,
        event: wl_shm::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<WaylandState>,
    ) {
        if let wl_shm::Event::Format {
            format: WEnum::Value(fmt),
        } = event
        {
            state.shm_formats.push(fmt);
        }
    }
}

// Dispatch for shm_pool
impl Dispatch<wl_shm_pool::WlShmPool, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _: &wl_shm_pool::WlShmPool,
        _event: wl_shm_pool::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<WaylandState>,
    ) {
        // wl_shm_pool has no events
    }
}

// Dispatch for output
impl Dispatch<wl_output::WlOutput, ()> for WaylandState {
    fn event(
        state: &mut Self,
        _: &wl_output::WlOutput,
        event: wl_output::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<WaylandState>,
    ) {
        if let wl_output::Event::Mode { width, height, .. } = event {
            state.output_width = width;
            state.output_height = height;
            debug!("Output mode: {}x{}", width, height);
        }
    }
}

// Dispatch for surface
impl Dispatch<wl_surface::WlSurface, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _: &wl_surface::WlSurface,
        _event: wl_surface::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<WaylandState>,
    ) {
        // Surface events handled elsewhere
    }
}

// Dispatch for buffer
impl Dispatch<wl_buffer::WlBuffer, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _: &wl_buffer::WlBuffer,
        event: wl_buffer::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<WaylandState>,
    ) {
        if let wl_buffer::Event::Release = event {
            // Buffer released by compositor, can reuse
        }
    }
}

// Dispatch for frame callback
impl Dispatch<wl_callback::WlCallback, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _: &wl_callback::WlCallback,
        _event: wl_callback::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<WaylandState>,
    ) {
        // Frame callback
    }
}

// Dispatch for layer shell
impl Dispatch<zwlr_layer_shell_v1::ZwlrLayerShellV1, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _: &zwlr_layer_shell_v1::ZwlrLayerShellV1,
        _event: zwlr_layer_shell_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<WaylandState>,
    ) {
        // Layer shell has no events
    }
}

// Dispatch for layer surface - this is where we get configure events
impl Dispatch<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1, ()> for WaylandState {
    fn event(
        state: &mut Self,
        layer_surface: &zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
        event: zwlr_layer_surface_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<WaylandState>,
    ) {
        match event {
            zwlr_layer_surface_v1::Event::Configure {
                serial,
                width,
                height,
            } => {
                debug!(
                    "Layer surface configure: {}x{} (serial {})",
                    width, height, serial
                );
                layer_surface.ack_configure(serial);

                // Use configured dimensions or fallback to output dimensions
                state.configured_width = if width > 0 {
                    width
                } else {
                    state.output_width as u32
                };
                state.configured_height = if height > 0 {
                    height
                } else {
                    state.output_height as u32
                };
                state.configured = true;
            }
            zwlr_layer_surface_v1::Event::Closed => {
                info!("Layer surface closed by compositor");
                state.should_close = true;
            }
            _ => {}
        }
    }
}

/// Wayland overlay window implementation using wlr-layer-shell.
///
/// This struct manages the fullscreen overlay window on Wayland using the
/// wlr-layer-shell protocol.
///
/// # Example
///
/// ```ignore
/// let mut overlay = WaylandOverlayWindow::new();
/// overlay.create()?;
/// overlay.set_opacity(0.8);
/// overlay.show();
/// // Event loop processing...
/// overlay.close();
/// ```
pub struct WaylandOverlayWindow {
    /// Wayland connection
    connection: Option<Connection>,
    /// Event queue
    event_queue: Option<wayland_client::EventQueue<WaylandState>>,
    /// State for event dispatching
    state: Option<WaylandState>,
    /// Whether the window is currently visible
    visible: bool,
    /// Current opacity (0.0 to 1.0)
    opacity: f64,
    /// Cached window bounds
    bounds: Rect,
    /// Shared memory file descriptor
    shm_fd: Option<std::fs::File>,
    /// Memory mapped buffer
    buffer_data: Option<memmap2::MmapMut>,
}

impl WaylandOverlayWindow {
    /// Creates a new Wayland overlay window instance.
    ///
    /// The window is not created until [`create()`](Self::create) is called.
    pub fn new() -> Self {
        Self {
            connection: None,
            event_queue: None,
            state: None,
            visible: false,
            opacity: 0.8,
            bounds: Rect::zero(),
            shm_fd: None,
            buffer_data: None,
        }
    }

    /// Sets the callback function for close button clicks.
    pub fn set_close_callback(callback: WaylandCloseCallback) {
        if let Ok(mut guard) = WAYLAND_CLOSE_CALLBACK.lock() {
            *guard = Some(callback);
        }
    }

    /// Clears the close callback.
    pub fn clear_close_callback() {
        if let Ok(mut guard) = WAYLAND_CLOSE_CALLBACK.lock() {
            *guard = None;
        }
    }

    /// Sets the timer display text.
    pub fn set_timer_text(text: Option<&str>) {
        if let Ok(mut timer_text) = WAYLAND_TIMER_TEXT.write() {
            *timer_text = text.map(|s| s.to_string());
        }
    }

    /// Creates the Wayland overlay window.
    pub fn create(&mut self) -> Result<(), WindowError> {
        // Connect to Wayland
        let connection = Connection::connect_to_env().map_err(|e| {
            WindowError::CreationFailed(format!("Failed to connect to Wayland: {}", e))
        })?;

        // Create event queue
        let mut event_queue = connection.new_event_queue();
        let qh = event_queue.handle();

        // Get the display and registry
        let display = connection.display();
        let _registry = display.get_registry(&qh, ());

        // Create state and do initial roundtrip to get globals
        let mut state = WaylandState::new();
        event_queue
            .roundtrip(&mut state)
            .map_err(|e| WindowError::CreationFailed(format!("Failed to roundtrip: {}", e)))?;

        // Check if layer shell is available
        if !state.layer_shell_available {
            return Err(WindowError::CreationFailed(
                "wlr-layer-shell protocol not available. \
                This Wayland compositor does not support overlay windows. \
                Try using a wlroots-based compositor (Sway, Wayfire, Hyprland) \
                or fall back to XWayland."
                    .into(),
            ));
        }

        // Do another roundtrip to get output info
        event_queue.roundtrip(&mut state).map_err(|e| {
            WindowError::CreationFailed(format!("Failed to get output info: {}", e))
        })?;

        // Verify we have the required globals
        let compositor = state
            .compositor
            .as_ref()
            .ok_or_else(|| WindowError::CreationFailed("wl_compositor not available".into()))?;

        let layer_shell = state.layer_shell.as_ref().ok_or_else(|| {
            WindowError::CreationFailed("zwlr_layer_shell_v1 not available".into())
        })?;

        let _shm = state
            .shm
            .as_ref()
            .ok_or_else(|| WindowError::CreationFailed("wl_shm not available".into()))?;

        // Create the surface
        let surface = compositor.create_surface(&qh, ());
        state.surface = Some(surface.clone());

        // Create the layer surface on the overlay layer
        let layer_surface = layer_shell.get_layer_surface(
            &surface,
            state.output.as_ref(),
            zwlr_layer_shell_v1::Layer::Overlay,
            "catshield-overlay".to_string(),
            &qh,
            (),
        );

        // Configure the layer surface for fullscreen overlay
        // Anchor to all edges to fill the screen
        layer_surface.set_anchor(
            zwlr_layer_surface_v1::Anchor::Top
                | zwlr_layer_surface_v1::Anchor::Bottom
                | zwlr_layer_surface_v1::Anchor::Left
                | zwlr_layer_surface_v1::Anchor::Right,
        );

        // Set exclusive zone to -1 to not reserve any space
        layer_surface.set_exclusive_zone(-1);

        // Request keyboard interactivity for potential input handling
        layer_surface
            .set_keyboard_interactivity(zwlr_layer_surface_v1::KeyboardInteractivity::Exclusive);

        state.layer_surface = Some(layer_surface);

        // Initial commit to get configure event
        surface.commit();

        // Wait for configure event
        event_queue
            .roundtrip(&mut state)
            .map_err(|e| WindowError::CreationFailed(format!("Failed to configure: {}", e)))?;

        if !state.configured {
            return Err(WindowError::CreationFailed(
                "Did not receive layer surface configure event".into(),
            ));
        }

        // Update bounds
        let width = state.configured_width as f64;
        let height = state.configured_height as f64;
        self.bounds = Rect::new(0.0, 0.0, width, height);

        debug!("Created Wayland overlay: {}x{}", width, height);

        // Store state
        self.connection = Some(connection);
        self.event_queue = Some(event_queue);
        self.state = Some(state);

        // Update global state
        WAYLAND_OVERLAY_OPACITY.store((self.opacity * 255.0) as u8, Ordering::SeqCst);

        info!("Created Wayland overlay window using wlr-layer-shell");
        Ok(())
    }

    /// Creates a shared memory buffer and draws the overlay content.
    fn draw(&mut self) -> Result<(), WindowError> {
        let state = self
            .state
            .as_ref()
            .ok_or_else(|| WindowError::Platform("State not initialized".into()))?;

        let surface = state
            .surface
            .as_ref()
            .ok_or_else(|| WindowError::Platform("Surface not created".into()))?;

        let shm = state
            .shm
            .as_ref()
            .ok_or_else(|| WindowError::Platform("SHM not available".into()))?;

        let connection = self
            .connection
            .as_ref()
            .ok_or_else(|| WindowError::Platform("Connection not available".into()))?;

        let event_queue = self
            .event_queue
            .as_ref()
            .ok_or_else(|| WindowError::Platform("Event queue not available".into()))?;

        let width = state.configured_width;
        let height = state.configured_height;
        let stride = width * 4; // 4 bytes per pixel (ARGB)
        let size = (stride * height) as usize;

        // Create shared memory file
        let file = create_shm_file(size)?;
        let fd = file.as_fd();

        // Memory map the file
        let mut mmap = unsafe {
            memmap2::MmapMut::map_mut(&file)
                .map_err(|e| WindowError::Platform(format!("Failed to mmap: {}", e)))?
        };

        // Draw the overlay content into the buffer
        self.draw_content(&mut mmap, width, height)?;

        // Create the shm pool and buffer
        let qh = event_queue.handle();
        let pool = shm.create_pool(fd, size as i32, &qh, ());
        let buffer = pool.create_buffer(
            0,
            width as i32,
            height as i32,
            stride as i32,
            wl_shm::Format::Argb8888,
            &qh,
            (),
        );

        // Attach buffer and commit
        surface.attach(Some(&buffer), 0, 0);
        surface.damage_buffer(0, 0, width as i32, height as i32);
        surface.commit();

        // Store for later cleanup
        self.shm_fd = Some(file);
        self.buffer_data = Some(mmap);

        // Flush to ensure compositor receives it
        connection
            .flush()
            .map_err(|e| WindowError::Platform(format!("Failed to flush: {}", e)))?;

        Ok(())
    }

    /// Draws the overlay content into a buffer.
    fn draw_content(&self, buffer: &mut [u8], width: u32, height: u32) -> Result<(), WindowError> {
        let is_hovered = WAYLAND_CLOSE_BUTTON_HOVERED.load(Ordering::SeqCst);
        let opacity_byte = WAYLAND_OVERLAY_OPACITY.load(Ordering::SeqCst);

        // Calculate background color with current opacity
        let bg_alpha = ((opacity_byte as u32 * 0xCC) / 255) as u8;
        let bg_color = ((bg_alpha as u32) << 24) | (BG_COLOR_ARGB & 0x00FFFFFF);

        // Fill background
        for y in 0..height {
            for x in 0..width {
                let offset = ((y * width + x) * 4) as usize;
                if offset + 3 < buffer.len() {
                    let color = bg_color;
                    buffer[offset] = (color & 0xFF) as u8; // Blue
                    buffer[offset + 1] = ((color >> 8) & 0xFF) as u8; // Green
                    buffer[offset + 2] = ((color >> 16) & 0xFF) as u8; // Red
                    buffer[offset + 3] = ((color >> 24) & 0xFF) as u8; // Alpha
                }
            }
        }

        // Draw close button
        let button_cx = width as i32 - CLOSE_BUTTON_MARGIN - CLOSE_BUTTON_RADIUS;
        let button_cy = height as i32 - CLOSE_BUTTON_MARGIN - CLOSE_BUTTON_RADIUS;
        let button_color = if is_hovered {
            CLOSE_BUTTON_HOVER_COLOR
        } else {
            CLOSE_BUTTON_COLOR
        };

        // Draw filled circle for close button
        for dy in -CLOSE_BUTTON_RADIUS..=CLOSE_BUTTON_RADIUS {
            for dx in -CLOSE_BUTTON_RADIUS..=CLOSE_BUTTON_RADIUS {
                if dx * dx + dy * dy <= CLOSE_BUTTON_RADIUS * CLOSE_BUTTON_RADIUS {
                    let px = button_cx + dx;
                    let py = button_cy + dy;
                    if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
                        let offset = ((py as u32 * width + px as u32) * 4) as usize;
                        if offset + 3 < buffer.len() {
                            buffer[offset] = (button_color & 0xFF) as u8;
                            buffer[offset + 1] = ((button_color >> 8) & 0xFF) as u8;
                            buffer[offset + 2] = ((button_color >> 16) & 0xFF) as u8;
                            buffer[offset + 3] = ((button_color >> 24) & 0xFF) as u8;
                        }
                    }
                }
            }
        }

        // Draw X on the close button
        let x_half = (CLOSE_BUTTON_RADIUS as f32 * 0.5) as i32;
        let line_width = 3;

        // Draw the two lines of the X
        for i in -x_half..=x_half {
            for w in -(line_width / 2)..=(line_width / 2) {
                // First line: top-left to bottom-right
                let px1 = button_cx + i;
                let py1 = button_cy + i + w;
                if px1 >= 0 && px1 < width as i32 && py1 >= 0 && py1 < height as i32 {
                    let offset = ((py1 as u32 * width + px1 as u32) * 4) as usize;
                    if offset + 3 < buffer.len() {
                        buffer[offset] = (TEXT_COLOR & 0xFF) as u8;
                        buffer[offset + 1] = ((TEXT_COLOR >> 8) & 0xFF) as u8;
                        buffer[offset + 2] = ((TEXT_COLOR >> 16) & 0xFF) as u8;
                        buffer[offset + 3] = ((TEXT_COLOR >> 24) & 0xFF) as u8;
                    }
                }

                // Second line: top-right to bottom-left
                let px2 = button_cx + i;
                let py2 = button_cy - i + w;
                if px2 >= 0 && px2 < width as i32 && py2 >= 0 && py2 < height as i32 {
                    let offset = ((py2 as u32 * width + px2 as u32) * 4) as usize;
                    if offset + 3 < buffer.len() {
                        buffer[offset] = (TEXT_COLOR & 0xFF) as u8;
                        buffer[offset + 1] = ((TEXT_COLOR >> 8) & 0xFF) as u8;
                        buffer[offset + 2] = ((TEXT_COLOR >> 16) & 0xFF) as u8;
                        buffer[offset + 3] = ((TEXT_COLOR >> 24) & 0xFF) as u8;
                    }
                }
            }
        }

        // Draw timer text if set (placeholder rendering)
        // TODO: Implement proper text rendering with a font library (e.g., freetype).
        // Currently renders dots as placeholders for each character position.
        // The X11 implementation uses image_text8 for basic text rendering.
        if let Ok(timer_text) = WAYLAND_TIMER_TEXT.read() {
            if let Some(ref text) = *timer_text {
                let text_y = height as i32 / 2;
                let text_x = width as i32 / 2 - (text.len() as i32 * 4);

                // Draw placeholder dots for each character position
                for (i, _c) in text.chars().enumerate() {
                    let px = text_x + (i as i32 * 10);
                    let py = text_y;
                    if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
                        // Draw a small dot for each character position
                        for dy in -2..=2 {
                            for dx in -2..=2 {
                                let dot_x = px + dx;
                                let dot_y = py + dy;
                                if dot_x >= 0
                                    && dot_x < width as i32
                                    && dot_y >= 0
                                    && dot_y < height as i32
                                {
                                    let offset =
                                        ((dot_y as u32 * width + dot_x as u32) * 4) as usize;
                                    if offset + 3 < buffer.len() {
                                        buffer[offset] = (TEXT_COLOR & 0xFF) as u8;
                                        buffer[offset + 1] = ((TEXT_COLOR >> 8) & 0xFF) as u8;
                                        buffer[offset + 2] = ((TEXT_COLOR >> 16) & 0xFF) as u8;
                                        buffer[offset + 3] = ((TEXT_COLOR >> 24) & 0xFF) as u8;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Shows the overlay window.
    pub fn show(&mut self) {
        if self.state.is_none() {
            warn!("Cannot show overlay: not created");
            return;
        }

        // Draw content and make visible
        if let Err(e) = self.draw() {
            error!("Failed to draw overlay: {}", e);
            return;
        }

        self.visible = true;
        WAYLAND_OVERLAY_VISIBLE.store(true, Ordering::SeqCst);
        info!("Showing Wayland overlay window");
    }

    /// Hides the overlay window.
    pub fn hide(&mut self) {
        if let Some(state) = &self.state {
            if let Some(surface) = &state.surface {
                // Attach null buffer to hide
                surface.attach(None, 0, 0);
                surface.commit();
            }
        }

        if let Some(conn) = &self.connection {
            let _ = conn.flush();
        }

        self.visible = false;
        WAYLAND_OVERLAY_VISIBLE.store(false, Ordering::SeqCst);
        info!("Hiding Wayland overlay window");
    }

    /// Sets the overlay opacity.
    pub fn set_opacity(&mut self, opacity: f64) {
        let clamped = opacity.clamp(0.0, 1.0);
        self.opacity = clamped;

        let opacity_byte = (clamped * 255.0) as u8;
        WAYLAND_OVERLAY_OPACITY.store(opacity_byte, Ordering::SeqCst);

        // Redraw if visible
        if self.visible {
            let _ = self.draw();
        }

        debug!("Set Wayland overlay opacity to {:.2}", clamped);
    }

    /// Returns the overlay bounds.
    pub fn bounds(&self) -> Rect {
        self.bounds
    }

    /// Closes the overlay window.
    pub fn close(&mut self) {
        // Destroy layer surface first
        if let Some(state) = &self.state {
            if let Some(layer_surface) = &state.layer_surface {
                layer_surface.destroy();
            }
            if let Some(surface) = &state.surface {
                surface.destroy();
            }
        }

        // Flush the connection
        if let Some(conn) = &self.connection {
            let _ = conn.flush();
        }

        // Clear state
        self.state = None;
        self.event_queue = None;
        self.connection = None;
        self.shm_fd = None;
        self.buffer_data = None;
        self.visible = false;
        self.bounds = Rect::zero();

        // Clear global state
        WAYLAND_OVERLAY_VISIBLE.store(false, Ordering::SeqCst);

        // Clear timer text
        if let Ok(mut timer_text) = WAYLAND_TIMER_TEXT.write() {
            *timer_text = None;
        }

        info!("Closed Wayland overlay window");
    }

    /// Returns whether the overlay is visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Processes pending Wayland events.
    pub fn process_events(&mut self) -> Result<bool, WindowError> {
        let event_queue = self
            .event_queue
            .as_mut()
            .ok_or_else(|| WindowError::Platform("Event queue not available".into()))?;

        let state = self
            .state
            .as_mut()
            .ok_or_else(|| WindowError::Platform("State not available".into()))?;

        // Non-blocking dispatch
        event_queue
            .dispatch_pending(state)
            .map_err(|e| WindowError::Platform(format!("Failed to dispatch events: {}", e)))?;

        // Check if compositor requested close
        if state.should_close {
            let callback = WAYLAND_CLOSE_CALLBACK
                .lock()
                .ok()
                .and_then(|guard| guard.clone());
            if let Some(cb) = callback {
                cb();
                return Ok(true);
            }
        }

        Ok(false)
    }
}

impl Default for WaylandOverlayWindow {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for WaylandOverlayWindow {
    fn drop(&mut self) {
        Self::clear_close_callback();
        self.close();
    }
}

/// Creates a shared memory file for Wayland buffer.
fn create_shm_file(size: usize) -> Result<std::fs::File, WindowError> {
    // Try memfd_create first (Linux 3.17+)
    #[cfg(target_os = "linux")]
    {
        use std::ffi::CString;
        use std::os::unix::io::FromRawFd;

        let name = CString::new("catshield-wayland-shm").unwrap();
        let fd = unsafe { libc::memfd_create(name.as_ptr(), libc::MFD_CLOEXEC) };

        if fd >= 0 {
            let file = unsafe { std::fs::File::from_raw_fd(fd) };
            file.set_len(size as u64)
                .map_err(|e| WindowError::Platform(format!("Failed to set shm size: {}", e)))?;
            return Ok(file);
        }
    }

    // Fallback to /dev/shm
    let path = format!("/dev/shm/catshield-wayland-{}", std::process::id());
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)
        .map_err(|e| WindowError::Platform(format!("Failed to create shm file: {}", e)))?;

    // Unlink immediately so file is deleted when closed
    let _ = std::fs::remove_file(&path);

    // Set size
    file.set_len(size as u64)
        .map_err(|e| WindowError::Platform(format!("Failed to set shm size: {}", e)))?;

    Ok(file)
}

/// Checks if the current session supports wlr-layer-shell.
pub fn is_wlr_layer_shell_available() -> bool {
    // Try to connect and check for the protocol
    let connection = match Connection::connect_to_env() {
        Ok(c) => c,
        Err(_) => return false,
    };

    let mut event_queue = connection.new_event_queue();
    let qh = event_queue.handle();

    let display = connection.display();
    let _registry = display.get_registry(&qh, ());

    let mut state = WaylandState::new();
    if event_queue.roundtrip(&mut state).is_err() {
        return false;
    }

    state.layer_shell_available
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wayland_overlay_new() {
        let overlay = WaylandOverlayWindow::new();
        assert!(!overlay.visible);
        assert!((overlay.opacity - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_wayland_overlay_default() {
        let overlay = WaylandOverlayWindow::default();
        assert!(!overlay.visible);
    }

    #[test]
    fn test_wayland_opacity_clamping() {
        let mut overlay = WaylandOverlayWindow::new();

        overlay.set_opacity(0.5);
        assert!((overlay.opacity - 0.5).abs() < 0.001);

        overlay.set_opacity(1.5);
        assert!((overlay.opacity - 1.0).abs() < 0.001);

        overlay.set_opacity(-0.5);
        assert!(overlay.opacity.abs() < 0.001);
    }

    #[test]
    fn test_wayland_bounds_initially_zero() {
        let overlay = WaylandOverlayWindow::new();
        let bounds = overlay.bounds();
        assert!(bounds.is_empty());
    }

    #[test]
    fn test_wayland_timer_text() {
        WaylandOverlayWindow::set_timer_text(Some("Test"));
        {
            let timer_text = WAYLAND_TIMER_TEXT.read().unwrap();
            assert!(timer_text.is_some());
            assert_eq!(timer_text.as_ref().unwrap(), "Test");
        }

        WaylandOverlayWindow::set_timer_text(None);
        {
            let timer_text = WAYLAND_TIMER_TEXT.read().unwrap();
            assert!(timer_text.is_none());
        }
    }

    #[test]
    fn test_wayland_global_atomics() {
        WAYLAND_CLOSE_BUTTON_HOVERED.store(false, Ordering::SeqCst);
        assert!(!WAYLAND_CLOSE_BUTTON_HOVERED.load(Ordering::SeqCst));
    }
}
