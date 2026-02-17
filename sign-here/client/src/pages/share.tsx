import { useRoute, useLocation } from "wouter";
import { useQuery, useMutation } from "@tanstack/react-query";
import { useState } from "react";
import { Shield, ShieldCheck, ShieldAlert, Lock, Download, FileText, Users, Clock, Fingerprint, ArrowLeft } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { StatusBadge } from "@/components/status-badge";
import { verifyZKProof, isWasmLoaded, loadZKVerifier } from "@/lib/zkVerifier";
import { apiRequest } from "@/lib/queryClient";
import { format } from "date-fns";

type VerifyStatus = "idle" | "loading" | "verifying" | "valid" | "invalid";

export default function SharePage() {
  const [, params] = useRoute("/share/:id");
  const [, setLocation] = useLocation();
  const envelopeId = params?.id || "";
  const [verifyStatus, setVerifyStatus] = useState<VerifyStatus>("idle");
  const [verifyMessage, setVerifyMessage] = useState("");
  const [wasmStatus, setWasmStatus] = useState<"pending" | "loaded" | "fallback">("pending");
  const [isDownloading, setIsDownloading] = useState(false);

  const { data: shareData, isLoading } = useQuery<{
    envelope: any;
    recipientCount: number;
    signedCount: number;
    zkData: any;
  }>({
    queryKey: ["/api/envelopes", envelopeId, "share"],
  });

  const proofMutation = useMutation({
    mutationFn: async () => {
      const res = await apiRequest("POST", `/api/envelopes/${envelopeId}/share-proof`);
      return res.json();
    },
  });

  const handleVerify = async () => {
    setVerifyStatus("loading");

    try {
      const loaded = await loadZKVerifier();
      setWasmStatus(loaded ? "loaded" : "fallback");

      setVerifyStatus("verifying");

      const proofResult = await proofMutation.mutateAsync();

      const result = await verifyZKProof(proofResult.proof, proofResult.publicInputs);

      if (result.valid) {
        setVerifyStatus("valid");
        setVerifyMessage(result.message);
      } else {
        setVerifyStatus("invalid");
        setVerifyMessage(result.message);
      }
    } catch (err: any) {
      setVerifyStatus("invalid");
      setVerifyMessage(err.message || "Verification failed");
    }
  };

  const downloadPdf = async () => {
    setIsDownloading(true);
    try {
      const response = await fetch(`/api/envelopes/${envelopeId}/bake`);
      if (!response.ok) throw new Error("Download failed");
      const blob = await response.blob();
      const a = document.createElement("a");
      a.href = URL.createObjectURL(blob);
      a.download = `${shareData?.envelope?.title || "document"}-certified.pdf`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(a.href);
    } catch {
    } finally {
      setIsDownloading(false);
    }
  };

  if (isLoading) {
    return (
      <div className="min-h-screen bg-background flex items-center justify-center p-6">
        <div className="max-w-md w-full space-y-4">
          <Skeleton className="h-8 w-48 mx-auto" />
          <Skeleton className="h-32 w-full" />
          <Skeleton className="h-12 w-full" />
        </div>
      </div>
    );
  }

  if (!shareData?.envelope) {
    return (
      <div className="min-h-screen bg-background flex items-center justify-center p-6">
        <Card className="max-w-md w-full">
          <CardContent className="p-6 text-center">
            <ShieldAlert className="w-8 h-8 text-destructive mx-auto mb-3" />
            <p className="text-sm font-medium">Document not found</p>
            <p className="text-[10px] text-muted-foreground mt-1">This share link may be invalid or expired</p>
          </CardContent>
        </Card>
      </div>
    );
  }

  const { envelope } = shareData;

  return (
    <div className="min-h-screen bg-background flex items-center justify-center p-6">
      <div className="max-w-md w-full space-y-4">
        <div className="flex justify-start mb-2">
          <Button variant="ghost" size="sm" onClick={() => setLocation("/")} data-testid="button-share-back">
            <ArrowLeft className="w-4 h-4 mr-1" />
            Dashboard
          </Button>
        </div>
        <div className="text-center mb-6">
          <div className="flex items-center justify-center gap-2 mb-2">
            <div className="w-8 h-8 rounded-md bg-primary flex items-center justify-center">
              <Shield className="w-4 h-4 text-primary-foreground" />
            </div>
            <span className="tracking-wide" style={{ fontFamily: "'Great Vibes', cursive", fontSize: '1.5rem' }} data-testid="text-share-brand">
              Sign Here
            </span>
          </div>
          <p className="text-[10px] text-muted-foreground tracking-wider uppercase">
            Zero-Knowledge Document Verification
          </p>
        </div>

        <Card>
          <CardContent className="p-4 space-y-3">
            <div className="flex items-start gap-3">
              <div className="w-9 h-9 rounded-full bg-primary/10 flex items-center justify-center shrink-0">
                <FileText className="w-4 h-4 text-primary" />
              </div>
              <div className="min-w-0 flex-1">
                <p className="text-xs font-semibold truncate" data-testid="text-share-title">{envelope.title}</p>
                {envelope.description && (
                  <p className="text-[10px] text-muted-foreground mt-0.5 line-clamp-2">{envelope.description}</p>
                )}
              </div>
              <StatusBadge status={envelope.status} />
            </div>

            <div className="grid grid-cols-3 gap-2 pt-2">
              <div className="text-center">
                <div className="flex items-center justify-center gap-1 text-[10px] text-muted-foreground mb-0.5">
                  <Users className="w-2.5 h-2.5" />
                  <span className="uppercase tracking-wider">Signers</span>
                </div>
                <p className="text-xs font-medium" data-testid="text-share-signers">
                  {shareData.signedCount}/{shareData.recipientCount}
                </p>
              </div>
              <div className="text-center">
                <div className="flex items-center justify-center gap-1 text-[10px] text-muted-foreground mb-0.5">
                  <Clock className="w-2.5 h-2.5" />
                  <span className="uppercase tracking-wider">Date</span>
                </div>
                <p className="text-xs font-medium">
                  {format(new Date(envelope.createdAt), "MMM d, yyyy")}
                </p>
              </div>
              <div className="text-center">
                <div className="flex items-center justify-center gap-1 text-[10px] text-muted-foreground mb-0.5">
                  <FileText className="w-2.5 h-2.5" />
                  <span className="uppercase tracking-wider">Pages</span>
                </div>
                <p className="text-xs font-medium">{envelope.pageCount}</p>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-4">
            {verifyStatus === "idle" && (
              <div className="text-center space-y-3">
                <div className="w-12 h-12 rounded-full bg-primary/10 flex items-center justify-center mx-auto">
                  <Lock className="w-5 h-5 text-primary" />
                </div>
                <div>
                  <p className="text-xs font-semibold">Zero-Knowledge Authorization</p>
                  <p className="text-[10px] text-muted-foreground mt-1">
                    Verify your authorization to access this document without revealing any content, signer identities, or private keys.
                  </p>
                </div>
                <Button
                  className="w-full"
                  onClick={handleVerify}
                  disabled={envelope.status !== "completed"}
                  data-testid="button-verify-proof"
                >
                  <Fingerprint className="w-3.5 h-3.5" />
                  {envelope.status === "completed" ? "Verify Authorization" : "Document Not Yet Certified"}
                </Button>
                {envelope.status !== "completed" && (
                  <p className="text-[9px] text-muted-foreground">
                    This document must be fully signed and certified before ZK verification is available.
                  </p>
                )}
              </div>
            )}

            {(verifyStatus === "loading" || verifyStatus === "verifying") && (
              <div className="text-center space-y-3 py-4">
                <div className="w-12 h-12 rounded-full bg-primary/10 flex items-center justify-center mx-auto animate-pulse">
                  <Shield className="w-5 h-5 text-primary" />
                </div>
                <p className="text-xs font-medium" data-testid="text-verify-status">
                  {verifyStatus === "loading" ? "Loading ZK verifier..." : "Verifying proof..."}
                </p>
                <p className="text-[10px] text-muted-foreground">
                  {wasmStatus === "loaded"
                    ? "WASM verifier active"
                    : wasmStatus === "fallback"
                    ? "Using fallback verifier"
                    : "Initializing cryptographic engine"}
                </p>
              </div>
            )}

            {verifyStatus === "valid" && (
              <div className="text-center space-y-3">
                <div className="w-12 h-12 rounded-full bg-emerald-500/10 flex items-center justify-center mx-auto">
                  <ShieldCheck className="w-5 h-5 text-emerald-500" />
                </div>
                <div>
                  <p className="text-xs font-semibold text-emerald-600 dark:text-emerald-400" data-testid="text-verify-success">
                    Access Granted
                  </p>
                  <p className="text-[10px] text-muted-foreground mt-1">{verifyMessage}</p>
                </div>
                <div className="flex items-center justify-center gap-1.5 text-[9px] text-muted-foreground">
                  <Fingerprint className="w-2.5 h-2.5" />
                  <span className="uppercase tracking-wider">
                    {wasmStatus === "loaded" ? "WASM ZK Verified" : "Cryptographic Fallback Verified"}
                  </span>
                </div>
                {envelope.pdfData === "has_pdf" && (
                  <Button
                    className="w-full"
                    onClick={downloadPdf}
                    disabled={isDownloading}
                    data-testid="button-download-shared"
                  >
                    <Download className="w-3.5 h-3.5" />
                    {isDownloading ? "Downloading..." : "Download Certified PDF"}
                  </Button>
                )}

                {shareData.zkData && (
                  <div className="text-left mt-4 p-3 rounded-md bg-muted/50 space-y-1">
                    <p className="text-[9px] font-medium uppercase tracking-wider text-muted-foreground mb-2">Certification Details</p>
                    {shareData.zkData.certifiedAt && (
                      <div className="flex items-center gap-2">
                        <span className="text-[9px] text-muted-foreground">HPTP Timestamp:</span>
                        <span className="text-[9px] font-mono text-primary/70" data-testid="text-share-hptp">
                          {shareData.zkData.certifiedAt.substring(0, 23)}
                        </span>
                      </div>
                    )}
                    {shareData.zkData.signerCount && (
                      <div className="flex items-center gap-2">
                        <span className="text-[9px] text-muted-foreground">Signers:</span>
                        <span className="text-[9px] font-medium">{shareData.zkData.signerCount}/{shareData.zkData.signerCount} completed</span>
                      </div>
                    )}
                    {shareData.zkData.mlDsaSignature && (
                      <div className="flex items-center gap-2">
                        <span className="text-[9px] text-muted-foreground">ML-DSA:</span>
                        <span className="text-[9px] font-mono text-primary/70">
                          {shareData.zkData.mlDsaSignature.substring(0, 20)}...
                        </span>
                      </div>
                    )}
                  </div>
                )}
              </div>
            )}

            {verifyStatus === "invalid" && (
              <div className="text-center space-y-3">
                <div className="w-12 h-12 rounded-full bg-destructive/10 flex items-center justify-center mx-auto">
                  <ShieldAlert className="w-5 h-5 text-destructive" />
                </div>
                <div>
                  <p className="text-xs font-semibold text-destructive" data-testid="text-verify-failed">
                    Verification Failed
                  </p>
                  <p className="text-[10px] text-muted-foreground mt-1">{verifyMessage}</p>
                </div>
                <Button variant="outline" className="w-full" onClick={handleVerify} data-testid="button-retry-verify">
                  Retry Verification
                </Button>
              </div>
            )}
          </CardContent>
        </Card>

        <p className="text-center text-[9px] text-muted-foreground">
          Powered by Ternary ZK Engine — Groth16 proofs with ML-DSA signatures
        </p>
      </div>
    </div>
  );
}
