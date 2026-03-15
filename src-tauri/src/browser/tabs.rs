use cocoa::appkit::NSView;
use cocoa::base::{id, nil, NO, YES};
use cocoa::foundation::{NSPoint, NSRect, NSSize, NSString};
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};
use objc::{class, msg_send, sel, sel_impl};
use std::sync::Once;

use super::state::{BrowserTab, BrowserWindow, BROWSER_STATE};
use super::toolbar;
use super::webview;

const TAB_BAR_HEIGHT: f64 = 28.0;
const TAB_MIN_WIDTH: f64 = 120.0;
const TAB_MAX_WIDTH: f64 = 200.0;
const TAB_HEIGHT: f64 = 24.0;
const TAB_PADDING: f64 = 8.0;
const NEW_TAB_BUTTON_WIDTH: f64 = 28.0;

/// Returns the tab bar height.
pub fn tab_bar_height() -> f64 {
    TAB_BAR_HEIGHT
}

/// Creates the tab bar NSView.
/// Returns the tab bar view (caller should add as subview).
pub unsafe fn create_tab_bar(window_label: &str) -> id {
    let tab_bar: id = msg_send![class!(NSView), alloc];
    let tab_bar: id = msg_send![tab_bar, initWithFrame: NSRect::new(
        NSPoint::new(0.0, 0.0),
        NSSize::new(800.0, TAB_BAR_HEIGHT),
    )];
    let _: () = msg_send![tab_bar, setAutoresizingMask: 18_u64]; // NSViewWidthSizable
    let _: () = msg_send![tab_bar, setWantsLayer: YES];

    // Subtle background
    let layer: id = msg_send![tab_bar, layer];
    let bg: id = msg_send![class!(NSColor), windowBackgroundColor];
    let cg_bg: id = msg_send![bg, CGColor];
    let _: () = msg_send![layer, setBackgroundColor: cg_bg];

    // Bottom border
    let border_color: id = msg_send![class!(NSColor), separatorColor];
    let cg_border: id = msg_send![border_color, CGColor];
    let _: () = msg_send![layer, setBorderColor: cg_border];
    let _: () = msg_send![layer, setBorderWidth: 0.5_f64];

    // Store window label for callbacks
    toolbar::store_window_label(tab_bar, window_label);

    // Add the "+" new tab button at the right
    let new_tab_btn = create_new_tab_button(window_label);
    tab_bar.addSubview_(new_tab_btn);

    tab_bar
}

/// Refreshes the tab bar UI to reflect the current tabs in the window.
/// Call this after adding/removing/switching tabs or when titles change.
pub fn refresh_tab_bar_for_window(window: &BrowserWindow) {
    unsafe {
        let tab_bar = window.tab_bar_view;
        if tab_bar == nil {
            return;
        }

        // Remove all existing tab button subviews (keep the + button which is last)
        let subviews: id = msg_send![tab_bar, subviews];
        let count: usize = msg_send![subviews, count];

        // Collect subviews to remove (all except the last one which is the + button)
        let mut to_remove: Vec<id> = Vec::new();
        for i in 0..count.saturating_sub(1) {
            let view: id = msg_send![subviews, objectAtIndex: i];
            to_remove.push(view);
        }
        for view in to_remove {
            let _: () = msg_send![view, removeFromSuperview];
        }

        // Calculate tab width
        let bar_frame: NSRect = msg_send![tab_bar, frame];
        let available_width = bar_frame.size.width - NEW_TAB_BUTTON_WIDTH - TAB_PADDING;
        let tab_count = window.tabs.len().max(1);
        let tab_width = (available_width / tab_count as f64)
            .max(TAB_MIN_WIDTH)
            .min(TAB_MAX_WIDTH);

        // Create tab buttons
        for (i, tab) in window.tabs.iter().enumerate() {
            let is_active = i == window.active_tab;
            let x = TAB_PADDING + (i as f64 * tab_width);
            let y = (TAB_BAR_HEIGHT - TAB_HEIGHT) / 2.0;

            let tab_button = create_tab_button(
                &tab.title,
                x,
                y,
                tab_width - 2.0, // small gap between tabs
                is_active,
                i,
                &window.label,
            );

            // Insert before the + button
            let plus_button: id = msg_send![subviews, lastObject];
            let _: () = msg_send![tab_bar, addSubview: tab_button positioned: 1_i64 relativeTo: plus_button]; // NSWindowBelow
        }
    }
}

unsafe fn create_tab_button(
    title: &str,
    x: f64,
    y: f64,
    width: f64,
    is_active: bool,
    tab_index: usize,
    window_label: &str,
) -> id {
    let button: id = msg_send![class!(NSButton), alloc];
    let button: id = msg_send![button, initWithFrame: NSRect::new(
        NSPoint::new(x, y),
        NSSize::new(width, TAB_HEIGHT),
    )];

    // Style
    let _: () = msg_send![button, setBordered: NO];
    let _: () = msg_send![button, setButtonType: 0_i64]; // Momentary light
    let _: () = msg_send![button, setWantsLayer: YES];

    let layer: id = msg_send![button, layer];
    let _: () = msg_send![layer, setCornerRadius: 6.0_f64];

    if is_active {
        let active_bg: id =
            msg_send![class!(NSColor), colorWithWhite: 0.0_f64 alpha: 0.06_f64];
        let cg_active: id = msg_send![active_bg, CGColor];
        let _: () = msg_send![layer, setBackgroundColor: cg_active];
    }

    // Title (truncated)
    let display_title = if title.is_empty() { "New Tab" } else { title };
    let truncated = if display_title.len() > 20 {
        format!("{}...", &display_title[..17])
    } else {
        display_title.to_string()
    };
    let ns_title = NSString::alloc(nil).init_str(&truncated);
    let _: () = msg_send![button, setTitle: ns_title];

    // Font
    let font: id = msg_send![class!(NSFont), systemFontOfSize: 11.0_f64];
    let _: () = msg_send![button, setFont: font];

    // Alignment
    let _: () = msg_send![button, setAlignment: 0_i64]; // NSTextAlignmentLeft

    // Store tab index and window label for the click handler
    let target = get_or_register_tab_target_class();
    let target_instance: id = msg_send![target, alloc];
    let target_instance: id = msg_send![target_instance, init];
    toolbar::store_window_label(target_instance, window_label);

    // Store tab index as tag
    let _: () = msg_send![button, setTag: tab_index as i64];
    let _: () = msg_send![button, setTarget: target_instance];
    let _: () = msg_send![button, setAction: sel!(tabClicked:)];

    button
}

unsafe fn create_new_tab_button(window_label: &str) -> id {
    let button: id = msg_send![class!(NSButton), alloc];
    let button: id = msg_send![button, initWithFrame: NSRect::new(
        NSPoint::new(0.0, (TAB_BAR_HEIGHT - TAB_HEIGHT) / 2.0),
        NSSize::new(NEW_TAB_BUTTON_WIDTH, TAB_HEIGHT),
    )];

    let _: () = msg_send![button, setBordered: NO];
    let _: () = msg_send![button, setButtonType: 0_i64];

    // "+" symbol
    let symbol: id = msg_send![class!(NSImage), imageWithSystemSymbolName:
        NSString::alloc(nil).init_str("plus")
        accessibilityDescription: nil];
    if symbol != nil {
        let config: id = msg_send![class!(NSImageSymbolConfiguration),
            configurationWithPointSize: 12.0_f64
            weight: 5_i64
        ];
        let sized: id = msg_send![symbol, imageWithSymbolConfiguration: config];
        let _: () = msg_send![button, setImage: sized];
    }

    let tint: id = msg_send![class!(NSColor), secondaryLabelColor];
    let _: () = msg_send![button, setContentTintColor: tint];

    // Stick to right edge
    let _: () = msg_send![button, setAutoresizingMask: 1_u64]; // NSViewMinXMargin

    let target = get_or_register_tab_target_class();
    let target_instance: id = msg_send![target, alloc];
    let target_instance: id = msg_send![target_instance, init];
    toolbar::store_window_label(target_instance, window_label);

    let _: () = msg_send![button, setTarget: target_instance];
    let _: () = msg_send![button, setAction: sel!(newTab:)];

    button
}

// -- Tab target class for button actions --

static REGISTER_TAB_TARGET: Once = Once::new();
static mut TAB_TARGET_CLASS: *const Class = std::ptr::null();

fn get_or_register_tab_target_class() -> &'static Class {
    unsafe {
        REGISTER_TAB_TARGET.call_once(|| {
            let superclass = class!(NSObject);
            let mut decl = ClassDecl::new("BrowserTabTarget", superclass).unwrap();

            decl.add_method(
                sel!(tabClicked:),
                tab_clicked_action as extern "C" fn(&Object, Sel, id),
            );
            decl.add_method(
                sel!(newTab:),
                new_tab_action as extern "C" fn(&Object, Sel, id),
            );

            TAB_TARGET_CLASS = decl.register();
        });
        &*TAB_TARGET_CLASS
    }
}

extern "C" fn tab_clicked_action(this: &Object, _sel: Sel, sender: id) {
    unsafe {
        let tab_index: i64 = msg_send![sender, tag];
        if let Some(label) = toolbar::get_window_label(this as *const Object as id) {
            switch_tab(&label, tab_index as usize);
        }
    }
}

extern "C" fn new_tab_action(this: &Object, _sel: Sel, _sender: id) {
    unsafe {
        if let Some(label) = toolbar::get_window_label(this as *const Object as id) {
            create_new_tab(&label, None);
        }
    }
}

// -- Public tab management functions --

/// Creates a new tab in the given browser window.
pub fn create_new_tab(window_label: &str, url: Option<&str>) {
    let mut state = BROWSER_STATE.lock().unwrap();
    if let Some(window) = state.windows.get_mut(window_label) {
        let initial_url = url.unwrap_or("about:blank");

        unsafe {
            let content_frame: NSRect = msg_send![window.content_area, frame];
            let webview_frame = NSRect::new(
                NSPoint::new(0.0, 0.0),
                NSSize::new(content_frame.size.width, content_frame.size.height),
            );

            let wk_webview =
                webview::create_webview(webview_frame, initial_url, window_label, window.data_store);

            // Hide the current active webview
            if let Some(current_tab) = window.tabs.get(window.active_tab) {
                let _: () = msg_send![current_tab.webview, setHidden: YES];
            }

            // Add the new webview to the content area
            window.content_area.addSubview_(wk_webview);

            let tab = BrowserTab {
                url: initial_url.to_string(),
                title: String::new(),
                favicon_url: None,
                webview: wk_webview,
                is_loading: true,
            };

            window.tabs.push(tab);
            window.active_tab = window.tabs.len() - 1;

            // Update toolbar URL field
            let ns_url = NSString::alloc(nil).init_str(initial_url);
            let _: () = msg_send![window.url_field, setStringValue: ns_url];
        }

        refresh_tab_bar_for_window(window);
    }
}

/// Switches to the tab at the given index.
pub fn switch_tab(window_label: &str, tab_index: usize) {
    let mut state = BROWSER_STATE.lock().unwrap();
    if let Some(window) = state.windows.get_mut(window_label) {
        if tab_index >= window.tabs.len() || tab_index == window.active_tab {
            return;
        }

        unsafe {
            // Hide current webview
            if let Some(current) = window.tabs.get(window.active_tab) {
                let _: () = msg_send![current.webview, setHidden: YES];
            }

            // Show new webview
            if let Some(new_tab) = window.tabs.get(tab_index) {
                let _: () = msg_send![new_tab.webview, setHidden: NO];

                // Update URL field
                let ns_url = NSString::alloc(nil).init_str(&new_tab.url);
                let _: () = msg_send![window.url_field, setStringValue: ns_url];

                // Update back/forward buttons
                let can_go_back: objc::runtime::BOOL = msg_send![new_tab.webview, canGoBack];
                let can_go_forward: objc::runtime::BOOL =
                    msg_send![new_tab.webview, canGoForward];
                let _: () = msg_send![window.back_button, setEnabled: can_go_back];
                let _: () = msg_send![window.forward_button, setEnabled: can_go_forward];
            }

            window.active_tab = tab_index;
        }

        refresh_tab_bar_for_window(window);
    }
}

/// Closes the tab at the given index.
pub fn close_tab(window_label: &str, tab_index: usize) {
    let mut state = BROWSER_STATE.lock().unwrap();
    if let Some(window) = state.windows.get_mut(window_label) {
        if tab_index >= window.tabs.len() {
            return;
        }

        // If this is the last tab, we could close the window
        // For now, don't allow closing the last tab
        if window.tabs.len() <= 1 {
            return;
        }

        unsafe {
            // Remove the webview from the content area
            let tab = &window.tabs[tab_index];
            let _: () = msg_send![tab.webview, removeFromSuperview];
        }

        window.tabs.remove(tab_index);

        // Adjust active_tab
        if window.active_tab >= window.tabs.len() {
            window.active_tab = window.tabs.len() - 1;
        } else if window.active_tab > tab_index {
            window.active_tab -= 1;
        }

        // Show the new active tab
        unsafe {
            if let Some(active) = window.tabs.get(window.active_tab) {
                let _: () = msg_send![active.webview, setHidden: NO];

                let ns_url = NSString::alloc(nil).init_str(&active.url);
                let _: () = msg_send![window.url_field, setStringValue: ns_url];
            }
        }

        refresh_tab_bar_for_window(window);
    }
}
