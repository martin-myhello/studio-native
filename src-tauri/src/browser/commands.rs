use cocoa::appkit::NSView;
use cocoa::base::{id, nil, YES};
use cocoa::foundation::{NSPoint, NSRect, NSSize, NSString};
use objc::{class, msg_send, sel, sel_impl};
use serde::Serialize;
use tauri::Manager;

use super::state::{BrowserTab, BrowserWindow, BROWSER_STATE};
use super::tabs;
use super::toolbar;
use super::webview;

#[derive(Serialize)]
pub struct PageInfo {
    pub url: String,
    pub title: String,
    pub favicon_url: Option<String>,
}

#[tauri::command]
pub fn open_browser_window(
    app: tauri::AppHandle,
    url: String,
    project_id: String,
) -> Result<(), String> {
    let label = format!(
        "browser-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    );

    let window = tauri::WindowBuilder::new(&app, &label)
        .title("Browser")
        .inner_size(1200.0, 800.0)
        .min_inner_size(400.0, 300.0)
        .build()
        .map_err(|e| e.to_string())?;

    #[cfg(target_os = "macos")]
    {
        let ns_window = window.ns_window().map_err(|e| e.to_string())? as id;

        unsafe {
            let _: () = msg_send![ns_window, setTitlebarAppearsTransparent: YES];
            let _: () = msg_send![ns_window, setTitleVisibility: 1_i64];

            let bg_color: id = msg_send![class!(NSColor), whiteColor];
            let _: () = msg_send![ns_window, setBackgroundColor: bg_color];

            let content_view: id = msg_send![ns_window, contentView];
            let content_frame: NSRect = msg_send![content_view, frame];

            // Create per-project data store for session isolation
            let data_store = webview::create_data_store_for_project(&project_id);

            // -- Toolbar (top) --
            let (toolbar_view, url_field, favicon_view, back_button, forward_button) =
                toolbar::create_toolbar(&label);

            let toolbar_h = toolbar::toolbar_height();
            let toolbar_frame = NSRect::new(
                NSPoint::new(0.0, content_frame.size.height - toolbar_h),
                NSSize::new(content_frame.size.width, toolbar_h),
            );
            let _: () = msg_send![toolbar_view, setFrame: toolbar_frame];
            let _: () = msg_send![toolbar_view, setAutoresizingMask: 18_u64 | 8_u64];
            content_view.addSubview_(toolbar_view);

            // -- Tab bar (below toolbar) --
            let tab_bar_h = tabs::tab_bar_height();
            let tab_bar_view = tabs::create_tab_bar(&label);
            let tab_bar_frame = NSRect::new(
                NSPoint::new(0.0, content_frame.size.height - toolbar_h - tab_bar_h),
                NSSize::new(content_frame.size.width, tab_bar_h),
            );
            let _: () = msg_send![tab_bar_view, setFrame: tab_bar_frame];
            let _: () = msg_send![tab_bar_view, setAutoresizingMask: 18_u64 | 8_u64];
            content_view.addSubview_(tab_bar_view);

            // -- Content area (below tab bar, fills remaining space) --
            let content_area_height = content_frame.size.height - toolbar_h - tab_bar_h;
            let content_area_frame = NSRect::new(
                NSPoint::new(0.0, 0.0),
                NSSize::new(content_frame.size.width, content_area_height),
            );
            let content_area: id = msg_send![class!(NSView), alloc];
            let content_area: id = msg_send![content_area, initWithFrame: content_area_frame];
            let _: () = msg_send![content_area, setAutoresizingMask: 18_u64 | 2_u64];
            content_view.addSubview_(content_area);

            // -- Initial WKWebView --
            let webview_frame = NSRect::new(
                NSPoint::new(0.0, 0.0),
                NSSize::new(content_area_frame.size.width, content_area_frame.size.height),
            );
            let wk_webview =
                webview::create_webview(webview_frame, &url, &label, data_store);
            content_area.addSubview_(wk_webview);

            let tab = BrowserTab {
                url: url.clone(),
                title: String::new(),
                favicon_url: None,
                webview: wk_webview,
                is_loading: true,
            };

            let browser_window = BrowserWindow {
                label: label.clone(),
                project_id,
                tabs: vec![tab],
                active_tab: 0,
                toolbar_view,
                url_field,
                favicon_view,
                back_button,
                forward_button,
                content_area,
                tab_bar_view,
                data_store,
            };

            let mut state = BROWSER_STATE.lock().unwrap();
            if state.app_handle.is_none() {
                state.app_handle = Some(app.clone());
            }
            state.windows.insert(label.clone(), browser_window);

            let ns_url = NSString::alloc(nil).init_str(&url);
            let _: () = msg_send![url_field, setStringValue: ns_url];

            // Refresh tab bar to show the initial tab
            if let Some(w) = state.windows.get(&label) {
                tabs::refresh_tab_bar_for_window(w);
            }
        }
    }

    Ok(())
}

#[tauri::command]
pub fn browser_navigate(window_label: String, url: String) -> Result<(), String> {
    let state = BROWSER_STATE.lock().unwrap();
    if let Some(window) = state.windows.get(&window_label) {
        if let Some(tab) = window.tabs.get(window.active_tab) {
            unsafe {
                webview::navigate_webview(tab.webview, &url);
            }
            Ok(())
        } else {
            Err("No active tab".to_string())
        }
    } else {
        Err("Browser window not found".to_string())
    }
}

#[tauri::command]
pub fn browser_go_back(window_label: String) -> Result<(), String> {
    webview::go_back(&window_label);
    Ok(())
}

#[tauri::command]
pub fn browser_go_forward(window_label: String) -> Result<(), String> {
    webview::go_forward(&window_label);
    Ok(())
}

#[tauri::command]
pub fn browser_get_page_info(window_label: String) -> Result<PageInfo, String> {
    let state = BROWSER_STATE.lock().unwrap();
    if let Some(window) = state.windows.get(&window_label) {
        if let Some(tab) = window.tabs.get(window.active_tab) {
            Ok(PageInfo {
                url: tab.url.clone(),
                title: tab.title.clone(),
                favicon_url: tab.favicon_url.clone(),
            })
        } else {
            Err("No active tab".to_string())
        }
    } else {
        Err("Browser window not found".to_string())
    }
}

#[tauri::command]
pub fn browser_new_tab(
    window_label: String,
    url: Option<String>,
) -> Result<(), String> {
    tabs::create_new_tab(&window_label, url.as_deref());
    Ok(())
}

#[tauri::command]
pub fn browser_close_tab(
    window_label: String,
    tab_index: usize,
) -> Result<(), String> {
    tabs::close_tab(&window_label, tab_index);
    Ok(())
}

#[tauri::command]
pub fn browser_switch_tab(
    window_label: String,
    tab_index: usize,
) -> Result<(), String> {
    tabs::switch_tab(&window_label, tab_index);
    Ok(())
}
