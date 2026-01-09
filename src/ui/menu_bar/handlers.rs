//! Menu action handlers for Cat Shield

use crate::ui::shield::activate_shield;
use crate::ui::state::SHIELD_ACTIVE;
use objc2::rc::Retained;
use objc2::{define_class, msg_send};
use objc2_app_kit::{NSMenuItem, NSWorkspace};
use objc2_foundation::{MainThreadMarker, NSObject, NSURL};
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

/// Empty ivars for the HelpActionHandler
pub struct HelpActionHandlerIvars {}

// Help action handler for opening URLs in the default browser
// This class provides targets for the Help submenu item actions
define_class!(
    #[unsafe(super(NSObject))]
    #[name = "HelpActionHandler"]
    #[ivars = HelpActionHandlerIvars]
    pub struct HelpActionHandler;

    impl HelpActionHandler {
        /// Action method called when "View Documentation" is clicked
        #[unsafe(method(viewDocumentation:))]
        unsafe fn view_documentation(&self, _sender: Option<&NSMenuItem>) {
            open_url("https://github.com/taearls/catshield/blob/main/README.md");
        }

        /// Action method called when "Report Issue" is clicked
        #[unsafe(method(reportIssue:))]
        unsafe fn report_issue(&self, _sender: Option<&NSMenuItem>) {
            open_url("https://github.com/taearls/catshield/issues/new");
        }
    }
);

impl HelpActionHandler {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = mtm.alloc::<HelpActionHandler>();
        let this = this.set_ivars(HelpActionHandlerIvars {});
        unsafe { msg_send![super(this), init] }
    }
}

/// Open a URL in the default browser using NSWorkspace
///
/// Returns `true` if the URL was successfully opened, `false` otherwise.
fn open_url(url_str: &str) -> bool {
    // Create NSURL from the string
    let ns_url_str = objc2_foundation::NSString::from_str(url_str);
    let Some(url) = NSURL::URLWithString(&ns_url_str) else {
        eprintln!("HelpActionHandler: invalid URL: {url_str}");
        return false;
    };
    let workspace = NSWorkspace::sharedWorkspace();
    let ok = workspace.openURL(&url);
    if !ok {
        eprintln!("HelpActionHandler: failed to open URL: {url_str}");
    }
    ok
}
