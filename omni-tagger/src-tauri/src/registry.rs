use tauri::AppHandle;
#[cfg(target_os = "windows")]
use std::process::Command;

#[tauri::command]
pub async fn register_context_menu(enable: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let exe_path = std::env::current_exe().map_err(|e| e.to_string())?;
        let exe_str = exe_path.to_str().ok_or("Invalid path")?;

        let command_str = format!("\"{}\" \"%1\"", exe_str);

        if enable {
            Command::new("reg")
                .args(&["add", "HKCU\\Software\\Classes\\*\\shell\\OmniTagger", "/ve", "/d", "Get Tags", "/f"])
                .output()
                .map_err(|e| format!("Failed to add registry key: {}", e))?;

            Command::new("reg")
                .args(&["add", "HKCU\\Software\\Classes\\*\\shell\\OmniTagger\\command", "/ve", "/d", &command_str, "/f"])
                .output()
                .map_err(|e| format!("Failed to add command key: {}", e))?;
        } else {
            Command::new("reg")
                .args(&["delete", "HKCU\\Software\\Classes\\*\\shell\\OmniTagger", "/f"])
                .output()
                .map_err(|e| format!("Failed to delete registry key: {}", e))?;
        }
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = enable; // suppress unused warning
        Err("Context menu registration is only supported on Windows".to_string())
    }
}

#[tauri::command]
pub async fn register_native_host(_app: AppHandle, extension_id: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        // 1. Get exe path and derive native_host.exe path
        let exe_path = std::env::current_exe().map_err(|e| e.to_string())?;
        let exe_dir = exe_path.parent().ok_or("Invalid path")?;
        let native_host_path = exe_dir.join("native_host.exe");

        if !native_host_path.exists() {
             return Err(format!("native_host.exe not found at {:?}", native_host_path));
        }

        // 2. Create JSON Manifest
        let manifest_content = serde_json::json!({
            "name": "com.omnitagger.host",
            "description": "OmniTagger Native Messaging Host",
            "path": native_host_path.to_str().unwrap_or("native_host.exe"),
            "type": "stdio",
            "allowed_origins": [
                format!("chrome-extension://{}/", extension_id)
            ]
        });

        let manifest_path = exe_dir.join("com.omnitagger.host.json");
        let file = std::fs::File::create(&manifest_path).map_err(|e| e.to_string())?;
        serde_json::to_writer_pretty(file, &manifest_content).map_err(|e| e.to_string())?;

        // 3. Add Registry Key
        // HKCU\Software\Google\Chrome\NativeMessagingHosts\com.omnitagger.host
        Command::new("reg")
            .args(&[
                "add",
                "HKCU\\Software\\Google\\Chrome\\NativeMessagingHosts\\com.omnitagger.host",
                "/ve",
                "/d",
                manifest_path.to_str().ok_or("Invalid path")?,
                "/f"
            ])
            .output()
            .map_err(|e| format!("Failed to register native host: {}", e))?;

        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = _app; // suppress unused warning
        let _ = extension_id; // suppress unused warning
        Err("Native host registration is only supported on Windows".to_string())
    }
}
