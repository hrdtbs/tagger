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

chrome.contextMenus.onClicked.addListener((info, tab) => {
  if (info.menuItemId === "omni-tagger-get-tags" && info.srcUrl) {
    console.log("Sending URL to native host:", info.srcUrl);

    // Send message to native host
    chrome.runtime.sendNativeMessage(
      "com.omnitagger.host",
      { url: info.srcUrl },
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
});
