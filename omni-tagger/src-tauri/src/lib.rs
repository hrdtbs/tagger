mod config;
mod model_manager;
mod processor;
mod registry;
mod state;
mod tagger;

use crate::config::{load_config, resolve_model_path, AppConfig};
use crate::processor::process_inputs;
use crate::state::AppState;
use crate::tagger::Tagger;
use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Emitter, Manager,
};

fn setup_menu(app: &mut tauri::App) -> tauri::Result<()> {
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
    Ok(())
}

fn init_model_loader(app_handle: tauri::AppHandle, config: AppConfig) {
    let model_path_str = config.model_path.clone();
    let tags_path_str = config.tags_path.clone();

    tauri::async_runtime::spawn(async move {
        let model_path = resolve_model_path(&app_handle, &model_path_str);
        let tags_path = resolve_model_path(&app_handle, &tags_path_str);

        if let Err(e) = model_manager::check_and_download_models(
            &app_handle,
            &model_path,
            &tags_path,
        )
        .await
        {
            let _ = app_handle.emit("model-download-error", e.to_string());
            return;
        }

        let state = app_handle.state::<AppState>();
        // check if loaded
        let is_loaded = state
            .tagger
            .lock()
            .map(|g| g.is_some())
            .unwrap_or_else(|e| {
                eprintln!("Failed to lock tagger state for reading: {}", e);
                false
            });

        if !is_loaded {
            match Tagger::new(
                model_path.to_str().unwrap_or(&model_path_str),
                tags_path.to_str().unwrap_or(&tags_path_str),
            ) {
                Ok(tagger) => match state.tagger.lock() {
                    Ok(mut guard) => {
                        *guard = Some(tagger);
                        println!("Tagger loaded successfully");
                        let _ = app_handle.emit("tagger-loaded", ());
                    }
                    Err(e) => eprintln!("Failed to lock tagger state for writing: {}", e),
                },
                Err(e) => {
                    eprintln!("Failed to load tagger: {}", e);
                }
            }
        }
    });
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
                    let _ = app_handle
                        .notification()
                        .builder()
                        .title("Error")
                        .body(format!("Processing failed: {}", e))
                        .show();
                }
            });
        }))
        .setup(|app| {
            setup_menu(app)?;

            let config = load_config(app.handle());
            {
                let state = app.state::<AppState>();
                let mut config_guard = state
                    .config
                    .lock()
                    .expect("Failed to lock config state");
                *config_guard = config.clone();
            }
            let app_handle = app.handle().clone();

            // Initial Arg Check (First Instance)
            let args: Vec<String> = std::env::args().collect();
            if args.len() > 1 {
                let args_clone = args.clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = process_inputs(&app_handle, args_clone).await {
                        eprintln!("Error processing inputs: {}", e);
                    }
                    app_handle.exit(0);
                });
                return Ok(());
            }

            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
            }

            init_model_loader(app.handle().clone(), config);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            config::get_config,
            config::set_config,
            config::check_model_exists,
            config::download_new_model,
            registry::register_context_menu,
            registry::register_native_host,
            registry::unregister_native_host
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
