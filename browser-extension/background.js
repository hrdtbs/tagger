chrome.runtime.onInstalled.addListener(() => {
  chrome.contextMenus.create({
    id: "omni-tagger-get-tags",
    title: "Get Tags",
    contexts: ["image"]
  });
});

function showNotification(title, message) {
  chrome.notifications.create({
    type: "basic",
    iconUrl: "icons/icon48.png",
    title: title,
    message: message,
    priority: 1
  });
}

// Helper to handle native messaging
function sendToNativeHost(message) {
  console.log("Sending to native host:", message.url ? "URL" : "Data URI");
  chrome.runtime.sendNativeMessage(
    "com.omnitagger.host",
    message,
    (response) => {
      if (chrome.runtime.lastError) {
        console.error("Native Messaging Error:", chrome.runtime.lastError);
        showNotification("Connection Error", "Could not connect to OmniTagger. Make sure the app is installed and native host is registered.");
      } else {
        console.log("Response:", response);
        if (response && response.status === "error") {
          showNotification("Error", "OmniTagger reported an error: " + response.message);
        } else if (response && response.status === "ok") {
           showNotification("OmniTagger", response.message || "Processing started...");
        }
      }
    }
  );
}

// This function is injected into the page to fetch and resize the image
async function fetchAndProcessImage(url) {
  try {
    const response = await fetch(url);
    if (!response.ok) throw new Error("Fetch failed: " + response.statusText);
    const blob = await response.blob();
    const bitmap = await createImageBitmap(blob);

    // Resize to max 512px to ensure payload is small (< 1MB for Native Messaging)
    const maxDim = 512;
    let width = bitmap.width;
    let height = bitmap.height;

    if (width > maxDim || height > maxDim) {
      const scale = maxDim / Math.max(width, height);
      width = Math.round(width * scale);
      height = Math.round(height * scale);
    }

    // Use OffscreenCanvas if available, else document canvas
    let canvas;
    let ctx;
    if (typeof OffscreenCanvas !== 'undefined') {
        canvas = new OffscreenCanvas(width, height);
        ctx = canvas.getContext('2d');
    } else {
        canvas = document.createElement('canvas');
        canvas.width = width;
        canvas.height = height;
        ctx = canvas.getContext('2d');
    }

    ctx.drawImage(bitmap, 0, 0, width, height);

    // Convert to Data URI (JPEG 0.9)
    if (canvas instanceof OffscreenCanvas) {
        const outBlob = await canvas.convertToBlob({ type: 'image/jpeg', quality: 0.9 });
        return new Promise((resolve, reject) => {
            const reader = new FileReader();
            reader.onloadend = () => resolve(reader.result);
            reader.onerror = reject;
            reader.readAsDataURL(outBlob);
        });
    } else {
        return canvas.toDataURL('image/jpeg', 0.9);
    }
  } catch (e) {
    console.error("Content script image processing error:", e);
    // Return null to indicate failure
    return null;
  }
}

chrome.contextMenus.onClicked.addListener((info, tab) => {
  if (info.menuItemId === "omni-tagger-get-tags" && info.srcUrl) {

    if (info.srcUrl.startsWith("data:")) {
      sendToNativeHost({ data: info.srcUrl });
      return;
    }

    // For http/https/blob, try to fetch and process in the tab context
    // This handles private images (auth cookies) and blob URLs, and resizes large images
    if (tab && tab.id) {
        chrome.scripting.executeScript({
            target: { tabId: tab.id },
            func: fetchAndProcessImage,
            args: [info.srcUrl]
        }, (results) => {
            if (chrome.runtime.lastError || !results || !results[0] || !results[0].result) {
                console.warn("Script execution failed or returned null:", chrome.runtime.lastError);
                // Fallback logic
                if (info.srcUrl.startsWith("blob:")) {
                     showNotification("Error", "Cannot process this image (Blob URL and script access failed).");
                } else {
                     console.log("Falling back to sending original URL");
                     sendToNativeHost({ url: info.srcUrl });
                }
            } else {
                console.log("Received processed data URI from content script");
                sendToNativeHost({ data: results[0].result });
            }
        });
    } else {
        // Fallback if no tab ID (unlikely for context menu?)
        console.warn("No tab ID available");
        if (!info.srcUrl.startsWith("blob:")) {
             sendToNativeHost({ url: info.srcUrl });
        }
    }
  }
});
