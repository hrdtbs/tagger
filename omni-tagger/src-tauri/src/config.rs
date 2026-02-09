use serde::{Deserialize, Serialize};
use std::fs;
use tauri::{State, Manager, AppHandle, path::BaseDirectory};
use crate::state::AppState;
use crate::tagger::Tagger;
use crate::model_manager;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub model_path: String,
    pub tags_path: String,
    pub threshold: f32,
    pub use_underscore: bool,
    pub exclusion_list: Vec<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            model_path: "models/model.onnx".to_string(),
            tags_path: "models/tags.csv".to_string(),
            threshold: 0.35,
            use_underscore: false,
            exclusion_list: Vec::new(),
        }
    }
}

pub fn load_config(app: &AppHandle) -> AppConfig {
    if let Ok(path) = app.path().resolve("config.json", BaseDirectory::AppConfig) {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(config) = serde_json::from_str(&content) {
                    return config;
                }
            }
        }
    }
    AppConfig::default()
}

pub fn save_config(app: &AppHandle, config: &AppConfig) -> Result<(), String> {
    let path = app
        .path()
        .resolve("config.json", BaseDirectory::AppConfig)
        .map_err(|e| e.to_string())?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let content = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn resolve_model_path(app: &AppHandle, path_str: &str) -> std::path::PathBuf {
    let path = std::path::Path::new(path_str);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        match app.path().resolve(path_str, BaseDirectory::AppLocalData) {
            Ok(p) => p,
            Err(_) => path.to_path_buf() // Fallback
        }
    }
}

#[tauri::command]
pub fn get_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    state.config.lock().map_err(|e| e.to_string()).map(|c| c.clone())
}

#[tauri::command]
pub async fn set_config(app: AppHandle, state: State<'_, AppState>, config: AppConfig) -> Result<(), String> {
    let mut config_guard = state.config.lock().map_err(|e| e.to_string())?;

    let model_changed = config_guard.model_path != config.model_path || config_guard.tags_path != config.tags_path;

    *config_guard = config.clone();
    save_config(&app, &config)?;

    if model_changed {
         let mut tagger_guard = state.tagger.lock().map_err(|e| e.to_string())?;
         let model_path = resolve_model_path(&app, &config.model_path);
         let tags_path = resolve_model_path(&app, &config.tags_path);

         match Tagger::new(model_path.to_str().unwrap_or(&config.model_path), tags_path.to_str().unwrap_or(&config.tags_path)) {
            Ok(tagger) => {
                *tagger_guard = Some(tagger);
                println!("Tagger reloaded successfully from {:?}", model_path);
            }
            Err(e) => {
                println!("Failed to reload tagger: {}", e);
                *tagger_guard = None;
                return Err(format!("Failed to reload tagger: {}", e));
            }
         }
    }
    Ok(())
}

#[tauri::command]
pub async fn check_model_exists(app: AppHandle, path_str: String) -> Result<bool, String> {
    let path = resolve_model_path(&app, &path_str);
    Ok(model_manager::check_file_exists(&path))
}

#[tauri::command]
pub async fn download_new_model(app: AppHandle, url: String, path_str: String) -> Result<(), String> {
    let path = resolve_model_path(&app, &path_str);
    model_manager::download_file(&app, &url, &path).await?;
    use tauri::Emitter;
    let _ = app.emit("model-download-finished", ());
    Ok(())
}
