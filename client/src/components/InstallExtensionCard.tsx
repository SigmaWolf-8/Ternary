import { useState } from "react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Terminal, Download } from "lucide-react";

let openInstallDialog: (() => void) | null = null;

export function triggerInstallDialog() {
  openInstallDialog?.();
}

export default function InstallExtensionDialog() {
  const [open, setOpen] = useState(false);
  const [downloaded, setDownloaded] = useState(false);

  openInstallDialog = () => { setOpen(true); setDownloaded(false); };

  const handleDownload = () => {
    window.location.href = "/api/install-extension";
    setDownloaded(true);
  };

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogContent className="sm:max-w-md" data-testid="dialog-install-extension">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2" data-testid="text-dialog-title">
            <Terminal className="w-5 h-5 text-primary" />
            Install TDNS Extension
          </DialogTitle>
          <DialogDescription>
            Resolve .plm ternary addresses in Edge, Chrome, Brave, and Arc.
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 pt-2">
          <Button
            onClick={handleDownload}
            className="w-full"
            variant={downloaded ? "outline" : "default"}
            data-testid="button-download-extension"
          >
            <Download className="w-4 h-4 mr-2" />
            {downloaded ? "Downloaded - run the file to install" : "Download Installer"}
          </Button>

          {downloaded && (
            <div className="rounded-md bg-muted/50 border border-border px-4 py-3 space-y-2">
              <p className="text-sm font-medium text-foreground">After running the installer:</p>
              <ol className="list-decimal list-inside text-sm text-muted-foreground space-y-1">
                <li>It downloads 9 files and opens your extensions page</li>
                <li>Enable <strong className="text-foreground">Developer mode</strong> (top-right toggle)</li>
                <li>Click <strong className="text-foreground">Load unpacked</strong></li>
                <li>Paste the folder path (already on your clipboard)</li>
              </ol>
            </div>
          )}

          {!downloaded && (
            <p className="text-xs text-muted-foreground text-center">
              One-click installer for Windows. Creates the extension folder and downloads all files automatically.
            </p>
          )}
        </div>
      </DialogContent>
    </Dialog>
  );
}
