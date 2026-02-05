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

struct AppState {
    last_capture: Mutex<Option<Vec<u8>>>,
    tagger: Mutex<Option<Tagger>>,
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

    let tags = if let Some(tagger) = tagger_guard.as_mut() {
        // Run inference
        let results = tagger.infer(&cropped, 0.35).map_err(|e| e.to_string())?;
        // Convert to string
        results.into_iter().map(|(t, _)| t).collect::<Vec<_>>().join(", ")
    } else {
        println!("Tagger not loaded, using fallback.");
        "1girl, solo, fallback_tag".to_string()
    };

    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set_text(tags.clone()).map_err(|e| e.to_string())?;

    Ok(tags)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState {
            last_capture: Mutex::new(None),
            tagger: Mutex::new(None),
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

            // Initialize Tagger
            let model_path = "models/model.onnx";
            let tags_path = "models/tags.csv";

            match Tagger::new(model_path, tags_path) {
                Ok(tagger) => {
                    let state = app.state::<AppState>();
                    *state.tagger.lock().unwrap() = Some(tagger);
                    println!("Tagger loaded successfully.");
                }
                Err(e) => {
                    println!("Failed to load tagger: {}. (Expected if models not present)", e);
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet, capture_screen, process_selection])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
