from playwright.sync_api import Page, expect, sync_playwright
import time

def verify_overlay(page: Page):
    # Scenario 1: Main Window (Settings)
    page.add_init_script("""
      window.__TAURI_INTERNALS__ = {
        metadata: {
          currentWindow: { label: "main" }
        },
        invoke: async (cmd, args) => {
            console.log("Invoke:", cmd, args);
            if (cmd === 'get_config') return {
                model_path: "mock_model.onnx",
                tags_path: "mock_tags.csv",
                threshold: 0.35,
                use_underscore: false,
                exclusion_list: []
            };
            return null;
        },
        transformCallback: (c) => c
      };
    """)

    page.goto("http://localhost:1420")
    expect(page.get_by_text("OmniTagger Settings")).to_be_visible()
    print("Settings view verified.")

    # Scenario 2: Overlay
    page.goto("about:blank")
    page.add_init_script("""
      window.__TAURI_INTERNALS__ = {
        metadata: {
          currentWindow: { label: "overlay-1" }
        },
        invoke: async (cmd, args) => {
            console.log("Invoke called:", cmd, args);
            if (cmd === 'get_overlay_image') {
                return "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==";
            }
            if (cmd === 'process_selection') {
                return "tag1, tag2";
            }
            if (cmd === 'close_all_overlays') {
                return;
            }
            return null;
        },
        transformCallback: (c) => c
      };
    """)

    # Capture console logs
    logs = []
    page.on("console", lambda msg: logs.append(msg.text))

    page.goto("http://localhost:1420")

    expect(page.locator("img[alt='Screen Capture']")).to_be_visible()
    print("Overlay view verified.")

    # Simulate selection
    page.mouse.move(50, 50)
    page.mouse.down()
    page.mouse.move(150, 150)
    page.mouse.up()

    # Wait a bit for async process
    time.sleep(1)

    # Check logs for process_selection and close_all_overlays
    process_called = any("process_selection" in log for log in logs)
    close_called = any("close_all_overlays" in log for log in logs)

    if process_called:
        print("process_selection called.")
    else:
        print("process_selection NOT called.")

    if close_called:
        print("close_all_overlays called.")
    else:
        print("close_all_overlays NOT called.")

    page.screenshot(path="/home/jules/verification/overlay_selection.png")

if __name__ == "__main__":
    import os
    os.makedirs("/home/jules/verification", exist_ok=True)
    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True)
        page = browser.new_page()
        try:
            verify_overlay(page)
        except Exception as e:
            print(f"Error: {e}")
            page.screenshot(path="/home/jules/verification/error.png")
            raise
        finally:
            browser.close()
