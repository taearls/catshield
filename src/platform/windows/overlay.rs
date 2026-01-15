//! Windows overlay window implementation for Cat Shield
//!
//! This module implements the [`OverlayWindow`] trait for Windows using:
//! - `CreateWindowEx` with `WS_EX_TOPMOST`, `WS_EX_TOOLWINDOW`, and `WS_EX_LAYERED`
//! - `SetLayeredWindowAttributes` for opacity control
//! - GDI+ for custom painting (close button and timer display)
//!
//! # Requirements
//!
//! The overlay window:
//! - Covers the entire primary display
//! - Stays on top of all other windows (`WS_EX_TOPMOST`)
//! - Is hidden from the taskbar (`WS_EX_TOOLWINDOW`)
//! - Supports opacity via layered window attributes
//! - Displays a close button and optional timer
//!
//! # Implementation Notes
//!
//! The window is created with `WS_POPUP` style for borderless fullscreen.
//! Custom painting is performed in the `WM_PAINT` handler using GDI functions.
//! The close button is tracked for mouse hover and click events.

use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicU8, Ordering};

use log::{debug, info, warn};
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreatePen, CreateSolidBrush, DeleteObject, Ellipse, EndPaint, FillRect,
    InvalidateRect, SelectObject, SetBkMode, SetTextColor, TextOutW, UpdateWindow, PAINTSTRUCT,
    PS_SOLID, TRANSPARENT,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, GetClientRect, GetSystemMetrics, KillTimer,
    RegisterClassW, SetLayeredWindowAttributes, SetTimer, ShowWindow, LWA_ALPHA, SM_CXSCREEN,
    SM_CYSCREEN, SW_HIDE, SW_SHOW, WM_DESTROY, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE,
    WM_PAINT, WM_TIMER, WNDCLASSW, WS_EX_LAYERED, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
};

use crate::platform::errors::WindowError;
use crate::platform::traits::OverlayWindow;
use crate::platform::types::Rect;

/// Window class name for the overlay window
static OVERLAY_CLASS_NAME: &[u16] = &[
    'C' as u16, 'a' as u16, 't' as u16, 'S' as u16, 'h' as u16, 'i' as u16, 'e' as u16, 'l' as u16,
    'd' as u16, 'O' as u16, 'v' as u16, 'e' as u16, 'r' as u16, 'l' as u16, 'a' as u16, 'y' as u16,
    0,
];

/// Timer ID for animation updates
const TIMER_ID: usize = 1;

/// Animation timer interval in milliseconds (60 FPS)
const TIMER_INTERVAL_MS: u32 = 16;

/// Close button radius in pixels
const CLOSE_BUTTON_RADIUS: i32 = 30;

/// Close button margin from bottom-right corner
const CLOSE_BUTTON_MARGIN: i32 = 50;

/// Background color (dark semi-transparent)
const BG_COLOR: COLORREF = COLORREF(0x00202020); // Dark gray RGB(32, 32, 32)

/// Close button color (red)
const CLOSE_BUTTON_COLOR: COLORREF = COLORREF(0x004040FF); // Red in BGR format (255, 64, 64)

/// Close button hover color (lighter red)
const CLOSE_BUTTON_HOVER_COLOR: COLORREF = COLORREF(0x006060FF); // Lighter red BGR

/// Text color (white)
const TEXT_COLOR: COLORREF = COLORREF(0x00FFFFFF); // White

/// Global state for the overlay window
static OVERLAY_HWND: AtomicPtr<std::ffi::c_void> = AtomicPtr::new(std::ptr::null_mut());

/// Global visibility state
static OVERLAY_VISIBLE: AtomicBool = AtomicBool::new(false);

/// Global opacity (0-255)
static OVERLAY_OPACITY: AtomicU8 = AtomicU8::new(200);

/// Mouse hover state for close button
static CLOSE_BUTTON_HOVERED: AtomicBool = AtomicBool::new(false);

/// Mouse down state for close button
static CLOSE_BUTTON_PRESSED: AtomicBool = AtomicBool::new(false);

/// Callback for close button click
static CLOSE_CALLBACK: AtomicPtr<()> = AtomicPtr::new(std::ptr::null_mut());

/// Timer text to display (null-terminated UTF-16, max 32 chars including null)
static TIMER_TEXT: std::sync::RwLock<Option<Vec<u16>>> = std::sync::RwLock::new(None);

/// Callback type for close button click
pub type CloseCallback = Box<dyn Fn() + Send + Sync>;

/// Windows overlay window implementation.
///
/// This struct manages the fullscreen overlay window on Windows.
/// It implements the [`OverlayWindow`] trait for platform abstraction.
///
/// # Example
///
/// ```ignore
/// let mut overlay = WindowsOverlayWindow::new();
/// overlay.create()?;
/// overlay.set_opacity(0.8);
/// overlay.show();
/// // ... later ...
/// overlay.close();
/// ```
pub struct WindowsOverlayWindow {
    /// Handle to the overlay window
    hwnd: HWND,
    /// Whether the window is currently visible
    visible: bool,
    /// Current opacity (0.0 to 1.0)
    opacity: f64,
    /// Cached window bounds
    bounds: Rect,
    /// Whether the window class has been registered
    class_registered: bool,
}

impl WindowsOverlayWindow {
    /// Creates a new Windows overlay window instance.
    ///
    /// The window is not created until [`create()`](Self::create) is called.
    pub fn new() -> Self {
        Self {
            hwnd: HWND::default(),
            visible: false,
            opacity: 0.8,
            bounds: Rect::zero(),
            class_registered: false,
        }
    }

    /// Sets the callback function for close button clicks.
    ///
    /// The callback will be invoked when the user clicks the close button.
    ///
    /// # Arguments
    ///
    /// * `callback` - A function that is called when the close button is clicked
    pub fn set_close_callback(callback: CloseCallback) {
        let boxed = Box::new(callback);
        let ptr = Box::into_raw(boxed) as *mut ();
        let old = CLOSE_CALLBACK.swap(ptr, Ordering::SeqCst);
        if !old.is_null() {
            // Clean up the old callback
            unsafe {
                drop(Box::from_raw(old as *mut CloseCallback));
            }
        }
    }

    /// Clears the close callback.
    pub fn clear_close_callback() {
        let old = CLOSE_CALLBACK.swap(std::ptr::null_mut(), Ordering::SeqCst);
        if !old.is_null() {
            unsafe {
                drop(Box::from_raw(old as *mut CloseCallback));
            }
        }
    }

    /// Sets the timer display text.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to display, or None to hide the timer
    pub fn set_timer_text(text: Option<&str>) {
        let mut timer_text = TIMER_TEXT.write().unwrap();
        *timer_text = text.map(|s| {
            let mut utf16: Vec<u16> = s.encode_utf16().collect();
            utf16.push(0); // Null terminate
            utf16
        });

        // Trigger a repaint if the window is visible
        let hwnd_ptr = OVERLAY_HWND.load(Ordering::SeqCst);
        if !hwnd_ptr.is_null() && OVERLAY_VISIBLE.load(Ordering::SeqCst) {
            unsafe {
                let hwnd = HWND(hwnd_ptr);
                let _ = InvalidateRect(hwnd, None, false);
            }
        }
    }

    /// Registers the window class for the overlay window.
    fn register_window_class(&mut self) -> Result<(), WindowError> {
        if self.class_registered {
            return Ok(());
        }

        let hinstance = unsafe { GetModuleHandleW(None) }.map_err(|e| {
            WindowError::CreationFailed(format!("Failed to get module handle: {e}"))
        })?;

        let wc = WNDCLASSW {
            lpfnWndProc: Some(overlay_window_proc),
            hInstance: hinstance.into(),
            lpszClassName: PCWSTR(OVERLAY_CLASS_NAME.as_ptr()),
            hbrBackground: unsafe { CreateSolidBrush(BG_COLOR) },
            ..Default::default()
        };

        let result = unsafe { RegisterClassW(&wc) };
        if result == 0 {
            // Class might already be registered, which is OK
            debug!("Overlay window class registration returned 0 (may already be registered)");
        }

        self.class_registered = true;
        Ok(())
    }

    /// Gets the screen dimensions.
    fn get_screen_size(&self) -> (i32, i32) {
        unsafe {
            let width = GetSystemMetrics(SM_CXSCREEN);
            let height = GetSystemMetrics(SM_CYSCREEN);
            (width, height)
        }
    }
}

impl Default for WindowsOverlayWindow {
    fn default() -> Self {
        Self::new()
    }
}

impl OverlayWindow for WindowsOverlayWindow {
    fn create(&mut self) -> Result<(), WindowError> {
        self.register_window_class()?;

        let hmodule = unsafe { GetModuleHandleW(None) }.map_err(|e| {
            WindowError::CreationFailed(format!("Failed to get module handle: {e}"))
        })?;
        let hinstance: HINSTANCE = hmodule.into();

        let (screen_width, screen_height) = self.get_screen_size();

        if screen_width <= 0 || screen_height <= 0 {
            return Err(WindowError::DisplayNotAvailable(
                "Failed to get screen dimensions".into(),
            ));
        }

        debug!(
            "Creating overlay window: {}x{}",
            screen_width, screen_height
        );

        // Create the window with layered, topmost, and toolwindow styles
        let ex_style = WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_TOOLWINDOW;

        let hwnd = unsafe {
            CreateWindowExW(
                ex_style,
                PCWSTR(OVERLAY_CLASS_NAME.as_ptr()),
                w!("Cat Shield Overlay"),
                WS_POPUP, // Borderless popup window
                0,
                0,
                screen_width,
                screen_height,
                None,
                None,
                hinstance,
                None,
            )
        }
        .map_err(|e| WindowError::CreationFailed(format!("CreateWindowExW failed: {e}")))?;

        if hwnd.is_invalid() {
            return Err(WindowError::CreationFailed(
                "CreateWindowExW returned invalid handle".into(),
            ));
        }

        // Set the initial opacity
        let opacity_byte = (self.opacity * 255.0).clamp(0.0, 255.0) as u8;
        unsafe {
            SetLayeredWindowAttributes(hwnd, COLORREF(0), opacity_byte, LWA_ALPHA).map_err(
                |e| WindowError::ConfigurationFailed(format!("Failed to set opacity: {e}")),
            )?;
        }

        self.hwnd = hwnd;
        self.bounds = Rect::new(0.0, 0.0, screen_width as f64, screen_height as f64);

        // Store the handle globally for the window procedure
        OVERLAY_HWND.store(hwnd.0, Ordering::SeqCst);
        OVERLAY_OPACITY.store(opacity_byte, Ordering::SeqCst);

        info!("Created Windows overlay window: {:?}", hwnd);
        Ok(())
    }

    fn show(&mut self) {
        if self.hwnd.is_invalid() {
            warn!("Cannot show overlay: window not created");
            return;
        }

        unsafe {
            let _ = ShowWindow(self.hwnd, SW_SHOW);
            let _ = UpdateWindow(self.hwnd);

            // Start the animation timer
            let _ = SetTimer(self.hwnd, TIMER_ID, TIMER_INTERVAL_MS, None);
        }

        self.visible = true;
        OVERLAY_VISIBLE.store(true, Ordering::SeqCst);
        info!("Showing overlay window");
    }

    fn hide(&mut self) {
        if self.hwnd.is_invalid() {
            return;
        }

        unsafe {
            // Stop the animation timer
            let _ = KillTimer(self.hwnd, TIMER_ID);
            let _ = ShowWindow(self.hwnd, SW_HIDE);
        }

        self.visible = false;
        OVERLAY_VISIBLE.store(false, Ordering::SeqCst);
        info!("Hiding overlay window");
    }

    fn set_opacity(&mut self, opacity: f64) {
        let clamped = opacity.clamp(0.0, 1.0);
        self.opacity = clamped;

        let opacity_byte = (clamped * 255.0) as u8;
        OVERLAY_OPACITY.store(opacity_byte, Ordering::SeqCst);

        if !self.hwnd.is_invalid() {
            unsafe {
                let _ = SetLayeredWindowAttributes(self.hwnd, COLORREF(0), opacity_byte, LWA_ALPHA);
            }
        }

        debug!("Set overlay opacity to {:.2}", clamped);
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn close(&mut self) {
        if !self.hwnd.is_invalid() {
            // Stop the timer first
            unsafe {
                let _ = KillTimer(self.hwnd, TIMER_ID);
            }

            // Clear the global handle before destroying
            OVERLAY_HWND.store(std::ptr::null_mut(), Ordering::SeqCst);
            OVERLAY_VISIBLE.store(false, Ordering::SeqCst);

            let _ = unsafe { DestroyWindow(self.hwnd) };
            self.hwnd = HWND::default();
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

impl Drop for WindowsOverlayWindow {
    fn drop(&mut self) {
        Self::clear_close_callback();
        self.close();
    }
}

/// Checks if a point is within the close button area.
fn is_point_in_close_button(x: i32, y: i32, window_width: i32, window_height: i32) -> bool {
    let button_center_x = window_width - CLOSE_BUTTON_MARGIN - CLOSE_BUTTON_RADIUS;
    let button_center_y = window_height - CLOSE_BUTTON_MARGIN - CLOSE_BUTTON_RADIUS;

    let dx = x - button_center_x;
    let dy = y - button_center_y;
    let distance_squared = dx * dx + dy * dy;

    distance_squared <= CLOSE_BUTTON_RADIUS * CLOSE_BUTTON_RADIUS
}

/// Window procedure for the overlay window.
///
/// Handles painting, mouse events, and timer updates.
unsafe extern "system" fn overlay_window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);

            if !hdc.is_invalid() {
                // Get window dimensions
                let mut rect = RECT::default();
                let _ = GetClientRect(hwnd, &mut rect);
                let width = rect.right - rect.left;
                let height = rect.bottom - rect.top;

                // Fill background
                let bg_brush = CreateSolidBrush(BG_COLOR);
                FillRect(hdc, &rect, bg_brush);
                let _ = DeleteObject(bg_brush);

                // Draw close button
                let button_center_x = width - CLOSE_BUTTON_MARGIN - CLOSE_BUTTON_RADIUS;
                let button_center_y = height - CLOSE_BUTTON_MARGIN - CLOSE_BUTTON_RADIUS;

                let is_hovered = CLOSE_BUTTON_HOVERED.load(Ordering::SeqCst);
                let button_color = if is_hovered {
                    CLOSE_BUTTON_HOVER_COLOR
                } else {
                    CLOSE_BUTTON_COLOR
                };

                let button_brush = CreateSolidBrush(button_color);
                let old_brush = SelectObject(hdc, button_brush);
                let null_pen = CreatePen(PS_SOLID, 0, button_color);
                let old_pen = SelectObject(hdc, null_pen);

                let _ = Ellipse(
                    hdc,
                    button_center_x - CLOSE_BUTTON_RADIUS,
                    button_center_y - CLOSE_BUTTON_RADIUS,
                    button_center_x + CLOSE_BUTTON_RADIUS,
                    button_center_y + CLOSE_BUTTON_RADIUS,
                );

                // Draw X on the close button
                let _ = SetBkMode(hdc, TRANSPARENT);
                let _ = SetTextColor(hdc, TEXT_COLOR);

                // Draw "X" centered on the button
                let x_text: Vec<u16> = "X".encode_utf16().collect();
                let _ = TextOutW(hdc, button_center_x - 5, button_center_y - 8, &x_text);

                // Restore old objects and clean up
                SelectObject(hdc, old_brush);
                SelectObject(hdc, old_pen);
                let _ = DeleteObject(button_brush);
                let _ = DeleteObject(null_pen);

                // Draw timer text if set
                if let Ok(timer_text) = TIMER_TEXT.read() {
                    if let Some(ref text) = *timer_text {
                        let _ = SetTextColor(hdc, TEXT_COLOR);
                        // Draw timer in the center of the screen
                        let text_x = width / 2 - 50;
                        let text_y = height / 2;
                        let _ = TextOutW(hdc, text_x, text_y, &text[..text.len() - 1]);
                        // Exclude null terminator
                    }
                }

                let _ = EndPaint(hwnd, &ps);
            }

            LRESULT(0)
        }
        WM_TIMER => {
            if wparam.0 == TIMER_ID {
                // Trigger a repaint for animation
                let _ = InvalidateRect(hwnd, None, false);
            }
            LRESULT(0)
        }
        WM_MOUSEMOVE => {
            let x = (lparam.0 & 0xFFFF) as i16 as i32;
            let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;

            let mut rect = RECT::default();
            let _ = GetClientRect(hwnd, &mut rect);
            let width = rect.right - rect.left;
            let height = rect.bottom - rect.top;

            let was_hovered = CLOSE_BUTTON_HOVERED.load(Ordering::SeqCst);
            let is_hovered = is_point_in_close_button(x, y, width, height);

            if was_hovered != is_hovered {
                CLOSE_BUTTON_HOVERED.store(is_hovered, Ordering::SeqCst);
                let _ = InvalidateRect(hwnd, None, false);
            }

            LRESULT(0)
        }
        WM_LBUTTONDOWN => {
            let x = (lparam.0 & 0xFFFF) as i16 as i32;
            let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;

            let mut rect = RECT::default();
            let _ = GetClientRect(hwnd, &mut rect);
            let width = rect.right - rect.left;
            let height = rect.bottom - rect.top;

            if is_point_in_close_button(x, y, width, height) {
                CLOSE_BUTTON_PRESSED.store(true, Ordering::SeqCst);
            }

            LRESULT(0)
        }
        WM_LBUTTONUP => {
            let x = (lparam.0 & 0xFFFF) as i16 as i32;
            let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;

            let mut rect = RECT::default();
            let _ = GetClientRect(hwnd, &mut rect);
            let width = rect.right - rect.left;
            let height = rect.bottom - rect.top;

            let was_pressed = CLOSE_BUTTON_PRESSED.swap(false, Ordering::SeqCst);

            if was_pressed && is_point_in_close_button(x, y, width, height) {
                debug!("Close button clicked");
                // Invoke the callback if set
                let ptr = CLOSE_CALLBACK.load(Ordering::SeqCst);
                if !ptr.is_null() {
                    let callback = &*(ptr as *const CloseCallback);
                    callback();
                }
            }

            LRESULT(0)
        }
        WM_DESTROY => {
            OVERLAY_HWND.store(std::ptr::null_mut(), Ordering::SeqCst);
            OVERLAY_VISIBLE.store(false, Ordering::SeqCst);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overlay_window_new() {
        let overlay = WindowsOverlayWindow::new();
        assert!(!overlay.visible);
        assert!(!overlay.class_registered);
        assert!((overlay.opacity - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_overlay_window_default() {
        let overlay = WindowsOverlayWindow::default();
        assert!(!overlay.visible);
        assert!(!overlay.class_registered);
    }

    #[test]
    fn test_is_point_in_close_button() {
        let width = 1920;
        let height = 1080;

        // Center of the close button
        let center_x = width - CLOSE_BUTTON_MARGIN - CLOSE_BUTTON_RADIUS;
        let center_y = height - CLOSE_BUTTON_MARGIN - CLOSE_BUTTON_RADIUS;

        // Point at center should be inside
        assert!(is_point_in_close_button(center_x, center_y, width, height));

        // Point far away should be outside
        assert!(!is_point_in_close_button(0, 0, width, height));
        assert!(!is_point_in_close_button(
            width / 2,
            height / 2,
            width,
            height
        ));
    }

    #[test]
    fn test_opacity_clamping() {
        let mut overlay = WindowsOverlayWindow::new();

        overlay.set_opacity(0.5);
        assert!((overlay.opacity - 0.5).abs() < 0.001);

        overlay.set_opacity(1.5);
        assert!((overlay.opacity - 1.0).abs() < 0.001);

        overlay.set_opacity(-0.5);
        assert!(overlay.opacity.abs() < 0.001);
    }

    #[test]
    fn test_bounds_initially_zero() {
        let overlay = WindowsOverlayWindow::new();
        let bounds = overlay.bounds();
        assert!(bounds.is_empty());
    }

    #[test]
    fn test_timer_text_lock() {
        // Test that timer text can be set and read
        WindowsOverlayWindow::set_timer_text(Some("Test"));
        {
            let timer_text = TIMER_TEXT.read().unwrap();
            assert!(timer_text.is_some());
        }

        WindowsOverlayWindow::set_timer_text(None);
        {
            let timer_text = TIMER_TEXT.read().unwrap();
            assert!(timer_text.is_none());
        }
    }

    #[test]
    fn test_global_atomics_default_state() {
        // These tests verify the default state of global atomics
        // Note: These may fail if other tests have modified the state
        // In a real scenario, you'd want test isolation
        assert!(!CLOSE_BUTTON_HOVERED.load(Ordering::SeqCst));
        assert!(!CLOSE_BUTTON_PRESSED.load(Ordering::SeqCst));
    }
}
