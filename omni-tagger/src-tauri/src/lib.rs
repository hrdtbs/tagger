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
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Emitter, Manager,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Vec<String>>();
    let active_tasks = Arc::new(AtomicUsize::new(0));
    let active_tasks_clone = Arc::clone(&active_tasks);

    tauri::Builder::default()
        .manage(AppState {
            tagger: Mutex::new(None),
            config: Mutex::new(AppConfig::default()),
            download_lock: tokio::sync::Mutex::new(()),
            input_tx: tx,
            active_tasks,
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_single_instance::init(move |app, argv, _cwd| {
            println!("Single Instance: {:?}", argv);
            let state = app.state::<AppState>();
            state.active_tasks.fetch_add(1, Ordering::SeqCst);
            let _ = state.input_tx.send(argv);
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
            *app.state::<AppState>().config.lock().expect("failed to lock config") = config.clone();
            let app_handle = app.handle().clone();

            // Setup background worker for queue processing
            let app_handle_worker = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                while let Some(args) = rx.recv().await {
                    if let Err(e) = process_inputs(&app_handle_worker, args).await {
                        eprintln!("Error processing inputs: {}", e);
                        use tauri_plugin_notification::NotificationExt;
                        let _ = app_handle_worker
                            .notification()
                            .builder()
                            .title("Error")
                            .body(format!("Processing failed: {}", e))
                            .show();
                    }

                    let remaining = active_tasks_clone.fetch_sub(1, Ordering::SeqCst) - 1;
                    if remaining == 0 {
                        // In CLI mode (initial args > 1), we exit when queue is empty
                        // But wait, the app might be kept alive if it's the first instance
                        // We will handle CLI exit separately if needed, or exit here if
                        // there was no GUI window intended.
                        let has_args = std::env::args().len() > 1;
                        if has_args {
                            app_handle_worker.exit(0);
                        }
                    }
                }
            });

            // Initial Arg Check (First Instance)
            let args: Vec<String> = std::env::args().collect();
            if args.len() > 1 {
                let state = app.state::<AppState>();
                state.active_tasks.fetch_add(1, Ordering::SeqCst);
                let _ = state.input_tx.send(args);
                return Ok(());
            }

            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
            }

            // Preload Tagger in background for GUI usage
            let model_path_str = config.model_path.clone();
            let tags_path_str = config.tags_path.clone();
            let preprocessing_config = config.preprocessing.clone();
            let app_handle_gui = app.handle().clone();

            tauri::async_runtime::spawn(async move {
                let model_path = resolve_model_path(&app_handle_gui, &model_path_str);
                let tags_path = resolve_model_path(&app_handle_gui, &tags_path_str);

                if let Err(e) = model_manager::check_and_download_models(
                    &app_handle_gui,
                    &model_path,
                    &tags_path,
                )
                .await
                {
                    let _ = app_handle_gui.emit("model-download-error", e.to_string());
                    return;
                }

                let state = app_handle_gui.state::<AppState>();
                let is_loaded = state.tagger.lock().expect("failed to lock tagger").is_some();

                if !is_loaded {
                    match Tagger::new(
                        model_path.to_str().unwrap_or(&model_path_str),
                        tags_path.to_str().unwrap_or(&tags_path_str),
                        preprocessing_config,
                    ) {
                        Ok(tagger) => {
                            *state.tagger.lock().expect("failed to lock tagger") = Some(tagger);
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
