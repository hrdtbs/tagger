mod tagger;

use base64::Engine;
use screenshots::Screen;
use std::sync::Mutex;
use std::time::Instant;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Emitter, Manager, State,
};
use tauri_plugin_global_shortcut::{Code, Modifiers, ShortcutState, Shortcut};
use crate::tagger::Tagger;
use image::{ImageEncoder, RgbaImage, imageops}; // Import ImageEncoder trait
use serde::{Deserialize, Serialize};
use std::fs;
use tauri::path::BaseDirectory;

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

fn load_config(app: &tauri::AppHandle) -> AppConfig {
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

fn save_config(app: &tauri::AppHandle, config: &AppConfig) -> Result<(), String> {
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

struct AppState {
    last_capture: Mutex<Option<Vec<u8>>>,
    tagger: Mutex<Option<Tagger>>,
    config: Mutex<AppConfig>,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn capture_screen(app: tauri::AppHandle, state: State<'_, AppState>) -> Result<String, String> {
    // Hide window to avoid capturing the overlay itself
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
    // Give some time for the window to hide and compositor to update
    std::thread::sleep(std::time::Duration::from_millis(200));

    let start = Instant::now();

    let screens = Screen::all();
    let mut captures = Vec::new();

    for screen in screens {
        let image = screen
            .capture()
            .ok_or("Failed to capture screen".to_string())?;
        captures.push((screen, image));
    }

    if captures.is_empty() {
        // If capture failed, ensure window is shown again (though frontend handles errors usually)
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.show();
        }
        return Err("No screens found".to_string());
    }

    // Calculate bounding box
    let min_x = captures.iter().map(|(s, _)| s.x).min().unwrap_or(0);
    let min_y = captures.iter().map(|(s, _)| s.y).min().unwrap_or(0);
    let max_x = captures
        .iter()
        .map(|(s, i)| s.x + i.width() as i32)
        .max()
        .unwrap_or(0);
    let max_y = captures
        .iter()
        .map(|(s, i)| s.y + i.height() as i32)
        .max()
        .unwrap_or(0);

    let total_width = (max_x - min_x) as u32;
    let total_height = (max_y - min_y) as u32;

    // Stitch images
    let mut stitched = RgbaImage::new(total_width, total_height);

    for (screen, image) in captures {
        let img_width = image.width();
        let img_height = image.height();
        let img_buffer = image.buffer();

        // Create RgbaImage from capture (assuming RGBA8)
        let sub_img = RgbaImage::from_raw(img_width, img_height, img_buffer.clone())
            .ok_or("Failed to create image buffer")?;

        let x_offset = (screen.x - min_x) as i64;
        let y_offset = (screen.y - min_y) as i64;

        imageops::overlay(&mut stitched, &sub_img, x_offset, y_offset);
    }

    // Encode to PNG
    let mut buffer = Vec::new();
    image::codecs::png::PngEncoder::new(&mut buffer)
        .write_image(
            &stitched,
            total_width,
            total_height,
            image::ColorType::Rgba8,
        )
        .map_err(|e| e.to_string())?;

    *state.last_capture.lock().map_err(|e| e.to_string())? = Some(buffer.clone());

    let b64 = base64::engine::general_purpose::STANDARD.encode(&buffer);

    println!("Screen captured in {:?}", start.elapsed());

    // Show window again
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }

    Ok(format!("data:image/png;base64,{}", b64))
}

#[tauri::command]
async fn process_selection(
    state: State<'_, AppState>,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
) -> Result<String, String> {
    let guard = state.last_capture.lock().map_err(|e| e.to_string())?;
    let data = guard.as_ref().ok_or("No capture found")?;

    let img = image::load_from_memory(data).map_err(|e| e.to_string())?;
    let cropped = img.crop_imm(x, y, w, h);

    // Tagger inference
    let mut tagger_guard = state.tagger.lock().map_err(|e| e.to_string())?;
    let config = state.config.lock().map_err(|e| e.to_string())?;

    let tags = if let Some(tagger) = tagger_guard.as_mut() {
        // Run inference
        let results = tagger.infer(&cropped, config.threshold).map_err(|e| e.to_string())?;

        // Filter and format
        let mut filtered: Vec<String> = results.into_iter()
            .map(|(t, _)| t)
            .filter(|t| !config.exclusion_list.contains(t))
            .collect();

        if config.use_underscore {
            filtered = filtered.iter().map(|t| t.replace(" ", "_")).collect();
        } else {
             filtered = filtered.iter().map(|t| t.replace("_", " ")).collect();
        }

        filtered.join(", ")
    } else {
        println!("Tagger not loaded, using fallback.");
        "1girl, solo, fallback_tag".to_string()
    };

    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set_text(tags.clone()).map_err(|e| e.to_string())?;

    Ok(tags)
}

#[tauri::command]
fn get_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    state.config.lock().map_err(|e| e.to_string()).map(|c| c.clone())
}

#[tauri::command]
async fn set_config(app: tauri::AppHandle, state: State<'_, AppState>, config: AppConfig) -> Result<(), String> {
    let mut config_guard = state.config.lock().map_err(|e| e.to_string())?;

    let model_changed = config_guard.model_path != config.model_path || config_guard.tags_path != config.tags_path;

    *config_guard = config.clone();

    // Save to disk
    save_config(&app, &config)?;

    if model_changed {
         // Reload Tagger
         let mut tagger_guard = state.tagger.lock().map_err(|e| e.to_string())?;
         match Tagger::new(&config.model_path, &config.tags_path) {
            Ok(tagger) => {
                *tagger_guard = Some(tagger);
                println!("Tagger reloaded successfully from {}", config.model_path);
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState {
            last_capture: Mutex::new(None),
            tagger: Mutex::new(None),
            config: Mutex::new(AppConfig::default()),
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_shortcut(Shortcut::new(Some(Modifiers::ALT | Modifiers::SHIFT), Code::KeyT))
                .unwrap()
                .with_handler(|app, shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                         if shortcut.matches(Modifiers::ALT | Modifiers::SHIFT, Code::KeyT) {
                            println!("Global hotkey pressed!");
                            // Do not show window immediately. Wait for capture_screen to handle it.
                            // if let Some(window) = app.get_webview_window("main") {
                            //     let _ = window.show();
                            //     let _ = window.set_focus();
                            // }
                            let _ = app.emit("show-overlay", ());
                        }
                    }
                })
                .build(),
        )
        .setup(|app| {
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let settings_i = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&settings_i, &quit_i])?;

            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    "settings" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                })
                .build(app)?;

            // Initialize Config and Tagger
            let state = app.state::<AppState>();
            let config = load_config(app.handle());

            // Reload tagger based on config
            match Tagger::new(&config.model_path, &config.tags_path) {
                Ok(tagger) => {
                    *state.tagger.lock().unwrap() = Some(tagger);
                    println!("Tagger loaded successfully from {}", config.model_path);
                }
                Err(e) => {
                    println!("Failed to load tagger: {}. (Expected if models not present)", e);
                }
            }
            *state.config.lock().unwrap() = config;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet, capture_screen, process_selection, get_config, set_config])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
