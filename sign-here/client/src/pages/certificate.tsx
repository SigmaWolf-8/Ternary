import { useRoute, Link } from "wouter";
import { useQuery } from "@tanstack/react-query";
import { useState } from "react";
import { format } from "date-fns";
import { Document, Page, pdfjs } from "react-pdf";
import "react-pdf/dist/Page/AnnotationLayer.css";
import "react-pdf/dist/Page/TextLayer.css";
import {
  ArrowLeft,
  Download,
  Shield,
  CheckCircle2,
  Clock,
  User,
  FileText,
  Lock,
  Fingerprint,
  ZoomIn,
  ZoomOut,
  ChevronLeft,
  ChevronRight,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { Badge } from "@/components/ui/badge";
import { useToast } from "@/hooks/use-toast";

pdfjs.GlobalWorkerOptions.workerSrc = `https://unpkg.com/pdfjs-dist@${pdfjs.version}/build/pdf.worker.min.mjs`;

interface CertificateData {
  envelopeId: string;
  title: string;
  description: string | null;
  status: string;
  pageCount: number;
  hasPdf: boolean;
  createdAt: string;
  updatedAt: string;
  plenumDocId: string | null;
  certification: {
    certifiedAt: string;
    signerCount: number;
    allSignersCompleted: boolean;
    hasZkProof: boolean;
    hasHptp: boolean;
  };
  signers: Array<{
    id: string;
    name: string;
    email: string;
    role: string;
    status: string;
    signedAt: string | null;
  }>;
  signatureCount: number;
  auditTrail: Array<{
    id: string;
    action: string;
    actorName: string | null;
    details: string | null;
    createdAt: string;
    hpTpTimestamp: string | null;
    metadata: any;
  }>;
}

function GoldSeal({ size = 120 }: { size?: number }) {
  const r = size / 2;
  const teeth = 24;
  const outerR = r;
  const innerR = r * 0.82;
  const points: string[] = [];
  for (let i = 0; i < teeth * 2; i++) {
    const angle = (Math.PI * 2 * i) / (teeth * 2) - Math.PI / 2;
    const rad = i % 2 === 0 ? outerR : innerR;
    points.push(`${r + rad * Math.cos(angle)},${r + rad * Math.sin(angle)}`);
  }

  return (
    <svg width={size} height={size} viewBox={`0 0 ${size} ${size}`} className="drop-shadow-lg">
      <polygon
        points={points.join(" ")}
        fill="url(#goldGradient)"
        stroke="hsl(40, 65%, 35%)"
        strokeWidth="1.5"
      />
      <defs>
        <radialGradient id="goldGradient" cx="40%" cy="35%">
          <stop offset="0%" stopColor="hsl(45, 80%, 65%)" />
          <stop offset="50%" stopColor="hsl(40, 70%, 50%)" />
          <stop offset="100%" stopColor="hsl(35, 65%, 35%)" />
        </radialGradient>
      </defs>
      <circle cx={r} cy={r} r={r * 0.6} fill="none" stroke="hsl(40, 50%, 30%)" strokeWidth="1" opacity="0.5" />
      <circle cx={r} cy={r} r={r * 0.52} fill="none" stroke="hsl(40, 50%, 30%)" strokeWidth="0.5" opacity="0.3" />
      <text
        x={r}
        y={r - 8}
        textAnchor="middle"
        fill="hsl(40, 20%, 15%)"
        fontSize="11"
        fontWeight="700"
        fontFamily="Inter, sans-serif"
        letterSpacing="0.1em"
      >
        CERTIFIED
      </text>
      <text
        x={r}
        y={r + 6}
        textAnchor="middle"
        fill="hsl(40, 20%, 15%)"
        fontSize="7"
        fontFamily="Inter, sans-serif"
        letterSpacing="0.15em"
      >
        SIGN HERE
      </text>
      <line x1={r - 20} y1={r + 13} x2={r + 20} y2={r + 13} stroke="hsl(40, 40%, 30%)" strokeWidth="0.5" opacity="0.5" />
      <text
        x={r}
        y={r + 22}
        textAnchor="middle"
        fill="hsl(40, 30%, 25%)"
        fontSize="5.5"
        fontFamily="monospace"
        letterSpacing="0.05em"
      >
        HPTP VERIFIED
      </text>
    </svg>
  );
}

export default function CertificatePage() {
  const [, params] = useRoute("/envelope/:id/certificate");
  const envelopeId = params?.id || "";
  const { toast } = useToast();
  const [pdfZoom, setPdfZoom] = useState(0.85);
  const [numPages, setNumPages] = useState(0);
  const [currentPage, setCurrentPage] = useState(1);
  const [isDownloading, setIsDownloading] = useState(false);

  const { data: cert, isLoading, error } = useQuery<CertificateData>({
    queryKey: ["/api/envelopes", envelopeId, "certificate"],
  });

  const downloadCertifiedPdf = async () => {
    setIsDownloading(true);
    try {
      const response = await fetch(`/api/envelopes/${envelopeId}/bake`);
      if (!response.ok) throw new Error("Download failed");
      const blob = await response.blob();
      const a = document.createElement("a");
      a.href = URL.createObjectURL(blob);
      a.download = `${cert?.title || "document"}-certified.pdf`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(a.href);
      toast({ title: "Certified PDF downloaded" });
    } catch {
      toast({ title: "Download failed", variant: "destructive" });
    } finally {
      setIsDownloading(false);
    }
  };

  if (isLoading) {
    return (
      <div className="flex-1 overflow-auto p-5">
        <div className="max-w-4xl mx-auto space-y-4">
          <Skeleton className="h-8 w-64" />
          <Skeleton className="h-64 w-full" />
          <Skeleton className="h-96 w-full" />
        </div>
      </div>
    );
  }

  if (error || !cert) {
    return (
      <div className="flex-1 flex items-center justify-center">
        <Card className="max-w-sm w-full">
          <CardContent className="p-8 text-center">
            <Shield className="w-10 h-10 text-muted-foreground mx-auto mb-3" />
            <h3 className="text-sm font-semibold mb-1">Certificate Not Available</h3>
            <p className="text-[11px] text-muted-foreground mb-4">
              {(error as any)?.message || "This document has not been certified yet. All signers must complete signing before certification."}
            </p>
            <Link href={`/envelope/${envelopeId}`}>
              <Button size="sm" variant="outline" data-testid="button-cert-back-to-detail">
                <ArrowLeft className="w-3 h-3" />
                Back to Envelope
              </Button>
            </Link>
          </CardContent>
        </Card>
      </div>
    );
  }

  const certDate = new Date(cert.certification.certifiedAt);
  const signedSigners = cert.signers.filter(s => s.status === "signed");
  const pdfUrl = cert.hasPdf ? `/api/envelopes/${envelopeId}/bake` : null;

  return (
    <div className="flex-1 overflow-auto">
      <div className="max-w-4xl mx-auto p-5 space-y-5">
        <div className="flex items-center justify-between gap-4 flex-wrap">
          <div className="flex items-center gap-2.5">
            <Link href={`/envelope/${envelopeId}`}>
              <Button size="icon" variant="ghost" data-testid="button-cert-back">
                <ArrowLeft className="w-3.5 h-3.5" />
              </Button>
            </Link>
            <div>
              <h1 className="text-sm font-semibold" data-testid="text-cert-title">Certificate of Completion</h1>
              <p className="text-[10px] text-muted-foreground">{cert.title}</p>
            </div>
          </div>
          <div className="flex items-center gap-1.5">
            {cert.hasPdf && (
              <Button
                size="sm"
                onClick={downloadCertifiedPdf}
                disabled={isDownloading}
                data-testid="button-download-certified-pdf"
              >
                <Download className="w-3 h-3" />
                {isDownloading ? "Downloading..." : "Download Certified PDF"}
              </Button>
            )}
          </div>
        </div>

        <Card className="overflow-hidden">
          <div
            className="relative p-6 md:p-8"
            style={{
              background: "linear-gradient(135deg, hsl(40, 25%, 8%) 0%, hsl(40, 15%, 12%) 40%, hsl(40, 20%, 10%) 100%)",
              borderBottom: "2px solid hsl(40, 65%, 40%)",
            }}
          >
            <div
              className="absolute inset-0 opacity-[0.03]"
              style={{
                backgroundImage: `repeating-linear-gradient(45deg, hsl(40, 65%, 50%) 0, hsl(40, 65%, 50%) 1px, transparent 0, transparent 20px)`,
              }}
            />

            <div className="relative flex flex-col md:flex-row items-center gap-6">
              <div className="shrink-0" data-testid="cert-gold-seal">
                <GoldSeal size={130} />
              </div>

              <div className="flex-1 text-center md:text-left min-w-0">
                <p
                  className="text-[9px] font-medium uppercase tracking-[0.3em] mb-2"
                  style={{ color: "hsl(40, 60%, 55%)" }}
                >
                  Certificate of Completion
                </p>
                <h2
                  className="text-lg md:text-xl font-semibold mb-1 truncate"
                  style={{ color: "hsl(40, 50%, 75%)" }}
                  data-testid="text-cert-doc-title"
                >
                  {cert.title}
                </h2>
                {cert.description && (
                  <p className="text-[11px] mb-3" style={{ color: "hsl(40, 20%, 55%)" }}>
                    {cert.description}
                  </p>
                )}
                <div className="flex items-center gap-4 justify-center md:justify-start flex-wrap">
                  <div className="flex items-center gap-1.5">
                    <CheckCircle2 className="w-3.5 h-3.5" style={{ color: "hsl(40, 65%, 55%)" }} />
                    <span className="text-[11px] font-medium" style={{ color: "hsl(40, 50%, 70%)" }}>
                      All {cert.certification.signerCount} signer{cert.certification.signerCount !== 1 ? "s" : ""} completed
                    </span>
                  </div>
                  <div className="flex items-center gap-1.5">
                    <Clock className="w-3 h-3" style={{ color: "hsl(40, 40%, 50%)" }} />
                    <span className="text-[11px]" style={{ color: "hsl(40, 30%, 55%)" }}>
                      {format(certDate, "MMMM d, yyyy 'at' h:mm:ss a")}
                    </span>
                  </div>
                </div>
              </div>
            </div>

            <div className="relative mt-6 pt-4" style={{ borderTop: "1px solid hsl(40, 40%, 20%)" }}>
              <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
                <div className="text-center">
                  <p className="text-[9px] uppercase tracking-widest mb-0.5" style={{ color: "hsl(40, 30%, 45%)" }}>
                    Envelope ID
                  </p>
                  <p className="text-[10px] font-mono truncate" style={{ color: "hsl(40, 30%, 60%)" }} data-testid="text-cert-envelope-id">
                    {cert.envelopeId.substring(0, 12)}...
                  </p>
                </div>
                <div className="text-center">
                  <p className="text-[9px] uppercase tracking-widest mb-0.5" style={{ color: "hsl(40, 30%, 45%)" }}>
                    Signatures
                  </p>
                  <p className="text-[10px] font-semibold" style={{ color: "hsl(40, 50%, 65%)" }} data-testid="text-cert-sig-count">
                    {cert.signatureCount}
                  </p>
                </div>
                <div className="text-center">
                  <p className="text-[9px] uppercase tracking-widest mb-0.5" style={{ color: "hsl(40, 30%, 45%)" }}>
                    Pages
                  </p>
                  <p className="text-[10px] font-semibold" style={{ color: "hsl(40, 50%, 65%)" }}>
                    {cert.pageCount}
                  </p>
                </div>
                <div className="text-center">
                  <p className="text-[9px] uppercase tracking-widest mb-0.5" style={{ color: "hsl(40, 30%, 45%)" }}>
                    Security
                  </p>
                  <div className="flex items-center justify-center gap-1">
                    {cert.certification.hasHptp && (
                      <Badge variant="outline" className="text-[8px] px-1.5 py-0" style={{ borderColor: "hsl(40, 50%, 40%)", color: "hsl(40, 50%, 60%)" }} data-testid="badge-cert-hptp">
                        HPTP
                      </Badge>
                    )}
                    {cert.certification.hasZkProof && (
                      <Badge variant="outline" className="text-[8px] px-1.5 py-0" style={{ borderColor: "hsl(40, 50%, 40%)", color: "hsl(40, 50%, 60%)" }} data-testid="badge-cert-zk">
                        ZK
                      </Badge>
                    )}
                    {cert.plenumDocId && (
                      <Badge variant="outline" className="text-[8px] px-1.5 py-0" style={{ borderColor: "hsl(40, 50%, 40%)", color: "hsl(40, 50%, 60%)" }} data-testid="badge-cert-cnsa">
                        CNSA 2.0
                      </Badge>
                    )}
                  </div>
                </div>
              </div>
            </div>
          </div>
        </Card>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <Card>
            <CardContent className="p-4">
              <div className="flex items-center gap-2 mb-3">
                <User className="w-3.5 h-3.5 text-primary" />
                <h3 className="text-[10px] font-medium uppercase tracking-wider text-muted-foreground">
                  Signers & Witnesses
                </h3>
              </div>
              <div className="space-y-2">
                {cert.signers.map((signer, i) => (
                  <div
                    key={signer.id}
                    className="flex items-center gap-2.5 p-2.5 rounded-md bg-muted/50"
                    data-testid={`cert-signer-${i}`}
                  >
                    <div className="w-7 h-7 rounded-full flex items-center justify-center shrink-0" style={{ backgroundColor: "hsl(40, 65%, 50%, 0.1)" }}>
                      <CheckCircle2 className="w-3.5 h-3.5 text-primary" />
                    </div>
                    <div className="flex-1 min-w-0">
                      <p className="text-xs font-medium truncate" data-testid={`text-cert-signer-name-${i}`}>{signer.name}</p>
                      <p className="text-[10px] text-muted-foreground truncate" data-testid={`text-cert-signer-email-${i}`}>{signer.email}</p>
                    </div>
                    <div className="text-right shrink-0">
                      <Badge variant="outline" className="text-[8px] capitalize mb-0.5">{signer.role}</Badge>
                      {signer.signedAt && (
                        <p className="text-[9px] text-muted-foreground">
                          {format(new Date(signer.signedAt), "MMM d, h:mm a")}
                        </p>
                      )}
                    </div>
                  </div>
                ))}
                {cert.signers.length === 0 && (
                  <p className="text-[10px] text-muted-foreground">No signers recorded</p>
                )}
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardContent className="p-4">
              <div className="flex items-center gap-2 mb-3">
                <Lock className="w-3.5 h-3.5 text-primary" />
                <h3 className="text-[10px] font-medium uppercase tracking-wider text-muted-foreground">
                  Security & Verification
                </h3>
              </div>
              <div className="space-y-2.5">
                <div className="flex items-center gap-2.5 p-2 rounded-md bg-muted/50">
                  <Shield className="w-4 h-4 text-primary shrink-0" />
                  <div className="min-w-0">
                    <p className="text-[11px] font-medium">HPTP Timestamp</p>
                    <p className="text-[9px] text-muted-foreground font-mono truncate" data-testid="text-cert-hptp">
                      {cert.certification.hasHptp ? cert.certification.certifiedAt : "Timestamp pending"}
                    </p>
                  </div>
                </div>
                <div className="flex items-center gap-2.5 p-2 rounded-md bg-muted/50">
                  <Fingerprint className="w-4 h-4 text-primary shrink-0" />
                  <div className="min-w-0">
                    <p className="text-[11px] font-medium">Zero-Knowledge Proof</p>
                    <p className="text-[9px] text-muted-foreground">
                      {cert.certification.hasZkProof ? "ZK proof generated and verified" : "No ZK proof attached"}
                    </p>
                  </div>
                </div>
                {cert.plenumDocId && (
                  <div className="flex items-center gap-2.5 p-2 rounded-md bg-muted/50">
                    <Lock className="w-4 h-4 text-primary shrink-0" />
                    <div className="min-w-0">
                      <p className="text-[11px] font-medium">PlenumNET Document</p>
                      <p className="text-[9px] text-muted-foreground font-mono truncate">
                        {cert.plenumDocId}
                      </p>
                    </div>
                  </div>
                )}
                <div className="flex items-center gap-2.5 p-2 rounded-md bg-muted/50">
                  <FileText className="w-4 h-4 text-primary shrink-0" />
                  <div className="min-w-0">
                    <p className="text-[11px] font-medium">Audit Trail</p>
                    <p className="text-[9px] text-muted-foreground">
                      {cert.auditTrail.length} event{cert.auditTrail.length !== 1 ? "s" : ""} recorded
                    </p>
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>
        </div>

        {pdfUrl && (
          <Card>
            <CardContent className="p-4">
              <div className="flex items-center justify-between gap-4 mb-3 flex-wrap">
                <div className="flex items-center gap-2">
                  <FileText className="w-3.5 h-3.5 text-primary" />
                  <h3 className="text-[10px] font-medium uppercase tracking-wider text-muted-foreground">
                    Certified Document
                  </h3>
                </div>
                <div className="flex items-center gap-1.5">
                  <Button
                    size="icon"
                    variant="ghost"
                    onClick={() => setPdfZoom(z => Math.max(0.4, z - 0.1))}
                    data-testid="button-cert-zoom-out"
                  >
                    <ZoomOut className="w-3.5 h-3.5" />
                  </Button>
                  <span className="text-[10px] text-muted-foreground tabular-nums w-10 text-center">
                    {Math.round(pdfZoom * 100)}%
                  </span>
                  <Button
                    size="icon"
                    variant="ghost"
                    onClick={() => setPdfZoom(z => Math.min(2, z + 0.1))}
                    data-testid="button-cert-zoom-in"
                  >
                    <ZoomIn className="w-3.5 h-3.5" />
                  </Button>
                  {numPages > 1 && (
                    <>
                      <div className="w-px h-4 bg-border mx-1" />
                      <Button
                        size="icon"
                        variant="ghost"
                        disabled={currentPage <= 1}
                        onClick={() => setCurrentPage(p => Math.max(1, p - 1))}
                        data-testid="button-cert-prev-page"
                      >
                        <ChevronLeft className="w-3.5 h-3.5" />
                      </Button>
                      <span className="text-[10px] text-muted-foreground tabular-nums">
                        {currentPage} / {numPages}
                      </span>
                      <Button
                        size="icon"
                        variant="ghost"
                        disabled={currentPage >= numPages}
                        onClick={() => setCurrentPage(p => Math.min(numPages, p + 1))}
                        data-testid="button-cert-next-page"
                      >
                        <ChevronRight className="w-3.5 h-3.5" />
                      </Button>
                    </>
                  )}
                </div>
              </div>
              <div
                className="flex justify-center rounded-md overflow-auto bg-muted/30 p-4"
                style={{
                  maxHeight: "600px",
                  boxShadow: "inset 2px 2px 6px rgba(0,0,0,0.3), inset -2px -2px 6px rgba(255,255,255,0.04)",
                }}
              >
                <Document
                  file={pdfUrl}
                  onLoadSuccess={(doc) => setNumPages(doc.numPages)}
                  loading={
                    <div className="flex items-center justify-center py-16">
                      <Skeleton className="h-[400px] w-[300px]" />
                    </div>
                  }
                  error={
                    <div className="flex flex-col items-center justify-center py-16 text-muted-foreground">
                      <FileText className="w-8 h-8 mb-2" />
                      <p className="text-xs">Failed to load certified PDF</p>
                    </div>
                  }
                >
                  <div
                    style={{ boxShadow: "0 4px 16px rgba(0,0,0,0.4), 0 2px 4px rgba(0,0,0,0.2)" }}
                  >
                    <Page
                      pageNumber={currentPage}
                      width={700 * pdfZoom}
                      renderAnnotationLayer={false}
                      renderTextLayer={false}
                    />
                  </div>
                </Document>
              </div>
            </CardContent>
          </Card>
        )}

        <Card>
          <CardContent className="p-4">
            <div className="flex items-center gap-2 mb-3">
              <Clock className="w-3.5 h-3.5 text-primary" />
              <h3 className="text-[10px] font-medium uppercase tracking-wider text-muted-foreground">
                Complete Audit Trail
              </h3>
            </div>
            {cert.auditTrail.length === 0 ? (
              <p className="text-[10px] text-muted-foreground">No audit events recorded</p>
            ) : (
              <div className="space-y-0">
                {cert.auditTrail.map((log, i) => (
                  <div key={log.id} className="flex gap-2.5" data-testid={`cert-audit-${i}`}>
                    <div className="flex flex-col items-center">
                      <div
                        className="w-2 h-2 rounded-full mt-1.5 shrink-0"
                        style={{ backgroundColor: "hsl(40, 65%, 50%)" }}
                      />
                      {i < cert.auditTrail.length - 1 && (
                        <div className="w-px flex-1" style={{ backgroundColor: "hsl(40, 30%, 25%)" }} />
                      )}
                    </div>
                    <div className="pb-3 min-w-0">
                      <p className="text-xs">{log.action}</p>
                      <div className="flex items-center gap-2 mt-0.5 flex-wrap">
                        {log.actorName && (
                          <span className="text-[10px] text-muted-foreground">{log.actorName}</span>
                        )}
                        <span className="text-[10px] text-muted-foreground">
                          {format(new Date(log.createdAt), "MMM d, h:mm:ss a")}
                        </span>
                        {log.hpTpTimestamp && (
                          <Badge variant="outline" className="text-[8px] px-1 py-0 font-mono" style={{ borderColor: "hsl(40, 50%, 40%)", color: "hsl(40, 50%, 60%)" }}>
                            HPTP: {log.hpTpTimestamp.substring(0, 23)}
                          </Badge>
                        )}
                      </div>
                      {log.details && (
                        <p className="text-[10px] text-muted-foreground mt-0.5">{log.details}</p>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            )}
          </CardContent>
        </Card>

        <div className="text-center py-4">
          <p className="text-[9px] text-muted-foreground uppercase tracking-widest">
            Sign Here v1.1.2 | PlenumNET v2.1 | CNSA 2.0 Compliant
          </p>
          <p className="text-[8px] text-muted-foreground/60 mt-1">
            This certificate was generated from an immutable audit trail and is independently verifiable
          </p>
        </div>
      </div>
    </div>
  );
}
