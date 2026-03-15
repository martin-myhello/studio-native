use std::collections::HashMap;
use std::sync::Mutex;

use cocoa::base::id;

use std::sync::LazyLock;

/// Represents a single browser tab within a browser window.
pub struct BrowserTab {
    pub url: String,
    pub title: String,
    pub favicon_url: Option<String>,
    /// Raw pointer to the WKWebView instance (retained).
    pub webview: id,
    pub is_loading: bool,
}

// Safety: WKWebView pointers are only accessed on the main thread
// via dispatch_async. The Mutex ensures single-threaded access to the map.
unsafe impl Send for BrowserTab {}

/// Represents a browser window with its toolbar and content area.
pub struct BrowserWindow {
    pub label: String,
    pub project_id: String,
    pub tabs: Vec<BrowserTab>,
    pub active_tab: usize,
    /// Raw pointer to the toolbar NSView (retained).
    pub toolbar_view: id,
    /// Raw pointer to the URL text field within the toolbar.
    pub url_field: id,
    /// Raw pointer to the favicon NSImageView within the toolbar.
    pub favicon_view: id,
    /// Raw pointer to the back button.
    pub back_button: id,
    /// Raw pointer to the forward button.
    pub forward_button: id,
    /// Raw pointer to the content area NSView that holds the active webview.
    pub content_area: id,
    /// Raw pointer to the tab bar NSView (between toolbar and content).
    pub tab_bar_view: id,
    /// Raw pointer to WKWebsiteDataStore for per-project session isolation.
    pub data_store: id,
}

unsafe impl Send for BrowserWindow {}

impl BrowserWindow {
    /// Returns a reference to the currently active tab, if any.
    pub fn active_tab(&self) -> Option<&BrowserTab> {
        self.tabs.get(self.active_tab)
    }

    /// Returns a mutable reference to the currently active tab, if any.
    pub fn active_tab_mut(&mut self) -> Option<&mut BrowserTab> {
        self.tabs.get_mut(self.active_tab)
    }
}

/// Global browser state — accessible from both Tauri commands and Cocoa callbacks.
pub static BROWSER_STATE: LazyLock<Mutex<BrowserStateInner>> =
    LazyLock::new(|| Mutex::new(BrowserStateInner::new()));

pub struct BrowserStateInner {
    /// Map from window label to BrowserWindow.
    pub windows: HashMap<String, BrowserWindow>,
    /// Tauri app handle for emitting events (set once during first window creation).
    pub app_handle: Option<tauri::AppHandle>,
}

impl BrowserStateInner {
    pub fn new() -> Self {
        Self {
            windows: HashMap::new(),
            app_handle: None,
        }
    }
}
