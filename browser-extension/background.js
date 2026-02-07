chrome.runtime.onInstalled.addListener(() => {
  chrome.contextMenus.create({
    id: "omni-tagger-get-tags",
    title: "Get Tags",
    contexts: ["image"]
  });
});

chrome.contextMenus.onClicked.addListener((info, tab) => {
  if (info.menuItemId === "omni-tagger-get-tags" && info.srcUrl) {
    console.log("Sending URL to native host:", info.srcUrl);
    chrome.runtime.sendNativeMessage(
      "com.omnitagger.host",
      { url: info.srcUrl },
      (response) => {
        if (chrome.runtime.lastError) {
          console.error("Native Messaging Error:", chrome.runtime.lastError);
        } else {
          console.log("Response:", response);
        }
      }
    );
  }
});
