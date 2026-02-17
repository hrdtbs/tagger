use base64::{engine::general_purpose, Engine as _};
use byteorder::{NativeEndian, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Deserialize)]
struct Request {
    url: Option<String>,
    data: Option<String>, // Base64 data URI if needed
}

#[derive(Serialize)]
struct Response {
    status: String,
    message: String,
}

fn main() -> io::Result<()> {
    loop {
        // Read 4 bytes length
        let length = match io::stdin().read_u32::<NativeEndian>() {
            Ok(len) => len as usize,
            Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => break, // Extension closed connection
            Err(e) => return Err(e),
        };

        // Read message body
        let mut buffer = vec![0u8; length];
        io::stdin().read_exact(&mut buffer)?;

        let request_str =
            String::from_utf8(buffer).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let response = match serde_json::from_str::<Request>(&request_str) {
            Ok(req) => handle_request(req),
            Err(e) => Response {
                status: "error".to_string(),
                message: format!("Invalid JSON: {}", e),
            },
        };

        send_response(&response)?;
    }
    Ok(())
}

fn handle_request(req: Request) -> Response {
    let mut command_args = Vec::new();

    if let Some(url) = req.url {
        command_args.push("--process-url".to_string());
        command_args.push(url);
    } else if let Some(data) = req.data {
        if data.starts_with("data:") {
            if let Some(comma_idx) = data.find(',') {
                let header = &data[..comma_idx];
                let base64_data = &data[comma_idx + 1..];

                let extension = if header.contains("image/png") {
                    "png"
                } else if header.contains("image/jpeg") || header.contains("image/jpg") {
                    "jpg"
                } else if header.contains("image/webp") {
                    "webp"
                } else {
                    "png"
                };

                match general_purpose::STANDARD.decode(base64_data) {
                    Ok(decoded) => {
                        let timestamp = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_millis();
                        let temp_dir = env::temp_dir();
                        let file_name = format!("omni_tagger_{}.{}", timestamp, extension);
                        let file_path = temp_dir.join(file_name);

                        match fs::write(&file_path, decoded) {
                            Ok(_) => {
                                command_args.push("--delete-after".to_string());
                                command_args.push(file_path.to_string_lossy().into_owned());
                            }
                            Err(e) => {
                                return Response {
                                    status: "error".to_string(),
                                    message: format!("Failed to write temp file: {}", e),
                                }
                            }
                        }
                    }
                    Err(e) => {
                        return Response {
                            status: "error".to_string(),
                            message: format!("Failed to decode base64: {}", e),
                        }
                    }
                }
            } else {
                return Response {
                    status: "error".to_string(),
                    message: "Invalid data URI format".to_string(),
                };
            }
        } else {
            return Response {
                status: "error".to_string(),
                message: "Invalid data URI format (must start with data:)".to_string(),
            };
        }
    } else {
        return Response {
            status: "error".to_string(),
            message: "No URL or data provided".to_string(),
        };
    };

    // Determine path to main executable
    // Assume it's in the same directory as this native_host binary
    let current_exe = match env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            return Response {
                status: "error".to_string(),
                message: format!("Failed to get exe path: {}", e),
            }
        }
    };

    let exe_dir = current_exe
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));

    // Check for "omni-tagger.exe" (Windows) or "omni-tagger" (Linux/Mac)
    #[cfg(target_os = "windows")]
    let app_name = "omni-tagger.exe";
    #[cfg(not(target_os = "windows"))]
    let app_name = "omni-tagger";

    // Check same directory (Dev)
    let app_path_local = exe_dir.join(app_name);
    // Check parent directory (Prod/Resources)
    let app_path_parent = exe_dir.parent().unwrap_or(exe_dir).join(app_name);

    let mut found_path = None;

    if app_path_local.exists() {
        found_path = Some(app_path_local.clone());
    } else if app_path_parent.exists() {
        found_path = Some(app_path_parent.clone());
    }

    #[cfg(not(target_os = "windows"))]
    if found_path.is_none() {
        // 3. Check /usr/bin and /usr/local/bin
        let p1 = std::path::PathBuf::from("/usr/bin").join(app_name);
        let p2 = std::path::PathBuf::from("/usr/local/bin").join(app_name);
        if p1.exists() {
            found_path = Some(p1);
        } else if p2.exists() {
            found_path = Some(p2);
        }
    }

    let app_path = match found_path {
        Some(p) => p,
        None => {
            return Response {
                status: "error".to_string(),
                message: format!(
                    "App executable not found. Searched at {:?}, {:?} (and system paths on Linux)",
                    app_path_local, app_path_parent
                ),
            };
        }
    };

    // Launch app
    // We use "start" on Windows to launch detached? Or just spawn.
    // If we spawn directly, it might be a child process.
    // We want to trigger the single instance mechanism.
    let mut cmd = Command::new(&app_path);
    for arg in command_args {
        cmd.arg(arg);
    }

    match cmd.spawn() {
        Ok(_) => Response {
            status: "ok".to_string(),
            message: "Processing started".to_string(),
        },
        Err(e) => Response {
            status: "error".to_string(),
            message: format!("Failed to launch app: {}", e),
        },
    }
}

fn send_response(response: &Response) -> io::Result<()> {
    let response_json = serde_json::to_string(response)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let bytes = response_json.as_bytes();
    let length = bytes.len() as u32;

    io::stdout().write_u32::<NativeEndian>(length)?;
    io::stdout().write_all(bytes)?;
    io::stdout().flush()?;
    Ok(())
}
