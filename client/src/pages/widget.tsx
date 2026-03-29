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
