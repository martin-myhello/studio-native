use tauri::{
    menu::{Menu, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder},
    App, Emitter, Manager,
};
use serde_json::json;

pub fn setup_menu(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    // --- App (Studio) menu ---
    let about = PredefinedMenuItem::about(app, Some("About Studio"), None)?;
    let settings = MenuItemBuilder::with_id("settings", "Settings...")
        .accelerator("CmdOrCtrl+,")
        .build(app)?;
    let hide = PredefinedMenuItem::hide(app, None)?;
    let hide_others = PredefinedMenuItem::hide_others(app, None)?;
    let show_all = PredefinedMenuItem::show_all(app, None)?;
    let quit = PredefinedMenuItem::quit(app, None)?;
    let check_update = MenuItemBuilder::with_id("check_update", "Check for Updates...")
        .build(app)?;
    let app_menu = SubmenuBuilder::new(app, "Studio")
        .item(&about)
        .item(&check_update)
        .separator()
        .item(&settings)
        .separator()
        .item(&hide)
        .item(&hide_others)
        .item(&show_all)
        .separator()
        .item(&quit)
        .build()?;

    // --- File menu ---
    let new_window = MenuItemBuilder::with_id("new_window", "New Window")
        .accelerator("CmdOrCtrl+N")
        .build(app)?;
    let new_tab = MenuItemBuilder::with_id("new_tab", "New Tab")
        .accelerator("CmdOrCtrl+T")
        .build(app)?;
    let close_tab = MenuItemBuilder::with_id("close_tab", "Close Tab")
        .accelerator("CmdOrCtrl+W")
        .build(app)?;

    let file_menu = SubmenuBuilder::new(app, "File")
        .item(&new_window)
        .item(&new_tab)
        .separator()
        .item(&close_tab)
        .build()?;

    // --- Edit menu ---
    let undo = PredefinedMenuItem::undo(app, None)?;
    let redo = PredefinedMenuItem::redo(app, None)?;
    let cut = PredefinedMenuItem::cut(app, None)?;
    let copy = PredefinedMenuItem::copy(app, None)?;
    let paste = PredefinedMenuItem::paste(app, None)?;
    let select_all = PredefinedMenuItem::select_all(app, None)?;

    let edit_menu = SubmenuBuilder::new(app, "Edit")
        .item(&undo)
        .item(&redo)
        .separator()
        .item(&cut)
        .item(&copy)
        .item(&paste)
        .item(&select_all)
        .build()?;

    // --- View menu ---
    let reload = MenuItemBuilder::with_id("reload", "Reload")
        .accelerator("CmdOrCtrl+R")
        .build(app)?;
    let force_reload = MenuItemBuilder::with_id("force_reload", "Force Reload")
        .accelerator("CmdOrCtrl+Shift+R")
        .build(app)?;
    let actual_size = MenuItemBuilder::with_id("actual_size", "Actual Size")
        .accelerator("CmdOrCtrl+0")
        .build(app)?;
    let zoom_in = MenuItemBuilder::with_id("zoom_in", "Zoom In")
        .accelerator("CmdOrCtrl+=")
        .build(app)?;
    let zoom_out = MenuItemBuilder::with_id("zoom_out", "Zoom Out")
        .accelerator("CmdOrCtrl+-")
        .build(app)?;

    let view_menu = SubmenuBuilder::new(app, "View")
        .item(&reload)
        .item(&force_reload)
        .separator()
        .item(&actual_size)
        .item(&zoom_in)
        .item(&zoom_out)
        .build()?;

    // --- Go menu (navigation) ---
    let go_files = MenuItemBuilder::with_id("go_files", "Files")
        .accelerator("CmdOrCtrl+1")
        .build(app)?;
    let go_progress = MenuItemBuilder::with_id("go_progress", "Progress")
        .accelerator("CmdOrCtrl+2")
        .build(app)?;
    let go_studio = MenuItemBuilder::with_id("go_studio", "Studio")
        .accelerator("CmdOrCtrl+3")
        .build(app)?;
    let go_timeline = MenuItemBuilder::with_id("go_timeline", "Timeline")
        .accelerator("CmdOrCtrl+4")
        .build(app)?;
    let search = MenuItemBuilder::with_id("search", "Search")
        .accelerator("CmdOrCtrl+K")
        .build(app)?;

    let go_menu = SubmenuBuilder::new(app, "Go")
        .item(&go_files)
        .item(&go_progress)
        .item(&go_studio)
        .item(&go_timeline)
        .separator()
        .item(&search)
        .build()?;

    // --- Window menu ---
    let minimize = PredefinedMenuItem::minimize(app, None)?;
    let zoom = PredefinedMenuItem::maximize(app, None)?;
    let fullscreen = PredefinedMenuItem::fullscreen(app, None)?;
    let next_tab = MenuItemBuilder::with_id("next_tab", "Show Next Tab")
        .accelerator("CmdOrCtrl+Shift+]")
        .build(app)?;
    let prev_tab = MenuItemBuilder::with_id("prev_tab", "Show Previous Tab")
        .accelerator("CmdOrCtrl+Shift+[")
        .build(app)?;

    let window_menu = SubmenuBuilder::new(app, "Window")
        .item(&minimize)
        .item(&zoom)
        .item(&fullscreen)
        .separator()
        .item(&next_tab)
        .item(&prev_tab)
        .build()?;

    // Build final menu
    let menu = Menu::with_items(
        app,
        &[
            &app_menu,
            &file_menu,
            &edit_menu,
            &view_menu,
            &go_menu,
            &window_menu,
        ],
    )?;
    app.set_menu(menu)?;

    // Handle menu events
    let app_handle = app.handle().clone();
    app.on_menu_event(move |_app, event| {
        let id = event.id().0.as_str();
        match id {
            // Check for updates
            "check_update" => {
                let handle = app_handle.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = crate::updater::check_for_updates(handle).await;
                });
            }

            // Navigate actions → emit to webview
            "settings" => emit_menu_action(&app_handle, "navigate", Some("settings")),
            "go_files" => emit_menu_action(&app_handle, "navigate", Some("files")),
            "go_progress" => emit_menu_action(&app_handle, "navigate", Some("progress")),
            "go_studio" => emit_menu_action(&app_handle, "navigate", Some("studio")),
            "go_timeline" => emit_menu_action(&app_handle, "navigate", Some("timeline")),
            "search" => emit_menu_action(&app_handle, "search", None),

            // New window
            "new_window" => {
                let _ = crate::open_new_window(app_handle.clone());
            }

            // View actions → direct webview control
            "reload" => {
                if let Some(w) = app_handle.get_webview_window("main") {
                    let _ = w.eval("window.location.reload()");
                }
            }
            "force_reload" => {
                if let Some(w) = app_handle.get_webview_window("main") {
                    let _ = w.eval("window.location.reload()");
                }
            }
            "actual_size" => {
                if let Some(w) = app_handle.get_webview_window("main") {
                    let _ = w.eval("document.body.style.zoom = '1'");
                }
            }
            "zoom_in" => {
                if let Some(w) = app_handle.get_webview_window("main") {
                    let _ = w.eval(
                        "document.body.style.zoom = (parseFloat(document.body.style.zoom || '1') + 0.1).toString()",
                    );
                }
            }
            "zoom_out" => {
                if let Some(w) = app_handle.get_webview_window("main") {
                    let _ = w.eval(
                        "document.body.style.zoom = Math.max(0.5, parseFloat(document.body.style.zoom || '1') - 0.1).toString()",
                    );
                }
            }

            // Browser tab shortcuts
            "new_tab" => {
                #[cfg(target_os = "macos")]
                if let Some(label) = get_focused_browser_label(&app_handle) {
                    crate::browser::tabs::create_new_tab(&label, None);
                }
            }
            "close_tab" => {
                #[cfg(target_os = "macos")]
                {
                    if let Some(label) = get_focused_browser_label(&app_handle) {
                        let state = crate::browser::state::BROWSER_STATE.lock().unwrap();
                        if let Some(window) = state.windows.get(&label) {
                            let idx = window.active_tab;
                            let tab_count = window.tabs.len();
                            drop(state);
                            if tab_count <= 1 {
                                // Close the browser window
                                if let Some(w) = app_handle.get_window(&label) {
                                    let _ = w.close();
                                }
                            } else {
                                crate::browser::tabs::close_tab(&label, idx);
                            }
                        }
                    } else {
                        // Not a browser window — close the focused window
                        if let Some(w) = app_handle.get_focused_window() {
                            let _ = w.close();
                        }
                    }
                }
                #[cfg(not(target_os = "macos"))]
                {
                    if let Some(w) = app_handle.get_focused_window() {
                        let _ = w.close();
                    }
                }
            }
            "next_tab" => {
                #[cfg(target_os = "macos")]
                if let Some(label) = get_focused_browser_label(&app_handle) {
                    let state = crate::browser::state::BROWSER_STATE.lock().unwrap();
                    if let Some(window) = state.windows.get(&label) {
                        let next = (window.active_tab + 1) % window.tabs.len();
                        drop(state);
                        crate::browser::tabs::switch_tab(&label, next);
                    }
                }
            }
            "prev_tab" => {
                #[cfg(target_os = "macos")]
                if let Some(label) = get_focused_browser_label(&app_handle) {
                    let state = crate::browser::state::BROWSER_STATE.lock().unwrap();
                    if let Some(window) = state.windows.get(&label) {
                        let prev = if window.active_tab == 0 {
                            window.tabs.len() - 1
                        } else {
                            window.active_tab - 1
                        };
                        drop(state);
                        crate::browser::tabs::switch_tab(&label, prev);
                    }
                }
            }

            _ => {}
        }
    });

    Ok(())
}

/// Returns the label of the focused browser window, if any.
fn get_focused_browser_label(app: &tauri::AppHandle) -> Option<String> {
    let focused = app.get_focused_window()?;
    let label = focused.label().to_string();
    if label.starts_with("browser-") {
        Some(label)
    } else {
        None
    }
}

fn emit_menu_action(app: &tauri::AppHandle, action: &str, tab: Option<&str>) {
    let payload = if let Some(tab) = tab {
        json!({ "action": action, "tab": tab })
    } else {
        json!({ "action": action })
    };

    // Emit to all windows
    let _ = app.emit("menu-action", payload);
}
