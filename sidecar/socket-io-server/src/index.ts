import { createServer, IncomingMessage, ServerResponse } from "http";
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

const preferredPort = parsePort(process.argv);
const MAX_PORT_RETRIES = 10;

function setupEmitEndpoint(
  httpServer: ReturnType<typeof createServer>,
  io: Server,
): void {
  httpServer.on("request", (req: IncomingMessage, res: ServerResponse) => {
    if (req.method === "POST" && req.url === "/emit") {
      let body = "";
      req.on("data", (chunk: Buffer) => {
        body += chunk.toString();
      });
      req.on("end", () => {
        try {
          const { namespace, event, data } = JSON.parse(body);
          if (!namespace || !event) {
            res.writeHead(400, { "Content-Type": "application/json" });
            res.end(JSON.stringify({ error: "namespace and event are required" }));
            return;
          }
          io.of(namespace).emit(event, data);
          res.writeHead(200, { "Content-Type": "application/json" });
          res.end(JSON.stringify({ ok: true }));
        } catch {
          res.writeHead(400, { "Content-Type": "application/json" });
          res.end(JSON.stringify({ error: "Invalid JSON body" }));
        }
      });
    }
  });
}

function startServer(port: number, attempt: number): void {
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
  setupEmitEndpoint(httpServer, io);

  httpServer.on("error", (err: NodeJS.ErrnoException) => {
    if (err.code === "EADDRINUSE" && attempt < MAX_PORT_RETRIES) {
      const nextPort = port + 1;
      console.error(`Port ${port} in use, retrying on ${nextPort}...`);
      io.close();
      startServer(nextPort, attempt + 1);
    } else {
      console.error(`Fatal server error: ${err.message}`);
      process.exit(1);
    }
  });

  httpServer.listen(port, "127.0.0.1", () => {
    // Rust sidecar manager watches for this exact line
    console.log(`READY:${port}`);
  });

  setupShutdown(io, httpServer);
}

function setupShutdown(io: Server, httpServer: ReturnType<typeof createServer>): void {
  function shutdown() {
    console.log("Shutting down Socket.IO server...");
    io.close(() => {
      httpServer.close(() => {
        process.exit(0);
      });
    });
    setTimeout(() => process.exit(0), 3000);
  }

  process.on("SIGTERM", shutdown);
  process.on("SIGINT", shutdown);
}

startServer(preferredPort, 0);
