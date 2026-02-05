mod tagger;

use base64::Engine;
use image::ImageFormat;
use screenshots::Screen;
use std::io::Cursor;
use std::sync::Mutex;
use std::time::Instant;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Emitter, Manager, State,
};
use tauri_plugin_global_shortcut::{Code, Modifiers, ShortcutState};

struct AppState {
    last_capture: Mutex<Option<Vec<u8>>>,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn capture_screen(state: State<'_, AppState>) -> Result<String, String> {
    let start = Instant::now();
    let screens = Screen::all().map_err(|e| e.to_string())?;

    // For simplicity, we capture the primary screen or the first one.
    if let Some(screen) = screens.first() {
        let image = screen.capture().map_err(|e| e.to_string())?;
        let buffer = image.to_png().map_err(|e| e.to_string())?;

        // Save to state
        *state.last_capture.lock().map_err(|e| e.to_string())? = Some(buffer.clone());

        let b64 = base64::engine::general_purpose::STANDARD.encode(&buffer);

        println!("Screen captured in {:?}", start.elapsed());
        Ok(format!("data:image/png;base64,{}", b64))
    } else {
        Err("No screens found".to_string())
    }
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

    // Decode image
    let img = image::load_from_memory(data).map_err(|e| e.to_string())?;

    // Crop
    let cropped = img.crop_imm(x, y, w, h);

    // Convert back to bytes for tagger (simulate)
    let mut cropped_bytes = Vec::new();
    cropped
        .write_to(&mut Cursor::new(&mut cropped_bytes), ImageFormat::Png)
        .map_err(|e| e.to_string())?;

    // Tagger
    let tags = tagger::extract_tags(&cropped_bytes);

    // Clipboard
    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set_text(tags.clone()).map_err(|e| e.to_string())?;

    Ok(tags)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState {
            last_capture: Mutex::new(None),
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_shortcut(Modifiers::ALT | Modifiers::SHIFT, Code::KeyT)
                .unwrap()
                .with_handler(|app, shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        if shortcut.matches(Modifiers::ALT | Modifiers::SHIFT, Code::KeyT) {
                            println!("Global hotkey pressed!");
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
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
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet, capture_screen, process_selection])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
