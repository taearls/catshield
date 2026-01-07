//! UI helper functions for Cat Shield

use objc2::rc::Retained;
use objc2_app_kit::{NSColor, NSFont, NSTextField};
use objc2_core_foundation::{CGFloat, CGRect};
use objc2_foundation::{MainThreadMarker, NSString};

/// Create a configured NSTextField label.
///
/// Creates a non-editable, non-selectable text field suitable for use as a label.
/// The text field has no border and can have a transparent or colored background.
pub fn create_label(
    mtm: MainThreadMarker,
    text: &str,
    frame: CGRect,
    font_size: CGFloat,
    text_color: &NSColor,
    is_bold: bool,
) -> Retained<NSTextField> {
    let label = NSTextField::new(mtm);
    label.setStringValue(&NSString::from_str(text));
    label.setEditable(false);
    label.setSelectable(false);
    label.setBordered(false);
    label.setDrawsBackground(false);
    label.setTextColor(Some(text_color));

    let font = if is_bold {
        NSFont::boldSystemFontOfSize(font_size)
    } else {
        NSFont::systemFontOfSize(font_size)
    };
    label.setFont(Some(&font));
    label.setFrame(frame);

    label
}
