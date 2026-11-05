/**
 * Page API: window.SignBridge.request({ method, ... })
 * Only available on allowlisted origins (see manifest host_permissions).
 */
(function () {
  const ORIGIN = window.location.origin;

  window.addEventListener("message", async (event) => {
    if (event.source !== window) return;
    const data = event.data;
    if (!data || data.channel !== "signbridge" || data.direction !== "request") {
      return;
    }

    const payload = {
      ...data.payload,
      origin: ORIGIN,
      id: data.payload?.id || String(data.requestId),
    };

    let reply;
    try {
      const result = await chrome.runtime.sendMessage({
        type: "signbridge",
        payload,
      });
      reply = {
        channel: "signbridge",
        direction: "response",
        requestId: data.requestId,
        ...result,
      };
    } catch (err) {
      reply = {
        channel: "signbridge",
        direction: "response",
        requestId: data.requestId,
        ok: false,
        error: String(err),
      };
    }
    window.postMessage(reply, ORIGIN);
  });

  // Convenience helper injected for pages that prefer a Promise API via a custom event.
  window.dispatchEvent(
    new CustomEvent("signbridge-ready", {
      detail: { version: "0.1.0", origin: ORIGIN },
    }),
  );
})();
