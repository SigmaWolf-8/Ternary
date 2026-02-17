import { WebSocketServer, WebSocket } from "ws";
import type { Server as HttpServer } from "http";
import type { IncomingMessage } from "http";

export interface WsOp {
  type: "field:add" | "field:update" | "field:delete" | "field:sync" | "presence:update" | "presence:leave" | "signer:activity" | "cursor:move";
  envelopeId: string;
  data: any;
  userId?: string;
  userName?: string;
  timestamp?: number;
}

interface RoomClient {
  ws: WebSocket;
  userId: string;
  userName: string;
  color: string;
  joinedAt: number;
}

const PRESENCE_COLORS = [
  "#f59e0b", "#10b981", "#8b5cf6", "#3b82f6",
  "#ef4444", "#06b6d4", "#ec4899", "#14b8a6",
];

const rooms = new Map<string, Map<string, RoomClient>>();

function getRoom(envelopeId: string): Map<string, RoomClient> {
  if (!rooms.has(envelopeId)) {
    rooms.set(envelopeId, new Map());
  }
  return rooms.get(envelopeId)!;
}

function broadcastToRoom(envelopeId: string, message: WsOp, excludeUserId?: string) {
  const room = rooms.get(envelopeId);
  if (!room) return;
  const payload = JSON.stringify(message);
  room.forEach((client, uid) => {
    if (uid !== excludeUserId && client.ws.readyState === WebSocket.OPEN) {
      client.ws.send(payload);
    }
  });
}

function getPresenceList(envelopeId: string): Array<{ userId: string; userName: string; color: string }> {
  const room = rooms.get(envelopeId);
  if (!room) return [];
  return Array.from(room.values()).map((c) => ({
    userId: c.userId,
    userName: c.userName,
    color: c.color,
  }));
}

export function setupWebSocket(httpServer: HttpServer) {
  const wss = new WebSocketServer({ noServer: true });

  httpServer.on("upgrade", (request: IncomingMessage, socket, head) => {
    const url = new URL(request.url || "/", `http://${request.headers.host}`);
    if (url.pathname === "/ws/collab") {
      wss.handleUpgrade(request, socket, head, (ws) => {
        wss.emit("connection", ws, request);
      });
    }
  });

  wss.on("connection", (ws: WebSocket, req: IncomingMessage) => {
    const url = new URL(req.url || "/", `http://${req.headers.host}`);
    const envelopeId = url.searchParams.get("envelopeId") || "";
    const userId = url.searchParams.get("userId") || `anon-${Date.now()}`;
    const userName = url.searchParams.get("userName") || "Anonymous";

    if (!envelopeId) {
      ws.close(4000, "Missing envelopeId");
      return;
    }

    const room = getRoom(envelopeId);
    const colorIdx = room.size % PRESENCE_COLORS.length;
    const client: RoomClient = {
      ws,
      userId,
      userName,
      color: PRESENCE_COLORS[colorIdx],
      joinedAt: Date.now(),
    };
    room.set(userId, client);

    ws.send(JSON.stringify({
      type: "presence:init",
      envelopeId,
      data: {
        yourColor: client.color,
        users: getPresenceList(envelopeId),
      },
    }));

    broadcastToRoom(envelopeId, {
      type: "presence:update",
      envelopeId,
      data: { users: getPresenceList(envelopeId) },
    }, userId);

    ws.on("message", (raw) => {
      try {
        const msg: WsOp = JSON.parse(raw.toString());
        msg.userId = userId;
        msg.userName = userName;
        msg.timestamp = Date.now();

        switch (msg.type) {
          case "field:add":
          case "field:update":
          case "field:delete":
          case "field:sync":
            broadcastToRoom(envelopeId, msg, userId);
            break;
          case "cursor:move":
            broadcastToRoom(envelopeId, msg, userId);
            break;
          case "signer:activity":
            broadcastToRoom(envelopeId, msg, userId);
            break;
          default:
            break;
        }
      } catch {}
    });

    ws.on("close", () => {
      room.delete(userId);
      if (room.size === 0) {
        rooms.delete(envelopeId);
      } else {
        broadcastToRoom(envelopeId, {
          type: "presence:leave",
          envelopeId,
          data: {
            userId,
            userName,
            users: getPresenceList(envelopeId),
          },
        });
      }
    });

    ws.on("error", () => {
      room.delete(userId);
    });
  });

  return wss;
}
