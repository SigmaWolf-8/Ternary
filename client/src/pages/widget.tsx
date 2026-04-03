// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division

import { LauncherProvider } from "@/components/LauncherPanel";

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
      <LauncherProvider widgetMode>
        <span />
      </LauncherProvider>
    </div>
  );
}
