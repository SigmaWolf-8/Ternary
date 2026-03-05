import { useState } from "react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Play, Copy, Terminal } from "lucide-react";
import { useToast } from "@/hooks/use-toast";

let openInstallDialog: (() => void) | null = null;

export function triggerInstallDialog() {
  openInstallDialog?.();
}

export default function InstallExtensionDialog() {
  const [open, setOpen] = useState(false);
  const { toast } = useToast();

  openInstallDialog = () => setOpen(true);

  const isWindows = typeof navigator !== "undefined" && navigator.userAgent.includes("Windows");
  const cmd = isWindows
    ? `irm ${window.location.origin}/api/install.ps1 | iex`
    : `curl -sL ${window.location.origin}/api/install.ps1 | bash`;
  const shellName = isWindows ? "Windows PowerShell" : "Terminal";

  const copyToClipboard = async () => {
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
  };

  const handleRunAndInstall = async () => {
    await copyToClipboard();
    setOpen(false);

    if (isWindows) {
      const psCmd = `powershell.exe -NoProfile -ExecutionPolicy Bypass -Command "${cmd}"`;
      const uri = `ms-powershell:command?command=${encodeURIComponent(cmd)}`;

      const opened = window.open(uri, "_blank");
      if (!opened) {
        try {
          window.location.href = uri;
        } catch {}
      }

      toast({
        title: "Launching PowerShell",
        description: "If PowerShell didn't open, press Win+X → PowerShell and paste the copied command.",
      });
    } else {
      toast({
        title: "Command copied",
        description: "Open your terminal and paste the command to install.",
      });
    }
  };

  const handleCopyOnly = async () => {
    await copyToClipboard();
    setOpen(false);
    toast({
      title: "Command copied",
      description: `Open ${shellName}, paste the command, and press Enter.`,
    });
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
          <div className="rounded-md bg-muted border border-border px-4 py-3">
            <code className="text-sm text-primary font-mono break-all" data-testid="text-install-command">
              {cmd}
            </code>
          </div>

          <Button
            onClick={handleRunAndInstall}
            className="w-full"
            data-testid="button-run-install"
          >
            <Play className="w-4 h-4 mr-2" />
            {isWindows ? "Open PowerShell & Install" : "Copy & Install"}
          </Button>

          <Button
            onClick={handleCopyOnly}
            className="w-full"
            variant="outline"
            data-testid="button-copy-command"
          >
            <Copy className="w-4 h-4 mr-2" />
            Copy Command Only
          </Button>

          <p className="text-xs text-muted-foreground text-center">
            Auto-detects all installed browsers. Requires {shellName}.
          </p>
        </div>
      </DialogContent>
    </Dialog>
  );
}
