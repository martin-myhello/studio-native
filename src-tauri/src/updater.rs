use serde::Serialize;
use tauri::Emitter;
use tauri_plugin_updater::UpdaterExt;

#[derive(Clone, Serialize)]
pub struct UpdateStatusPayload {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub percent: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

fn emit_status(app: &tauri::AppHandle, payload: UpdateStatusPayload) {
    let _ = app.emit("update-status", payload);
}

#[tauri::command]
pub async fn check_for_updates(app: tauri::AppHandle) -> Result<(), String> {
    emit_status(
        &app,
        UpdateStatusPayload {
            status: "checking".to_string(),
            version: None,
            percent: None,
            message: None,
        },
    );

    let updater = match app.updater() {
        Ok(u) => u,
        Err(e) => {
            emit_status(
                &app,
                UpdateStatusPayload {
                    status: "error".to_string(),
                    version: None,
                    percent: None,
                    message: Some(e.to_string()),
                },
            );
            return Ok(());
        }
    };

    match updater.check().await {
        Ok(Some(update)) => {
            emit_status(
                &app,
                UpdateStatusPayload {
                    status: "available".to_string(),
                    version: Some(update.version.clone()),
                    percent: None,
                    message: None,
                },
            );
        }
        Ok(None) => {
            emit_status(
                &app,
                UpdateStatusPayload {
                    status: "up-to-date".to_string(),
                    version: None,
                    percent: None,
                    message: None,
                },
            );
        }
        Err(e) => {
            emit_status(
                &app,
                UpdateStatusPayload {
                    status: "error".to_string(),
                    version: None,
                    percent: None,
                    message: Some(e.to_string()),
                },
            );
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn download_update(app: tauri::AppHandle) -> Result<(), String> {
    let updater = app.updater().map_err(|e| e.to_string())?;

    match updater.check().await {
        Ok(Some(update)) => {
            let version = update.version.clone();
            emit_status(
                &app,
                UpdateStatusPayload {
                    status: "downloading".to_string(),
                    version: Some(version.clone()),
                    percent: Some(0.0),
                    message: None,
                },
            );

            let app_clone = app.clone();
            let version_clone = version.clone();

            match update
                .download_and_install(
                    |chunk_length, content_length| {
                        if let Some(total) = content_length {
                            let percent = (chunk_length as f64 / total as f64) * 100.0;
                            emit_status(
                                &app_clone,
                                UpdateStatusPayload {
                                    status: "downloading".to_string(),
                                    version: Some(version_clone.clone()),
                                    percent: Some(percent),
                                    message: None,
                                },
                            );
                        }
                    },
                    || {},
                )
                .await
            {
                Ok(_) => {
                    emit_status(
                        &app,
                        UpdateStatusPayload {
                            status: "ready".to_string(),
                            version: Some(version),
                            percent: None,
                            message: None,
                        },
                    );
                }
                Err(e) => {
                    emit_status(
                        &app,
                        UpdateStatusPayload {
                            status: "error".to_string(),
                            version: None,
                            percent: None,
                            message: Some(e.to_string()),
                        },
                    );
                }
            }
        }
        Ok(None) => {
            emit_status(
                &app,
                UpdateStatusPayload {
                    status: "up-to-date".to_string(),
                    version: None,
                    percent: None,
                    message: None,
                },
            );
        }
        Err(e) => {
            emit_status(
                &app,
                UpdateStatusPayload {
                    status: "error".to_string(),
                    version: None,
                    percent: None,
                    message: Some(e.to_string()),
                },
            );
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn install_update(app: tauri::AppHandle) -> Result<(), String> {
    app.restart();
}

/// Spawn a background task that checks for updates on startup and every 4 hours
pub fn spawn_update_checker(app: &tauri::AppHandle) {
    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        // Initial check after 10 seconds
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        let _ = check_for_updates(app_handle.clone()).await;

        // Then check every 4 hours
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(4 * 60 * 60));
        loop {
            interval.tick().await;
            let _ = check_for_updates(app_handle.clone()).await;
        }
    });
}
