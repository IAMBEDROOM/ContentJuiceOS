// ContentJuiceOS Browser Source Base Script
(function () {
  "use strict";

  var params = new URLSearchParams(window.location.search);
  var debug = params.get("debug") === "true";

  var statusIndicator = document.getElementById("status-indicator");
  var renderContainer = document.getElementById("render-container");

  if (debug && statusIndicator) {
    statusIndicator.style.display = "block";
  }

  function setStatus(state) {
    if (!statusIndicator) return;
    statusIndicator.className = state;
  }

  setStatus("disconnected");

  fetch("/config")
    .then(function (res) { return res.json(); })
    .then(function (config) {
      var socketUrl = "http://127.0.0.1:" + config.socketIoPort + "/overlays";

      var socket = io(socketUrl, {
        transports: ["websocket", "polling"],
        reconnection: true,
        reconnectionAttempts: 5,
        reconnectionDelay: 1000,
      });

      socket.on("connect", function () {
        console.log("[ContentJuiceOS] Connected to /overlays");
        setStatus("connected");
      });

      socket.on("disconnect", function (reason) {
        console.log("[ContentJuiceOS] Disconnected:", reason);
        setStatus("disconnected");
      });

      socket.on("reconnect_attempt", function (attempt) {
        console.log("[ContentJuiceOS] Reconnecting, attempt:", attempt);
        setStatus("reconnecting");
      });

      socket.on("connect_error", function (err) {
        console.error("[ContentJuiceOS] Connection error:", err.message);
        setStatus("disconnected");
      });

      socket.on("render", function (data) {
        if (data && typeof data.html === "string" && renderContainer) {
          renderContainer.innerHTML = data.html;
        }
      });

      window.addEventListener("beforeunload", function () {
        socket.disconnect();
      });

      window.ContentJuiceOS = {
        socket: socket,
        config: config,
        version: "0.1.0",
      };
    })
    .catch(function (err) {
      console.error("[ContentJuiceOS] Failed to fetch config:", err);
    });
})();
