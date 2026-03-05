import { useState } from "react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import { Terminal, FolderOpen, ToggleRight, Puzzle, Globe } from "lucide-react";

let openInstallDialog: (() => void) | null = null;

export function triggerInstallDialog() {
  openInstallDialog?.();
}

export default function InstallExtensionDialog() {
  const [open, setOpen] = useState(false);

  openInstallDialog = () => setOpen(true);

  const steps = [
    {
      icon: FolderOpen,
      title: "Unzip the download",
      desc: "Extract plenumnet-tdns-extension.zip to any folder",
    },
    {
      icon: Globe,
      title: "Open your browser's extensions page",
      desc: "edge://extensions or chrome://extensions or brave://extensions",
    },
    {
      icon: ToggleRight,
      title: "Enable Developer mode",
      desc: "Toggle in the top-right corner of the extensions page",
    },
    {
      icon: Puzzle,
      title: "Click \"Load unpacked\"",
      desc: "Select the folder you extracted the zip into",
    },
  ];

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogContent className="sm:max-w-md" data-testid="dialog-install-extension">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2" data-testid="text-dialog-title">
            <Terminal className="w-5 h-5 text-primary" />
            Install TDNS Extension
          </DialogTitle>
          <DialogDescription>
            Your download has started. Follow these steps to finish installing.
          </DialogDescription>
        </DialogHeader>

        <ol className="space-y-4 pt-2">
          {steps.map((step, i) => (
            <li key={i} className="flex items-start gap-3" data-testid={`step-install-${i + 1}`}>
              <span className="flex items-center justify-center w-7 h-7 rounded-full bg-primary/10 text-primary shrink-0 text-sm font-semibold">
                {i + 1}
              </span>
              <div>
                <p className="text-sm font-medium text-foreground">{step.title}</p>
                <p className="text-xs text-muted-foreground mt-0.5">{step.desc}</p>
              </div>
            </li>
          ))}
        </ol>

        <div className="mt-4 rounded-md bg-muted/50 border border-border px-4 py-3">
          <p className="text-sm font-medium text-foreground mb-1">Then try it:</p>
          <p className="text-sm text-muted-foreground">
            Type <code className="text-xs bg-muted px-1.5 py-0.5 rounded text-primary font-mono">google.plm</code> in the address bar and press Enter.
          </p>
        </div>
      </DialogContent>
    </Dialog>
  );
}
