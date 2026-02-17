import { createContext, useContext, useState, useEffect } from "react";
import { Minus, Plus } from "lucide-react";
import { Button } from "@/components/ui/button";

interface ZoomContextType {
  zoom: number;
  setZoom: (z: number) => void;
}

const ZoomContext = createContext<ZoomContextType>({ zoom: 100, setZoom: () => {} });

const ZOOM_MIN = 70;
const ZOOM_MAX = 130;
const ZOOM_STEP = 10;

export function ZoomProvider({ children }: { children: React.ReactNode }) {
  const [zoom, setZoom] = useState<number>(() => {
    if (typeof window !== "undefined") {
      const saved = localStorage.getItem("signhere-zoom");
      return saved ? parseInt(saved, 10) : 90;
    }
    return 90;
  });

  useEffect(() => {
    document.documentElement.style.fontSize = `${zoom}%`;
    localStorage.setItem("signhere-zoom", String(zoom));
  }, [zoom]);

  return (
    <ZoomContext.Provider value={{ zoom, setZoom }}>
      {children}
    </ZoomContext.Provider>
  );
}

export function useZoom() {
  return useContext(ZoomContext);
}

export function ZoomControl() {
  const { zoom, setZoom } = useZoom();

  return (
    <div className="flex items-center gap-0.5" data-testid="zoom-control">
      <Button
        size="icon"
        variant="ghost"
        onClick={() => setZoom(Math.max(ZOOM_MIN, zoom - ZOOM_STEP))}
        disabled={zoom <= ZOOM_MIN}
        data-testid="button-zoom-out"
      >
        <Minus className="w-3 h-3" />
      </Button>
      <button
        className="text-[10px] text-muted-foreground tabular-nums w-8 text-center tracking-wide"
        onClick={() => setZoom(90)}
        data-testid="button-zoom-reset"
      >
        {zoom}%
      </button>
      <Button
        size="icon"
        variant="ghost"
        onClick={() => setZoom(Math.min(ZOOM_MAX, zoom + ZOOM_STEP))}
        disabled={zoom >= ZOOM_MAX}
        data-testid="button-zoom-in"
      >
        <Plus className="w-3 h-3" />
      </Button>
    </div>
  );
}
