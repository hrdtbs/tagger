use anyhow::{Context, Result};
use crate::config::{get_config, resolve_model_path};
use crate::state::AppState;
use crate::tagger::Tagger;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;

pub async fn process_inputs(app: &AppHandle, args: Vec<String>) -> Result<()> {
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

async fn process_image_url(app: &AppHandle, url: String) -> Result<()> {
    // Download image
    // Using reqwest
    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .send()
        .await
        .context("Failed to send request")?;
    let bytes = resp.bytes().await.context("Failed to read response body")?;
    let img = image::load_from_memory(&bytes).context("Failed to load image from URL")?;

    run_inference_and_notify(app, img).await
}

async fn process_image_file(app: &AppHandle, path: PathBuf) -> Result<()> {
    let img = image::open(&path).context("Failed to open image file")?;
    run_inference_and_notify(app, img).await
}

async fn run_inference_and_notify(app: &AppHandle, img: image::DynamicImage) -> Result<()> {
    let state = app.state::<AppState>();

    let config = get_config(state.clone()).map_err(anyhow::Error::msg)?;

    // Quick check if loaded
    let is_loaded = state
        .tagger
        .lock()
        .map_err(|_| anyhow::anyhow!("Mutex poisoned"))?
        .is_some();

    if !is_loaded {
        let model_path = resolve_model_path(app, &config.model_path);
        let tags_path = resolve_model_path(app, &config.tags_path);

        let tagger = Tagger::new(
            model_path.to_str().unwrap_or(&config.model_path),
            tags_path.to_str().unwrap_or(&config.tags_path),
        )
        .context("Failed to initialize tagger")?;

        *state
            .tagger
            .lock()
            .map_err(|_| anyhow::anyhow!("Mutex poisoned"))? = Some(tagger);
    }

    let mut tagger_guard = state
        .tagger
        .lock()
        .map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;
    let tagger = tagger_guard
        .as_mut()
        .ok_or_else(|| anyhow::anyhow!("Tagger not available"))?;

    let results = tagger.infer(&img, config.threshold).context("Inference failed")?;

    let mut filtered: Vec<String> = results
        .into_iter()
        .map(|(t, _)| t)
        .filter(|t| !config.exclusion_list.contains(t))
        .collect();

    if config.use_underscore {
        filtered = filtered.iter().map(|t| t.replace(" ", "_")).collect();
    } else {
        filtered = filtered.iter().map(|t| t.replace("_", " ")).collect();
    }
    let tags_str = filtered.join(", ");

    let mut clipboard = arboard::Clipboard::new().context("Failed to initialize clipboard")?;
    clipboard
        .set_text(tags_str.clone())
        .context("Failed to set clipboard text")?;

    let _ = app
        .notification()
        .builder()
        .title("Tags Copied!")
        .body(&tags_str)
        .show();

    Ok(())
}
