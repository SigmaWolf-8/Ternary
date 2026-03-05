import { useState } from "react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Copy, Check, Terminal } from "lucide-react";

let openInstallDialog: (() => void) | null = null;

export function triggerInstallDialog() {
  openInstallDialog?.();
}

export default function InstallExtensionDialog() {
  const [open, setOpen] = useState(false);
  const [copied, setCopied] = useState(false);

  openInstallDialog = () => setOpen(true);

  const isWindows = typeof navigator !== "undefined" && navigator.userAgent.includes("Windows");
  const cmd = isWindows
    ? `irm ${window.location.origin}/install.ps1 | iex`
    : `curl -sL ${window.location.origin}/install.ps1 | bash`;

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(cmd);
    } catch {
      const ta = document.createElement("textarea");
      ta.value = cmd;
      ta.style.position = "fixed";
      ta.style.opacity = "0";
      document.body.appendChild(ta);
      ta.select();
      document.execCommand("copy");
      document.body.removeChild(ta);
    }
    setCopied(true);
    setTimeout(() => setCopied(false), 3000);
  };

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogContent className="sm:max-w-md" data-testid="dialog-install-extension">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2" data-testid="text-dialog-title">
            <Terminal className="w-5 h-5 text-primary" />
            Install TDNS Browser Extension
          </DialogTitle>
          <DialogDescription>
            Resolves .plm addresses in Chrome, Edge, Brave, Firefox, Opera &amp; Vivaldi.
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 pt-2">
          <p className="text-sm text-muted-foreground">
            Copy the command below and paste it into {isWindows ? "Windows PowerShell" : "your terminal"}:
          </p>

          <div className="rounded-md bg-muted border border-border px-4 py-3">
            <code className="text-sm text-primary font-mono break-all" data-testid="text-install-command">
              {cmd}
            </code>
          </div>

          <Button
            onClick={handleCopy}
            className="w-full"
            variant={copied ? "outline" : "default"}
            data-testid="button-copy-command"
          >
            {copied ? (
              <>
                <Check className="w-4 h-4 mr-2" />
                Copied — paste in {isWindows ? "PowerShell" : "Terminal"}
              </>
            ) : (
              <>
                <Copy className="w-4 h-4 mr-2" />
                Copy Install Command
              </>
            )}
          </Button>

          <p className="text-xs text-muted-foreground text-center">
            Auto-detects all installed browsers. Requires {isWindows ? "PowerShell" : "Terminal"}.
          </p>
        </div>
      </DialogContent>
    </Dialog>
  );
}
