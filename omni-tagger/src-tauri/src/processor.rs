use crate::config::{get_config, resolve_model_path};
use crate::model_manager;
use crate::state::AppState;
use crate::tagger::Tagger;
use anyhow::{Context, Result};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;

pub async fn process_inputs(app: &AppHandle, args: Vec<String>) -> Result<(), String> {
    process_inputs_with_actions(
        args,
        |url| process_image_url(app, url),
        |path| process_image_file(app, path),
    )
    .await
    .map_err(|e| e.to_string())
}

pub async fn process_inputs_with_actions<FUrl, FutUrl, FFile, FutFile>(
    args: Vec<String>,
    url_processor: FUrl,
    file_processor: FFile,
) -> Result<()>
where
    FUrl: FnOnce(String) -> FutUrl,
    FutUrl: std::future::Future<Output = Result<()>>,
    FFile: FnOnce(PathBuf) -> FutFile,
    FutFile: std::future::Future<Output = Result<()>>,
{
    if args.len() <= 1 {
        return Ok(());
    }

    let mut idx = 1;
    let mut delete_after = false;

    if args.len() > idx && args[idx] == "--delete-after" {
        delete_after = true;
        idx += 1;
    }

    if args.len() <= idx {
        return Ok(());
    }

    let arg = args[idx].clone();

    if arg == "--process-url" {
        if args.len() > idx + 1 {
            let url = args[idx + 1].clone();
            url_processor(url).await?;
        }
    } else if !arg.starts_with("--") {
        let path = PathBuf::from(arg);
        let result = file_processor(path.clone()).await;

        if delete_after {
            if let Err(e) = std::fs::remove_file(&path) {
                eprintln!("Failed to delete temp file {:?}: {}", path, e);
            } else {
                println!("Deleted temp file {:?}", path);
            }
        }
        result?;
    }

    Ok(())
}

async fn process_image_url(app: &AppHandle, url: String) -> Result<()> {
    // Validate URL to prevent SSRF
    let parsed_url = url::Url::parse(&url).context("Invalid URL format")?;

    // 1. Check Scheme (only http/https)
    let scheme = parsed_url.scheme();
    if scheme != "http" && scheme != "https" {
        return Err(anyhow::anyhow!("Invalid URL scheme. Only HTTP and HTTPS are allowed."));
    }

    // 2. Resolve and check Host (Reject localhost, loopback, private networks)
    let host_str = parsed_url.host_str().ok_or_else(|| anyhow::anyhow!("URL has no host"))?;

    // Simple blocklist for common local/private hostnames
    let lower_host = host_str.to_lowercase();
    if lower_host == "localhost" || lower_host.ends_with(".localhost") || lower_host == "broadcasthost" {
        return Err(anyhow::anyhow!("URL resolves to a restricted local hostname"));
    }

    // Resolve IPs and check for private/loopback/unspecified
    use std::net::ToSocketAddrs;

    // If it's an IP or a hostname, we try to resolve it to an IP address.
    // We add a dummy port (80) because ToSocketAddrs requires it, even though we just want the IP.
    let addr_str = format!("{}:80", host_str);

    // Note: This relies on the system DNS resolver.
    if let Ok(mut addrs) = addr_str.to_socket_addrs() {
        if let Some(addr) = addrs.next() {
            let ip = addr.ip();
            if ip.is_loopback() || ip.is_unspecified() || ip.is_multicast() {
                 return Err(anyhow::anyhow!("URL resolves to a restricted IP address (loopback/unspecified/multicast)"));
            }

            // Check for private IPv4
            if let std::net::IpAddr::V4(ipv4) = ip {
                if ipv4.is_private() || ipv4.is_link_local() {
                     return Err(anyhow::anyhow!("URL resolves to a restricted private IPv4 address"));
                }
            }
            // Check for private IPv6 (Unique Local Addresses fc00::/7)
            if let std::net::IpAddr::V6(ipv6) = ip {
                if (ipv6.segments()[0] & 0xfe00) == 0xfc00 {
                     return Err(anyhow::anyhow!("URL resolves to a restricted private IPv6 address"));
                }
            }
        }
    } else {
        // If it can't resolve, reqwest will also fail, but we don't necessarily want to block it here
        // if it's just a temporary DNS issue. However, for strict SSRF, failing closed is safer.
        return Err(anyhow::anyhow!("Failed to resolve hostname for URL validation"));
    }

    // Download image
    // Using reqwest with redirects disabled to prevent SSRF via redirect to localhost
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .context("Failed to build client")?;
    let mut resp = client
        .get(&url)
        .send()
        .await
        .context("Failed to send request")?;

    // Check content length header if available
    const MAX_DOWNLOAD_SIZE: u64 = 20 * 1024 * 1024; // 20 MB limit
    if let Some(content_length) = resp.content_length() {
        if content_length > MAX_DOWNLOAD_SIZE {
            return Err(anyhow::anyhow!("File size {} exceeds maximum allowed size of {} bytes", content_length, MAX_DOWNLOAD_SIZE));
        }
    }

    // Stream download and enforce size limit manually
    let mut bytes = Vec::new();
    while let Some(chunk) = resp.chunk().await.context("Failed to get chunk")? {
        if (bytes.len() + chunk.len()) as u64 > MAX_DOWNLOAD_SIZE {
            return Err(anyhow::anyhow!("File size exceeds maximum allowed size of {} bytes", MAX_DOWNLOAD_SIZE));
        }
        bytes.extend_from_slice(&chunk);
    }

    let img = image::load_from_memory(&bytes).context("Failed to load image from URL")?;

    run_inference_and_notify(app, img).await
}

async fn process_image_file(app: &AppHandle, path: PathBuf) -> Result<()> {
    let img = image::open(&path).context(format!("Failed to open image at {:?}", path))?;
    run_inference_and_notify(app, img).await
}

async fn run_inference_and_notify(app: &AppHandle, img: image::DynamicImage) -> Result<()> {
    let state = app.state::<AppState>();

    let config = get_config(state.clone()).map_err(|e| anyhow::anyhow!(e))?;

    // Quick check if loaded
    let is_loaded = state
        .tagger
        .lock()
        .map_err(|_| anyhow::anyhow!("Failed to lock tagger"))?
        .is_some();

    if !is_loaded {
        // Load it now
        let model_path = resolve_model_path(app, &config.model_path);
        let tags_path = resolve_model_path(app, &config.tags_path);

        model_manager::check_and_download_models(app, &model_path, &tags_path)
            .await
            .context("Failed to check/download models")?;

        let tagger = Tagger::new(
            model_path.to_str().unwrap_or(&config.model_path),
            tags_path.to_str().unwrap_or(&config.tags_path),
            config.preprocessing.clone(),
        )?;

        *state
            .tagger
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock tagger"))? = Some(tagger);
    }

    let mut tagger_guard = state
        .tagger
        .lock()
        .map_err(|_| anyhow::anyhow!("Failed to lock tagger"))?;
    let tagger = tagger_guard
        .as_mut()
        .ok_or_else(|| anyhow::anyhow!("Tagger not available"))?;

    let results = tagger.infer(&img, config.threshold)?;

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

    let mut clipboard = arboard::Clipboard::new().context("Failed to access clipboard")?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    #[tokio::test]
    async fn test_process_inputs_with_actions_delete_after() {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("omni_tagger_test_delete.tmp");

        // Create a dummy file
        {
            let mut file = fs::File::create(&file_path).unwrap();
            writeln!(file, "dummy content").unwrap();
        }

        assert!(file_path.exists());

        let args = vec![
            "app_name".to_string(),
            "--delete-after".to_string(),
            file_path.to_string_lossy().to_string(),
        ];

        let result =
            process_inputs_with_actions(args, |_| async { Ok(()) }, |_| async { Ok(()) }).await;

        assert!(result.is_ok());
        assert!(
            !file_path.exists(),
            "File should be deleted after processing"
        );
    }

    #[tokio::test]
    async fn test_process_inputs_with_actions_no_delete() {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("omni_tagger_test_keep.tmp");

        // Create a dummy file
        {
            let mut file = fs::File::create(&file_path).unwrap();
            writeln!(file, "dummy content").unwrap();
        }

        assert!(file_path.exists());

        let args = vec![
            "app_name".to_string(),
            file_path.to_string_lossy().to_string(),
        ];

        let result =
            process_inputs_with_actions(args, |_| async { Ok(()) }, |_| async { Ok(()) }).await;

        assert!(result.is_ok());
        assert!(file_path.exists(), "File should NOT be deleted");

        // Cleanup
        fs::remove_file(file_path).unwrap();
    }
}
