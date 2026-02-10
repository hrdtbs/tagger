use std::io::{self, Read, Write};
use byteorder::{ReadBytesExt, WriteBytesExt, NativeEndian};
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::env;

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

        let request_str = String::from_utf8(buffer).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

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
    let target = if let Some(url) = req.url {
        url
    } else if let Some(data) = req.data {
        data // Assuming data URI or path
    } else {
        return Response { status: "error".to_string(), message: "No URL or data provided".to_string() };
    };

    // Determine path to main executable
    // Assume it's in the same directory as this native_host binary
    let current_exe = match env::current_exe() {
        Ok(p) => p,
        Err(e) => return Response { status: "error".to_string(), message: format!("Failed to get exe path: {}", e) },
    };

    let exe_dir = current_exe.parent().unwrap_or_else(|| std::path::Path::new("."));

    // Check for "omni-tagger.exe" (Windows) or "omni-tagger" (Linux/Mac)
    #[cfg(target_os = "windows")]
    let app_name = "omni-tagger.exe";
    #[cfg(not(target_os = "windows"))]
    let app_name = "omni-tagger";

    // Check same directory (Dev)
    let app_path_local = exe_dir.join(app_name);
    // Check parent directory (Prod/Resources)
    let app_path_parent = exe_dir.parent().unwrap_or(exe_dir).join(app_name);

    let app_path = if app_path_local.exists() {
        app_path_local
    } else if app_path_parent.exists() {
        app_path_parent
    } else {
         return Response {
             status: "error".to_string(),
             message: format!("App executable not found. Searched at {:?} and {:?}", app_path_local, app_path_parent)
         };
    };

    // Launch app
    // We use "start" on Windows to launch detached? Or just spawn.
    // If we spawn directly, it might be a child process.
    // We want to trigger the single instance mechanism.
    match Command::new(&app_path)
        .arg("--process-url")
        .arg(&target)
        .spawn() {
            Ok(_) => Response { status: "ok".to_string(), message: "Processing started".to_string() },
            Err(e) => Response { status: "error".to_string(), message: format!("Failed to launch app: {}", e) },
        }
}

fn send_response(response: &Response) -> io::Result<()> {
    let response_json = serde_json::to_string(response).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let bytes = response_json.as_bytes();
    let length = bytes.len() as u32;

    io::stdout().write_u32::<NativeEndian>(length)?;
    io::stdout().write_all(bytes)?;
    io::stdout().flush()?;
    Ok(())
}
