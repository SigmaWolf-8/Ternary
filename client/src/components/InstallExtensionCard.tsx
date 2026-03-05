import { useState } from "react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Terminal, Copy, Check } from "lucide-react";
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

  const cmd = "irm https://raw.githubusercontent.com/SigmaWolf-8/Ternary/main/services/tdns-v2/install.ps1 | iex";

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
    toast({ title: "Copied to clipboard", description: "Paste into Windows PowerShell and press Enter." });
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
            Resolve .plm ternary addresses directly in Edge, Chrome, Brave, and Arc.
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 pt-2">
          <div className="space-y-1.5">
            <p className="text-sm font-medium text-foreground">Run in Windows PowerShell:</p>
            <div className="rounded-md bg-muted border border-border px-4 py-3 flex items-start justify-between gap-3">
              <code className="text-xs text-primary font-mono break-all leading-relaxed select-all" data-testid="text-install-command">
                {cmd}
              </code>
              <Button
                onClick={handleCopy}
                variant="ghost"
                size="sm"
                className="shrink-0 mt-[-2px]"
                data-testid="button-copy-command"
              >
                {copied ? <Check className="w-4 h-4 text-green-500" /> : <Copy className="w-4 h-4" />}
              </Button>
            </div>
          </div>

          <div className="text-sm text-muted-foreground space-y-2">
            <p>The installer downloads 9 files, detects your browsers, and opens the extensions page. Then:</p>
            <ol className="list-decimal list-inside space-y-1 text-sm">
              <li>Enable <strong className="text-foreground">Developer mode</strong> (top-right toggle)</li>
              <li>Click <strong className="text-foreground">Load unpacked</strong></li>
              <li>Paste the folder path (already on your clipboard)</li>
            </ol>
          </div>

          <div className="rounded-md bg-muted/50 border border-border px-4 py-3 space-y-1">
            <p className="text-sm font-medium text-foreground">Pin the icon (Edge)</p>
            <p className="text-xs text-muted-foreground">
              Click the puzzle piece icon next to the address bar, find PlenumNET TDNS, and click the pin/eye icon to show it in your toolbar.
            </p>
          </div>

          <Button onClick={handleCopy} className="w-full" data-testid="button-run-install">
            {copied ? <Check className="w-4 h-4 mr-2" /> : <Copy className="w-4 h-4 mr-2" />}
            {copied ? "Copied!" : "Copy Install Command"}
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}
