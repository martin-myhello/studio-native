mod models;

#[cfg(target_os = "ios")]
mod mobile;

use tauri::{
    plugin::{Builder, TauriPlugin},
    Runtime,
};

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_pencilkit);

#[cfg(target_os = "ios")]
mod commands {
    use super::models::*;
    use tauri::{Manager, Runtime};

    #[tauri::command]
    pub fn is_available<R: Runtime>(app: tauri::AppHandle<R>) -> Result<bool, String> {
        app.state::<super::mobile::PencilKit<R>>()
            .is_available()
            .map(|r| r.available)
    }

    #[tauri::command]
    pub fn show<R: Runtime>(app: tauri::AppHandle<R>) -> Result<(), String> {
        app.state::<super::mobile::PencilKit<R>>().show()
    }

    #[tauri::command]
    pub fn hide<R: Runtime>(app: tauri::AppHandle<R>) -> Result<(), String> {
        app.state::<super::mobile::PencilKit<R>>().hide()
    }

    #[tauri::command]
    pub fn clear<R: Runtime>(app: tauri::AppHandle<R>) -> Result<(), String> {
        app.state::<super::mobile::PencilKit<R>>().clear()
    }

    #[tauri::command]
    pub fn get_drawing<R: Runtime>(
        app: tauri::AppHandle<R>,
    ) -> Result<GetDrawingResponse, String> {
        app.state::<super::mobile::PencilKit<R>>().get_drawing()
    }

    #[tauri::command]
    pub fn set_drawing<R: Runtime>(
        app: tauri::AppHandle<R>,
        data: String,
    ) -> Result<(), String> {
        app.state::<super::mobile::PencilKit<R>>().set_drawing(data)
    }

    #[tauri::command]
    pub fn get_image<R: Runtime>(
        app: tauri::AppHandle<R>,
    ) -> Result<GetImageResponse, String> {
        app.state::<super::mobile::PencilKit<R>>().get_image()
    }

    #[tauri::command]
    pub fn set_tool<R: Runtime>(
        app: tauri::AppHandle<R>,
        tool: String,
    ) -> Result<(), String> {
        app.state::<super::mobile::PencilKit<R>>().set_tool(tool)
    }
}

#[cfg(not(target_os = "ios"))]
mod commands {
    use super::models::*;

    #[tauri::command]
    pub fn is_available() -> Result<bool, String> {
        Ok(false)
    }

    #[tauri::command]
    pub fn show() -> Result<(), String> {
        Ok(())
    }

    #[tauri::command]
    pub fn hide() -> Result<(), String> {
        Ok(())
    }

    #[tauri::command]
    pub fn clear() -> Result<(), String> {
        Ok(())
    }

    #[tauri::command]
    pub fn get_drawing() -> Result<GetDrawingResponse, String> {
        Ok(GetDrawingResponse { data: None })
    }

    #[tauri::command]
    pub fn set_drawing(_data: String) -> Result<(), String> {
        Ok(())
    }

    #[tauri::command]
    pub fn get_image() -> Result<GetImageResponse, String> {
        Ok(GetImageResponse {
            image: None,
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        })
    }

    #[tauri::command]
    pub fn set_tool(_tool: String) -> Result<(), String> {
        Ok(())
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("pencilkit")
        .invoke_handler(tauri::generate_handler![
            commands::is_available,
            commands::show,
            commands::hide,
            commands::clear,
            commands::get_drawing,
            commands::set_drawing,
            commands::get_image,
            commands::set_tool,
        ])
        .setup(|app, api| {
            #[cfg(target_os = "ios")]
            {
                let handle = api.register_ios_plugin(init_plugin_pencilkit)?;
                app.manage(mobile::PencilKit(handle));
            }
            let _ = (app, api);
            Ok(())
        })
        .build()
}
