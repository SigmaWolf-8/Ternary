import { useState } from "react";

export default function InstallExtensionCard() {
  const [copied, setCopied] = useState(false);

  const isWindows = typeof navigator !== "undefined" && navigator.userAgent.includes("Windows");
  const cmd = isWindows
    ? `irm ${window.location.origin}/install.ps1 | iex`
    : `curl -sL ${window.location.origin}/install.ps1 | bash`;

  const handleCopy = () => {
    navigator.clipboard.writeText(cmd);
    setCopied(true);
    setTimeout(() => setCopied(false), 3000);
  };

  return (
    <div className="rounded-lg border border-[#2A2520] bg-[#1A1816] p-5 max-w-md" data-testid="card-install-extension">
      <h3 className="text-sm font-semibold text-[#D4A017] tracking-wide uppercase mb-1">
        Install TDNS Extension
      </h3>
      <p className="text-xs text-[#8A8578] mb-4">
        Resolves .plm addresses in Chrome, Edge, Brave, Firefox, Opera &amp; Vivaldi.
      </p>

      <button
        onClick={handleCopy}
        data-testid="button-install-extension"
        className={`w-full py-2.5 px-4 rounded-md text-sm font-semibold transition-colors ${
          copied
            ? "bg-[#059669] text-white"
            : "bg-[#D4A017] text-[#090807] hover:bg-[#E8B42E]"
        }`}
      >
        {copied ? "✓ Copied — paste in PowerShell" : "Install Extension"}
      </button>

      {copied && (
        <div className="mt-3 rounded-md bg-[#0F0E0D] border border-[#2A2520] px-3 py-2">
          <code className="text-xs text-[#D4A017] font-mono break-all" data-testid="text-install-command">{cmd}</code>
        </div>
      )}

      <p className="text-[10px] text-[#5A5548] mt-3">
        Auto-detects all installed browsers. Requires {isWindows ? "PowerShell" : "Terminal"}.
      </p>
    </div>
  );
}
