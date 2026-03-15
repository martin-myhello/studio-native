use cocoa::base::{id, nil, NO, YES};
use cocoa::foundation::{NSRect, NSString};
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel, BOOL};
use objc::{class, msg_send, sel, sel_impl};
use std::sync::Once;

use super::state::BROWSER_STATE;
use super::toolbar;

// -- Per-project session isolation (Phase 3) --

/// Creates a per-project WKWebsiteDataStore using a UUID derived from the project ID.
/// Each project gets its own cookies, localStorage, and cache (macOS 14+).
pub unsafe fn create_data_store_for_project(project_id: &str) -> id {
    let hash = {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        project_id.hash(&mut hasher);
        hasher.finish()
    };
    let uuid_bytes: [u8; 16] = {
        let mut bytes = [0u8; 16];
        bytes[..8].copy_from_slice(&hash.to_le_bytes());
        let hash2 = hash.wrapping_mul(0x517cc1b727220a95);
        bytes[8..].copy_from_slice(&hash2.to_le_bytes());
        bytes[6] = (bytes[6] & 0x0f) | 0x40;
        bytes[8] = (bytes[8] & 0x3f) | 0x80;
        bytes
    };

    let uuid: id = msg_send![class!(NSUUID), alloc];
    let uuid: id = msg_send![uuid, initWithUUIDBytes: uuid_bytes.as_ptr()];

    // WKWebsiteDataStore.dataStoreForIdentifier: (macOS 14+)
    let data_store: id = msg_send![class!(WKWebsiteDataStore), dataStoreForIdentifier: uuid];
    data_store
}

// -- WKWebView creation --

/// Creates a new WKWebView with the given data store for per-project session isolation.
pub unsafe fn create_webview(
    frame: NSRect,
    initial_url: &str,
    window_label: &str,
    data_store: id,
) -> id {
    let config: id = msg_send![class!(WKWebViewConfiguration), alloc];
    let config: id = msg_send![config, init];

    if data_store != nil {
        let _: () = msg_send![config, setWebsiteDataStore: data_store];
    }

    let prefs: id = msg_send![config, preferences];
    let _: () = msg_send![prefs, setJavaScriptEnabled: YES];
    let _: () = msg_send![config, setAllowsInlineMediaPlayback: YES];

    let webview: id = msg_send![class!(WKWebView), alloc];
    let webview: id = msg_send![webview, initWithFrame: frame configuration: config];
    let _: () = msg_send![webview, setAutoresizingMask: 18_u64];
    let _: () = msg_send![webview, setAllowsBackForwardNavigationGestures: YES];

    let delegate = create_navigation_delegate(window_label);
    let _: () = msg_send![webview, setNavigationDelegate: delegate];

    navigate_webview(webview, initial_url);
    webview
}

pub unsafe fn navigate_webview(webview: id, url_str: &str) {
    let url_string = ensure_url_scheme(url_str);
    let ns_url_string = NSString::alloc(nil).init_str(&url_string);
    let url: id = msg_send![class!(NSURL), URLWithString: ns_url_string];
    if url != nil {
        let request: id = msg_send![class!(NSURLRequest), requestWithURL: url];
        let _: () = msg_send![webview, loadRequest: request];
    }
}

fn ensure_url_scheme(url: &str) -> String {
    let trimmed = url.trim();
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed.to_string()
    } else if trimmed.contains('.') && !trimmed.contains(' ') {
        format!("https://{}", trimmed)
    } else {
        format!(
            "https://www.google.com/search?q={}",
            urlencoding_simple(trimmed)
        )
    }
}

fn urlencoding_simple(s: &str) -> String {
    let mut result = String::with_capacity(s.len() * 3);
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            b' ' => result.push('+'),
            _ => {
                result.push('%');
                result.push_str(&format!("{:02X}", byte));
            }
        }
    }
    result
}

// -- Public navigation functions --

pub fn navigate_active_tab(window_label: &str, url: &str) {
    let state = BROWSER_STATE.lock().unwrap();
    if let Some(window) = state.windows.get(window_label) {
        if let Some(tab) = window.tabs.get(window.active_tab) {
            unsafe { navigate_webview(tab.webview, url); }
        }
    }
}

pub fn go_back(window_label: &str) {
    let state = BROWSER_STATE.lock().unwrap();
    if let Some(window) = state.windows.get(window_label) {
        if let Some(tab) = window.tabs.get(window.active_tab) {
            unsafe { let _: () = msg_send![tab.webview, goBack]; }
        }
    }
}

pub fn go_forward(window_label: &str) {
    let state = BROWSER_STATE.lock().unwrap();
    if let Some(window) = state.windows.get(window_label) {
        if let Some(tab) = window.tabs.get(window.active_tab) {
            unsafe { let _: () = msg_send![tab.webview, goForward]; }
        }
    }
}

// -- WKNavigationDelegate --

static REGISTER_NAV_DELEGATE: Once = Once::new();
static mut NAV_DELEGATE_CLASS: *const Class = std::ptr::null();

unsafe fn create_navigation_delegate(window_label: &str) -> id {
    REGISTER_NAV_DELEGATE.call_once(|| {
        let superclass = class!(NSObject);
        let mut decl = ClassDecl::new("BrowserNavigationDelegate", superclass).unwrap();

        decl.add_method(
            sel!(webView:didCommitNavigation:),
            did_commit_navigation as extern "C" fn(&Object, Sel, id, id),
        );
        decl.add_method(
            sel!(webView:didFinishNavigation:),
            did_finish_navigation as extern "C" fn(&Object, Sel, id, id),
        );
        decl.add_method(
            sel!(webView:didStartProvisionalNavigation:),
            did_start_provisional_navigation as extern "C" fn(&Object, Sel, id, id),
        );
        decl.add_method(
            sel!(webView:didFailNavigation:withError:),
            did_fail_navigation as extern "C" fn(&Object, Sel, id, id, id),
        );
        // Phase 5: download support
        decl.add_method(
            sel!(webView:navigationResponse:didBecomeDownload:),
            nav_response_did_become_download as extern "C" fn(&Object, Sel, id, id, id),
        );
        decl.add_method(
            sel!(webView:decidePolicyForNavigationResponse:decisionHandler:),
            decide_policy_for_response as extern "C" fn(&Object, Sel, id, id, id),
        );

        if let Some(protocol) = objc::runtime::Protocol::get("WKNavigationDelegate") {
            decl.add_protocol(protocol);
        }

        NAV_DELEGATE_CLASS = decl.register();
    });

    let delegate: id = msg_send![NAV_DELEGATE_CLASS, alloc];
    let delegate: id = msg_send![delegate, init];
    toolbar::store_window_label(delegate, window_label);
    delegate
}

extern "C" fn did_start_provisional_navigation(
    this: &Object, _sel: Sel, _webview: id, _navigation: id,
) {
    unsafe {
        if let Some(label) = toolbar::get_window_label(this as *const Object as id) {
            let mut state = BROWSER_STATE.lock().unwrap();
            if let Some(window) = state.windows.get_mut(&label) {
                if let Some(tab) = window.active_tab_mut() {
                    tab.is_loading = true;
                }
            }
        }
    }
}

extern "C" fn did_commit_navigation(this: &Object, _sel: Sel, webview: id, _navigation: id) {
    unsafe {
        let url: id = msg_send![webview, URL];
        if url == nil { return; }
        let url_string: id = msg_send![url, absoluteString];
        let cstr: *const i8 = msg_send![url_string, UTF8String];
        if cstr.is_null() { return; }
        let url_str = std::ffi::CStr::from_ptr(cstr).to_string_lossy().into_owned();

        if let Some(label) = toolbar::get_window_label(this as *const Object as id) {
            let mut state = BROWSER_STATE.lock().unwrap();
            if let Some(window) = state.windows.get_mut(&label) {
                if let Some(tab) = window.active_tab_mut() {
                    tab.url = url_str.clone();
                }
                let ns_url = NSString::alloc(nil).init_str(&url_str);
                let _: () = msg_send![window.url_field, setStringValue: ns_url];
                let can_go_back: BOOL = msg_send![webview, canGoBack];
                let can_go_forward: BOOL = msg_send![webview, canGoForward];
                let _: () = msg_send![window.back_button, setEnabled: can_go_back];
                let _: () = msg_send![window.forward_button, setEnabled: can_go_forward];
            }
        }
    }
}

extern "C" fn did_finish_navigation(this: &Object, _sel: Sel, webview: id, _navigation: id) {
    unsafe {
        let title: id = msg_send![webview, title];
        let title_str = if title != nil {
            let cstr: *const i8 = msg_send![title, UTF8String];
            if !cstr.is_null() {
                std::ffi::CStr::from_ptr(cstr).to_string_lossy().into_owned()
            } else { String::new() }
        } else { String::new() };

        if let Some(label) = toolbar::get_window_label(this as *const Object as id) {
            let mut state = BROWSER_STATE.lock().unwrap();
            if let Some(window) = state.windows.get_mut(&label) {
                if let Some(tab) = window.active_tab_mut() {
                    tab.title = title_str.clone();
                    tab.is_loading = false;
                }
                super::tabs::refresh_tab_bar_for_window(window);
            }
        }

        fetch_favicon(webview, this as *const Object as id);
    }
}

extern "C" fn did_fail_navigation(
    this: &Object, _sel: Sel, _webview: id, _navigation: id, _error: id,
) {
    unsafe {
        if let Some(label) = toolbar::get_window_label(this as *const Object as id) {
            let mut state = BROWSER_STATE.lock().unwrap();
            if let Some(window) = state.windows.get_mut(&label) {
                if let Some(tab) = window.active_tab_mut() {
                    tab.is_loading = false;
                }
            }
        }
    }
}

// -- Download handling (Phase 5) --

extern "C" fn decide_policy_for_response(
    _this: &Object, _sel: Sel, _webview: id, navigation_response: id, decision_handler: id,
) {
    unsafe {
        let can_show: BOOL = msg_send![navigation_response, canShowMIMEType];
        let policy: i64 = if can_show == NO { 2 } else { 1 };
        let block_ptr = decision_handler as *mut block::Block<(i64,), ()>;
        (*block_ptr).call((policy,));
    }
}

extern "C" fn nav_response_did_become_download(
    _this: &Object, _sel: Sel, _webview: id, _navigation_response: id, download: id,
) {
    unsafe {
        let delegate = create_download_delegate();
        let _: () = msg_send![download, setDelegate: delegate];
    }
}

static REGISTER_DOWNLOAD_DELEGATE: Once = Once::new();
static mut DOWNLOAD_DELEGATE_CLASS: *const Class = std::ptr::null();

unsafe fn create_download_delegate() -> id {
    REGISTER_DOWNLOAD_DELEGATE.call_once(|| {
        let superclass = class!(NSObject);
        let mut decl = ClassDecl::new("BrowserDownloadDelegate", superclass).unwrap();
        decl.add_method(
            sel!(download:decideDestinationUsingResponse:suggestedFilename:completionHandler:),
            download_decide_destination as extern "C" fn(&Object, Sel, id, id, id, id),
        );
        decl.add_method(
            sel!(downloadDidFinish:),
            download_did_finish as extern "C" fn(&Object, Sel, id),
        );
        if let Some(protocol) = objc::runtime::Protocol::get("WKDownloadDelegate") {
            decl.add_protocol(protocol);
        }
        DOWNLOAD_DELEGATE_CLASS = decl.register();
    });
    let delegate: id = msg_send![DOWNLOAD_DELEGATE_CLASS, alloc];
    let delegate: id = msg_send![delegate, init];
    delegate
}

extern "C" fn download_decide_destination(
    _this: &Object, _sel: Sel, _download: id, _response: id,
    suggested_filename: id, completion_handler: id,
) {
    unsafe {
        let file_manager: id = msg_send![class!(NSFileManager), defaultManager];
        let desktop_urls: id = msg_send![file_manager,
            URLsForDirectory: 12_u64  // NSDesktopDirectory
            inDomains: 1_u64          // NSUserDomainMask
        ];
        let desktop_url: id = msg_send![desktop_urls, firstObject];
        let dest_url = if desktop_url != nil {
            msg_send![desktop_url, URLByAppendingPathComponent: suggested_filename]
        } else {
            nil
        };
        let block_ptr = completion_handler as *mut block::Block<(id,), ()>;
        (*block_ptr).call((dest_url,));
    }
}

extern "C" fn download_did_finish(_this: &Object, _sel: Sel, _download: id) {}

// -- Favicon fetching --

unsafe fn fetch_favicon(webview: id, _delegate: id) {
    let js_code = NSString::alloc(nil).init_str(
        r#"(function() {
            var icons = document.querySelectorAll("link[rel*='icon']");
            if (icons.length > 0) { return icons[icons.length - 1].href; }
            return location.origin + '/favicon.ico';
        })()"#,
    );
    let _: () = msg_send![webview, evaluateJavaScript: js_code completionHandler: nil];
}
