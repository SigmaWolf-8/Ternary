import { useState, useEffect } from "react";
import { LauncherProvider, useLauncher } from "@/components/LauncherPanel";

function WidgetAutoOpen() {
  const { panelState, setPanelState } = useLauncher();

  useEffect(() => {
    if (panelState === "CLOSED") {
      setPanelState("OPENING");
    }
  }, [panelState, setPanelState]);

  return null;
}

export default function WidgetPage() {
  return (
    <div
      data-testid="widget-page"
      style={{
        width: "100vw",
        height: "100vh",
        background: "#0F0C0A",
        overflow: "hidden",
        position: "relative",
      }}
    >
      <LauncherProvider>
        <WidgetAutoOpen />
      </LauncherProvider>
    </div>
  );
}
