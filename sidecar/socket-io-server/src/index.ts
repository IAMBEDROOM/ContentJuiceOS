import { createServer } from "http";
import { Server } from "socket.io";
import { registerOverlaysNamespace } from "./namespaces/overlays.js";
import { registerControlNamespace } from "./namespaces/control.js";

function parsePort(args: string[]): number {
  const portIndex = args.indexOf("--port");
  if (portIndex !== -1 && args[portIndex + 1]) {
    const parsed = parseInt(args[portIndex + 1], 10);
    if (!isNaN(parsed) && parsed > 0 && parsed < 65536) {
      return parsed;
    }
  }
  return 4849;
}

const port = parsePort(process.argv);

const httpServer = createServer();
const io = new Server(httpServer, {
  cors: {
    origin: "*",
    methods: ["GET", "POST"],
  },
  transports: ["websocket", "polling"],
});

const overlaysNsp = io.of("/overlays");
const controlNsp = io.of("/control");
registerOverlaysNamespace(overlaysNsp);
registerControlNamespace(controlNsp, overlaysNsp);

httpServer.listen(port, "127.0.0.1", () => {
  // Rust sidecar manager watches for this exact line
  console.log(`READY:${port}`);
});

function shutdown() {
  console.log("Shutting down Socket.IO server...");
  io.close(() => {
    httpServer.close(() => {
      process.exit(0);
    });
  });
  // Force exit after 3 seconds if graceful shutdown hangs
  setTimeout(() => process.exit(0), 3000);
}

process.on("SIGTERM", shutdown);
process.on("SIGINT", shutdown);
