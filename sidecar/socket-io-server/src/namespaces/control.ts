import type { Namespace } from "socket.io";

export function registerControlNamespace(nsp: Namespace): void {
  nsp.on("connection", (socket) => {
    console.log(`[control] client connected: ${socket.id}`);

    socket.on("ping", (data, callback) => {
      const response = {
        message: "pong",
        timestamp: new Date().toISOString(),
        receivedData: data,
      };
      if (typeof callback === "function") {
        callback(response);
      } else {
        socket.emit("pong", response);
      }
    });

    socket.on("disconnect", (reason) => {
      console.log(`[control] client disconnected: ${socket.id} (${reason})`);
    });
  });
}
