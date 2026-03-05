#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::fs;
#[cfg(target_os = "windows")]
use std::process::Command;
use tauri::AppHandle;
use tauri::{path::BaseDirectory, Manager};

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
        // On Linux, we create a .desktop file in ~/.local/share/applications/
        let data_local_dir = app
            .path()
            .data_dir()
            .map_err(|e: tauri::Error| e.to_string())?;
        let applications_dir = data_local_dir.join("applications");

        if !applications_dir.exists() {
            fs::create_dir_all(&applications_dir).map_err(|e| e.to_string())?;
        }

        let desktop_file_path = applications_dir.join("omni-tagger-context.desktop");

        if enable {
            let exe_path = std::env::current_exe().map_err(|e| e.to_string())?;
            let exe_str = exe_path.to_str().ok_or("Invalid path")?;

            // Generate content
            let content = generate_desktop_file_content(exe_str);
            fs::write(&desktop_file_path, content)
                .map_err(|e| format!("Failed to write desktop file: {}", e))?;
        } else if desktop_file_path.exists() {
            fs::remove_file(&desktop_file_path)
                .map_err(|e| format!("Failed to remove desktop file: {}", e))?;
        }
        Ok(())
    }
    #[cfg(target_os = "macos")]
    {
        let home_dir = app.path().home_dir().map_err(|e| e.to_string())?;
        let services_dir = home_dir.join("Library/Services");

        if !services_dir.exists() {
            fs::create_dir_all(&services_dir).map_err(|e| e.to_string())?;
        }

        let workflow_dir = services_dir.join("OmniTagger.workflow");

        if enable {
            let exe_path = std::env::current_exe().map_err(|e| e.to_string())?;
            let exe_str = exe_path.to_str().ok_or("Invalid path")?;

            let contents_dir = workflow_dir.join("Contents");
            if !contents_dir.exists() {
                fs::create_dir_all(&contents_dir).map_err(|e| e.to_string())?;
            }

            let document_path = contents_dir.join("document.wflow");
            let content = generate_macos_workflow_content(exe_str);
            fs::write(&document_path, content)
                .map_err(|e| format!("Failed to write workflow file: {}", e))?;
        } else if workflow_dir.exists() {
            fs::remove_dir_all(&workflow_dir)
                .map_err(|e| format!("Failed to remove workflow directory: {}", e))?;
        }
        Ok(())
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        let _ = app;
        let _ = enable;
        Err("Context menu registration is only supported on Windows, Linux, and macOS".to_string())
    }
}

#[cfg(target_os = "macos")]
fn generate_macos_workflow_content(exe_path: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>AMApplicationBuild</key>
	<string>523</string>
	<key>AMApplicationVersion</key>
	<string>2.10</string>
	<key>AMDocumentVersion</key>
	<string>2</string>
	<key>actions</key>
	<array>
		<dict>
			<key>action</key>
			<dict>
				<key>AMAccepts</key>
				<dict>
					<key>Container</key>
					<string>List</string>
					<key>Optional</key>
					<true/>
					<key>Types</key>
					<array>
						<string>com.apple.cocoa.string</string>
					</array>
				</dict>
				<key>AMActionVersion</key>
				<string>2.0.3</string>
				<key>AMApplication</key>
				<array>
					<string>Automator</string>
				</array>
				<key>AMParameterProperties</key>
				<dict>
					<key>COMMAND_STRING</key>
					<dict/>
					<key>CheckedForUserDefaultShell</key>
					<dict/>
					<key>inputMethod</key>
					<dict/>
					<key>shell</key>
					<dict/>
					<key>source</key>
					<dict/>
				</dict>
				<key>AMProvides</key>
				<dict>
					<key>Container</key>
					<string>List</string>
					<key>Types</key>
					<array>
						<string>com.apple.cocoa.string</string>
					</array>
				</dict>
				<key>ActionBundlePath</key>
				<string>/System/Library/Automator/Run Shell Script.action</string>
				<key>ActionName</key>
				<string>Run Shell Script</string>
				<key>ActionParameters</key>
				<dict>
					<key>COMMAND_STRING</key>
					<string>"{}" "$@"</string>
					<key>CheckedForUserDefaultShell</key>
					<true/>
					<key>inputMethod</key>
					<integer>1</integer>
					<key>shell</key>
					<string>/bin/bash</string>
					<key>source</key>
					<string></string>
				</dict>
				<key>BundleIdentifier</key>
				<string>com.apple.RunShellScript</string>
				<key>CFBundleVersion</key>
				<string>2.0.3</string>
				<key>CanShowSelectedItemsWhenRun</key>
				<false/>
				<key>CanShowWhenRun</key>
				<true/>
				<key>Category</key>
				<array>
					<string>AMCategoryUtilities</string>
				</array>
				<key>Class Name</key>
				<string>RunShellScriptAction</string>
				<key>InputUUID</key>
				<string>3A6284CB-0AF9-4A73-A33A-E6726D1FC1AD</string>
				<key>Keywords</key>
				<array>
					<string>Shell</string>
					<string>Script</string>
					<string>Command</string>
					<string>Run</string>
					<string>Unix</string>
				</array>
				<key>OutputUUID</key>
				<string>1E6BAFE3-300D-47C0-8B98-9FFDFB24F0FA</string>
				<key>UUID</key>
				<string>47E20612-4B6A-4A00-88A9-9FE769C5CDBA</string>
				<key>UnlocalizedApplications</key>
				<array>
					<string>Automator</string>
				</array>
				<key>arguments</key>
				<dict>
					<key>0</key>
					<dict>
						<key>default value</key>
						<integer>0</integer>
						<key>name</key>
						<string>inputMethod</string>
						<key>required</key>
						<string>0</string>
						<key>type</key>
						<string>0</string>
						<key>uuid</key>
						<string>0</string>
					</dict>
					<key>1</key>
					<dict>
						<key>default value</key>
						<false/>
						<key>name</key>
						<string>CheckedForUserDefaultShell</string>
						<key>required</key>
						<string>0</string>
						<key>type</key>
						<string>0</string>
						<key>uuid</key>
						<string>1</string>
					</dict>
					<key>2</key>
					<dict>
						<key>default value</key>
						<string></string>
						<key>name</key>
						<string>source</string>
						<key>required</key>
						<string>0</string>
						<key>type</key>
						<string>0</string>
						<key>uuid</key>
						<string>2</string>
					</dict>
					<key>3</key>
					<dict>
						<key>default value</key>
						<string></string>
						<key>name</key>
						<string>COMMAND_STRING</string>
						<key>required</key>
						<string>0</string>
						<key>type</key>
						<string>0</string>
						<key>uuid</key>
						<string>3</string>
					</dict>
					<key>4</key>
					<dict>
						<key>default value</key>
						<string>/bin/sh</string>
						<key>name</key>
						<string>shell</string>
						<key>required</key>
						<string>0</string>
						<key>type</key>
						<string>0</string>
						<key>uuid</key>
						<string>4</string>
					</dict>
				</dict>
				<key>isViewVisible</key>
				<true/>
				<key>location</key>
				<string>353.000000:305.000000</string>
				<key>nibPath</key>
				<string>/System/Library/Automator/Run Shell Script.action/Contents/Resources/Base.lproj/main.nib</string>
			</dict>
			<key>isViewVisible</key>
			<true/>
		</dict>
	</array>
	<key>connectors</key>
	<dict/>
	<key>workflowMetaData</key>
	<dict>
		<key>applicationBundleID</key>
		<string>com.apple.finder</string>
		<key>applicationBundleIDsByPath</key>
		<dict>
			<key>/System/Library/CoreServices/Finder.app</key>
			<string>com.apple.finder</string>
		</dict>
		<key>applicationPath</key>
		<string>/System/Library/CoreServices/Finder.app</string>
		<key>applicationPaths</key>
		<array>
			<string>/System/Library/CoreServices/Finder.app</string>
		</array>
		<key>inputTypeIdentifier</key>
		<string>com.apple.Automator.fileSystemObject.image</string>
		<key>outputTypeIdentifier</key>
		<string>com.apple.Automator.nothing</string>
		<key>presentationMode</key>
		<integer>15</integer>
		<key>processesInput</key>
		<false/>
		<key>serviceApplicationBundleID</key>
		<string>com.apple.finder</string>
		<key>serviceApplicationPath</key>
		<string>/System/Library/CoreServices/Finder.app</string>
		<key>serviceInputTypeIdentifier</key>
		<string>com.apple.Automator.fileSystemObject.image</string>
		<key>serviceOutputTypeIdentifier</key>
		<string>com.apple.Automator.nothing</string>
		<key>serviceProcessesInput</key>
		<false/>
		<key>systemImageName</key>
		<string>NSTouchBarTagIcon</string>
		<key>useAutomaticInputType</key>
		<false/>
		<key>workflowTypeIdentifier</key>
		<string>com.apple.Automator.servicesMenu</string>
	</dict>
</dict>
</plist>
"#,
        exe_path
    )
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
"#,
        exe_path, exe_path
    )
}

#[tauri::command]
pub async fn register_native_host(
    app: AppHandle,
    extension_id: String,
    browser: Option<String>,
) -> Result<(), String> {
    let browser_type = browser.unwrap_or_else(|| "chromium".to_string());

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

        if browser_type == "firefox" {
            // Firefox Logic
            let manifest_content = generate_firefox_manifest_content(
                native_host_path.to_str().unwrap_or("native_host.exe"),
                &extension_id,
            );

            // We need a separate manifest file for Firefox because content differs
            let manifest_path = exe_dir.join("com.omnitagger.host-firefox.json");
            let file = std::fs::File::create(&manifest_path).map_err(|e| e.to_string())?;
            serde_json::to_writer_pretty(file, &manifest_content).map_err(|e| e.to_string())?;

            let key = "HKCU\\Software\\Mozilla\\NativeMessagingHosts\\com.omnitagger.host";
            Command::new("reg")
                .args(&[
                    "add",
                    key,
                    "/ve",
                    "/d",
                    manifest_path.to_str().ok_or("Invalid path")?,
                    "/f",
                ])
                .output()
                .map_err(|e| format!("Failed to register native host for Firefox: {}", e))?;
        } else {
            // Chromium Logic
            let manifest_content = generate_manifest_content(
                native_host_path.to_str().unwrap_or("native_host.exe"),
                &extension_id,
            );

            let manifest_path = exe_dir.join("com.omnitagger.host.json");
            let file = std::fs::File::create(&manifest_path).map_err(|e| e.to_string())?;
            serde_json::to_writer_pretty(file, &manifest_content).map_err(|e| e.to_string())?;

            // 3. Add Registry Key
            // Iterate over Chrome, Edge, and Brave registry paths
            let registry_keys = vec![
                "HKCU\\Software\\Google\\Chrome\\NativeMessagingHosts\\com.omnitagger.host",
                "HKCU\\Software\\Microsoft\\Edge\\NativeMessagingHosts\\com.omnitagger.host",
                "HKCU\\Software\\BraveSoftware\\Brave-Browser\\NativeMessagingHosts\\com.omnitagger.host",
            ];

            for key in registry_keys {
                Command::new("reg")
                    .args(&[
                        "add",
                        key,
                        "/ve",
                        "/d",
                        manifest_path.to_str().ok_or("Invalid path")?,
                        "/f",
                    ])
                    .output()
                    .map_err(|e| format!("Failed to register native host for {}: {}", key, e))?;
            }
        }
        Ok(())
    }
    #[cfg(target_os = "linux")]
    {
        // 1. Get native_host path
        let resource_path = app
            .path()
            .resolve("native_host.exe", BaseDirectory::Resource)
            .map_err(|e| format!("Failed to resolve resource path: {}", e))?;

        // In dev, it might be in target/release/native_host (no exe) or target/debug/native_host
        // But the build script copies it to resources/native_host.exe even on Linux
        let native_host_path = if resource_path.exists() {
            resource_path
        } else {
            // Fallback dev path logic similar to Windows but checking for no-extension too
            let exe_path = std::env::current_exe().map_err(|e| e.to_string())?;
            let exe_dir = exe_path.parent().ok_or("Invalid path")?;
            let p = exe_dir.join("native_host");
            if p.exists() {
                p
            } else {
                exe_dir.join("native_host.exe")
            }
        };

        if !native_host_path.exists() {
            return Err(format!("native_host not found at {:?}", native_host_path));
        }

        // Generate wrapper script for Flatpak/Snap support
        let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
        if !app_data_dir.exists() {
            fs::create_dir_all(&app_data_dir).map_err(|e| e.to_string())?;
        }
        let wrapper_path = app_data_dir.join("native-host-wrapper.sh");

        let wrapper_content = format!(
            "#!/bin/sh\nif command -v flatpak-spawn >/dev/null 2>&1; then\n    exec flatpak-spawn --host \"{}\" \"$@\"\nelse\n    exec \"{}\" \"$@\"\nfi\n",
            native_host_path.to_str().ok_or("Invalid path")?,
            native_host_path.to_str().ok_or("Invalid path")?
        );

        fs::write(&wrapper_path, wrapper_content).map_err(|e| format!("Failed to write wrapper script: {}", e))?;

        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&wrapper_path).map_err(|e| e.to_string())?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&wrapper_path, perms).map_err(|e| e.to_string())?;

        let executable_path_str = wrapper_path.to_str().ok_or("Invalid wrapper path")?;

        if browser_type == "firefox" {
            // Firefox Logic
            let manifest_content = generate_firefox_manifest_content(
                executable_path_str,
                &extension_id,
            );

            // Use home_dir for ~/.mozilla
            let home_dir = app.path().home_dir().map_err(|e| e.to_string())?;
            let mozilla_native_hosts_dir = home_dir.join(".mozilla/native-messaging-hosts");

            let mut targets = vec![mozilla_native_hosts_dir.clone()];

            // Flatpak
            targets.push(
                home_dir.join(".var/app/org.mozilla.firefox/.mozilla/native-messaging-hosts"),
            );

            // Snap
            targets.push(home_dir.join("snap/firefox/common/.mozilla/native-messaging-hosts"));

            for target_dir in targets {
                if !target_dir.exists() {
                    let _ = fs::create_dir_all(&target_dir);
                }
                let manifest_path = target_dir.join("com.omnitagger.host.json");
                let file = match fs::File::create(&manifest_path) {
                    Ok(f) => f,
                    Err(_) => continue,
                };
                let _ = serde_json::to_writer_pretty(file, &manifest_content);
            }
        } else {
            // Chromium Logic
            let manifest_content = generate_manifest_content(
                executable_path_str,
                &extension_id,
            );

            // 3. Write to browser config directories
            let config_dir = app.path().config_dir().map_err(|e| e.to_string())?;

            // Common paths for Chrome, Chromium, Edge
            let mut targets = vec![
                config_dir.join("google-chrome/NativeMessagingHosts"),
                config_dir.join("chromium/NativeMessagingHosts"),
                config_dir.join("microsoft-edge/NativeMessagingHosts"),
                config_dir.join("BraveSoftware/Brave-Browser/NativeMessagingHosts"),
            ];

            // Flatpak paths
            let home_dir = app.path().home_dir().map_err(|e| e.to_string())?;
            targets.push(
                home_dir
                    .join(".var/app/com.google.Chrome/config/google-chrome/NativeMessagingHosts"),
            );
            targets.push(
                home_dir
                    .join(".var/app/org.chromium.Chromium/config/chromium/NativeMessagingHosts"),
            );
            targets
                .push(home_dir.join(
                    ".var/app/com.microsoft.Edge/config/microsoft-edge/NativeMessagingHosts",
                ));
            targets.push(home_dir.join(".var/app/com.brave.Browser/config/BraveSoftware/Brave-Browser/NativeMessagingHosts"));

            // Snap paths
            targets
                .push(home_dir.join("snap/chromium/current/.config/chromium/NativeMessagingHosts"));
            targets.push(home_dir.join(
                "snap/brave/current/.config/BraveSoftware/Brave-Browser/NativeMessagingHosts",
            ));

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
                        serde_json::to_writer_pretty(file, &manifest_content)
                            .map_err(|e| e.to_string())?;
                        success_count += 1;
                    }
                }
            }

            if success_count == 0 {
                // Maybe no browser installed or paths differ.
                // We can force create google-chrome path?
                // Let's create the google-chrome one by default just in case.
                let default_dir = config_dir.join("google-chrome/NativeMessagingHosts");
                if !default_dir.exists() {
                    let _ = fs::create_dir_all(&default_dir);
                }
                let manifest_path = default_dir.join("com.omnitagger.host.json");
                let file = fs::File::create(&manifest_path).map_err(|e| e.to_string())?;
                serde_json::to_writer_pretty(file, &manifest_content).map_err(|e| e.to_string())?;
            }
        }

        Ok(())
    }
    #[cfg(target_os = "macos")]
    {
        // 1. Get native_host path
        let resource_path = app
            .path()
            .resolve("native_host.exe", BaseDirectory::Resource)
            .map_err(|e| format!("Failed to resolve resource path: {}", e))?;

        let native_host_path = if resource_path.exists() {
            resource_path
        } else {
            let exe_path = std::env::current_exe().map_err(|e| e.to_string())?;
            let exe_dir = exe_path.parent().ok_or("Invalid path")?;
            let p = exe_dir.join("native_host");
            if p.exists() {
                p
            } else {
                exe_dir.join("native_host.exe")
            }
        };

        if !native_host_path.exists() {
            return Err(format!("native_host not found at {:?}", native_host_path));
        }

        let home_dir = app.path().home_dir().map_err(|e| e.to_string())?;

        if browser_type == "firefox" {
            // Firefox Logic
            let manifest_content = generate_firefox_manifest_content(
                native_host_path.to_str().ok_or("Invalid path")?,
                &extension_id,
            );

            let mozilla_native_hosts_dir =
                home_dir.join("Library/Application Support/Mozilla/NativeMessagingHosts");

            if !mozilla_native_hosts_dir.exists() {
                fs::create_dir_all(&mozilla_native_hosts_dir).map_err(|e| e.to_string())?;
            }

            let manifest_path = mozilla_native_hosts_dir.join("com.omnitagger.host.json");
            let file = fs::File::create(&manifest_path).map_err(|e| e.to_string())?;
            serde_json::to_writer_pretty(file, &manifest_content).map_err(|e| e.to_string())?;
        } else {
            // Chromium Logic
            let manifest_content = generate_manifest_content(
                native_host_path.to_str().ok_or("Invalid path")?,
                &extension_id,
            );

            // 3. Write to browser config directories
            let targets = vec![
                home_dir.join("Library/Application Support/Google/Chrome/NativeMessagingHosts"),
                home_dir.join("Library/Application Support/Chromium/NativeMessagingHosts"),
                home_dir.join("Library/Application Support/Microsoft Edge/NativeMessagingHosts"),
                home_dir.join(
                    "Library/Application Support/BraveSoftware/Brave-Browser/NativeMessagingHosts",
                ),
            ];

            let mut success_count = 0;

            for dir in targets {
                if let Some(parent) = dir.parent() {
                    if parent.exists() {
                        if !dir.exists() {
                            let _ = fs::create_dir_all(&dir);
                        }
                        let manifest_path = dir.join("com.omnitagger.host.json");
                        let file = fs::File::create(&manifest_path).map_err(|e| e.to_string())?;
                        serde_json::to_writer_pretty(file, &manifest_content)
                            .map_err(|e| e.to_string())?;
                        success_count += 1;
                    }
                }
            }

            if success_count == 0 {
                // Default to Chrome
                let default_dir =
                    home_dir.join("Library/Application Support/Google/Chrome/NativeMessagingHosts");
                if !default_dir.exists() {
                    let _ = fs::create_dir_all(&default_dir);
                }
                let manifest_path = default_dir.join("com.omnitagger.host.json");
                let file = fs::File::create(&manifest_path).map_err(|e| e.to_string())?;
                serde_json::to_writer_pretty(file, &manifest_content).map_err(|e| e.to_string())?;
            }
        }

        Ok(())
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        let _ = app;
        let _ = extension_id;
        let _ = browser_type;
        Err("Native host registration is only supported on Windows, Linux, and macOS".to_string())
    }
}

#[tauri::command]
pub async fn unregister_native_host(app: AppHandle, browser: Option<String>) -> Result<(), String> {
    let browser_type = browser.unwrap_or_else(|| "chromium".to_string());

    #[cfg(target_os = "windows")]
    {
        let _ = app; // Unused in Windows unregistration logic

        if browser_type == "firefox" {
            let key = "HKCU\\Software\\Mozilla\\NativeMessagingHosts\\com.omnitagger.host";
            Command::new("reg")
                .args(&["delete", key, "/f"])
                .output()
                .map_err(|e| format!("Failed to delete registry key for Firefox: {}", e))?;
        } else {
            let registry_keys = vec![
                "HKCU\\Software\\Google\\Chrome\\NativeMessagingHosts\\com.omnitagger.host",
                "HKCU\\Software\\Microsoft\\Edge\\NativeMessagingHosts\\com.omnitagger.host",
                "HKCU\\Software\\BraveSoftware\\Brave-Browser\\NativeMessagingHosts\\com.omnitagger.host",
            ];

            for key in registry_keys {
                // We ignore errors here because the key might not exist for all browsers
                let _ = Command::new("reg").args(&["delete", key, "/f"]).output();
            }
        }
        Ok(())
    }
    #[cfg(target_os = "linux")]
    {
        if browser_type == "firefox" {
            let home_dir = app.path().home_dir().map_err(|e| e.to_string())?;
            let targets = vec![
                home_dir.join(".mozilla/native-messaging-hosts/com.omnitagger.host.json"),
                home_dir.join(".var/app/org.mozilla.firefox/.mozilla/native-messaging-hosts/com.omnitagger.host.json"),
                home_dir.join("snap/firefox/common/.mozilla/native-messaging-hosts/com.omnitagger.host.json"),
            ];
            for path in targets {
                if path.exists() {
                    let _ = fs::remove_file(&path);
                }
            }
        } else {
            let config_dir = app.path().config_dir().map_err(|e| e.to_string())?;
            let home_dir = app.path().home_dir().map_err(|e| e.to_string())?;

            let targets = vec![
                config_dir.join("google-chrome/NativeMessagingHosts/com.omnitagger.host.json"),
                config_dir.join("chromium/NativeMessagingHosts/com.omnitagger.host.json"),
                config_dir.join("microsoft-edge/NativeMessagingHosts/com.omnitagger.host.json"),
                config_dir.join("BraveSoftware/Brave-Browser/NativeMessagingHosts/com.omnitagger.host.json"),

                // Flatpak
                home_dir.join(".var/app/com.google.Chrome/config/google-chrome/NativeMessagingHosts/com.omnitagger.host.json"),
                home_dir.join(".var/app/org.chromium.Chromium/config/chromium/NativeMessagingHosts/com.omnitagger.host.json"),
                home_dir.join(".var/app/com.microsoft.Edge/config/microsoft-edge/NativeMessagingHosts/com.omnitagger.host.json"),
                home_dir.join(".var/app/com.brave.Browser/config/BraveSoftware/Brave-Browser/NativeMessagingHosts/com.omnitagger.host.json"),

                // Snap
                home_dir.join("snap/chromium/current/.config/chromium/NativeMessagingHosts/com.omnitagger.host.json"),
                home_dir.join("snap/brave/current/.config/BraveSoftware/Brave-Browser/NativeMessagingHosts/com.omnitagger.host.json"),
            ];

            for path in targets {
                if path.exists() {
                    // Ignore errors if removal fails (e.g. permission issues), but we should probably log them?
                    // For now, let's just try to remove.
                    let _ = fs::remove_file(&path);
                }
            }
        }
        Ok(())
    }
    #[cfg(target_os = "macos")]
    {
        let home_dir = app.path().home_dir().map_err(|e| e.to_string())?;

        if browser_type == "firefox" {
            let manifest_path = home_dir.join(
                "Library/Application Support/Mozilla/NativeMessagingHosts/com.omnitagger.host.json",
            );
            if manifest_path.exists() {
                fs::remove_file(&manifest_path)
                    .map_err(|e| format!("Failed to remove manifest: {}", e))?;
            }
        } else {
            let targets = vec![
                home_dir.join("Library/Application Support/Google/Chrome/NativeMessagingHosts/com.omnitagger.host.json"),
                home_dir.join("Library/Application Support/Chromium/NativeMessagingHosts/com.omnitagger.host.json"),
                home_dir.join("Library/Application Support/Microsoft Edge/NativeMessagingHosts/com.omnitagger.host.json"),
                home_dir.join(
                    "Library/Application Support/BraveSoftware/Brave-Browser/NativeMessagingHosts/com.omnitagger.host.json",
                ),
            ];

            for path in targets {
                if path.exists() {
                    let _ = fs::remove_file(&path);
                }
            }
        }
        Ok(())
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        let _ = app;
        let _ = browser_type;
        Err("Native host unregistration is only supported on Windows, Linux, and macOS".to_string())
    }
}

fn generate_manifest_content(native_host_path: &str, extension_id: &str) -> serde_json::Value {
    serde_json::json!({
        "name": "com.omnitagger.host",
        "description": "OmniTagger Native Messaging Host",
        "path": native_host_path,
        "type": "stdio",
        "allowed_origins": [
            format!("chrome-extension://{}/", extension_id)
        ]
    })
}

fn generate_firefox_manifest_content(
    native_host_path: &str,
    extension_id: &str,
) -> serde_json::Value {
    serde_json::json!({
        "name": "com.omnitagger.host",
        "description": "OmniTagger Native Messaging Host",
        "path": native_host_path,
        "type": "stdio",
        "allowed_extensions": [
            extension_id
        ]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "linux")]
    fn test_generate_desktop_file_content() {
        let content = generate_desktop_file_content("/usr/bin/omni-tagger");
        assert!(content.contains("Exec=\"/usr/bin/omni-tagger\" %F"));
        assert!(content.contains("[Desktop Entry]"));
        assert!(content.contains("MimeType=image/jpeg;"));
    }

    #[test]
    fn test_generate_manifest_content() {
        let json = generate_manifest_content("/usr/bin/native_host", "abcdefg");

        assert_eq!(json["name"], "com.omnitagger.host");
        assert_eq!(json["path"], "/usr/bin/native_host");
        assert_eq!(json["allowed_origins"][0], "chrome-extension://abcdefg/");
    }

    #[test]
    fn test_generate_firefox_manifest_content() {
        let json =
            generate_firefox_manifest_content("/usr/bin/native_host", "extension@omnitagger");

        assert_eq!(json["name"], "com.omnitagger.host");
        assert_eq!(json["path"], "/usr/bin/native_host");
        assert_eq!(json["allowed_extensions"][0], "extension@omnitagger");
        assert!(json.get("allowed_origins").is_none());
    }
}
