/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Applied Physics Division
 *
 * PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
 * Patent(s) Pending.
 *
 * This file is part of the Salvi Framework / PlenumNET platform.
 * Unauthorized copying, modification, distribution, or use of this file,
 * via any medium, is strictly prohibited without the prior written
 * permission of Capomastro Holdings Ltd.
 *
 * See LICENSE in the repository root for full terms.
 */
import { useState, useEffect, useRef, useCallback } from "react";
import type { Field as FieldType } from "@shared/schema";

export interface CollabUser {
  userId: string;
  userName: string;
  color: string;
}

export interface CursorPosition {
  userId: string;
  userName: string;
  color: string;
  page: number;
  x: number;
  y: number;
}

export interface SignerActivity {
  userId: string;
  userName: string;
  recipientId: string;
  fieldId: string;
  action: "focus" | "signed" | "viewing";
  timestamp: number;
}

interface WsMessage {
  type: string;
  envelopeId: string;
  data: any;
  userId?: string;
  userName?: string;
  timestamp?: number;
}

interface UseCollabOptions {
  envelopeId: string;
  userId: string;
  userName: string;
  enabled?: boolean;
  onRemoteFieldAdd?: (field: FieldType) => void;
  onRemoteFieldUpdate?: (field: Partial<FieldType> & { id: string }) => void;
  onRemoteFieldDelete?: (fieldId: string) => void;
  onRemoteFieldSync?: (fields: FieldType[]) => void;
  onConflict?: (info: { fieldId: string; userName: string; action: string }) => void;
}

export function useCollab({
  envelopeId,
  userId,
  userName,
  enabled = true,
  onRemoteFieldAdd,
  onRemoteFieldUpdate,
  onRemoteFieldDelete,
  onRemoteFieldSync,
  onConflict,
}: UseCollabOptions) {
  const wsRef = useRef<WebSocket | null>(null);
  const [connected, setConnected] = useState(false);
  const [presence, setPresence] = useState<CollabUser[]>([]);
  const [myColor, setMyColor] = useState("#f59e0b");
  const [cursors, setCursors] = useState<Map<string, CursorPosition>>(new Map());
  const [signerActivities, setSignerActivities] = useState<SignerActivity[]>([]);
  const reconnectTimer = useRef<NodeJS.Timeout | null>(null);
  const recentLocalOps = useRef<Set<string>>(new Set());

  const connect = useCallback(() => {
    if (!enabled || !envelopeId) return;

    const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
    const url = `${protocol}//${window.location.host}/ws/collab?envelopeId=${encodeURIComponent(envelopeId)}&userId=${encodeURIComponent(userId)}&userName=${encodeURIComponent(userName)}`;

    const ws = new WebSocket(url);
    wsRef.current = ws;

    ws.onopen = () => {
      setConnected(true);
      if (reconnectTimer.current) {
        clearTimeout(reconnectTimer.current);
        reconnectTimer.current = null;
      }
    };

    ws.onmessage = (event) => {
      try {
        const msg: WsMessage = JSON.parse(event.data);
        switch (msg.type) {
          case "presence:init":
            setMyColor(msg.data.yourColor);
            setPresence(msg.data.users);
            break;
          case "presence:update":
          case "presence:leave":
            setPresence(msg.data.users);
            break;
          case "field:add":
            onRemoteFieldAdd?.(msg.data.field);
            if (msg.userName) {
              onConflict?.({ fieldId: msg.data.field.id, userName: msg.userName, action: "added" });
            }
            break;
          case "field:update": {
            const opKey = `${msg.data.field?.id}-${msg.timestamp}`;
            if (!recentLocalOps.current.has(opKey)) {
              onRemoteFieldUpdate?.(msg.data.field);
              if (msg.userName) {
                onConflict?.({ fieldId: msg.data.field.id, userName: msg.userName, action: "moved" });
              }
            }
            break;
          }
          case "field:delete":
            onRemoteFieldDelete?.(msg.data.fieldId);
            if (msg.userName) {
              onConflict?.({ fieldId: msg.data.fieldId, userName: msg.userName, action: "deleted" });
            }
            break;
          case "field:sync":
            onRemoteFieldSync?.(msg.data.fields);
            break;
          case "cursor:move":
            if (msg.userId) {
              setCursors((prev) => {
                const next = new Map(prev);
                next.set(msg.userId!, {
                  userId: msg.userId!,
                  userName: msg.userName || "User",
                  color: msg.data.color || "#f59e0b",
                  page: msg.data.page,
                  x: msg.data.x,
                  y: msg.data.y,
                });
                return next;
              });
            }
            break;
          case "signer:activity":
            setSignerActivities((prev) => {
              const filtered = prev.filter((a) => a.userId !== msg.userId);
              return [...filtered, { ...msg.data, userId: msg.userId, userName: msg.userName, timestamp: msg.timestamp }];
            });
            break;
        }
      } catch {}
    };

    ws.onclose = () => {
      setConnected(false);
      wsRef.current = null;
      if (enabled) {
        reconnectTimer.current = setTimeout(connect, 2000);
      }
    };

    ws.onerror = () => {
      ws.close();
    };
  }, [envelopeId, userId, userName, enabled, onRemoteFieldAdd, onRemoteFieldUpdate, onRemoteFieldDelete, onRemoteFieldSync, onConflict]);

  useEffect(() => {
    connect();
    return () => {
      if (reconnectTimer.current) clearTimeout(reconnectTimer.current);
      if (wsRef.current) {
        wsRef.current.close();
        wsRef.current = null;
      }
    };
  }, [connect]);

  const send = useCallback((msg: Omit<WsMessage, "envelopeId">) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({ ...msg, envelopeId }));
    }
  }, [envelopeId]);

  const emitFieldAdd = useCallback((field: FieldType) => {
    send({ type: "field:add", data: { field } });
  }, [send]);

  const emitFieldUpdate = useCallback((field: Partial<FieldType> & { id: string }) => {
    send({ type: "field:update", data: { field } });
  }, [send]);

  const emitFieldDelete = useCallback((fieldId: string) => {
    send({ type: "field:delete", data: { fieldId } });
  }, [send]);

  const emitFieldSync = useCallback((fields: FieldType[]) => {
    send({ type: "field:sync", data: { fields } });
  }, [send]);

  const emitCursorMove = useCallback((page: number, x: number, y: number) => {
    send({ type: "cursor:move", data: { page, x, y, color: myColor } });
  }, [send, myColor]);

  const emitSignerActivity = useCallback((recipientId: string, fieldId: string, action: "focus" | "signed" | "viewing") => {
    send({ type: "signer:activity", data: { recipientId, fieldId, action } });
  }, [send]);

  return {
    connected,
    presence,
    myColor,
    cursors,
    signerActivities,
    emitFieldAdd,
    emitFieldUpdate,
    emitFieldDelete,
    emitFieldSync,
    emitCursorMove,
    emitSignerActivity,
  };
}
