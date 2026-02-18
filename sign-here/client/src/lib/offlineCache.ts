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
import type { Field as FieldType, Envelope, Recipient } from "@shared/schema";

const DB_NAME = "signhere-offline";
const DB_VERSION = 1;

interface PendingOp {
  id: string;
  envelopeId: string;
  type: "save_fields" | "sign_field";
  data: any;
  timestamp: number;
}

function openDB(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open(DB_NAME, DB_VERSION);
    request.onupgradeneeded = (event) => {
      const db = (event.target as IDBOpenDBRequest).result;
      if (!db.objectStoreNames.contains("envelopes")) {
        db.createObjectStore("envelopes", { keyPath: "id" });
      }
      if (!db.objectStoreNames.contains("fields")) {
        const fieldStore = db.createObjectStore("fields", { keyPath: "id" });
        fieldStore.createIndex("envelopeId", "envelopeId", { unique: false });
      }
      if (!db.objectStoreNames.contains("recipients")) {
        const recipientStore = db.createObjectStore("recipients", { keyPath: "id" });
        recipientStore.createIndex("envelopeId", "envelopeId", { unique: false });
      }
      if (!db.objectStoreNames.contains("pendingOps")) {
        db.createObjectStore("pendingOps", { keyPath: "id" });
      }
    };
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error);
  });
}

export async function cacheEnvelope(envelope: Envelope): Promise<void> {
  const db = await openDB();
  const tx = db.transaction("envelopes", "readwrite");
  tx.objectStore("envelopes").put(envelope);
  return new Promise((resolve, reject) => {
    tx.oncomplete = () => resolve();
    tx.onerror = () => reject(tx.error);
  });
}

export async function getCachedEnvelope(id: string): Promise<Envelope | null> {
  const db = await openDB();
  const tx = db.transaction("envelopes", "readonly");
  const request = tx.objectStore("envelopes").get(id);
  return new Promise((resolve) => {
    request.onsuccess = () => resolve(request.result || null);
    request.onerror = () => resolve(null);
  });
}

export async function cacheFields(envelopeId: string, fields: FieldType[]): Promise<void> {
  const db = await openDB();
  const tx = db.transaction("fields", "readwrite");
  const store = tx.objectStore("fields");
  const index = store.index("envelopeId");
  const existingReq = index.getAllKeys(envelopeId);
  existingReq.onsuccess = () => {
    const keys = existingReq.result;
    for (const key of keys) {
      store.delete(key);
    }
    for (const field of fields) {
      store.put(field);
    }
  };
  return new Promise((resolve, reject) => {
    tx.oncomplete = () => resolve();
    tx.onerror = () => reject(tx.error);
  });
}

export async function getCachedFields(envelopeId: string): Promise<FieldType[]> {
  const db = await openDB();
  const tx = db.transaction("fields", "readonly");
  const index = tx.objectStore("fields").index("envelopeId");
  const request = index.getAll(envelopeId);
  return new Promise((resolve) => {
    request.onsuccess = () => resolve(request.result || []);
    request.onerror = () => resolve([]);
  });
}

export async function cacheRecipients(envelopeId: string, recipients: Recipient[]): Promise<void> {
  const db = await openDB();
  const tx = db.transaction("recipients", "readwrite");
  const store = tx.objectStore("recipients");
  const index = store.index("envelopeId");
  const existingReq = index.getAllKeys(envelopeId);
  existingReq.onsuccess = () => {
    const keys = existingReq.result;
    for (const key of keys) {
      store.delete(key);
    }
    for (const r of recipients) {
      store.put(r);
    }
  };
  return new Promise((resolve, reject) => {
    tx.oncomplete = () => resolve();
    tx.onerror = () => reject(tx.error);
  });
}

export async function getCachedRecipients(envelopeId: string): Promise<Recipient[]> {
  const db = await openDB();
  const tx = db.transaction("recipients", "readonly");
  const index = tx.objectStore("recipients").index("envelopeId");
  const request = index.getAll(envelopeId);
  return new Promise((resolve) => {
    request.onsuccess = () => resolve(request.result || []);
    request.onerror = () => resolve([]);
  });
}

export async function addPendingOp(op: PendingOp): Promise<void> {
  const db = await openDB();
  const tx = db.transaction("pendingOps", "readwrite");
  tx.objectStore("pendingOps").put(op);
  return new Promise((resolve, reject) => {
    tx.oncomplete = () => resolve();
    tx.onerror = () => reject(tx.error);
  });
}

export async function getPendingOps(): Promise<PendingOp[]> {
  const db = await openDB();
  const tx = db.transaction("pendingOps", "readonly");
  const request = tx.objectStore("pendingOps").getAll();
  return new Promise((resolve) => {
    request.onsuccess = () => resolve(request.result || []);
    request.onerror = () => resolve([]);
  });
}

export async function removePendingOp(id: string): Promise<void> {
  const db = await openDB();
  const tx = db.transaction("pendingOps", "readwrite");
  tx.objectStore("pendingOps").delete(id);
  return new Promise((resolve, reject) => {
    tx.oncomplete = () => resolve();
    tx.onerror = () => reject(tx.error);
  });
}

export async function syncPendingOps(): Promise<number> {
  const ops = await getPendingOps();
  let synced = 0;

  for (const op of ops) {
    try {
      if (op.type === "save_fields") {
        const res = await fetch(`/api/envelopes/${op.envelopeId}/fields`, {
          method: "PUT",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ fields: op.data }),
        });
        if (res.ok) {
          await removePendingOp(op.id);
          synced++;
        }
      } else if (op.type === "sign_field") {
        const res = await fetch(`/api/envelopes/${op.envelopeId}/sign`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(op.data),
        });
        if (res.ok) {
          await removePendingOp(op.id);
          synced++;
        }
      }
    } catch {
    }
  }

  return synced;
}

export function useOnlineStatus() {
  const getStatus = () => navigator.onLine;
  return getStatus;
}
