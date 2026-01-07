//! Menu action handlers for Cat Shield

use crate::ui::shield::activate_shield;
use crate::ui::state::SHIELD_ACTIVE;
use objc2::rc::Retained;
use objc2::{define_class, msg_send};
use objc2_app_kit::NSMenuItem;
use objc2_foundation::{MainThreadMarker, NSObject};
use std::sync::atomic::Ordering;

/// Empty ivars for the MenuActionHandler
pub struct MenuActionHandlerIvars {}

// Menu action handler for the "Start Protection" menu item
// This class provides a target for the menu item action selector
define_class!(
    #[unsafe(super(NSObject))]
    #[name = "MenuActionHandler"]
    #[ivars = MenuActionHandlerIvars]
    pub struct MenuActionHandler;

    impl MenuActionHandler {
        /// Action method called when "Start Protection" is clicked
        #[unsafe(method(startProtection:))]
        unsafe fn start_protection(&self, _sender: Option<&NSMenuItem>) {
            // Prevent double-activation
            if SHIELD_ACTIVE.load(Ordering::SeqCst) {
                return;
            }

            // Call the activate_shield function
            if let Some(mtm) = MainThreadMarker::new() {
                activate_shield(mtm);
            }
        }
    }
);

impl MenuActionHandler {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = mtm.alloc::<MenuActionHandler>();
        let this = this.set_ivars(MenuActionHandlerIvars {});
        unsafe { msg_send![super(this), init] }
    }
}
