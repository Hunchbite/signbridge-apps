const HOST_NAME = "com.hunchbite.signbridge";

function sendNative(message) {
  return new Promise((resolve, reject) => {
    chrome.runtime.sendNativeMessage(HOST_NAME, message, (response) => {
      if (chrome.runtime.lastError) {
        reject(new Error(chrome.runtime.lastError.message));
        return;
      }
      resolve(response);
    });
  });
}

chrome.runtime.onMessage.addListener((msg, _sender, sendResponse) => {
  if (!msg || msg.type !== "signbridge") {
    return false;
  }
  sendNative(msg.payload)
    .then((response) => sendResponse({ ok: true, response }))
    .catch((err) => sendResponse({ ok: false, error: String(err.message || err) }));
  return true;
});
