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
import axios from "axios";

const PLENUM_BASE = "https://plenumnet.replit.app/api/salvi";
const API_KEY = process.env.PLENUM_API_KEY || "";

interface PlenumResponse<T> {
  data: T;
}

export async function secureDoc(
  payload: Buffer,
  tenantId: string
): Promise<{ id: string }> {
  try {
    const res = await axios.post<PlenumResponse<{ id: string }>>(
      `${PLENUM_BASE}/phase/split`,
      { payload: payload.toString("base64"), tenantId },
      { headers: { "x-api-key": API_KEY }, timeout: 15000 }
    );
    return res.data.data;
  } catch (err: any) {
    console.warn("PlenumDB secureDoc fallback:", err.message);
    return { id: `local-${Date.now()}-${Math.random().toString(36).slice(2, 8)}` };
  }
}

export async function witnessSign(
  fieldId: string,
  value: string,
  tenantId: string
): Promise<any> {
  try {
    const res = await axios.post(
      `${PLENUM_BASE}/witness/sign`,
      { fieldId, value, tenantId },
      { headers: { "x-api-key": API_KEY }, timeout: 10000 }
    );
    return res.data;
  } catch (err: any) {
    console.warn("Witness sign fallback:", err.message);
    return { witnessed: false, fallback: true, timestamp: new Date().toISOString() };
  }
}

export async function getHPTP(): Promise<string> {
  try {
    const res = await axios.get(`${PLENUM_BASE}/timing/self-test`, {
      headers: { "x-api-key": API_KEY },
      timeout: 5000,
    });
    return res.data.epoch || new Date().toISOString();
  } catch (err: any) {
    console.warn("HPTP timing fallback:", err.message);
    return new Date().toISOString();
  }
}

export async function mlDsaSign(
  payload: string,
  tenantId: string
): Promise<{ signature: string }> {
  try {
    const res = await axios.post(
      `${PLENUM_BASE}/crypto/ml-dsa`,
      { payload, tenantId },
      { headers: { "x-api-key": API_KEY }, timeout: 10000 }
    );
    return res.data;
  } catch (err: any) {
    console.warn("ML-DSA sign fallback:", err.message);
    return { signature: `fallback-sig-${Date.now()}` };
  }
}

export interface CNSA2Result {
  plenumDocId: string;
  mlDsaSignature: string;
  hptpTimestamp: string;
  phaseSplit: boolean;
  quantumSecured: boolean;
}

export async function cnsa2SecureDocument(
  base64Data: string,
  tenantId: string,
  documentHash: string
): Promise<CNSA2Result> {
  const hptpTimestamp = await getHPTP();

  const payload = Buffer.from(base64Data, "base64");
  const { id: plenumDocId } = await secureDoc(payload, tenantId);
  const phaseSplit = !plenumDocId.startsWith("local-");

  const sigPayload = JSON.stringify({
    documentHash,
    plenumDocId,
    timestamp: hptpTimestamp,
    tenantId,
  });
  const { signature: mlDsaSignature } = await mlDsaSign(sigPayload, tenantId);

  return {
    plenumDocId,
    mlDsaSignature,
    hptpTimestamp,
    phaseSplit,
    quantumSecured: phaseSplit && !mlDsaSignature.startsWith("fallback-"),
  };
}

export async function healthCheck(): Promise<{ status: string; timestamp: string; keyValid?: boolean }> {
  const timestamp = new Date().toISOString();
  let reachable = false;
  let keyValid = false;

  try {
    const res = await axios.get(`${PLENUM_BASE}/timing/self-test`, { timeout: 5000 });
    reachable = true;
  } catch {
    return { status: "unreachable", timestamp, keyValid: false };
  }

  if (API_KEY) {
    try {
      const res = await axios.get(`${PLENUM_BASE}/timing/hptp`, {
        headers: { "x-api-key": API_KEY },
        timeout: 5000,
      });
      keyValid = res.status === 200;
    } catch (err: any) {
      keyValid = false;
    }
  }

  return { status: "connected", timestamp, keyValid };
}
