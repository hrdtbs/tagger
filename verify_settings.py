from playwright.sync_api import sync_playwright

def run(p):
    browser = p.chromium.launch(headless=True)
    page = browser.new_page()

    # Mock Tauri internals
    page.add_init_script("""
        window.__TAURI_INTERNALS__ = {};
        window.__TAURI_INTERNALS__.invoke = async (cmd, args) => {
            console.log(`[Mock Invoke] ${cmd}`, args);
            if (cmd === 'get_config') {
                return {
                    model_path: 'models/model.onnx',
                    tags_path: 'models/tags.csv',
                    threshold: 0.35,
                    use_underscore: false,
                    exclusion_list: [],
                    preprocessing: {
                        input_size: 448,
                        format: 'bgr',
                        normalize: false
                    }
                };
            }
            if (cmd === 'check_model_exists') {
                return true;
            }
            if (cmd === 'set_config') {
                console.log('set_config called', args);
                return;
            }
            return null;
        };

        window.__TAURI_INTERNALS__.plugins = {
            invoke: window.__TAURI_INTERNALS__.invoke
        };

        window.__TAURI__ = {
            invoke: window.__TAURI_INTERNALS__.invoke,
            event: {
                listen: async () => { return () => {}; }
            }
        };
    """)

    page.goto("http://localhost:1420")

    # Wait for settings to load
    page.wait_for_selector("text=OmniTagger Settings")

    print("Page loaded")

    # Expand Advanced Model Settings
    page.click("text=Advanced Model Settings")

    # Check if inputs are visible
    page.wait_for_selector("text=Input Size (px)")
    page.wait_for_selector("text=Color Format")
    page.wait_for_selector("text=Normalize (0-1 range)")

    # Wait a bit for transition
    page.wait_for_timeout(500)

    # Take screenshot
    page.screenshot(path="/home/jules/verification/settings_advanced.png")
    print("Screenshot saved to /home/jules/verification/settings_advanced.png")

    browser.close()

with sync_playwright() as p:
    run(p)
