use serde_json::Value;
use tauri::{plugin::PluginHandle, Runtime};

use crate::models::*;

pub struct PencilKit<R: Runtime>(pub PluginHandle<R>);

impl<R: Runtime> PencilKit<R> {
    pub fn is_available(&self) -> Result<IsAvailableResponse, String> {
        self.0
            .run_mobile_plugin::<IsAvailableResponse>("isAvailable", ())
            .map_err(|e| e.to_string())
    }

    pub fn show(&self) -> Result<(), String> {
        self.0
            .run_mobile_plugin::<Value>("show", ())
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    pub fn hide(&self) -> Result<(), String> {
        self.0
            .run_mobile_plugin::<Value>("hide", ())
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    pub fn clear(&self) -> Result<(), String> {
        self.0
            .run_mobile_plugin::<Value>("clear", ())
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    pub fn get_drawing(&self) -> Result<GetDrawingResponse, String> {
        self.0
            .run_mobile_plugin::<GetDrawingResponse>("getDrawing", ())
            .map_err(|e| e.to_string())
    }

    pub fn set_drawing(&self, data: String) -> Result<(), String> {
        self.0
            .run_mobile_plugin::<Value>("setDrawing", SetDrawingArgs { data })
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    pub fn get_image(&self) -> Result<GetImageResponse, String> {
        self.0
            .run_mobile_plugin::<GetImageResponse>("getImage", ())
            .map_err(|e| e.to_string())
    }

    pub fn set_tool(&self, tool: String) -> Result<(), String> {
        self.0
            .run_mobile_plugin::<Value>("setTool", SetToolArgs { tool })
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
}
