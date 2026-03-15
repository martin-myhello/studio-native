#[cfg(target_os = "macos")]
mod browser;
mod menu;
mod updater;

use tauri::Manager;

#[cfg(target_os = "macos")]
fn setup_macos_window(window: &tauri::WebviewWindow) {
    use cocoa::appkit::{NSColor, NSWindow};
    use cocoa::base::{id, nil};

    if let Ok(ns_window) = window.ns_window() {
        let ns_window = ns_window as id;
        unsafe {
            let bg_color = NSColor::colorWithSRGBRed_green_blue_alpha_(
                nil,
                1.0,
                1.0,
                1.0,
                1.0,
            );
            ns_window.setBackgroundColor_(bg_color);
        }
    }
}

const STUDIO_URL: &str = "https://studio.myhello.io";

/// Returns true if the URL is an allowed domain (myhello.io, supabase, googleapis)
fn is_allowed_url(url: &str) -> bool {
    let allowed_patterns = [
        "http://localhost",
        "https://studio.myhello.io",
        "https://myhello.io",
        "https://www.myhello.io",
        // Supabase for API calls
        ".supabase.co",
        // Google APIs for calendar integration
        ".googleapis.com",
        // Google auth
        "accounts.google.com",
    ];
    for pattern in &allowed_patterns {
        if url.starts_with(pattern) || url.contains(pattern) {
            return true;
        }
    }
    false
}

#[tauri::command]
fn open_external(url: String) -> Result<(), String> {
    open::that(&url).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_platform() -> String {
    #[cfg(target_os = "macos")]
    {
        "macos".to_string()
    }
    #[cfg(target_os = "ios")]
    {
        // Distinguish iPhone from iPad using screen size heuristic
        // On iOS, UIDevice info isn't directly available from Rust,
        // so the web app's JS detection handles iPhone vs iPad
        "ios".to_string()
    }
    #[cfg(not(any(target_os = "macos", target_os = "ios")))]
    {
        "unknown".to_string()
    }
}

#[tauri::command]
fn open_new_window(app: tauri::AppHandle) -> Result<(), String> {
    let label = format!(
        "studio-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    );

    let _window = tauri::WebviewWindowBuilder::new(
        &app,
        &label,
        tauri::WebviewUrl::External(STUDIO_URL.parse().unwrap()),
    )
    .title("Studio")
    .inner_size(1280.0, 860.0)
    .min_inner_size(380.0, 380.0)
    .decorations(true)
    .hidden_title(true)
    .title_bar_style(tauri::TitleBarStyle::Overlay)
    .traffic_light_position(tauri::Position::Logical(tauri::LogicalPosition::new(16.0, 24.0)))
    .on_navigation(|url| {
        // Allow navigation to myhello.io and partner domains
        is_allowed_url(url.as_str())
    })
    .build()
    .map_err(|e| e.to_string())?;

    #[cfg(target_os = "macos")]
    setup_macos_window(&_window);

    Ok(())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_window_state::Builder::new().with_filename("window-state.json").build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_pencilkit::init())
        .invoke_handler(tauri::generate_handler![
            open_new_window,
            open_external,
            get_platform,
            updater::check_for_updates,
            updater::download_update,
            updater::install_update,
            #[cfg(target_os = "macos")]
            browser::commands::open_browser_window,
            #[cfg(target_os = "macos")]
            browser::commands::browser_navigate,
            #[cfg(target_os = "macos")]
            browser::commands::browser_go_back,
            #[cfg(target_os = "macos")]
            browser::commands::browser_go_forward,
            #[cfg(target_os = "macos")]
            browser::commands::browser_get_page_info,
            #[cfg(target_os = "macos")]
            browser::commands::browser_new_tab,
            #[cfg(target_os = "macos")]
            browser::commands::browser_close_tab,
            #[cfg(target_os = "macos")]
            browser::commands::browser_switch_tab,
        ])
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();

            #[cfg(target_os = "macos")]
            setup_macos_window(&window);

            menu::setup_menu(app)?;

            // Start background update checker
            updater::spawn_update_checker(app.handle());

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
