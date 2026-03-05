use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct IsAvailableResponse {
    pub available: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetDrawingArgs {
    pub data: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetToolArgs {
    pub tool: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetDrawingResponse {
    pub data: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetImageResponse {
    pub image: Option<String>,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}
