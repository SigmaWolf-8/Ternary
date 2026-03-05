import { useState } from "react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Copy, Terminal, Check } from "lucide-react";
import { useToast } from "@/hooks/use-toast";

let openInstallDialog: (() => void) | null = null;

export function triggerInstallDialog() {
  openInstallDialog?.();
}

export default function InstallExtensionDialog() {
  const [open, setOpen] = useState(false);
  const [copied, setCopied] = useState(false);
  const { toast } = useToast();

  openInstallDialog = () => { setOpen(true); setCopied(false); };

  const scriptUrl = "https://raw.githubusercontent.com/SigmaWolf-8/Ternary/main/services/tdns-v2/install.ps1";
  const cmd = `irm ${scriptUrl} | iex`;

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
    setCopied(true);
    toast({
      title: "Command copied",
      description: "Open Windows PowerShell, paste the command, and press Enter.",
    });
    setTimeout(() => setCopied(false), 3000);
  };

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogContent className="sm:max-w-lg" data-testid="dialog-install-extension">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2" data-testid="text-dialog-title">
            <Terminal className="w-5 h-5 text-primary" />
            Install TDNS Browser Extension
          </DialogTitle>
          <DialogDescription>
            Resolves .plm ternary addresses directly in Edge, Chrome, and Brave.
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 pt-2">
          <div className="space-y-2">
            <p className="text-sm font-medium text-foreground">Step 1: Copy and run in PowerShell</p>
            <div className="rounded-md bg-muted border border-border px-4 py-3 flex items-center justify-between gap-3">
              <code className="text-sm text-primary font-mono break-all select-all" data-testid="text-install-command">
                {cmd}
              </code>
              <Button
                onClick={copyToClipboard}
                variant="ghost"
                size="sm"
                className="shrink-0"
                data-testid="button-copy-command"
              >
                {copied ? <Check className="w-4 h-4 text-green-500" /> : <Copy className="w-4 h-4" />}
              </Button>
            </div>
          </div>

          <div className="space-y-2">
            <p className="text-sm font-medium text-foreground">Step 2: Load the extension</p>
            <ol className="text-sm text-muted-foreground space-y-1 list-decimal list-inside">
              <li>Open <code className="text-xs bg-muted px-1 rounded">edge://extensions</code> in your browser</li>
              <li>Enable <strong>Developer mode</strong> (top-right toggle)</li>
              <li>Click <strong>Load unpacked</strong></li>
              <li>Paste the folder path (already copied by the installer)</li>
            </ol>
          </div>

          <div className="space-y-2">
            <p className="text-sm font-medium text-foreground">Step 3: Try it</p>
            <p className="text-sm text-muted-foreground">
              Type <code className="text-xs bg-muted px-1 rounded text-primary">google.plm</code> in the address bar and press Enter.
            </p>
          </div>

          <Button
            onClick={copyToClipboard}
            className="w-full"
            data-testid="button-run-install"
          >
            {copied ? <Check className="w-4 h-4 mr-2" /> : <Copy className="w-4 h-4 mr-2" />}
            {copied ? "Copied!" : "Copy Install Command"}
          </Button>

          <p className="text-xs text-muted-foreground text-center">
            Requires Windows PowerShell. Downloads 9 files to %LocalAppData%\PlenumNET.
          </p>
        </div>
      </DialogContent>
    </Dialog>
  );
}
