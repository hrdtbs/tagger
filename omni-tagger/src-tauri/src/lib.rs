mod tagger;
mod model_manager;

use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Emitter, State, Manager,
};
use crate::tagger::Tagger;
use serde::{Deserialize, Serialize};
use std::fs;
use tauri::path::BaseDirectory;
use std::process::Command;
use std::path::PathBuf;

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
        match app.path().resolve(path_str, BaseDirectory::AppLocalData) {
            Ok(p) => p,
            Err(_) => path.to_path_buf() // Fallback
        }
    }
}

struct AppState {
    tagger: Mutex<Option<Tagger>>,
    config: Mutex<AppConfig>,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
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
async fn register_context_menu(enable: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let exe_path = std::env::current_exe().map_err(|e| e.to_string())?;
        let exe_str = exe_path.to_str().ok_or("Invalid path")?;

        let command_str = format!("\"{}\" \"%1\"", exe_str);

        if enable {
            Command::new("reg")
                .args(&["add", "HKCU\\Software\\Classes\\*\\shell\\OmniTagger", "/ve", "/d", "Get Tags", "/f"])
                .output()
                .map_err(|e| format!("Failed to add registry key: {}", e))?;

            Command::new("reg")
                .args(&["add", "HKCU\\Software\\Classes\\*\\shell\\OmniTagger\\command", "/ve", "/d", &command_str, "/f"])
                .output()
                .map_err(|e| format!("Failed to add command key: {}", e))?;
        } else {
            Command::new("reg")
                .args(&["delete", "HKCU\\Software\\Classes\\*\\shell\\OmniTagger", "/f"])
                .output()
                .map_err(|e| format!("Failed to delete registry key: {}", e))?;
        }
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        Err("Context menu registration is only supported on Windows".to_string())
    }
}

#[tauri::command]
async fn register_native_host(_app: tauri::AppHandle, extension_id: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        // 1. Get exe path and derive native_host.exe path
        let exe_path = std::env::current_exe().map_err(|e| e.to_string())?;
        let exe_dir = exe_path.parent().ok_or("Invalid path")?;
        let native_host_path = exe_dir.join("native_host.exe");

        if !native_host_path.exists() {
             return Err(format!("native_host.exe not found at {:?}", native_host_path));
        }

        // 2. Create JSON Manifest
        let manifest_content = serde_json::json!({
            "name": "com.omnitagger.host",
            "description": "OmniTagger Native Messaging Host",
            "path": native_host_path.to_str().unwrap_or("native_host.exe"),
            "type": "stdio",
            "allowed_origins": [
                format!("chrome-extension://{}/", extension_id)
            ]
        });

        let manifest_path = exe_dir.join("com.omnitagger.host.json");
        let file = std::fs::File::create(&manifest_path).map_err(|e| e.to_string())?;
        serde_json::to_writer_pretty(file, &manifest_content).map_err(|e| e.to_string())?;

        // 3. Add Registry Key
        // HKCU\Software\Google\Chrome\NativeMessagingHosts\com.omnitagger.host
        Command::new("reg")
            .args(&[
                "add",
                "HKCU\\Software\\Google\\Chrome\\NativeMessagingHosts\\com.omnitagger.host",
                "/ve",
                "/d",
                manifest_path.to_str().ok_or("Invalid path")?,
                "/f"
            ])
            .output()
            .map_err(|e| format!("Failed to register native host: {}", e))?;

        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        Err("Native host registration is only supported on Windows".to_string())
    }
}


async fn process_inputs(app: &tauri::AppHandle, args: Vec<String>) -> Result<(), String> {
    if args.len() <= 1 {
        return Ok(());
    }

    // args[1] could be file path or flag
    let arg1 = args[1].clone();

    if arg1 == "--process-url" {
        if args.len() > 2 {
            let url = args[2].clone();
            process_image_url(app, url).await?;
        }
    } else if !arg1.starts_with("--") {
        let path = PathBuf::from(arg1);
        process_image_file(app, path).await?;
    }

    Ok(())
}

async fn process_image_url(app: &tauri::AppHandle, url: String) -> Result<(), String> {
    // Download image
    // Using reqwest
    let client = reqwest::Client::new();
    let resp = client.get(&url).send().await.map_err(|e| e.to_string())?;
    let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
    let img = image::load_from_memory(&bytes).map_err(|e| format!("Failed to load image from URL: {}", e))?;

    run_inference_and_notify(app, img).await
}

async fn process_image_file(app: &tauri::AppHandle, path: PathBuf) -> Result<(), String> {
    let img = image::open(&path).map_err(|e| format!("Failed to open image: {}", e))?;
    run_inference_and_notify(app, img).await
}

async fn run_inference_and_notify(app: &tauri::AppHandle, img: image::DynamicImage) -> Result<(), String> {
    let state = app.state::<AppState>();

    let config = get_config(state.clone())?;

    // Quick check if loaded
    let is_loaded = state.tagger.lock().map_err(|e| e.to_string())?.is_some();

    if !is_loaded {
        // Load it now (blocking/async mixed?)
        let model_path = resolve_model_path(app, &config.model_path);
        let tags_path = resolve_model_path(app, &config.tags_path);

        let tagger = Tagger::new(
            model_path.to_str().unwrap_or(&config.model_path),
            tags_path.to_str().unwrap_or(&config.tags_path)
        ).map_err(|e| e.to_string())?;

        *state.tagger.lock().map_err(|e| e.to_string())? = Some(tagger);
    }

    let mut tagger_guard = state.tagger.lock().map_err(|e| e.to_string())?;
    let tagger = tagger_guard.as_mut().ok_or("Tagger not available")?;

    let results = tagger.infer(&img, config.threshold).map_err(|e| e.to_string())?;

    let mut filtered: Vec<String> = results.into_iter()
        .map(|(t, _)| t)
        .filter(|t| !config.exclusion_list.contains(t))
        .collect();

    if config.use_underscore {
        filtered = filtered.iter().map(|t| t.replace(" ", "_")).collect();
    } else {
         filtered = filtered.iter().map(|t| t.replace("_", " ")).collect();
    }
    let tags_str = filtered.join(", ");

    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set_text(tags_str.clone()).map_err(|e| e.to_string())?;

    use tauri_plugin_notification::NotificationExt;
    let _ = app.notification().builder()
        .title("Tags Copied!")
        .body(&tags_str)
        .show();

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState {
            tagger: Mutex::new(None),
            config: Mutex::new(AppConfig::default()),
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            println!("Single Instance: {:?}", argv);
            let app_handle = app.clone();
            let args = argv.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = process_inputs(&app_handle, args).await {
                    eprintln!("Error processing inputs: {}", e);
                     use tauri_plugin_notification::NotificationExt;
                     let _ = app_handle.notification().builder()
                        .title("Error")
                        .body(format!("Processing failed: {}", e))
                        .show();
                }
            });
        }))
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

            let config = load_config(app.handle());
            *app.state::<AppState>().config.lock().unwrap() = config.clone();
            let app_handle = app.handle().clone();

            // Initial Arg Check (First Instance)
            let args: Vec<String> = std::env::args().collect();
            if args.len() > 1 {
                let args_clone = args.clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = process_inputs(&app_handle, args_clone).await {
                         eprintln!("Error processing inputs: {}", e);
                    }
                });
                return Ok(());
            }

            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
            }

            // Preload Tagger in background for GUI usage
            let model_path_str = config.model_path.clone();
            let tags_path_str = config.tags_path.clone();
            let app_handle_gui = app.handle().clone();

            tauri::async_runtime::spawn(async move {
                let model_path = resolve_model_path(&app_handle_gui, &model_path_str);
                let tags_path = resolve_model_path(&app_handle_gui, &tags_path_str);

                if let Err(e) = model_manager::check_and_download_models(&app_handle_gui, &model_path, &tags_path).await {
                     let _ = app_handle_gui.emit("model-download-error", e.clone());
                     return;
                }

                let state = app_handle_gui.state::<AppState>();
                let is_loaded = state.tagger.lock().unwrap().is_some();

                if !is_loaded {
                    match Tagger::new(model_path.to_str().unwrap_or(&model_path_str), tags_path.to_str().unwrap_or(&tags_path_str)) {
                        Ok(tagger) => {
                            *state.tagger.lock().unwrap() = Some(tagger);
                            println!("Tagger loaded successfully");
                            let _ = app_handle_gui.emit("tagger-loaded", ());
                        }
                        Err(e) => {
                            eprintln!("Failed to load tagger: {}", e);
                        }
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            get_config,
            set_config,
            check_model_exists,
            download_new_model,
            register_context_menu,
            register_native_host
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
