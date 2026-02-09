use tauri::{AppHandle, Manager};
use std::path::PathBuf;
use crate::state::AppState;
use crate::config::{get_config, resolve_model_path};
use crate::tagger::Tagger;
use tauri_plugin_notification::NotificationExt;

pub async fn process_inputs(app: &AppHandle, args: Vec<String>) -> Result<(), String> {
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

async fn process_image_url(app: &AppHandle, url: String) -> Result<(), String> {
    // Download image
    // Using reqwest
    let client = reqwest::Client::new();
    let resp = client.get(&url).send().await.map_err(|e| e.to_string())?;
    let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
    let img = image::load_from_memory(&bytes).map_err(|e| format!("Failed to load image from URL: {}", e))?;

    run_inference_and_notify(app, img).await
}

async fn process_image_file(app: &AppHandle, path: PathBuf) -> Result<(), String> {
    let img = image::open(&path).map_err(|e| format!("Failed to open image: {}", e))?;
    run_inference_and_notify(app, img).await
}

async fn run_inference_and_notify(app: &AppHandle, img: image::DynamicImage) -> Result<(), String> {
    let state = app.state::<AppState>();

    // We can't use get_config command directly as it expects State<_>.
    // But we have AppState.
    // Actually get_config in config.rs takes State<'_, AppState>.
    // But here we are in async function, we can just access the state directly.
    // Or we can call get_config(state).
    // Let's reuse logic from config.rs, but get_config takes State, which we have.

    // However, get_config is a command, it returns Result<AppConfig, String>.
    // And it takes State<'_, AppState>.
    // app.state::<AppState>() returns State<AppState>.
    // So get_config(state) works.

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

    let _ = app.notification().builder()
        .title("Tags Copied!")
        .body(&tags_str)
        .show();

    Ok(())
}
