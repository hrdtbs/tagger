mod tagger;
mod model_manager;

use base64::Engine;
use screenshots::Screen;
use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Emitter, State, Manager, // Added Manager trait
    WebviewWindowBuilder, WebviewUrl, PhysicalPosition,
};
use tauri_plugin_global_shortcut::{Code, Modifiers, ShortcutState, Shortcut};
use crate::tagger::Tagger;
use image::{ImageEncoder, RgbaImage, DynamicImage}; // Import ImageEncoder trait
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

fn resolve_model_path(app: &tauri::AppHandle, path_str: &str) -> std::path::PathBuf {
    let path = std::path::Path::new(path_str);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        // Default to AppLocalData for relative paths
        match app.path().resolve(path_str, BaseDirectory::AppLocalData) {
            Ok(p) => p,
            Err(_) => path.to_path_buf() // Fallback
        }
    }
}

struct AppState {
    // Store captured screens individually: (Screen Info, Image Buffer)
    captures: Mutex<Vec<(Screen, RgbaImage)>>,
    tagger: Mutex<Option<Tagger>>,
    config: Mutex<AppConfig>,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

async fn capture_all_screens(app: &tauri::AppHandle) -> Result<(), String> {
    // Hide main window
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
    // Give some time for the window to hide
    std::thread::sleep(std::time::Duration::from_millis(200));

    let screens = Screen::all();
    let mut captured_data = Vec::new();

    for screen in screens {
        let image = screen.capture().ok_or("Failed to capture screen".to_string())?;
        let img_width = image.width();
        let img_height = image.height();
        let img_buffer = image.buffer();

        let rgba = RgbaImage::from_raw(img_width, img_height, img_buffer.clone())
            .ok_or("Failed to create image buffer")?;

        captured_data.push((screen, rgba));
    }

    let state = app.state::<AppState>();
    *state.captures.lock().map_err(|e| e.to_string())? = captured_data;

    Ok(())
}

fn create_overlay_windows(app: &tauri::AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();
    let captures = state.captures.lock().map_err(|e| e.to_string())?;

    for (i, (screen, _)) in captures.iter().enumerate() {
        let label = format!("overlay-{}", i);

        let window = if let Some(w) = app.get_webview_window(&label) {
            w
        } else {
            WebviewWindowBuilder::new(
                app,
                &label,
                WebviewUrl::App("index.html".into()),
            )
            .title("Overlay")
            // .transparent(true) // Removed due to compilation error
            .decorations(false)
            .visible(false) // Start hidden
            .always_on_top(true)
            .skip_taskbar(true)
            .build()
            .map_err(|e: tauri::Error| e.to_string())?
        };

        // Position window on the correct screen
        let pos = PhysicalPosition::new(screen.x, screen.y);
        window.set_position(pos).map_err(|e: tauri::Error| e.to_string())?;

        // Ensure fullscreen
        window.set_fullscreen(true).map_err(|e: tauri::Error| e.to_string())?;

        // Ensure always on top and hidden from taskbar before showing
        window.set_always_on_top(true).map_err(|e: tauri::Error| e.to_string())?;
        window.set_skip_taskbar(true).map_err(|e: tauri::Error| e.to_string())?;

        window.show().map_err(|e: tauri::Error| e.to_string())?;
        window.set_focus().map_err(|e: tauri::Error| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn capture_screen(_app: tauri::AppHandle, _state: State<'_, AppState>) -> Result<String, String> {
    // Deprecated in favor of capture_all_screens + create_overlay_windows
    Err("Deprecated".to_string())
}

#[tauri::command]
async fn get_overlay_image(state: State<'_, AppState>, screen_index: usize) -> Result<String, String> {
    let captures = state.captures.lock().map_err(|e| e.to_string())?;

    if let Some((_, image)) = captures.get(screen_index) {
        let mut buffer = Vec::new();
        image::codecs::png::PngEncoder::new(&mut buffer)
            .write_image(
                image,
                image.width(),
                image.height(),
                image::ColorType::Rgba8,
            )
            .map_err(|e| e.to_string())?;

        let b64 = base64::engine::general_purpose::STANDARD.encode(&buffer);
        Ok(format!("data:image/png;base64,{}", b64))
    } else {
        Err("Screen index out of bounds".to_string())
    }
}

#[tauri::command]
async fn close_all_overlays(app: tauri::AppHandle) -> Result<(), String> {
    for window in app.webview_windows().values() {
        if window.label().starts_with("overlay-") {
            let _ = window.close();
        }
    }

    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }

    Ok(())
}

#[tauri::command]
async fn process_selection(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    screen_index: usize,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
) -> Result<String, String> {
    let captures = state.captures.lock().map_err(|e| e.to_string())?;
    let (_, img) = captures.get(screen_index).ok_or("Screen index out of bounds")?;

    // Convert to DynamicImage to use crop_imm
    let dyn_img = DynamicImage::ImageRgba8(img.clone());
    let cropped = dyn_img.crop_imm(x, y, w, h);

    let tags = run_inference(&state, cropped)?;

    // Copy to clipboard
    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set_text(tags.clone()).map_err(|e| e.to_string())?;

    // Emit event for Settings window
    let _ = app.emit("tag-generated", tags.clone());

    // Show main window to display result
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }

    Ok(tags)
}

// Placeholder for future logic to reuse tagger code if needed
fn run_inference(
    state: &State<'_, AppState>,
    cropped: DynamicImage,
) -> Result<String, String> {
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
    Ok(tags)
}


#[tauri::command]
async fn check_model_exists(app: tauri::AppHandle, path_str: String) -> Result<bool, String> {
    let path = resolve_model_path(&app, &path_str);
    Ok(model_manager::check_file_exists(&path))
}

#[tauri::command]
async fn download_new_model(app: tauri::AppHandle, url: String, path_str: String) -> Result<(), String> {
    let path = resolve_model_path(&app, &path_str);
    model_manager::download_file(&app, &url, &path).await?;
    let _ = app.emit("model-download-finished", ());
    Ok(())
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState {
            captures: Mutex::new(Vec::new()),
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
                            let app_handle = app.clone();
                            tauri::async_runtime::spawn(async move {
                                if let Err(e) = capture_all_screens(&app_handle).await {
                                    eprintln!("Failed to capture screens: {}", e);
                                    return;
                                }
                                if let Err(e) = create_overlay_windows(&app_handle) {
                                    eprintln!("Failed to create overlay windows: {}", e);
                                }
                            });
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
            *state.config.lock().unwrap() = config.clone();

            let app_handle = app.handle().clone();
            let model_path_str = config.model_path.clone();
            let tags_path_str = config.tags_path.clone();

            tauri::async_runtime::spawn(async move {
                let model_path = resolve_model_path(&app_handle, &model_path_str);
                let tags_path = resolve_model_path(&app_handle, &tags_path_str);

                // Ensure models exist
                if let Err(e) = model_manager::check_and_download_models(&app_handle, &model_path, &tags_path).await {
                    eprintln!("Failed to download models: {}", e);
                     let _ = app_handle.emit("model-download-error", e.clone());
                     use tauri_plugin_notification::NotificationExt;
                     let _ = app_handle.notification().builder()
                        .title("OmniTagger Error")
                        .body(format!("Failed to download models: {}", e))
                        .show();
                     return;
                }

                // Load Tagger
                match Tagger::new(model_path.to_str().unwrap_or(&model_path_str), tags_path.to_str().unwrap_or(&tags_path_str)) {
                    Ok(tagger) => {
                        let state = app_handle.state::<AppState>();
                        *state.tagger.lock().unwrap() = Some(tagger);
                        println!("Tagger loaded successfully");
                        let _ = app_handle.emit("tagger-loaded", ());
                    }
                    Err(e) => {
                        eprintln!("Failed to load tagger: {}", e);
                        use tauri_plugin_notification::NotificationExt;
                         let _ = app_handle.notification().builder()
                            .title("OmniTagger Error")
                            .body(format!("Failed to load model: {}", e))
                            .show();
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            capture_screen,
            process_selection,
            get_config,
            set_config,
            get_overlay_image,
            close_all_overlays,
            check_model_exists,
            download_new_model
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
