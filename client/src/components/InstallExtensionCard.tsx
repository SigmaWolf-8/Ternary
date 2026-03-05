import { useState } from "react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Terminal, Download, FolderOpen, ToggleRight, Puzzle } from "lucide-react";

let openInstallDialog: (() => void) | null = null;

export function triggerInstallDialog() {
  openInstallDialog?.();
}

export default function InstallExtensionDialog() {
  const [open, setOpen] = useState(false);

  openInstallDialog = () => setOpen(true);

  const handleDownload = () => {
    window.location.href = "/api/extension-zip";
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
            Resolve .plm ternary addresses directly in Edge, Chrome, or Brave.
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 pt-2">
          <Button
            onClick={handleDownload}
            className="w-full"
            data-testid="button-download-extension"
          >
            <Download className="w-4 h-4 mr-2" />
            Download Extension
          </Button>

          <ol className="space-y-3">
            <li className="flex items-start gap-3" data-testid="step-install-1">
              <span className="flex items-center justify-center w-6 h-6 rounded-full bg-primary/10 text-primary shrink-0 text-xs font-semibold">1</span>
              <div>
                <p className="text-sm font-medium text-foreground flex items-center gap-1.5"><FolderOpen className="w-3.5 h-3.5" /> Unzip the download</p>
                <p className="text-xs text-muted-foreground">Extract the zip to any folder</p>
              </div>
            </li>
            <li className="flex items-start gap-3" data-testid="step-install-2">
              <span className="flex items-center justify-center w-6 h-6 rounded-full bg-primary/10 text-primary shrink-0 text-xs font-semibold">2</span>
              <div>
                <p className="text-sm font-medium text-foreground flex items-center gap-1.5"><ToggleRight className="w-3.5 h-3.5" /> Open extensions page</p>
                <p className="text-xs text-muted-foreground">Go to <code className="bg-muted px-1 rounded">edge://extensions</code> and enable Developer mode</p>
              </div>
            </li>
            <li className="flex items-start gap-3" data-testid="step-install-3">
              <span className="flex items-center justify-center w-6 h-6 rounded-full bg-primary/10 text-primary shrink-0 text-xs font-semibold">3</span>
              <div>
                <p className="text-sm font-medium text-foreground flex items-center gap-1.5"><Puzzle className="w-3.5 h-3.5" /> Load unpacked</p>
                <p className="text-xs text-muted-foreground">Click "Load unpacked" and select the extracted folder</p>
              </div>
            </li>
          </ol>

          <div className="rounded-md bg-muted/50 border border-border px-4 py-3">
            <p className="text-sm text-muted-foreground">
              Then type <code className="text-xs bg-muted px-1.5 py-0.5 rounded text-primary font-mono">google.plm</code> in the address bar.
            </p>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
