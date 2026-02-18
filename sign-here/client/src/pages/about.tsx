import { Shield, Lock, FileCheck, Fingerprint, Clock, Globe, Layers, FileText, Award, CheckCircle, Combine, FileOutput, MapPin, Download, Share, Plus } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription } from "@/components/ui/dialog";
import { usePwaInstall } from "@/hooks/use-pwa-install";

const certifications = [
  {
    title: "CNSA 2.0 Compliant Encryption",
    icon: Lock,
    badge: "Active",
    description:
      "All documents are encrypted at rest using HKDF-SHA512 key derivation with AES-256-GCM, meeting the NSA's Commercial National Security Algorithm Suite 2.0 requirements for quantum-resistant symmetric encryption.",
  },
  {
    title: "ML-DSA Post-Quantum Signatures",
    icon: Fingerprint,
    badge: "Active",
    description:
      "Every document is digitally signed using Module-Lattice Digital Signature Algorithm (ML-DSA, FIPS 204) at three critical points: upload, each individual signing event, and final certification — providing a complete chain of quantum-resistant authentication.",
  },
  {
    title: "PlenumNET Dual-Phase Encryption",
    icon: Layers,
    badge: "Active",
    description:
      "Documents are routed through PlenumNET's phase/split quantum-resistant pipeline for dual-phase encryption, ensuring forward secrecy against future quantum computing threats. When unavailable, local mode provides continued operation with standard CNSA 2.0 protection.",
  },
  {
    title: "HPTP Femtosecond Timestamping",
    icon: Clock,
    badge: "Active",
    description:
      "High-Precision Time Protocol timestamps provide femtosecond-accurate event recording for all signing and certification activities, creating a cryptographically verifiable audit timeline that meets the highest standards of non-repudiation.",
  },
  {
    title: "Zero-Knowledge Proof Verification",
    icon: Shield,
    badge: "Active",
    description:
      "Completed documents can be securely shared using Groth16-style zero-knowledge proofs compiled to WebAssembly. Recipients verify document authenticity without exposing sensitive content — enabling trustless, privacy-preserving document sharing.",
  },
  {
    title: "IP Geolocation Audit Logging",
    icon: MapPin,
    badge: "Active",
    description:
      "Every signing event and document access is logged with IP address, geographic location (city, region, country), coordinates, and ISP data. This provides enterprise-grade non-repudiation and supports legal audit trail requirements across jurisdictions.",
  },
  {
    title: "ESIGN, UETA & International Compliance",
    icon: FileCheck,
    badge: "Equivalent",
    description:
      "Sign Here's signing workflow satisfies the requirements of the U.S. Electronic Signatures in Global and National Commerce Act (ESIGN, 2000) and the Uniform Electronic Transactions Act (UETA), providing legally binding electronic signatures in all 50 U.S. states. In Canada, the platform aligns with the Personal Information Protection and Electronic Documents Act (PIPEDA), the Uniform Electronic Commerce Act (UECA), and provincial legislation including Ontario's Electronic Commerce Act and Quebec's Act to Establish a Legal Framework for Information Technology. Internationally, Sign Here's identity verification, audit trail integrity, and tamper-evident certification support enforceability under the UNCITRAL Model Law on Electronic Signatures, providing a foundation for legal recognition across signatory nations worldwide.",
  },
  {
    title: "eIDAS Regulation Equivalence",
    icon: Globe,
    badge: "Equivalent",
    description:
      "The platform's audit trail, signer authentication, and tamper-evident sealing align with the European Union's eIDAS Regulation standards for Advanced Electronic Signatures (AdES), supporting cross-border legal recognition across EU and EEA member states.",
  },
  {
    title: "21 CFR Part 11 Alignment",
    icon: FileText,
    badge: "Equivalent",
    description:
      "Sign Here's comprehensive audit trails, signer identification, tamper detection, and record retention capabilities align with FDA 21 CFR Part 11 requirements for electronic records and signatures in regulated industries.",
  },
  {
    title: "SOC 2 Type II Controls",
    icon: Award,
    badge: "Equivalent",
    description:
      "Multi-tenant isolation with row-level security, encrypted data at rest and in transit, access logging, and IP geo-tracking provide controls equivalent to SOC 2 Type II trust service criteria for security, availability, and confidentiality.",
  },
];

const capabilities = [
  "Create and manage document envelopes with multiple recipients",
  "Assign signer, viewer, and witness roles to each recipient",
  "Upload PDF, DOCX, XLSX, and CSV files with automatic PDF conversion",
  "PDF Stapler: merge and stitch multiple PDFs into a single document with page reordering",
  "Document Converter: automatic server-side DOCX, XLSX, and CSV to PDF conversion via LibreOffice",
  "Visually place signature, date, text, checkbox, and initials fields with drag, resize, and snap",
  "Collect signatures via typed input (8 custom embedded fonts) or freehand drawing",
  "Generate tamper-evident sealed PDFs with embedded TTF custom fonts",
  "Append a certification page with all signatures, timestamps, and audit summary",
  "Track every action in a detailed audit trail with HPTP femtosecond timestamps",
  "IP geolocation logging on every signing event for legal non-repudiation",
  "Share completed documents via zero-knowledge proof verification",
  "Email notifications to recipients via Resend API with branded templates",
  "Dark and light modes with Swiss Banker black-and-gold aesthetic",
  "Site-wide zoom control (70% to 130%)",
];

export default function AboutPage() {
  const { canInstall, isInstalled, install, showIosGuide, dismissIosGuide, iosDevice } = usePwaInstall();

  return (
    <div className="flex-1 overflow-y-auto">
      <div className="max-w-4xl mx-auto px-6 py-8 space-y-8">
        <div>
          <h1 className="text-2xl font-semibold tracking-tight" data-testid="text-about-title">
            About Sign Here
          </h1>
          <p className="text-sm text-muted-foreground mt-1">
            Enterprise-grade e-signature platform with quantum-resistant security
          </p>
        </div>

        <Card>
          <CardHeader>
            <CardTitle className="text-sm tracking-wider uppercase">What is Sign Here?</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3 text-sm leading-relaxed text-muted-foreground">
            <p>
              Sign Here is a next-generation electronic signature platform built for organizations that demand the highest levels of document security and legal defensibility. Inspired by industry leaders like DocuSign, Sign Here goes further by integrating post-quantum cryptography, femtosecond-precision audit trails, and zero-knowledge proof verification into every signing workflow.
            </p>
            <p>
              Built as a module within the Ternary ecosystem, Sign Here leverages PlenumNET's quantum-resistant infrastructure to protect documents against both current and future computational threats — ensuring your signed agreements remain tamper-proof and legally binding for decades to come.
            </p>
          </CardContent>
        </Card>

        <div>
          <h2 className="text-sm font-medium tracking-wider uppercase mb-4">Platform Capabilities</h2>
          <Card>
            <CardContent className="pt-5">
              <ul className="space-y-2.5">
                {capabilities.map((item, i) => (
                  <li key={i} className="flex items-start gap-2.5 text-sm">
                    <CheckCircle className="w-3.5 h-3.5 text-primary mt-0.5 shrink-0" />
                    <span className="text-muted-foreground">{item}</span>
                  </li>
                ))}
              </ul>
            </CardContent>
          </Card>
        </div>

        <div>
          <h2 className="text-sm font-medium tracking-wider uppercase mb-4">
            Document Tools
          </h2>
          <div className="grid gap-4 sm:grid-cols-2">
            <Card>
              <CardHeader className="flex flex-row items-start gap-2 pb-2">
                <Combine className="w-4 h-4 text-primary shrink-0 mt-0.5" />
                <div>
                  <CardTitle className="text-xs font-medium">PDF Stapler</CardTitle>
                  <Badge variant="default" className="text-[10px] mt-1">Built-in</Badge>
                </div>
              </CardHeader>
              <CardContent>
                <p className="text-xs leading-relaxed text-muted-foreground">
                  Merge multiple PDF documents into a single unified file. Upload several PDFs at envelope creation or add pages later in the editor. Supports drag-and-drop page reordering and uses pdf-lib for lossless stitching — no external services required.
                </p>
              </CardContent>
            </Card>
            <Card>
              <CardHeader className="flex flex-row items-start gap-2 pb-2">
                <FileOutput className="w-4 h-4 text-primary shrink-0 mt-0.5" />
                <div>
                  <CardTitle className="text-xs font-medium">Document Converter</CardTitle>
                  <Badge variant="default" className="text-[10px] mt-1">Built-in</Badge>
                </div>
              </CardHeader>
              <CardContent>
                <p className="text-xs leading-relaxed text-muted-foreground">
                  Automatically converts DOCX, XLSX, and CSV files to PDF on upload using server-side LibreOffice headless rendering. Preserves formatting, tables, and layouts. The converted PDF is then encrypted and stored with the same CNSA 2.0 security as native PDF uploads.
                </p>
              </CardContent>
            </Card>
          </div>
        </div>

        <div>
          <h2 className="text-sm font-medium tracking-wider uppercase mb-4">
            Certifications &amp; Compliance
          </h2>
          <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
            {certifications.map((cert) => (
              <Card key={cert.title} data-testid={`card-cert-${cert.title.toLowerCase().replace(/\s+/g, "-")}`}>
                <CardHeader className="flex flex-row items-start justify-between gap-2 pb-2">
                  <div className="flex items-center gap-2">
                    <cert.icon className="w-4 h-4 text-primary shrink-0" />
                    <CardTitle className="text-xs font-medium leading-tight">{cert.title}</CardTitle>
                  </div>
                  <Badge
                    variant={cert.badge === "Active" ? "default" : "secondary"}
                    className="text-[10px] shrink-0"
                  >
                    {cert.badge}
                  </Badge>
                </CardHeader>
                <CardContent>
                  <p className="text-xs leading-relaxed text-muted-foreground">{cert.description}</p>
                </CardContent>
              </Card>
            ))}
          </div>
        </div>

        <Card>
          <CardHeader>
            <CardTitle className="text-sm tracking-wider uppercase">Security Architecture</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3 text-sm leading-relaxed text-muted-foreground">
            <p>
              Sign Here employs a defense-in-depth security model with dual-phase encryption across all database tables. Every sensitive field — recipient names, email addresses, signature values, envelope titles, audit details, and document content — is encrypted at rest using CNSA 2.0 compliant HKDF-SHA512 derived keys with AES-256-GCM. Document payloads are additionally routed through PlenumNET's dual-phase quantum-resistant pipeline for forward secrecy.
            </p>
            <p>
              All data in flight is protected by TLS 1.3 encryption between the application and database, and between the application and PlenumNET services. Every cryptographic operation — from initial upload to final certification — is signed using ML-DSA (Module-Lattice Digital Signature Algorithm), a NIST-standardized post-quantum signature scheme resistant to attacks by both classical and quantum computers.
            </p>
            <p>
              Multi-tenant isolation is enforced at the database level through PostgreSQL row-level security policies, ensuring complete data separation between organizations. All access events are logged with IP geolocation data and HPTP femtosecond timestamps for enterprise-grade audit compliance. Eight database tables are covered by field-level encryption: tenants, users, envelopes, recipients, fields, audit logs, templates, and WBS tags.
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between gap-2">
            <div className="flex items-center gap-2">
              <Download className="w-4 h-4 text-primary shrink-0" />
              <CardTitle className="text-sm tracking-wider uppercase">Install App</CardTitle>
            </div>
            {isInstalled && (
              <Badge variant="default" className="text-[10px]">Installed</Badge>
            )}
          </CardHeader>
          <CardContent className="space-y-4">
            <p className="text-xs leading-relaxed text-muted-foreground">
              Install Sign Here as a standalone app on your device for quick access, offline support, and a full-screen experience without browser chrome.
            </p>
            {!isInstalled ? (
              <Button
                onClick={() => install()}
                data-testid="button-pwa-install"
              >
                <Download className="w-3.5 h-3.5 mr-2" />
                Install Sign Here
              </Button>
            ) : (
              <p className="text-xs text-muted-foreground">
                Sign Here is installed on this device.
              </p>
            )}
          </CardContent>
        </Card>

        {iosDevice && (
          <Dialog open={showIosGuide} onOpenChange={(open) => !open && dismissIosGuide()}>
            <DialogContent className="max-w-sm">
              <DialogHeader>
                <DialogTitle className="text-sm">Install Sign Here</DialogTitle>
                <DialogDescription className="text-xs">
                  To install this app on your device:
                </DialogDescription>
              </DialogHeader>
              <ol className="space-y-3 text-xs text-muted-foreground pl-1">
                <li className="flex items-start gap-2">
                  <span className="font-semibold text-foreground shrink-0">1.</span>
                  <span>Tap the <Share className="inline w-3.5 h-3.5 text-foreground -mt-0.5" /> Share button in Safari</span>
                </li>
                <li className="flex items-start gap-2">
                  <span className="font-semibold text-foreground shrink-0">2.</span>
                  <span>Scroll down and tap <Plus className="inline w-3.5 h-3.5 text-foreground -mt-0.5" /> Add to Home Screen</span>
                </li>
                <li className="flex items-start gap-2">
                  <span className="font-semibold text-foreground shrink-0">3.</span>
                  <span>Tap Add to confirm</span>
                </li>
              </ol>
            </DialogContent>
          </Dialog>
        )}

        <Card>
          <CardContent className="pt-5">
            <div className="flex items-center justify-between gap-4 flex-wrap">
              <div className="space-y-1">
                <p className="text-xs font-medium tracking-wider uppercase">Version</p>
                <p className="text-sm text-muted-foreground" data-testid="text-app-version">v1.1.0</p>
              </div>
              <div className="space-y-1">
                <p className="text-xs font-medium tracking-wider uppercase">Platform</p>
                <p className="text-sm text-muted-foreground" data-testid="text-platform-version">PlenumNET v2.1</p>
              </div>
              <div className="space-y-1">
                <p className="text-xs font-medium tracking-wider uppercase">Phase</p>
                <p className="text-sm text-muted-foreground" data-testid="text-phase-status">Phase 4 — Complete</p>
              </div>
              <div className="space-y-1">
                <p className="text-xs font-medium tracking-wider uppercase">Build</p>
                <p className="text-sm text-muted-foreground" data-testid="text-build-date">February 2026</p>
              </div>
            </div>
          </CardContent>
        </Card>

        <div className="text-center pb-4">
          <p className="text-[10px] text-muted-foreground tracking-wider uppercase">
            Sign Here — Signed | Sealed | Delivered
          </p>
        </div>
      </div>
    </div>
  );
}
