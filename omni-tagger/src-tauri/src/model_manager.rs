use tauri::{AppHandle, Emitter, Manager};
use std::path::Path;
use std::fs::{self, File};
use std::io::Write;
use futures_util::StreamExt;

const MODEL_URL: &str = "https://huggingface.co/SmilingWolf/wd-v1-4-swinv2-tagger-v2/resolve/main/model.onnx";
const TAGS_URL: &str = "https://huggingface.co/SmilingWolf/wd-v1-4-swinv2-tagger-v2/resolve/main/selected_tags.csv";

#[derive(Clone, serde::Serialize)]
struct DownloadProgress {
    file: String,
    total: u64,
    downloaded: u64,
    percent: f64,
}

pub async fn check_and_download_models(app: &AppHandle, model_path: &Path, tags_path: &Path) -> Result<(), String> {
    if !model_path.exists() {
        download_file(app, MODEL_URL, model_path).await?;
    }

    if !tags_path.exists() {
        download_file(app, TAGS_URL, tags_path).await?;
    }

    // Emit finished event
    let _ = app.emit("model-download-finished", ());

    Ok(())
}

async fn download_file(app: &AppHandle, url: &str, dest: &Path) -> Result<(), String> {
    let client = reqwest::Client::new();
    let res = client.get(url)
        .send()
        .await
        .map_err(|e| format!("Failed to connect: {}", e))?;

    let total_size = res.content_length().unwrap_or(0);

    // Create parent directory if it doesn't exist
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    let mut file = File::create(dest).map_err(|e| format!("Failed to create file: {}", e))?;
    let mut stream = res.bytes_stream();
    let mut downloaded: u64 = 0;

    let filename = dest.file_name().and_then(|s| s.to_str()).unwrap_or("unknown").to_string();

    while let Some(item) = stream.next().await {
        let chunk = item.map_err(|e| format!("Error while downloading: {}", e))?;
        file.write_all(&chunk).map_err(|e| format!("Error while writing to file: {}", e))?;

        downloaded += chunk.len() as u64;

        if total_size > 0 {
             let percent = (downloaded as f64 / total_size as f64) * 100.0;
             let _ = app.emit("model-download-progress", DownloadProgress {
                 file: filename.clone(),
                 total: total_size,
                 downloaded,
                 percent,
             });
        }
    }

    Ok(())
}
