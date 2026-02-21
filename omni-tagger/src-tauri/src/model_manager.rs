use anyhow::{anyhow, Context, Result};
use futures_util::StreamExt;
use std::path::Path;
use tauri::{AppHandle, Emitter};
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;

const SWINV2_MODEL_URL: &str =
    "https://huggingface.co/SmilingWolf/wd-v1-4-swinv2-tagger-v2/resolve/main/model.onnx";
const CONVNEXT_MODEL_URL: &str =
    "https://huggingface.co/SmilingWolf/wd-v1-4-convnext-tagger-v2/resolve/main/model.onnx";
const CONVNEXTV2_MODEL_URL: &str =
    "https://huggingface.co/SmilingWolf/wd-v1-4-convnextv2-tagger-v2/resolve/main/model.onnx";

const TAGS_URL: &str =
    "https://huggingface.co/SmilingWolf/wd-v1-4-swinv2-tagger-v2/resolve/main/selected_tags.csv";

#[derive(Clone, serde::Serialize)]
struct DownloadProgress {
    file: String,
    total: u64,
    downloaded: u64,
    percent: f64,
}

fn get_model_url(path: &Path) -> Option<&'static str> {
    let file_name = path.file_name()?.to_str()?;
    match file_name {
        "model.onnx" => Some(SWINV2_MODEL_URL),
        "convnext.onnx" => Some(CONVNEXT_MODEL_URL),
        "convnextv2.onnx" => Some(CONVNEXTV2_MODEL_URL),
        _ => None,
    }
}

pub async fn check_and_download_models(
    app: &AppHandle,
    model_path: &Path,
    tags_path: &Path,
) -> Result<()> {
    if !model_path.exists() {
        if let Some(url) = get_model_url(model_path) {
            download_file(app, url, model_path).await?;
        } else {
            return Err(anyhow!("Model file not found at {:?} and cannot be automatically downloaded. Please ensure the path is correct or download the model manually.", model_path));
        }
    }

    if !tags_path.exists() {
        download_file(app, TAGS_URL, tags_path).await?;
    }

    // Emit finished event
    let _ = app.emit("model-download-finished", ());

    Ok(())
}

pub fn check_file_exists(path: &Path) -> bool {
    path.exists()
}

pub async fn download_file(app: &AppHandle, url: &str, dest: &Path) -> Result<()> {
    let client = reqwest::Client::new();
    let res = client
        .get(url)
        .send()
        .await
        .context("Failed to connect")?;

    let total_size = res.content_length().unwrap_or(0);

    // Create parent directory if it doesn't exist
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)
            .await
            .context("Failed to create directory")?;
    }

    let mut file = File::create(dest).await.context("Failed to create file")?;
    let mut stream = res.bytes_stream();
    let mut downloaded: u64 = 0;

    let filename = dest
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    while let Some(item) = stream.next().await {
        let chunk = item.context("Error while downloading")?;
        file.write_all(&chunk)
            .await
            .context("Error while writing to file")?;

        downloaded += chunk.len() as u64;

        if total_size > 0 {
            let percent = (downloaded as f64 / total_size as f64) * 100.0;
            let _ = app.emit(
                "model-download-progress",
                DownloadProgress {
                    file: filename.clone(),
                    total: total_size,
                    downloaded,
                    percent,
                },
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_check_file_exists() {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("omni_tagger_test_model.onnx");

        // Ensure file does not exist initially
        if file_path.exists() {
            fs::remove_file(&file_path).unwrap();
        }

        assert!(!check_file_exists(&file_path));

        // Create dummy file
        fs::write(&file_path, "dummy content").unwrap();

        assert!(check_file_exists(&file_path));

        // Clean up
        fs::remove_file(file_path).unwrap();
    }

    #[test]
    fn test_get_model_url() {
        assert_eq!(
            get_model_url(Path::new("models/model.onnx")),
            Some(SWINV2_MODEL_URL)
        );
        assert_eq!(
            get_model_url(Path::new("/abs/path/to/convnext.onnx")),
            Some(CONVNEXT_MODEL_URL)
        );
        assert_eq!(
            get_model_url(Path::new("convnextv2.onnx")),
            Some(CONVNEXTV2_MODEL_URL)
        );
        assert_eq!(get_model_url(Path::new("custom_model.onnx")), None);
        assert_eq!(get_model_url(Path::new("some/other/file.txt")), None);
    }
}
