use cocoa::appkit::{
    NSButton, NSTextField, NSView, NSViewHeightSizable, NSViewWidthSizable,
    NSBackingStoreBuffered,
};
use cocoa::base::{id, nil, NO, YES};
use cocoa::foundation::{NSAutoreleasePool, NSPoint, NSRect, NSSize, NSString};
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel, BOOL};
use objc::{class, msg_send, sel, sel_impl};
use serde_json::json;
use std::os::raw::c_void;
use std::sync::Once;
use tauri::Emitter;

const TOOLBAR_HEIGHT: f64 = 44.0;
const BUTTON_SIZE: f64 = 28.0;
const URL_BAR_HEIGHT: f64 = 28.0;
const PADDING: f64 = 12.0;
const BUTTON_SPACING: f64 = 4.0;
const FAVICON_SIZE: f64 = 16.0;

/// Creates the custom toolbar NSView for a browser window.
/// Returns (toolbar_view, url_field, favicon_view, back_button, forward_button).
pub unsafe fn create_toolbar(window_label: &str) -> (id, id, id, id, id) {
    // Create the toolbar container view
    let toolbar: id = msg_send![class!(NSView), alloc];
    let toolbar: id = msg_send![toolbar, initWithFrame: NSRect::new(
        NSPoint::new(0.0, 0.0),
        NSSize::new(800.0, TOOLBAR_HEIGHT),
    )];
    let _: () = msg_send![toolbar, setAutoresizingMask: NSViewWidthSizable];

    // Add a subtle bottom border via a layer
    let _: () = msg_send![toolbar, setWantsLayer: YES];
    let layer: id = msg_send![toolbar, layer];
    let border_color: id = msg_send![class!(NSColor), separatorColor];
    let cg_color: id = msg_send![border_color, CGColor];
    let _: () = msg_send![layer, setBorderColor: cg_color];
    let _: () = msg_send![layer, setBorderWidth: 0.5_f64];

    // -- Back button (left side) --
    let back_button = create_nav_button("chevron.left", PADDING, window_label, sel!(goBack:));
    let _: () = msg_send![toolbar, addSubview: back_button];

    // -- Forward button --
    let forward_x = PADDING + BUTTON_SIZE + BUTTON_SPACING;
    let forward_button =
        create_nav_button("chevron.right", forward_x, window_label, sel!(goForward:));
    let _: () = msg_send![toolbar, addSubview: forward_button];

    // -- Favicon image view --
    let url_bar_x = forward_x + BUTTON_SIZE + PADDING;
    let favicon_y = (TOOLBAR_HEIGHT - FAVICON_SIZE) / 2.0;
    let favicon_view: id = msg_send![class!(NSImageView), alloc];
    let favicon_view: id = msg_send![favicon_view, initWithFrame: NSRect::new(
        NSPoint::new(url_bar_x, favicon_y),
        NSSize::new(FAVICON_SIZE, FAVICON_SIZE),
    )];
    // Set a default globe icon
    let globe_image: id = msg_send![class!(NSImage), imageWithSystemSymbolName:
        NSString::alloc(nil).init_str("globe")
        accessibilityDescription: nil];
    let _: () = msg_send![favicon_view, setImage: globe_image];
    let _: () = msg_send![favicon_view, setImageScaling: 2_i64]; // NSImageScaleProportionallyUpOrDown
    let _: () = msg_send![toolbar, addSubview: favicon_view];

    // -- URL text field (rounded, pill-shaped) --
    let url_field_x = url_bar_x + FAVICON_SIZE + 6.0;
    let url_field_y = (TOOLBAR_HEIGHT - URL_BAR_HEIGHT) / 2.0;
    // Right side: pin button (BUTTON_SIZE) + padding
    let right_reserved = PADDING + BUTTON_SIZE + PADDING;

    let url_field: id = msg_send![class!(NSTextField), alloc];
    let url_field: id = msg_send![url_field, initWithFrame: NSRect::new(
        NSPoint::new(url_field_x, url_field_y),
        NSSize::new(400.0, URL_BAR_HEIGHT), // width will be adjusted by autoresizing
    )];

    // Style the URL field
    let _: () = msg_send![url_field, setBezeled: YES];
    let _: () = msg_send![url_field, setBezelStyle: 1_i64]; // NSTextFieldRoundedBezel
    let _: () = msg_send![url_field, setEditable: YES];
    let _: () = msg_send![url_field, setSelectable: YES];
    let _: () = msg_send![url_field, setDrawsBackground: YES];

    // Placeholder text
    let placeholder = NSString::alloc(nil).init_str("Enter URL or search...");
    let _: () = msg_send![url_field, setPlaceholderString: placeholder];

    // Font size 13px to match the app's topbar
    let font: id = msg_send![class!(NSFont), systemFontOfSize: 13.0_f64];
    let _: () = msg_send![url_field, setFont: font];

    // Make it stretch horizontally
    let _: () = msg_send![url_field, setAutoresizingMask: NSViewWidthSizable];

    // Set up the URL field delegate for Enter key handling
    let delegate = create_url_field_delegate(window_label);
    let _: () = msg_send![url_field, setDelegate: delegate];

    let _: () = msg_send![toolbar, addSubview: url_field];

    // -- Pin/bookmark button (right side) --
    let pin_button = create_symbol_button("pin", 0.0, TOOLBAR_HEIGHT);
    let _: () = msg_send![pin_button, setAutoresizingMask: 1_u64]; // NSViewMinXMargin (stick to right)

    // Wire up pin button action
    let pin_target_class = get_or_register_toolbar_target_class();
    let pin_target: id = msg_send![pin_target_class, alloc];
    let pin_target: id = msg_send![pin_target, init];
    store_window_label(pin_target, window_label);
    let _: () = msg_send![pin_button, setTarget: pin_target];
    let _: () = msg_send![pin_button, setAction: sel!(pinPage:)];

    let _: () = msg_send![toolbar, addSubview: pin_button];

    // Store window label as associated object on toolbar for callbacks
    store_window_label(toolbar, window_label);

    (toolbar, url_field, favicon_view, back_button, forward_button)
}

/// Updates the toolbar layout when the window resizes.
/// Call this after adding the toolbar to the window's content view.
pub unsafe fn layout_toolbar(toolbar: id, window_width: f64) {
    let frame = NSRect::new(
        NSPoint::new(0.0, 0.0),
        NSSize::new(window_width, TOOLBAR_HEIGHT),
    );
    let _: () = msg_send![toolbar, setFrame: frame];
}

/// Returns the toolbar height constant.
pub fn toolbar_height() -> f64 {
    TOOLBAR_HEIGHT
}

// -- Internal helpers --

unsafe fn create_nav_button(symbol_name: &str, x: f64, _window_label: &str, action: Sel) -> id {
    let button = create_symbol_button(symbol_name, x, TOOLBAR_HEIGHT);

    // Set target/action — the target class handles dispatching
    let target_class = get_or_register_toolbar_target_class();
    let target: id = msg_send![target_class, alloc];
    let target: id = msg_send![target, init];
    let _: () = msg_send![button, setTarget: target];
    let _: () = msg_send![button, setAction: action];

    button
}

unsafe fn create_symbol_button(symbol_name: &str, x: f64, toolbar_height: f64) -> id {
    let y = (toolbar_height - BUTTON_SIZE) / 2.0;
    let button: id = msg_send![class!(NSButton), alloc];
    let button: id = msg_send![button, initWithFrame: NSRect::new(
        NSPoint::new(x, y),
        NSSize::new(BUTTON_SIZE, BUTTON_SIZE),
    )];

    // Borderless, transparent button
    let _: () = msg_send![button, setButtonType: 0_i64]; // NSButtonTypeMomentaryLight
    let _: () = msg_send![button, setBordered: NO];
    let _: () = msg_send![button, setBezelStyle: 0_i64];

    // Use SF Symbol
    let symbol: id = msg_send![class!(NSImage), imageWithSystemSymbolName:
        NSString::alloc(nil).init_str(symbol_name)
        accessibilityDescription: nil];
    if symbol != nil {
        // Configure symbol size
        let config: id = msg_send![class!(NSImageSymbolConfiguration),
            configurationWithPointSize: 14.0_f64
            weight: 5_i64  // NSFontWeightMedium
        ];
        let sized_symbol: id = msg_send![symbol, imageWithSymbolConfiguration: config];
        let _: () = msg_send![button, setImage: sized_symbol];
    }

    // Tint color
    let tint: id = msg_send![class!(NSColor), secondaryLabelColor];
    let _: () = msg_send![button, setContentTintColor: tint];

    button
}

extern "C" {
    fn objc_setAssociatedObject(
        object: *mut Object,
        key: *const c_void,
        value: *mut Object,
        policy: usize,
    );
    fn objc_getAssociatedObject(object: *mut Object, key: *const c_void) -> *mut Object;
}

/// Stores the window label string as an associated object on an NSView,
/// so callbacks can retrieve it to look up BrowserState.
pub unsafe fn store_window_label(view: id, label: &str) {
    let key = b"browserWindowLabel\0".as_ptr() as *const c_void;
    let ns_label = NSString::alloc(nil).init_str(label);
    objc_setAssociatedObject(
        view as *mut Object,
        key,
        ns_label as *mut Object,
        1, // OBJC_ASSOCIATION_RETAIN_NONATOMIC
    );
}

/// Retrieves the window label from an associated object.
pub unsafe fn get_window_label(view: id) -> Option<String> {
    let key = b"browserWindowLabel\0".as_ptr() as *const c_void;
    let ns_label = objc_getAssociatedObject(view as *mut Object, key) as id;
    if ns_label == nil {
        return None;
    }
    let cstr: *const i8 = msg_send![ns_label, UTF8String];
    if cstr.is_null() {
        return None;
    }
    Some(std::ffi::CStr::from_ptr(cstr).to_string_lossy().into_owned())
}

// -- NSTextField delegate for URL field Enter key --

static REGISTER_DELEGATE: Once = Once::new();
static mut DELEGATE_CLASS: *const Class = std::ptr::null();

unsafe fn create_url_field_delegate(window_label: &str) -> id {
    REGISTER_DELEGATE.call_once(|| {
        let superclass = class!(NSObject);
        let mut decl = ClassDecl::new("BrowserURLFieldDelegate", superclass).unwrap();

        // control:textView:doCommandBySelector:
        decl.add_method(
            sel!(control:textView:doCommandBySelector:),
            url_field_command as extern "C" fn(&Object, Sel, id, id, Sel) -> BOOL,
        );

        DELEGATE_CLASS = decl.register();
    });

    let delegate: id = msg_send![DELEGATE_CLASS, alloc];
    let delegate: id = msg_send![delegate, init];

    // Store window label on the delegate so we can look up state in callbacks
    store_window_label(delegate, window_label);

    delegate
}

extern "C" fn url_field_command(
    this: &Object,
    _sel: Sel,
    control: id,
    _text_view: id,
    command_selector: Sel,
) -> BOOL {
    unsafe {
        // Check if the user pressed Enter (insertNewline:)
        if command_selector == sel!(insertNewline:) {
            // Get the URL string from the text field
            let string_value: id = msg_send![control, stringValue];
            let cstr: *const i8 = msg_send![string_value, UTF8String];
            if !cstr.is_null() {
                let url_str = std::ffi::CStr::from_ptr(cstr)
                    .to_string_lossy()
                    .into_owned();

                // Get window label from delegate
                if let Some(label) = get_window_label(this as *const Object as id) {
                    // Navigate the active tab to this URL
                    // This is dispatched through the webview module
                    super::webview::navigate_active_tab(&label, &url_str);
                }
            }
            return YES; // We handled this command
        }
        NO
    }
}

// -- Toolbar target class for button actions --

static REGISTER_TARGET: Once = Once::new();
static mut TARGET_CLASS: *const Class = std::ptr::null();

fn get_or_register_toolbar_target_class() -> &'static Class {
    unsafe {
        REGISTER_TARGET.call_once(|| {
            let superclass = class!(NSObject);
            let mut decl = ClassDecl::new("BrowserToolbarTarget", superclass).unwrap();

            decl.add_method(
                sel!(goBack:),
                go_back_action as extern "C" fn(&Object, Sel, id),
            );
            decl.add_method(
                sel!(goForward:),
                go_forward_action as extern "C" fn(&Object, Sel, id),
            );
            decl.add_method(
                sel!(pinPage:),
                pin_page_action as extern "C" fn(&Object, Sel, id),
            );

            TARGET_CLASS = decl.register();
        });
        &*TARGET_CLASS
    }
}

extern "C" fn go_back_action(_this: &Object, _sel: Sel, sender: id) {
    unsafe {
        // Walk up from sender to find the toolbar, then get the window label
        let superview: id = msg_send![sender, superview];
        if let Some(label) = get_window_label(superview) {
            super::webview::go_back(&label);
        }
    }
}

extern "C" fn go_forward_action(_this: &Object, _sel: Sel, sender: id) {
    unsafe {
        let superview: id = msg_send![sender, superview];
        if let Some(label) = get_window_label(superview) {
            super::webview::go_forward(&label);
        }
    }
}

extern "C" fn pin_page_action(this: &Object, _sel: Sel, _sender: id) {
    unsafe {
        if let Some(label) = get_window_label(this as *const Object as id) {
            let state = super::state::BROWSER_STATE.lock().unwrap();
            if let Some(window) = state.windows.get(&label) {
                if let Some(tab) = window.tabs.get(window.active_tab) {
                    let payload = json!({
                        "url": tab.url,
                        "title": tab.title,
                        "favicon_url": tab.favicon_url,
                    });
                    if let Some(ref app) = state.app_handle {
                        let _ = app.emit("browser-pin-request", payload);
                    }
                }
            }
        }
    }
}
