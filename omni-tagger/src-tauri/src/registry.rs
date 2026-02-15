use tauri::AppHandle;
use tauri::{path::BaseDirectory, Manager};
#[cfg(target_os = "windows")]
use std::process::Command;
#[cfg(target_os = "linux")]
use std::fs;
#[cfg(target_os = "linux")]
use std::path::Path;

#[tauri::command]
pub async fn register_context_menu(app: AppHandle, enable: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let _ = app; // unused on Windows for now, as we use current_exe directly
        let exe_path = std::env::current_exe().map_err(|e| e.to_string())?;
        let exe_str = exe_path.to_str().ok_or("Invalid path")?;

        let command_str = format!("\"{}\" \"%1\"", exe_str);

        if enable {
            Command::new("reg")
                .args(&[
                    "add",
                    "HKCU\\Software\\Classes\\*\\shell\\OmniTagger",
                    "/ve",
                    "/d",
                    "Get Tags",
                    "/f",
                ])
                .output()
                .map_err(|e| format!("Failed to add registry key: {}", e))?;

            Command::new("reg")
                .args(&[
                    "add",
                    "HKCU\\Software\\Classes\\*\\shell\\OmniTagger\\command",
                    "/ve",
                    "/d",
                    &command_str,
                    "/f",
                ])
                .output()
                .map_err(|e| format!("Failed to add command key: {}", e))?;
        } else {
            Command::new("reg")
                .args(&[
                    "delete",
                    "HKCU\\Software\\Classes\\*\\shell\\OmniTagger",
                    "/f",
                ])
                .output()
                .map_err(|e| format!("Failed to delete registry key: {}", e))?;
        }
        Ok(())
    }
    #[cfg(target_os = "linux")]
    {
        let data_dir = app.path().data_dir().map_err(|e| e.to_string())?;
        let applications_dir = data_dir.join("applications");
        let exe_path = std::env::current_exe().map_err(|e| e.to_string())?;

        internal_register_context_menu_linux(&exe_path, &applications_dir, enable)
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        let _ = app;
        let _ = enable;
        Err("Context menu registration is only supported on Windows and Linux".to_string())
    }
}

#[cfg(target_os = "linux")]
fn internal_register_context_menu_linux(exe_path: &Path, applications_dir: &Path, enable: bool) -> Result<(), String> {
    if !applications_dir.exists() {
         fs::create_dir_all(&applications_dir).map_err(|e| e.to_string())?;
    }

    let desktop_file_path = applications_dir.join("omni-tagger-context.desktop");

    if enable {
        let exe_str = exe_path.to_str().ok_or("Invalid path")?;
        let content = generate_desktop_file_content(exe_str);
        fs::write(&desktop_file_path, content).map_err(|e| format!("Failed to write desktop file: {}", e))?;
    } else {
        if desktop_file_path.exists() {
            fs::remove_file(&desktop_file_path).map_err(|e| format!("Failed to remove desktop file: {}", e))?;
        }
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn generate_desktop_file_content(exe_path: &str) -> String {
    format!(
r#"[Desktop Entry]
Type=Application
Name=OmniTagger
Comment=Get AI Tags for images
Exec="{}" %F
Icon=omni-tagger
Terminal=false
Categories=Graphics;Utility;
MimeType=image/jpeg;image/png;image/webp;
Actions=GetTags;

[Desktop Action GetTags]
Name=Get Tags
Exec="{}" %F
"#, exe_path, exe_path)
}

#[tauri::command]
pub async fn register_native_host(app: AppHandle, extension_id: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        // 1. Get native_host.exe path
        // Try to resolve from resources first (Production)
        let resource_path = app
            .path()
            .resolve("native_host.exe", BaseDirectory::Resource)
            .map_err(|e| format!("Failed to resolve resource path: {}", e))?;

        let native_host_path = if resource_path.exists() {
            resource_path
        } else {
            // Fallback: Check alongside main executable (Dev environment)
            let exe_path = std::env::current_exe().map_err(|e| e.to_string())?;
            let exe_dir = exe_path.parent().ok_or("Invalid path")?;
            exe_dir.join("native_host.exe")
        };

        if !native_host_path.exists() {
            return Err(format!(
                "native_host.exe not found at {:?}",
                native_host_path
            ));
        }

        let exe_dir = native_host_path.parent().ok_or("Invalid path")?;

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
                "/f",
            ])
            .output()
            .map_err(|e| format!("Failed to register native host: {}", e))?;

        Ok(())
    }
    #[cfg(target_os = "linux")]
    {
        // 1. Get native_host path
        let resource_path = app
            .path()
            .resolve("native_host.exe", BaseDirectory::Resource)
            .map_err(|e| format!("Failed to resolve resource path: {}", e))?;

        let native_host_path = if resource_path.exists() {
            resource_path
        } else {
             // Fallback dev path logic
             let exe_path = std::env::current_exe().map_err(|e| e.to_string())?;
             let exe_dir = exe_path.parent().ok_or("Invalid path")?;
             let p = exe_dir.join("native_host");
             if p.exists() { p } else { exe_dir.join("native_host.exe") }
        };

        if !native_host_path.exists() {
             return Err(format!("native_host not found at {:?}", native_host_path));
        }

        // 2. Determine config dir
        let config_dir = app.path().config_dir().map_err(|e| e.to_string())?;

        internal_register_native_host_linux(&native_host_path, &config_dir, &extension_id)
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        let _ = app;
        let _ = extension_id;
        Err("Native host registration is only supported on Windows and Linux".to_string())
    }
}

#[cfg(target_os = "linux")]
fn internal_register_native_host_linux(native_host_path: &Path, config_dir: &Path, extension_id: &str) -> Result<(), String> {
    let manifest_content = generate_linux_manifest(native_host_path, extension_id)?;

    // Common paths for Chrome, Chromium, Edge
    let targets = vec![
        config_dir.join("google-chrome/NativeMessagingHosts"),
        config_dir.join("chromium/NativeMessagingHosts"),
        config_dir.join("microsoft-edge/NativeMessagingHosts"),
        config_dir.join("BraveSoftware/Brave-Browser/NativeMessagingHosts"),
    ];

    let mut success_count = 0;

    for dir in targets {
        // Only write if the parent browser directory exists (to avoid polluting unrelated configs)
        if let Some(parent) = dir.parent() {
            if parent.exists() {
                 if !dir.exists() {
                     let _ = fs::create_dir_all(&dir);
                 }
                 let manifest_path = dir.join("com.omnitagger.host.json");
                 let file = fs::File::create(&manifest_path).map_err(|e| e.to_string())?;
                 serde_json::to_writer_pretty(file, &manifest_content).map_err(|e| e.to_string())?;
                 success_count += 1;
            }
        }
    }

    if success_count == 0 {
         // Create default google-chrome path
         let default_dir = config_dir.join("google-chrome/NativeMessagingHosts");
         if !default_dir.exists() {
             let _ = fs::create_dir_all(&default_dir);
         }
         let manifest_path = default_dir.join("com.omnitagger.host.json");
         let file = fs::File::create(&manifest_path).map_err(|e| e.to_string())?;
         serde_json::to_writer_pretty(file, &manifest_content).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn generate_linux_manifest(native_host_path: &std::path::Path, extension_id: &str) -> Result<serde_json::Value, String> {
    let path_str = native_host_path.to_str().ok_or("Invalid native host path")?;
    Ok(serde_json::json!({
        "name": "com.omnitagger.host",
        "description": "OmniTagger Native Messaging Host",
        "path": path_str,
        "type": "stdio",
        "allowed_origins": [
            format!("chrome-extension://{}/", extension_id)
        ]
    }))
}


#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(target_os = "linux")]
    use std::path::PathBuf;

    #[test]
    #[cfg(target_os = "linux")]
    fn test_generate_desktop_file_content() {
        let content = generate_desktop_file_content("/usr/bin/omni-tagger");
        assert!(content.contains("Exec=\"/usr/bin/omni-tagger\" %F"));
        assert!(content.contains("[Desktop Entry]"));
        assert!(content.contains("MimeType=image/jpeg;"));
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_generate_linux_manifest() {
        let path = std::path::Path::new("/usr/bin/native_host");
        let json = generate_linux_manifest(path, "abcdefg").unwrap();

        assert_eq!(json["name"], "com.omnitagger.host");
        assert_eq!(json["path"], "/usr/bin/native_host");
        assert_eq!(json["allowed_origins"][0], "chrome-extension://abcdefg/");
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_internal_register_context_menu_linux() {
        let temp_dir = std::env::temp_dir().join("omni_test_context_menu");
        let applications_dir = temp_dir.join("applications");
        let exe_path = PathBuf::from("/usr/bin/omni-tagger-test");

        // Clean up
        if applications_dir.exists() {
            let _ = std::fs::remove_dir_all(&temp_dir);
        }

        // Test Enable
        let res = internal_register_context_menu_linux(&exe_path, &applications_dir, true);
        assert!(res.is_ok());

        let desktop_file = applications_dir.join("omni-tagger-context.desktop");
        assert!(desktop_file.exists());
        let content = std::fs::read_to_string(&desktop_file).unwrap();
        assert!(content.contains("/usr/bin/omni-tagger-test"));

        // Test Disable
        let res = internal_register_context_menu_linux(&exe_path, &applications_dir, false);
        assert!(res.is_ok());
        assert!(!desktop_file.exists());

        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_internal_register_native_host_linux() {
        let temp_dir = std::env::temp_dir().join("omni_test_native_host");
        let config_dir = temp_dir.join("config");
        let native_host_path = PathBuf::from("/usr/bin/native_host_test");
        let ext_id = "test_extension_id";

        // Clean up
        if config_dir.exists() {
            let _ = std::fs::remove_dir_all(&temp_dir);
        }

        // Ensure parent 'google-chrome' exists to test that path
        let chrome_dir = config_dir.join("google-chrome");
        std::fs::create_dir_all(&chrome_dir).unwrap();

        let res = internal_register_native_host_linux(&native_host_path, &config_dir, ext_id);
        assert!(res.is_ok());

        let manifest_file = chrome_dir.join("NativeMessagingHosts/com.omnitagger.host.json");
        assert!(manifest_file.exists());

        let content = std::fs::read_to_string(&manifest_file).unwrap();
        assert!(content.contains("test_extension_id"));
        assert!(content.contains("/usr/bin/native_host_test"));

        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
