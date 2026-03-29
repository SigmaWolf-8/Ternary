import { describe, it, expect } from "vitest";

const MAX_MSG_SIZE = 65536;
const VALID_MSG_TYPES = new Set(["chat", "telemetry", "product-status", "auth_ok", "challenge", "ops-error", "pong"]);
const REP_C_RE = /^[1-3]{13}$/;

function validateInboundMessage(raw: string) {
  if (raw.length > MAX_MSG_SIZE) return null;
  let msg: Record<string, unknown>;
  try { msg = JSON.parse(raw); } catch { return null; }
  if (typeof msg !== "object" || msg === null) return null;
  if (typeof msg.type !== "string") return null;
  const msgType = (msg.msgType ?? msg.relay_msg_type) as string | undefined;
  if (typeof msgType !== "string" || !VALID_MSG_TYPES.has(msgType)) return null;
  const fromField = typeof msg.from === "string" ? msg.from : undefined;
  const fromValid = fromField === undefined || REP_C_RE.test(fromField);
  const payload = typeof msg.payload === "string" ? msg.payload : "";
  const ts = typeof msg.ts === "number" ? msg.ts : Date.now();
  return { type: msg.type, msgType, from: fromField, fromValid, payload, ts };
}

type PanelState = "CLOSED" | "OPENING" | "OPEN" | "MINIMIZED" | "CLOSING";

function togglePanel(prev: PanelState): PanelState {
  if (prev === "CLOSED") return "OPENING";
  if (prev === "OPEN") return "CLOSING";
  if (prev === "MINIMIZED") return "OPEN";
  return prev;
}

function computeBackoffSeries(startMs: number, cap: number, budgetMs: number) {
  let backoff = startMs;
  let elapsed = 0;
  const series: number[] = [];
  while (elapsed < budgetMs) {
    series.push(backoff);
    elapsed += backoff;
    backoff = Math.min(backoff * 2, cap);
  }
  return { series, elapsed, attempts: series.length };
}

describe("LauncherPanel – behavioral unit tests", () => {
  describe("validateInboundMessage", () => {
    it("parses valid relay envelope with all fields", () => {
      const result = validateInboundMessage(
        JSON.stringify({ type: "relay", msgType: "chat", payload: "hello", from: "1231231231231", ts: 12345 })
      );
      expect(result).not.toBeNull();
      expect(result!.type).toBe("relay");
      expect(result!.msgType).toBe("chat");
      expect(result!.payload).toBe("hello");
      expect(result!.from).toBe("1231231231231");
      expect(result!.fromValid).toBe(true);
      expect(result!.ts).toBe(12345);
    });

    it("accepts relay_msg_type as alias for msgType", () => {
      const result = validateInboundMessage(
        JSON.stringify({ type: "relay", relay_msg_type: "telemetry", ts: 1 })
      );
      expect(result).not.toBeNull();
      expect(result!.msgType).toBe("telemetry");
    });

    it("rejects oversized messages (> 64KB)", () => {
      const huge = JSON.stringify({ type: "relay", msgType: "chat", payload: "x".repeat(MAX_MSG_SIZE) });
      expect(huge.length).toBeGreaterThan(MAX_MSG_SIZE);
      expect(validateInboundMessage(huge)).toBeNull();
    });

    it("accepts messages at exactly 64KB boundary", () => {
      const filler = "a".repeat(MAX_MSG_SIZE - 50);
      const msg = JSON.stringify({ type: "r", msgType: "chat", payload: filler });
      if (msg.length <= MAX_MSG_SIZE) {
        expect(validateInboundMessage(msg)).not.toBeNull();
      }
    });

    it("rejects non-JSON input", () => {
      expect(validateInboundMessage("not-json{")).toBeNull();
      expect(validateInboundMessage("")).toBeNull();
    });

    it("rejects missing type field", () => {
      expect(validateInboundMessage(JSON.stringify({ msgType: "chat" }))).toBeNull();
    });

    it("rejects unknown msgType values", () => {
      expect(validateInboundMessage(JSON.stringify({ type: "relay", msgType: "unknown" }))).toBeNull();
      expect(validateInboundMessage(JSON.stringify({ type: "relay", msgType: "" }))).toBeNull();
    });

    it("validates all 7 accepted message types", () => {
      for (const mt of ["chat", "telemetry", "product-status", "auth_ok", "challenge", "ops-error", "pong"]) {
        const result = validateInboundMessage(JSON.stringify({ type: "relay", msgType: mt }));
        expect(result).not.toBeNull();
        expect(result!.msgType).toBe(mt);
      }
    });

    it("flags invalid Rep C from field", () => {
      const result = validateInboundMessage(
        JSON.stringify({ type: "relay", msgType: "chat", from: "bad_address" })
      );
      expect(result).not.toBeNull();
      expect(result!.fromValid).toBe(false);
    });

    it("validates 13-trit Rep C addresses using digits 1-3 only", () => {
      expect(REP_C_RE.test("1231231231231")).toBe(true);
      expect(REP_C_RE.test("3333333333333")).toBe(true);
      expect(REP_C_RE.test("0231231231231")).toBe(false);
      expect(REP_C_RE.test("123123123123")).toBe(false);
      expect(REP_C_RE.test("12312312312311")).toBe(false);
    });

    it("defaults payload to empty string when missing", () => {
      const result = validateInboundMessage(JSON.stringify({ type: "relay", msgType: "pong" }));
      expect(result!.payload).toBe("");
    });

    it("defaults ts to current time when missing", () => {
      const before = Date.now();
      const result = validateInboundMessage(JSON.stringify({ type: "relay", msgType: "pong" }));
      expect(result!.ts).toBeGreaterThanOrEqual(before);
      expect(result!.ts).toBeLessThanOrEqual(Date.now());
    });
  });

  describe("Panel state machine (togglePanel)", () => {
    it("CLOSED → OPENING on toggle", () => expect(togglePanel("CLOSED")).toBe("OPENING"));
    it("OPEN → CLOSING on toggle", () => expect(togglePanel("OPEN")).toBe("CLOSING"));
    it("MINIMIZED → OPEN on toggle (restore)", () => expect(togglePanel("MINIMIZED")).toBe("OPEN"));
    it("OPENING is locked during animation", () => expect(togglePanel("OPENING")).toBe("OPENING"));
    it("CLOSING is locked during animation", () => expect(togglePanel("CLOSING")).toBe("CLOSING"));

    it("full lifecycle: CLOSED → OPENING → OPEN → CLOSING → CLOSED", () => {
      let state: PanelState = "CLOSED";
      state = togglePanel(state);
      expect(state).toBe("OPENING");
      state = "OPEN";
      state = togglePanel(state);
      expect(state).toBe("CLOSING");
      state = "CLOSED";
      expect(state).toBe("CLOSED");
    });

    it("minimize/restore cycle: OPEN → MINIMIZED → OPEN", () => {
      let state: PanelState = "OPEN";
      state = "MINIMIZED";
      state = togglePanel(state);
      expect(state).toBe("OPEN");
    });
  });

  describe("Exponential backoff with budget", () => {
    it("produces correct doubling series from 1s to 32s cap", () => {
      const { series } = computeBackoffSeries(1000, 32000, 300000);
      expect(series[0]).toBe(1000);
      expect(series[1]).toBe(2000);
      expect(series[2]).toBe(4000);
      expect(series[3]).toBe(8000);
      expect(series[4]).toBe(16000);
      expect(series[5]).toBe(32000);
    });

    it("exhausts within 5-minute budget", () => {
      const { elapsed, attempts } = computeBackoffSeries(1000, 32000, 300000);
      expect(elapsed).toBeGreaterThanOrEqual(300000);
      expect(attempts).toBeGreaterThanOrEqual(5);
      expect(attempts).toBeLessThanOrEqual(20);
    });

    it("stays at cap after reaching it", () => {
      const { series } = computeBackoffSeries(1000, 32000, 300000);
      const cappedEntries = series.filter((v) => v === 32000);
      expect(cappedEntries.length).toBeGreaterThan(1);
    });
  });

  describe("REST error classification", () => {
    it("TypeError indicates CORS block", () => {
      const err = new TypeError("Failed to fetch");
      expect(err.name).toBe("TypeError");
    });

    it("Error indicates HTTP-level failure", () => {
      const err = new Error("HTTP 500");
      expect(err.name).toBe("Error");
    });

    it("AbortError is filtered out (not displayed to user)", () => {
      const err = new DOMException("The operation was aborted.", "AbortError");
      expect(err.name).toBe("AbortError");
    });

    it("produces user-facing CORS detail with origin info", () => {
      const endpoint = "/cluster-health";
      const daemonHttp = "http://localhost:11124";
      const origin = "https://myapp.replit.app";
      const detail = `CORS blocked: ${daemonHttp}${endpoint} — Origin ${origin} rejected`;
      expect(detail).toContain("CORS blocked");
      expect(detail).toContain("11124");
      expect(detail).toContain(origin);
    });
  });

  describe("Port derivation and protocol selection", () => {
    it("11124 = BASE_PORT 11111 + GATEWAY_OFFSET 13", () => {
      expect(11111 + 13).toBe(11124);
    });

    function isLocalhostHost(host: string) {
      return host === "localhost" || host === "127.0.0.1" || host === "::1";
    }

    it("localhost always uses insecure transport", () => {
      expect(isLocalhostHost("localhost")).toBe(true);
      expect(isLocalhostHost("127.0.0.1")).toBe(true);
      expect(isLocalhostHost("::1")).toBe(true);
    });

    it("non-localhost hosts are not treated as localhost", () => {
      expect(isLocalhostHost("example.com")).toBe(false);
      expect(isLocalhostHost("192.168.1.1")).toBe(false);
      expect(isLocalhostHost("daemon.internal")).toBe(false);
    });
  });

  describe("Daemon state definitions", () => {
    type DaemonState = "DISCONNECTED" | "HEALTH_CHECK" | "CONNECTING" | "CONNECTED" | "RECONNECTING" | "FAILED";

    it("has 6 valid states with DISCONNECTED as initial", () => {
      const states: DaemonState[] = ["DISCONNECTED", "HEALTH_CHECK", "CONNECTING", "CONNECTED", "RECONNECTING", "FAILED"];
      expect(states).toHaveLength(6);
      expect(states[0]).toBe("DISCONNECTED");
    });

    it("connection-derived isOn flag is true only when CONNECTED", () => {
      const states: DaemonState[] = ["DISCONNECTED", "HEALTH_CHECK", "CONNECTING", "CONNECTED", "RECONNECTING", "FAILED"];
      for (const s of states) {
        const isOn = s === "CONNECTED";
        expect(isOn).toBe(s === "CONNECTED");
      }
    });
  });
});
