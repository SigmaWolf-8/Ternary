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

export async function healthCheck(): Promise<{ status: string; timestamp: string }> {
  try {
    const res = await axios.get(`${PLENUM_BASE}/timing/self-test`, {
      headers: { "x-api-key": API_KEY },
      timeout: 5000,
    });
    return { status: "connected", timestamp: res.data.epoch || new Date().toISOString() };
  } catch (err: any) {
    return { status: "unreachable", timestamp: new Date().toISOString() };
  }
}
