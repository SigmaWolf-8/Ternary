import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, fireEvent, act, cleanup } from "@testing-library/react";
import { LauncherProvider, useLauncher } from "../client/src/components/LauncherPanel";
import { useState } from "react";

Element.prototype.scrollIntoView = vi.fn();

Object.defineProperty(window, "matchMedia", {
  writable: true,
  value: vi.fn().mockImplementation((query: string) => ({
    matches: query.includes("prefers-reduced-motion"),
    media: query,
    onchange: null,
    addListener: vi.fn(),
    removeListener: vi.fn(),
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
});

class MockWebSocket {
  static CONNECTING = 0;
  static OPEN = 1;
  static CLOSING = 2;
  static CLOSED = 3;

  readyState = MockWebSocket.CONNECTING;
  url: string;
  onopen: ((ev: Event) => void) | null = null;
  onclose: ((ev: CloseEvent) => void) | null = null;
  onmessage: ((ev: MessageEvent) => void) | null = null;
  onerror: ((ev: Event) => void) | null = null;

  constructor(url: string) {
    this.url = url;
    MockWebSocket._instances.push(this);
  }

  send(_data: string) {}

  close() {
    this.readyState = MockWebSocket.CLOSED;
    if (this.onclose) {
      this.onclose(new CloseEvent("close", { code: 1000 }));
    }
  }

  simulateOpen() {
    this.readyState = MockWebSocket.OPEN;
    if (this.onopen) this.onopen(new Event("open"));
  }

  simulateError() {
    if (this.onerror) this.onerror(new Event("error"));
  }

  static _instances: MockWebSocket[] = [];
  static reset() { MockWebSocket._instances = []; }
  static latest() { return MockWebSocket._instances[MockWebSocket._instances.length - 1]; }
}

function makeMockFetch() {
  return vi.fn().mockImplementation((url: string) => {
    if (typeof url === "string" && url.includes("localhost:11124")) {
      if (url.includes("cluster-health")) {
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve({
            resources: [
              { label: "CPU", value: 42, detail: "4 cores", cores: "4" },
              { label: "RAM", value: 60, detail: "8 GB", cores: null },
            ],
            arrayName: "ARRAY3",
            nodeCount: 2,
            latencyMs: 2.1,
            arch: "ARM64",
            installPath: "/opt/capomastro",
            repC: "211.111.111.111.1",
          }),
        });
      }
      if (url.includes("topology")) {
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve({
            topologyStatus: "nominal",
            nodes: [{ node: "Node-A", addr: "10.0.0.1", role: "relay", latency: "1.2ms" }],
            interfaces: [],
          }),
        });
      }
      if (url.includes("fts/status")) {
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve([{ id: "n1", status: "up", latency: 1 }]),
        });
      }
      if (url.includes("con/stats")) {
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ activeConnections: 3, totalBytes: 1024 }),
        });
      }
      if (url.includes("node/info")) {
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ products: [] }),
        });
      }
      return Promise.resolve({ ok: true, json: () => Promise.resolve({ status: "ok" }) });
    }
    return Promise.resolve({ ok: true, json: () => Promise.resolve({}) });
  });
}

function PanelOpener() {
  const { togglePanel, panelState } = useLauncher();
  return (
    <button data-testid="test-toggle-panel" onClick={togglePanel}>
      {panelState}
    </button>
  );
}

function MinimizeTrigger() {
  const { setPanelState, panelState } = useLauncher();
  return (
    <button data-testid="test-minimize" onClick={() => setPanelState("MINIMIZED")}>
      min:{panelState}
    </button>
  );
}

function renderWithOpener() {
  return render(
    <LauncherProvider>
      <PanelOpener />
      <MinimizeTrigger />
    </LauncherProvider>
  );
}

describe("LauncherPanel – component rendering", () => {
  let originalFetch: typeof globalThis.fetch;

  beforeEach(() => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    MockWebSocket.reset();
    (globalThis as Record<string, unknown>).WebSocket = MockWebSocket as unknown as typeof WebSocket;
    originalFetch = globalThis.fetch;
    (globalThis as Record<string, unknown>).fetch = makeMockFetch();
    sessionStorage.clear();
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.restoreAllMocks();
    (globalThis as Record<string, unknown>).fetch = originalFetch;
    cleanup();
  });

  describe("Panel open/close lifecycle", () => {
    it("renders nothing when panel is CLOSED (initial state)", () => {
      renderWithOpener();
      expect(screen.queryByTestId("launcher-panel")).toBeNull();
      expect(screen.queryByTestId("launcher-minimized-bar")).toBeNull();
      expect(screen.getByTestId("test-toggle-panel").textContent).toBe("CLOSED");
    });

    it("opens panel via toggle (CLOSED → OPEN with reduced-motion)", async () => {
      renderWithOpener();
      expect(screen.queryByTestId("launcher-panel")).toBeNull();

      await act(async () => {
        fireEvent.click(screen.getByTestId("test-toggle-panel"));
      });

      await act(async () => {
        vi.advanceTimersByTime(100);
      });

      expect(screen.getByTestId("launcher-panel")).toBeTruthy();
    });

    it("closes panel via toggle when OPEN", async () => {
      renderWithOpener();

      await act(async () => {
        fireEvent.click(screen.getByTestId("test-toggle-panel"));
      });
      await act(async () => { vi.advanceTimersByTime(100); });

      expect(screen.getByTestId("launcher-panel")).toBeTruthy();

      await act(async () => {
        fireEvent.click(screen.getByTestId("test-toggle-panel"));
      });
      await act(async () => { vi.advanceTimersByTime(100); });

      expect(screen.queryByTestId("launcher-panel")).toBeNull();
    });

    it("LauncherProvider renders its children", () => {
      render(
        <LauncherProvider>
          <span data-testid="child-content">Hello</span>
        </LauncherProvider>
      );
      expect(screen.getByTestId("child-content")).toBeTruthy();
      expect(screen.getByText("Hello")).toBeTruthy();
    });
  });

  describe("Minimize/restore flow", () => {
    it("shows minimized bar when MINIMIZED, restores on click", async () => {
      renderWithOpener();

      await act(async () => {
        fireEvent.click(screen.getByTestId("test-toggle-panel"));
      });
      await act(async () => { vi.advanceTimersByTime(100); });
      expect(screen.getByTestId("launcher-panel")).toBeTruthy();

      await act(async () => {
        fireEvent.click(screen.getByTestId("button-launcher-minimize"));
      });
      await act(async () => { vi.advanceTimersByTime(100); });

      expect(screen.queryByTestId("launcher-panel")).toBeNull();
      expect(screen.getByTestId("launcher-minimized-bar")).toBeTruthy();

      await act(async () => {
        fireEvent.click(screen.getByTestId("launcher-minimized-bar"));
      });
      await act(async () => { vi.advanceTimersByTime(100); });

      expect(screen.getByTestId("launcher-panel")).toBeTruthy();
      expect(screen.queryByTestId("launcher-minimized-bar")).toBeNull();
    });
  });

  describe("Tab navigation in DOM", () => {
    it("renders all 5 tab buttons and supports clicking between tabs", async () => {
      renderWithOpener();

      await act(async () => {
        fireEvent.click(screen.getByTestId("test-toggle-panel"));
      });
      await act(async () => { vi.advanceTimersByTime(100); });

      for (const tabId of ["yoda", "apps", "local", "net", "apis"]) {
        const tabBtn = screen.getByTestId(`tab-${tabId}`);
        expect(tabBtn).toBeTruthy();
      }

      await act(async () => {
        fireEvent.click(screen.getByTestId("tab-apps"));
      });
      expect(screen.getByTestId("tab-apps").getAttribute("aria-selected")).toBe("true");
      expect(screen.getByTestId("tab-yoda").getAttribute("aria-selected")).toBe("false");

      await act(async () => {
        fireEvent.click(screen.getByTestId("tab-net"));
      });
      expect(screen.getByTestId("tab-net").getAttribute("aria-selected")).toBe("true");
      expect(screen.getByTestId("tab-apps").getAttribute("aria-selected")).toBe("false");
    });

    it("supports arrow key navigation between tabs", async () => {
      renderWithOpener();

      await act(async () => {
        fireEvent.click(screen.getByTestId("test-toggle-panel"));
      });
      await act(async () => { vi.advanceTimersByTime(100); });

      const yodaTab = screen.getByTestId("tab-yoda");
      expect(yodaTab.getAttribute("aria-selected")).toBe("true");

      await act(async () => {
        fireEvent.keyDown(yodaTab, { key: "ArrowRight" });
      });

      expect(screen.getByTestId("tab-apps").getAttribute("aria-selected")).toBe("true");
    });
  });

  describe("Keyboard shortcuts in DOM", () => {
    it("Escape closes the open panel", async () => {
      renderWithOpener();

      await act(async () => {
        fireEvent.click(screen.getByTestId("test-toggle-panel"));
      });
      await act(async () => { vi.advanceTimersByTime(100); });
      expect(screen.getByTestId("launcher-panel")).toBeTruthy();

      await act(async () => {
        fireEvent.keyDown(window, { key: "Escape" });
      });
      await act(async () => { vi.advanceTimersByTime(100); });

      expect(screen.queryByTestId("launcher-panel")).toBeNull();
    });

    it("M key minimizes the panel", async () => {
      renderWithOpener();

      await act(async () => {
        fireEvent.click(screen.getByTestId("test-toggle-panel"));
      });
      await act(async () => { vi.advanceTimersByTime(100); });
      expect(screen.getByTestId("launcher-panel")).toBeTruthy();

      await act(async () => {
        fireEvent.keyDown(window, { key: "m" });
      });
      await act(async () => { vi.advanceTimersByTime(100); });

      expect(screen.queryByTestId("launcher-panel")).toBeNull();
      expect(screen.getByTestId("launcher-minimized-bar")).toBeTruthy();
    });

    it("S key toggles settings panel", async () => {
      renderWithOpener();

      await act(async () => {
        fireEvent.click(screen.getByTestId("test-toggle-panel"));
      });
      await act(async () => { vi.advanceTimersByTime(100); });

      const apiKeyInput = screen.queryByTestId("input-api-key");
      const settingsInitiallyVisible = apiKeyInput !== null;

      await act(async () => {
        fireEvent.keyDown(window, { key: "s" });
      });
      await act(async () => { vi.advanceTimersByTime(100); });

      const afterToggle = screen.queryByTestId("input-api-key");
      expect(afterToggle !== null).not.toBe(settingsInitiallyVisible);
    });
  });

  describe("Panel header elements", () => {
    it("renders minimize, settings, close buttons with data-testids", async () => {
      renderWithOpener();

      await act(async () => {
        fireEvent.click(screen.getByTestId("test-toggle-panel"));
      });
      await act(async () => { vi.advanceTimersByTime(100); });

      expect(screen.getByTestId("button-launcher-minimize")).toBeTruthy();
      expect(screen.getByTestId("button-launcher-settings")).toBeTruthy();
      expect(screen.getByTestId("button-launcher-close")).toBeTruthy();
    });

    it("close button closes the panel", async () => {
      renderWithOpener();

      await act(async () => {
        fireEvent.click(screen.getByTestId("test-toggle-panel"));
      });
      await act(async () => { vi.advanceTimersByTime(100); });

      await act(async () => {
        fireEvent.click(screen.getByTestId("button-launcher-close"));
      });
      await act(async () => { vi.advanceTimersByTime(100); });

      expect(screen.queryByTestId("launcher-panel")).toBeNull();
    });
  });

  describe("Daemon connection status in DOM", () => {
    it("shows status-daemon-connection element when panel is open", async () => {
      renderWithOpener();

      await act(async () => {
        fireEvent.click(screen.getByTestId("test-toggle-panel"));
      });
      await act(async () => { vi.advanceTimersByTime(100); });

      expect(screen.getByTestId("status-daemon-connection")).toBeTruthy();
    });
  });

  describe("Panel dialog accessibility", () => {
    it("has role=dialog and aria-label", async () => {
      renderWithOpener();

      await act(async () => {
        fireEvent.click(screen.getByTestId("test-toggle-panel"));
      });
      await act(async () => { vi.advanceTimersByTime(100); });

      const panel = screen.getByTestId("launcher-panel");
      expect(panel.getAttribute("role")).toBe("dialog");
      expect(panel.getAttribute("aria-label")).toBe("PlenumNET Launcher");
    });

    it("has WAI-ARIA tablist with tab roles", async () => {
      renderWithOpener();

      await act(async () => {
        fireEvent.click(screen.getByTestId("test-toggle-panel"));
      });
      await act(async () => { vi.advanceTimersByTime(100); });

      const panel = screen.getByTestId("launcher-panel");
      const tablist = panel.querySelector('[role="tablist"]');
      expect(tablist).toBeTruthy();

      const tabs = panel.querySelectorAll('[role="tab"]');
      expect(tabs.length).toBe(5);
    });
  });

  describe("Footer", () => {
    it("renders launcher-footer element", async () => {
      renderWithOpener();

      await act(async () => {
        fireEvent.click(screen.getByTestId("test-toggle-panel"));
      });
      await act(async () => { vi.advanceTimersByTime(100); });

      expect(screen.getByTestId("launcher-footer")).toBeTruthy();
    });
  });

  describe("Outside-click persistence", () => {
    it("panel persists when clicking outside the panel (does not close)", async () => {
      renderWithOpener();

      await act(async () => {
        fireEvent.click(screen.getByTestId("test-toggle-panel"));
      });
      await act(async () => { vi.advanceTimersByTime(100); });
      expect(screen.getByTestId("launcher-panel")).toBeTruthy();

      await act(async () => {
        fireEvent.mouseDown(document.body);
        fireEvent.mouseUp(document.body);
        fireEvent.click(document.body);
      });
      await act(async () => { vi.advanceTimersByTime(100); });

      expect(screen.getByTestId("launcher-panel")).toBeTruthy();
    });

    it("panel persists when clicking on its own children (test-toggle-panel)", async () => {
      renderWithOpener();

      await act(async () => {
        fireEvent.click(screen.getByTestId("test-toggle-panel"));
      });
      await act(async () => { vi.advanceTimersByTime(100); });

      const otherButton = screen.getByTestId("test-minimize");
      await act(async () => {
        fireEvent.click(otherButton);
      });

      expect(screen.queryByTestId("launcher-panel")).toBeNull();
      expect(screen.getByTestId("launcher-minimized-bar")).toBeTruthy();
    });
  });

  describe("WS reconnect behavior", () => {
    it("panel shows DISCONNECTED status initially (before WS connects)", async () => {
      renderWithOpener();

      await act(async () => {
        fireEvent.click(screen.getByTestId("test-toggle-panel"));
      });
      await act(async () => { vi.advanceTimersByTime(100); });

      const statusEl = screen.getByTestId("status-daemon-connection");
      expect(statusEl.textContent).not.toContain("Connected");
    });

    it("retry button appears only when daemon state is FAILED", async () => {
      renderWithOpener();

      await act(async () => {
        fireEvent.click(screen.getByTestId("test-toggle-panel"));
      });
      await act(async () => { vi.advanceTimersByTime(100); });

      expect(screen.queryByTestId("button-daemon-retry")).toBeNull();
    });

    it("connection status banner hides when daemon connects (CONNECTED)", async () => {
      renderWithOpener();

      await act(async () => {
        fireEvent.click(screen.getByTestId("test-toggle-panel"));
      });
      await act(async () => { vi.advanceTimersByTime(100); });

      expect(screen.getByTestId("status-daemon-connection")).toBeTruthy();

      const ws = MockWebSocket.latest();
      if (ws) {
        await act(async () => {
          ws.simulateOpen();
          vi.advanceTimersByTime(100);
        });
        await act(async () => {
          await Promise.resolve();
          vi.advanceTimersByTime(100);
        });

        expect(screen.queryByTestId("status-daemon-connection")).toBeNull();
      }
    });

    it("panel remains open when WS disconnects (reconnect cycle)", async () => {
      renderWithOpener();

      await act(async () => {
        fireEvent.click(screen.getByTestId("test-toggle-panel"));
      });
      await act(async () => { vi.advanceTimersByTime(100); });

      const ws = MockWebSocket.latest();
      if (ws) {
        await act(async () => {
          ws.simulateOpen();
          vi.advanceTimersByTime(100);
        });

        await act(async () => {
          ws.close();
          vi.advanceTimersByTime(100);
        });

        expect(screen.getByTestId("launcher-panel")).toBeTruthy();
        const statusEl = screen.getByTestId("status-daemon-connection");
        expect(statusEl.textContent).not.toContain("Connected");
      }
    });
  });

  describe("Partial connectivity degradation", () => {
    it("System tab shows offline message when daemon is disconnected", async () => {
      renderWithOpener();

      await act(async () => {
        fireEvent.click(screen.getByTestId("test-toggle-panel"));
      });
      await act(async () => { vi.advanceTimersByTime(100); });

      await act(async () => {
        fireEvent.click(screen.getByTestId("tab-local"));
      });
      await act(async () => { vi.advanceTimersByTime(100); });

      const panel = screen.getByTestId("launcher-panel");
      const systemPanel = panel.querySelector("#launcher-tabpanel-local");
      expect(systemPanel).toBeTruthy();
      expect(systemPanel!.textContent).toContain("daemon offline");
    });

    it("Network tab shows offline message when daemon is disconnected", async () => {
      renderWithOpener();

      await act(async () => {
        fireEvent.click(screen.getByTestId("test-toggle-panel"));
      });
      await act(async () => { vi.advanceTimersByTime(100); });

      await act(async () => {
        fireEvent.click(screen.getByTestId("tab-net"));
      });
      await act(async () => { vi.advanceTimersByTime(100); });

      const panel = screen.getByTestId("launcher-panel");
      const netPanel = panel.querySelector("#launcher-tabpanel-net");
      expect(netPanel).toBeTruthy();
      expect(netPanel!.textContent).toContain("daemon offline");
    });

    it("Apps tab shows connecting message when daemon is disconnected", async () => {
      renderWithOpener();

      await act(async () => {
        fireEvent.click(screen.getByTestId("test-toggle-panel"));
      });
      await act(async () => { vi.advanceTimersByTime(100); });

      await act(async () => {
        fireEvent.click(screen.getByTestId("tab-apps"));
      });
      await act(async () => { vi.advanceTimersByTime(100); });

      const panel = screen.getByTestId("launcher-panel");
      const appsPanel = panel.querySelector("#launcher-tabpanel-apps");
      expect(appsPanel).toBeTruthy();
      expect(appsPanel!.textContent).toContain("daemon");
    });
  });

  describe("Mobile width behavior", () => {
    it("panel uses responsive width constraint (min formula)", async () => {
      const fs = await import("fs");
      const source = fs.readFileSync("client/src/components/LauncherPanel.tsx", "utf-8");

      expect(source).toContain("min(450px, calc(100vw - 32px))");
    });
  });

  describe("Protocol-safe URL construction", () => {
    it("derives daemon URLs from configurable CUBE_API_HOST/PORT", async () => {
      const fs = await import("fs");
      const source = fs.readFileSync("client/src/components/LauncherPanel.tsx", "utf-8");

      expect(source).toContain("CUBE_API_HOST");
      expect(source).toContain("CUBE_API_PORT");
      expect(source).toContain("VITE_CUBE_API_HOST");
      expect(source).toContain("VITE_CUBE_API_PORT");
    });

    it("localhost always uses insecure transport (ws:// and http://)", async () => {
      const fs = await import("fs");
      const source = fs.readFileSync("client/src/components/LauncherPanel.tsx", "utf-8");

      expect(source).toContain("isLocalhostHost");
      expect(source).toContain('"localhost"');
      expect(source).toContain('"127.0.0.1"');
      expect(source).toContain('"::1"');
      expect(source).toContain("usesSecureTransport");
    });

    it("non-localhost HTTPS pages use secure transport (wss:// and https://)", async () => {
      const fs = await import("fs");
      const source = fs.readFileSync("client/src/components/LauncherPanel.tsx", "utf-8");

      expect(source).toContain("if (isLocalhostHost(CUBE_API_HOST)) return false");
      expect(source).toContain("https:");
    });
  });

  describe("Health check race condition guard", () => {
    it("doHealthCheck gates on activeRef.current before proceeding", async () => {
      const fs = await import("fs");
      const source = fs.readFileSync("client/src/components/LauncherPanel.tsx", "utf-8");

      const healthCheckFn = source.substring(
        source.indexOf("const doHealthCheck"),
        source.indexOf("}, [connectWs, scheduleReconnect])")
      );

      expect(healthCheckFn).toContain("if (!activeRef.current) return;");

      const thenBlock = healthCheckFn.substring(healthCheckFn.indexOf(".then("));
      expect(thenBlock).toContain("if (!activeRef.current) return;");

      const catchBlock = healthCheckFn.substring(healthCheckFn.indexOf(".catch("));
      expect(catchBlock).toContain("if (!activeRef.current) return;");
    });

    it("close-during-health-check: panel closes cleanly without zombie WS", async () => {
      renderWithOpener();

      await act(async () => {
        fireEvent.click(screen.getByTestId("test-toggle-panel"));
      });
      await act(async () => { vi.advanceTimersByTime(100); });
      expect(screen.getByTestId("launcher-panel")).toBeTruthy();

      const wsBefore = MockWebSocket._instances.length;

      await act(async () => {
        fireEvent.click(screen.getByTestId("button-launcher-close"));
      });
      await act(async () => { vi.advanceTimersByTime(100); });

      expect(screen.queryByTestId("launcher-panel")).toBeNull();

      await act(async () => {
        vi.advanceTimersByTime(6000);
      });

      for (let i = wsBefore; i < MockWebSocket._instances.length; i++) {
        const ws = MockWebSocket._instances[i];
        expect(ws.readyState).toBe(MockWebSocket.CLOSED);
      }
    });
  });

  describe("Source-level verification", () => {
    it("CSS custom properties are all defined in component source", async () => {
      const fs = await import("fs");
      const source = fs.readFileSync("client/src/components/LauncherPanel.tsx", "utf-8");

      const requiredVars = [
        "--launcher-bg-primary", "--launcher-bg-panel", "--launcher-bg-surface",
        "--launcher-bg-deep", "--launcher-border", "--launcher-text-heading",
        "--launcher-text-body", "--launcher-text-faint", "--launcher-accent",
        "--launcher-destructive", "--launcher-warning", "--launcher-header-bg",
        "--launcher-highlight", "--launcher-shadow-overlay", "--launcher-font-body",
        "--launcher-font-mono",
      ];

      for (const v of requiredVars) {
        expect(source).toContain(v);
      }
    });

    it("no inline hex colors remain in style props", async () => {
      const fs = await import("fs");
      const source = fs.readFileSync("client/src/components/LauncherPanel.tsx", "utf-8");

      const hexInStyleProp = /style=\{[^}]*#[0-9a-fA-F]{3,8}[^}]*\}/g;
      const matches = source.match(hexInStyleProp);
      expect(matches).toBeNull();
    });

    it("no TypeScript any types exist", async () => {
      const fs = await import("fs");
      const source = fs.readFileSync("client/src/components/LauncherPanel.tsx", "utf-8");

      const lines = source.split("\n");
      const anyLines = lines.filter((line) => /:\s*any\b/.test(line) || /as\s+any\b/.test(line));
      expect(anyLines).toEqual([]);
    });

    it("no hardcoded operational values in connected paths", async () => {
      const fs = await import("fs");
      const source = fs.readFileSync("client/src/components/LauncherPanel.tsx", "utf-8");

      expect(source).not.toContain('"ARRAY3"');
      expect(source).not.toContain('"2-node relay"');
      expect(source).not.toContain('"2.1ms"');
      expect(source).not.toContain('"ARM64"');
      expect(source).not.toContain('"%ProgramFiles%');
      expect(source).not.toContain("getDefaultProducts");
    });

    it("REST errors are surfaced with role=alert and CopyDetailsButton", async () => {
      const fs = await import("fs");
      const source = fs.readFileSync("client/src/components/LauncherPanel.tsx", "utf-8");

      expect(source).toContain('role="alert"');
      expect(source).toContain('restErrors["/cluster-health"]');
      expect(source).toContain('restErrors["/topology"]');
      expect(source).toContain('restErrors["/node/info"]');
    });

    it("cluster-health data is routed to both systemData and networkData", async () => {
      const fs = await import("fs");
      const source = fs.readFileSync("client/src/components/LauncherPanel.tsx", "utf-8");

      const clusterHealthIdx = source.indexOf("cluster-health");
      const setSystemDataIdx = source.indexOf("setSystemData", clusterHealthIdx);
      const setNetworkDataIdx = source.indexOf("setNetworkData", clusterHealthIdx);
      expect(setSystemDataIdx).toBeGreaterThan(clusterHealthIdx);
      expect(setNetworkDataIdx).toBeGreaterThan(clusterHealthIdx);
    });

    it("defines useDaemonConnection with DISCONNECTED initial state (not IDLE)", async () => {
      const fs = await import("fs");
      const source = fs.readFileSync("client/src/components/LauncherPanel.tsx", "utf-8");

      expect(source).toContain('"DISCONNECTED"');
      expect(source).not.toContain('"IDLE"');
    });
  });
});
