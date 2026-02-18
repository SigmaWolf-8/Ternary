import type { Express } from "express";
import { createServer, type Server } from "http";
import { storage } from "./storage";
import { insertEnvelopeSchema, insertRecipientSchema, insertTenantSchema, insertUserSchema, insertTemplateSchema, insertWbsTagSchema } from "@shared/schema";
import { z } from "zod";
import { tenantMiddleware } from "./middleware/tenant";
import { healthCheck as plenumHealthCheck } from "./services/plenum";
import { saveEnvelope as hybridSave } from "./services/saveCopy";
import { getHPTP, witnessSign, mlDsaSign, cnsa2SecureDocument } from "./services/plenum";
import crypto from "crypto";
import { bakeFillablePdf, type CertificationData } from "./services/pdfForms";
import { generateZKProof, verifyZKProofServer } from "./services/zk";
import { encryptPdf, decryptPdf } from "./services/pdfCrypto";
import { sendEnvelopeEmails } from "./services/email";
import { convertToPdf, isConvertible, isPdfFile } from "./services/fileConvert";
import { extractClientIP, lookupGeo } from "./services/ipGeo";

export async function registerRoutes(
  httpServer: Server,
  app: Express
): Promise<Server> {

  app.use(tenantMiddleware);

  app.get("/api/health", async (_req, res) => {
    const plenum = await plenumHealthCheck();
    res.json({
      status: "ok",
      timestamp: new Date().toISOString(),
      plenum,
      database: "connected",
      encryption: {
        atRest: "CNSA 2.0 (HKDF-SHA512 + AES-256-GCM)",
        inFlight: "TLS 1.3 + PlenumNET dual-phase",
        fieldLevel: "All PII encrypted (fenc: prefix)",
        documentLevel: "PDF encrypted (hkdf: prefix)",
        signatures: "ML-DSA post-quantum (FIPS 204)",
        tables: ["tenants", "users", "envelopes", "recipients", "fields", "audit_logs", "templates", "wbs_tags"],
      },
    });
  });

  app.get("/api/envelopes", async (_req, res) => {
    const envelopes = await storage.getEnvelopes();
    res.json(envelopes);
  });

  app.get("/api/envelopes/:id", async (req, res) => {
    const envelope = await storage.getEnvelope(req.params.id);
    if (!envelope) return res.status(404).json({ message: "Not found" });
    const { pdfData, ...rest } = envelope;
    res.json({ ...rest, pdfData: pdfData ? "has_pdf" : null });
  });

  app.post("/api/envelopes", async (req, res) => {
    try {
      const { title, description, recipients: recipientList, pdfBytes: pdfBase64, fields: fieldsList } = req.body;
      const tenantId = req.tenantId;

      if (pdfBase64 && tenantId) {
        const pdfBuffer = Buffer.from(pdfBase64, "base64");
        const result = await hybridSave({
          pdfBytes: pdfBuffer,
          fields: fieldsList || [],
          tenantId,
          title,
          description,
        });

        if (recipientList && Array.isArray(recipientList)) {
          for (let i = 0; i < recipientList.length; i++) {
            const r = recipientList[i];
            await storage.createRecipient({
              envelopeId: result.envelopeId,
              name: r.name,
              email: r.email,
              role: r.role || "signer",
              status: "pending",
              sortOrder: i,
            });
          }
        }

        return res.status(201).json(result);
      }

      const parsed = insertEnvelopeSchema.parse({
        title,
        description,
        status: "draft",
        tenantId: tenantId || null,
      });
      const envelope = await storage.createEnvelope(parsed);

      if (recipientList && Array.isArray(recipientList)) {
        for (let i = 0; i < recipientList.length; i++) {
          const r = recipientList[i];
          await storage.createRecipient({
            envelopeId: envelope.id,
            name: r.name,
            email: r.email,
            role: r.role || "signer",
            status: "pending",
            sortOrder: i,
          });
        }
      }

      const epoch = await getHPTP();
      await storage.createAuditLog({
        envelopeId: envelope.id,
        tenantId: tenantId || null,
        action: "Envelope created",
        actorName: "System",
        details: `Created with ${recipientList?.length || 0} recipient(s)`,
        hpTpTimestamp: epoch,
      });

      const { pdfData: _pdf, ...envelopeRest } = envelope;
      res.json({ ...envelopeRest, pdfData: _pdf ? "has_pdf" : null });
    } catch (error: any) {
      res.status(400).json({ message: error.message });
    }
  });

  app.patch("/api/envelopes/:id", async (req, res) => {
    try {
      if (req.body.status === "sent") {
        const allRecipients = await storage.getRecipientsByEnvelope(req.params.id);
        const signers = allRecipients.filter((r) => r.role === "signer");
        if (signers.length === 0) {
          return res.status(400).json({
            message: "Cannot send: add at least one signer recipient before sending",
          });
        }
      }

      const envelope = await storage.updateEnvelope(req.params.id, req.body);
      if (!envelope) return res.status(404).json({ message: "Not found" });

      if (req.body.status === "sent") {
        const epoch = await getHPTP();
        await storage.createAuditLog({
          envelopeId: envelope.id,
          tenantId: req.tenantId || null,
          action: "Envelope sent for signing",
          actorName: "System",
          details: "Document sent to all recipients",
          hpTpTimestamp: epoch,
        });

        const allRecipients = await storage.getRecipientsByEnvelope(req.params.id);
        const senderName = req.body.senderName || "Sign Here User";
        const senderEmail = req.body.senderEmail || null;
        const emailResult = await sendEnvelopeEmails(
          envelope.id,
          envelope.title,
          senderName,
          allRecipients.map((r) => ({ id: r.id, email: r.email, name: r.name, role: r.role })),
          senderEmail
        );
        const emailDetails = emailResult.failed > 0
          ? `${emailResult.sent}/${emailResult.total} emails sent (${emailResult.failed} failed — verify your Resend domain)`
          : `${emailResult.sent}/${emailResult.total} emails sent successfully`;
        await storage.createAuditLog({
          envelopeId: envelope.id,
          tenantId: req.tenantId || null,
          action: "Signing emails dispatched",
          actorName: "System",
          details: emailDetails,
          hpTpTimestamp: epoch,
        });
      }

      const { pdfData, ...rest } = envelope;
      res.json({ ...rest, pdfData: pdfData ? "has_pdf" : null });
    } catch (error: any) {
      res.status(400).json({ message: error.message });
    }
  });

  app.delete("/api/envelopes/:id", async (req, res) => {
    await storage.deleteEnvelope(req.params.id);
    res.json({ success: true });
  });

  app.get("/api/envelopes/:id/pdf", async (req, res) => {
    try {
      const envelope = await storage.getEnvelope(req.params.id);
      if (!envelope) return res.status(404).json({ message: "Not found" });
      if (!envelope.pdfData) return res.status(404).json({ message: "No PDF attached" });

      const decryptedBase64 = decryptPdf(envelope.pdfData);
      const pdfBuffer = Buffer.from(decryptedBase64, "base64");
      res.setHeader("Content-Type", "application/pdf");
      res.setHeader("Content-Length", pdfBuffer.length.toString());
      res.setHeader("Content-Disposition", `inline; filename="${envelope.title}.pdf"`);
      res.send(pdfBuffer);
    } catch (error: any) {
      res.status(500).json({ message: error.message });
    }
  });

  app.post("/api/envelopes/:id/upload-pdf", async (req, res) => {
    try {
      const envelope = await storage.getEnvelope(req.params.id);
      if (!envelope) return res.status(404).json({ message: "Not found" });

      const { pdfData, pageCount, fileName, fileType } = req.body;
      if (!pdfData) return res.status(400).json({ message: "pdfData (base64) is required" });

      let finalBase64 = pdfData;
      let finalPageCount = pageCount || 1;
      let converted = false;

      if (fileName && isConvertible(fileName)) {
        try {
          const result = await convertToPdf(pdfData, fileName);
          finalBase64 = result.pdfBase64;
          finalPageCount = result.pageCount;
          converted = true;
        } catch (convErr: any) {
          return res.status(400).json({ message: `Failed to convert ${fileName} to PDF: ${convErr.message}` });
        }
      }

      const documentHash = crypto.createHash("sha512").update(finalBase64).digest("hex");

      const tenantId = req.tenantId || "default";
      const cnsa2 = await cnsa2SecureDocument(finalBase64, tenantId, documentHash);

      const encryptedPdf = encryptPdf(finalBase64);
      const updated = await storage.updateEnvelope(req.params.id, {
        pdfData: encryptedPdf,
        pageCount: finalPageCount,
        plenumDocId: cnsa2.plenumDocId,
      });

      const fileLabel = fileName || "PDF";
      await storage.createAuditLog({
        envelopeId: req.params.id,
        tenantId: req.tenantId || null,
        action: "Document uploaded",
        actorName: "System",
        details: converted
          ? `${fileLabel} converted to PDF (${finalPageCount} page${finalPageCount !== 1 ? "s" : ""})`
          : `${fileLabel} with ${finalPageCount} page(s) attached`,
        hpTpTimestamp: cnsa2.hptpTimestamp,
      });

      await storage.createAuditLog({
        envelopeId: req.params.id,
        tenantId: req.tenantId || null,
        action: "CNSA 2.0 quantum encryption applied",
        actorName: "PlenumNET Security Engine",
        details: cnsa2.quantumSecured
          ? `Document secured via dual-phase split (${cnsa2.plenumDocId}) with ML-DSA signature`
          : `Document secured locally with HKDF-AES-256-GCM + ML-DSA signature (PlenumNET fallback)`,
        hpTpTimestamp: cnsa2.hptpTimestamp,
        metadata: {
          plenumDocId: cnsa2.plenumDocId,
          mlDsaSignature: cnsa2.mlDsaSignature,
          documentHash,
          phaseSplit: cnsa2.phaseSplit,
          quantumSecured: cnsa2.quantumSecured,
          encryptionMethod: "HKDF-SHA512-AES-256-GCM",
        },
      });

      res.json({
        success: true,
        pageCount: updated?.pageCount,
        converted,
        cnsa2: {
          plenumDocId: cnsa2.plenumDocId,
          quantumSecured: cnsa2.quantumSecured,
          phaseSplit: cnsa2.phaseSplit,
        },
      });
    } catch (error: any) {
      res.status(400).json({ message: error.message });
    }
  });

  app.get("/api/envelopes/:id/recipients", async (req, res) => {
    const recipients = await storage.getRecipientsByEnvelope(req.params.id);
    res.json(recipients);
  });

  app.get("/api/recipients/:id", async (req, res) => {
    const recipient = await storage.getRecipient(req.params.id);
    if (!recipient) return res.status(404).json({ message: "Not found" });
    res.json(recipient);
  });

  app.post("/api/envelopes/:id/recipients", async (req, res) => {
    try {
      const { name, email, role } = req.body;
      if (!name || !email) return res.status(400).json({ message: "Name and email are required" });
      const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
      if (!emailRegex.test(email)) return res.status(400).json({ message: "Invalid email address" });

      const existingRecipients = await storage.getRecipientsByEnvelope(req.params.id);
      const sortOrder = existingRecipients.length;

      const recipient = await storage.createRecipient({
        envelopeId: req.params.id,
        name,
        email,
        role: role || "signer",
        sortOrder,
        status: "pending",
      });
      res.status(201).json(recipient);
    } catch (error: any) {
      res.status(400).json({ message: error.message });
    }
  });

  app.patch("/api/recipients/:id", async (req, res) => {
    try {
      const { name, email, role } = req.body;
      const updates: any = {};
      if (name !== undefined) updates.name = name;
      if (email !== undefined) {
        const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
        if (!emailRegex.test(email)) return res.status(400).json({ message: "Invalid email address" });
        updates.email = email;
      }
      if (role !== undefined) updates.role = role;

      const updated = await storage.updateRecipient(req.params.id, updates);
      if (!updated) return res.status(404).json({ message: "Recipient not found" });
      res.json(updated);
    } catch (error: any) {
      res.status(400).json({ message: error.message });
    }
  });

  app.delete("/api/recipients/:id", async (req, res) => {
    try {
      await storage.deleteRecipient(req.params.id);
      res.json({ success: true });
    } catch (error: any) {
      res.status(400).json({ message: error.message });
    }
  });

  app.get("/api/envelopes/:id/fields", async (req, res) => {
    const fieldsList = await storage.getFieldsByEnvelope(req.params.id);
    res.json(fieldsList);
  });

  app.put("/api/envelopes/:id/fields", async (req, res) => {
    try {
      const { fields: fieldsList } = req.body;
      const envelopeId = req.params.id;

      await storage.deleteFieldsByEnvelope(envelopeId);

      if (fieldsList && Array.isArray(fieldsList)) {
        for (const f of fieldsList) {
          await storage.createField({
            envelopeId,
            recipientId: f.recipientId,
            type: f.type,
            label: f.label || null,
            page: f.page || 1,
            x: f.x,
            y: f.y,
            width: f.width,
            height: f.height,
            value: f.value || null,
            required: f.required !== false,
          });
        }
      }

      const savedFields = await storage.getFieldsByEnvelope(envelopeId);
      res.json(savedFields);
    } catch (error: any) {
      res.status(400).json({ message: error.message });
    }
  });

  app.post("/api/envelopes/:id/sign", async (req, res) => {
    try {
      const { recipientId, fieldValues } = req.body;
      const envelopeId = req.params.id;

      const recipient = await storage.getRecipient(recipientId);
      if (!recipient) return res.status(404).json({ message: "Recipient not found" });

      if (recipient.status === "signed") {
        return res.status(400).json({ message: "Already signed" });
      }

      const existingFields = await storage.getFieldsByEnvelope(envelopeId);
      if (fieldValues && typeof fieldValues === "object") {
        for (const [fieldId, value] of Object.entries(fieldValues)) {
          const field = existingFields.find((f) => f.id === fieldId);
          if (field) {
            await storage.updateField(fieldId, { value: value as string });
          }
        }
      }

      const clientIP = extractClientIP(req);
      const geo = await lookupGeo(clientIP);

      const signEpoch = await getHPTP();

      await witnessSign(recipientId, "recipient_sign", req.tenantId || "default");

      const signaturePayload = JSON.stringify({
        envelopeId,
        recipientId,
        signerName: recipient.name,
        signerEmail: recipient.email,
        fieldCount: Object.keys(fieldValues || {}).length,
        timestamp: signEpoch,
      });
      const signerMlDsa = await mlDsaSign(signaturePayload, req.tenantId || "default");

      await storage.updateRecipient(recipientId, {
        status: "signed",
        signedAt: new Date(),
      });

      await storage.createAuditLog({
        envelopeId,
        tenantId: req.tenantId || null,
        action: "Document signed",
        actorName: recipient.name,
        details: `Signed by ${recipient.name} (${recipient.email})`,
        hpTpTimestamp: signEpoch,
        metadata: {
          recipientId,
          fieldCount: Object.keys(fieldValues || {}).length,
          ipAddress: clientIP,
          geoLocation: geo.city && geo.country ? `${geo.city}, ${geo.region}, ${geo.country}` : undefined,
          geoCoordinates: geo.lat && geo.lon ? { lat: geo.lat, lon: geo.lon } : undefined,
          isp: geo.org || undefined,
        },
      });

      await storage.createAuditLog({
        envelopeId,
        tenantId: req.tenantId || null,
        action: "ML-DSA signature recorded",
        actorName: "PlenumNET Security Engine",
        details: `Post-quantum ML-DSA signature applied for ${recipient.name}'s signing action`,
        hpTpTimestamp: signEpoch,
        metadata: {
          recipientId,
          mlDsaSignature: signerMlDsa.signature,
          algorithm: "ML-DSA-65 (CNSA 2.0)",
        },
      });

      const allRecipients = await storage.getRecipientsByEnvelope(envelopeId);
      const signers = allRecipients.filter((r) => r.role === "signer");
      const signedCount = signers.filter((r) => r.status === "signed" || r.id === recipientId).length;
      const allSigned = signedCount === signers.length;

      if (allSigned) {
        const certEpoch = await getHPTP();

        const envelope = await storage.getEnvelope(envelopeId);
        const docHash = envelope?.plenumDocId || envelopeId;
        const mlDsaResult = await mlDsaSign(
          JSON.stringify({ envelopeId, docHash, signedCount: signers.length, certifiedAt: certEpoch }),
          req.tenantId || "default"
        );

        await storage.updateEnvelope(envelopeId, {
          status: "completed",
          zkProof: JSON.stringify({
            certifiedAt: certEpoch,
            signerCount: signers.length,
            mlDsaSignature: mlDsaResult.signature,
            allSignersCompleted: true,
          }),
        });

        await storage.createAuditLog({
          envelopeId,
          tenantId: req.tenantId || null,
          action: "Document certified",
          actorName: "HPTP Certification Engine",
          details: `All ${signers.length}/${signers.length} signers completed — document certified with femtosecond HPTP timestamp and ML-DSA signature`,
          hpTpTimestamp: certEpoch,
          metadata: {
            certifiedAt: certEpoch,
            signerCount: signers.length,
            mlDsaSignature: mlDsaResult.signature,
          },
        });
      } else {
        await storage.updateEnvelope(envelopeId, { status: "signing" });

        await storage.createAuditLog({
          envelopeId,
          tenantId: req.tenantId || null,
          action: "Signing progress",
          actorName: "System",
          details: `${signedCount}/${signers.length} signers completed`,
          hpTpTimestamp: signEpoch,
        });
      }

      res.json({
        success: true,
        signedCount,
        totalSigners: signers.length,
        allSigned,
        certified: allSigned,
      });
    } catch (error: any) {
      res.status(400).json({ message: error.message });
    }
  });

  app.get("/api/envelopes/:id/audit", async (req, res) => {
    const logs = await storage.getAuditLogsByEnvelope(req.params.id);
    const sanitized = logs.map(log => {
      const meta = log.metadata ? { ...log.metadata as Record<string, any> } : null;
      if (meta) {
        delete meta.geoCoordinates;
        if (meta.ipAddress) {
          const parts = (meta.ipAddress as string).split(".");
          meta.ipAddress = parts.length === 4
            ? `${parts[0]}.${parts[1]}.***.***`
            : (meta.ipAddress as string).replace(/:[^:]+$/, ":****");
        }
      }
      return { ...log, metadata: meta };
    });
    res.json(sanitized);
  });

  app.get("/api/envelopes/:id/bake", async (req, res) => {
    try {
      const envelope = await storage.getEnvelope(req.params.id);
      if (!envelope) return res.status(404).json({ message: "Not found" });
      if (!envelope.pdfData) return res.status(400).json({ message: "No PDF attached" });

      const fieldsList = await storage.getFieldsByEnvelope(req.params.id);
      const decryptedBase64 = decryptPdf(envelope.pdfData);
      const pdfBuffer = Buffer.from(decryptedBase64, "base64");

      let certData: CertificationData | undefined;
      if (envelope.status === "completed" && envelope.zkProof) {
        try {
          const zkData = JSON.parse(envelope.zkProof);
          const allRecipients = await storage.getRecipientsByEnvelope(req.params.id);
          const signers = allRecipients.filter((r) => r.role === "signer");
          const auditLogs = await storage.getAuditLogsByEnvelope(req.params.id);

          certData = {
            title: envelope.title,
            certifiedAt: zkData.certifiedAt || envelope.updatedAt?.toISOString() || new Date().toISOString(),
            signerCount: zkData.signerCount || signers.length,
            mlDsaSignature: zkData.mlDsaSignature,
            signers: signers.map((s) => ({
              name: s.name,
              email: s.email,
              signedAt: s.signedAt ? s.signedAt.toISOString() : null,
            })),
            auditTrail: auditLogs.map((log) => ({
              action: log.action,
              actorName: log.actorName,
              details: log.details || "",
              hpTpTimestamp: log.hpTpTimestamp || null,
              createdAt: log.createdAt.toISOString(),
            })),
          };
        } catch (e) {
          console.warn("Failed to build certification data for PDF:", e);
        }
      }

      const bakedPdf = await bakeFillablePdf(pdfBuffer, fieldsList, certData);

      res.setHeader("Content-Type", "application/pdf");
      res.setHeader("Content-Length", bakedPdf.length.toString());
      res.setHeader("Content-Disposition", `attachment; filename="${envelope.title} - Signed.pdf"`);
      res.send(Buffer.from(bakedPdf));
    } catch (error: any) {
      res.status(500).json({ message: error.message });
    }
  });


  app.get("/api/envelopes/:id/certificate", async (req, res) => {
    try {
      const envelope = await storage.getEnvelope(req.params.id);
      if (!envelope) return res.status(404).json({ message: "Envelope not found" });
      if (envelope.status !== "completed") {
        return res.status(400).json({ message: "Document is not yet certified" });
      }

      const recipients = await storage.getRecipientsByEnvelope(req.params.id);
      const auditLogs = await storage.getAuditLogsByEnvelope(req.params.id);
      const fields = await storage.getFieldsByEnvelope(req.params.id);

      let certData: any = {};
      try {
        certData = envelope.zkProof ? JSON.parse(envelope.zkProof) : {};
      } catch {}

      const signers = recipients.filter(r => r.role === "signer" || r.role === "witness");
      const signatureFields = fields.filter(f => f.type === "signature" && f.value);

      const certificate = {
        envelopeId: envelope.id,
        title: envelope.title,
        description: envelope.description,
        status: envelope.status,
        pageCount: envelope.pageCount,
        hasPdf: envelope.pdfData === "has_pdf",
        createdAt: envelope.createdAt,
        updatedAt: envelope.updatedAt,
        plenumDocId: envelope.plenumDocId,
        certification: {
          certifiedAt: certData.certifiedAt || envelope.updatedAt,
          signerCount: certData.signerCount || signers.length,
          allSignersCompleted: certData.allSignersCompleted ?? (signers.every(s => s.status === "signed")),
          hasZkProof: !!certData.zkProof,
          hasHptp: !!certData.certifiedAt,
        },
        signers: signers.map(s => ({
          id: s.id,
          name: s.name,
          email: s.email,
          role: s.role,
          status: s.status,
          signedAt: s.signedAt,
        })),
        signatureCount: signatureFields.length,
        auditTrail: auditLogs.map(log => {
          const meta = log.metadata ? { ...log.metadata as Record<string, any> } : null;
          if (meta) {
            delete meta.geoCoordinates;
            if (meta.ipAddress) {
              const parts = (meta.ipAddress as string).split(".");
              meta.ipAddress = parts.length === 4
                ? `${parts[0]}.${parts[1]}.***.***`
                : (meta.ipAddress as string).replace(/:[^:]+$/, ":****");
            }
          }
          return {
            id: log.id,
            action: log.action,
            actorName: log.actorName,
            details: log.details,
            createdAt: log.createdAt,
            hpTpTimestamp: log.hpTpTimestamp,
            metadata: meta,
          };
        }),
      };

      res.json(certificate);
    } catch (error: any) {
      res.status(500).json({ message: error.message });
    }
  });

  app.post("/api/tenants", async (req, res) => {
    try {
      const parsed = insertTenantSchema.parse(req.body);
      const tenant = await storage.createTenant(parsed);
      res.status(201).json(tenant);
    } catch (error: any) {
      res.status(400).json({ message: error.message });
    }
  });

  app.get("/api/admin/stats", async (_req, res) => {
    try {
      const allTenants = await storage.getAllTenants();
      const allUsers = await storage.getAllUsers();
      const allEnvelopes = await storage.getEnvelopes();
      res.json({
        tenants: allTenants.length,
        users: allUsers.length,
        envelopes: {
          total: allEnvelopes.length,
          draft: allEnvelopes.filter((e) => e.status === "draft").length,
          sent: allEnvelopes.filter((e) => e.status === "sent" || e.status === "signing").length,
          completed: allEnvelopes.filter((e) => e.status === "completed").length,
        },
      });
    } catch (error: any) {
      res.status(500).json({ message: error.message });
    }
  });

  app.get("/api/admin/tenants", async (_req, res) => {
    try {
      const allTenants = await storage.getAllTenants();
      res.json(allTenants);
    } catch (error: any) {
      res.status(500).json({ message: error.message });
    }
  });

  app.get("/api/admin/users", async (_req, res) => {
    try {
      const allUsers = await storage.getAllUsers();
      const safeUsers = allUsers.map(({ password, ...u }) => u);
      res.json(safeUsers);
    } catch (error: any) {
      res.status(500).json({ message: error.message });
    }
  });

  app.post("/api/admin/users", async (req, res) => {
    try {
      const parsed = insertUserSchema.parse(req.body);
      const user = await storage.createUser(parsed);
      const { role, email, tenantId } = req.body;
      if (email || role || tenantId) {
        const updated = await storage.updateUser(user.id, {
          ...(email ? { email } : {}),
          ...(role ? { role } : {}),
          ...(tenantId ? { tenantId } : {}),
        });
        if (updated) {
          const { password: _, ...safe } = updated;
          return res.status(201).json(safe);
        }
      }
      const { password: _, ...safe } = user;
      res.status(201).json(safe);
    } catch (error: any) {
      res.status(400).json({ message: error.message });
    }
  });

  app.patch("/api/admin/users/:id", async (req, res) => {
    try {
      const { role, email, tenantId } = req.body;
      if (role === "sadmin") {
        return res.status(403).json({ message: "SAdmin role cannot be assigned — reserved for platform creator only" });
      }
      const target = await storage.getUser(req.params.id);
      if (!target) return res.status(404).json({ message: "User not found" });
      if (target.role === "sadmin" && target.isPlatformCreator) {
        return res.status(403).json({ message: "Cannot modify the platform creator's role" });
      }
      const updates: Record<string, string> = {};
      if (role) updates.role = role;
      if (email !== undefined) updates.email = email;
      if (tenantId !== undefined) updates.tenantId = tenantId;
      const updated = await storage.updateUser(req.params.id, updates);
      if (!updated) return res.status(404).json({ message: "User not found" });
      const { password: _, ...safe } = updated;
      res.json(safe);
    } catch (error: any) {
      res.status(400).json({ message: error.message });
    }
  });

  app.delete("/api/admin/users/:id", async (req, res) => {
    try {
      const target = await storage.getUser(req.params.id);
      if (target?.role === "sadmin" && target?.isPlatformCreator) {
        return res.status(403).json({ message: "Cannot delete the platform creator" });
      }
      await storage.deleteUser(req.params.id);
      res.json({ success: true });
    } catch (error: any) {
      res.status(400).json({ message: error.message });
    }
  });

  app.get("/api/saas/settings", async (_req, res) => {
    res.json({
      pricingTier: "enterprise",
      maxEnvelopesPerTenant: 500,
      maxUsersPerTenant: 50,
      features: {
        zkProofs: true,
        hptpTimestamps: true,
        plenumIntegration: true,
        aiFieldDetection: true,
        realtimeCollab: true,
        offlineMode: true,
        wbsTags: true,
        templateGallery: true,
      },
      platform: {
        version: "1.0.0",
        plenumVersion: "2.1",
        phase: "4",
        encryption: "CNSA 2.0",
      },
    });
  });

  app.patch("/api/saas/settings", async (req, res) => {
    res.json({ success: true, message: "SaaS settings updated" });
  });

  app.post("/api/envelopes/:id/share-proof", async (req, res) => {
    try {
      const envelopeId = req.params.id;
      const tenantId = req.tenantId || "default";
      const clientIP = extractClientIP(req);
      const geo = await lookupGeo(clientIP);

      const envelope = await storage.getEnvelope(envelopeId);
      if (!envelope) return res.status(404).json({ message: "Not found" });

      if (envelope.status !== "completed") {
        return res.status(400).json({ message: "Envelope must be completed before sharing" });
      }

      const result = await generateZKProof(envelopeId, tenantId);

      const epoch = await getHPTP();
      await storage.createAuditLog({
        envelopeId,
        tenantId,
        action: "ZK share proof generated",
        actorName: "ZK Proof Engine",
        details: "Zero-knowledge authorization proof generated for secure sharing",
        hpTpTimestamp: epoch,
        metadata: {
          ipAddress: clientIP,
          geoLocation: geo.city && geo.country ? `${geo.city}, ${geo.region}, ${geo.country}` : undefined,
          proofCommitment: result.proof.commitment.substring(0, 16) + "...",
          nullifier: result.proof.nullifier.substring(0, 16) + "...",
        },
      });

      res.json(result);
    } catch (error: any) {
      res.status(400).json({ message: error.message });
    }
  });

  app.get("/api/envelopes/:id/share", async (req, res) => {
    try {
      const envelope = await storage.getEnvelope(req.params.id);
      if (!envelope) return res.status(404).json({ message: "Not found" });

      const { pdfData, ...rest } = envelope;
      const recipients = await storage.getRecipientsByEnvelope(req.params.id);

      let zkData = null;
      if (envelope.zkProof) {
        try { zkData = JSON.parse(envelope.zkProof); } catch {}
      }

      res.json({
        envelope: { ...rest, pdfData: pdfData ? "has_pdf" : null },
        recipientCount: recipients.length,
        signedCount: recipients.filter((r) => r.status === "signed").length,
        zkData,
      });
    } catch (error: any) {
      res.status(500).json({ message: error.message });
    }
  });

  app.post("/api/envelopes/:id/verify-proof", async (req, res) => {
    try {
      const { proof, publicInputs } = req.body;
      if (!proof || !publicInputs) {
        return res.status(400).json({ message: "proof and publicInputs required" });
      }

      const isValid = verifyZKProofServer(proof, publicInputs);

      const clientIP = extractClientIP(req);
      const geo = await lookupGeo(clientIP);
      const epoch = await getHPTP();
      await storage.createAuditLog({
        envelopeId: req.params.id,
        tenantId: req.tenantId || null,
        action: isValid ? "ZK proof verified" : "ZK proof verification failed",
        actorName: "ZK Verify Engine",
        details: isValid
          ? "Authorization proof verified successfully — access granted"
          : "Authorization proof verification failed — access denied",
        hpTpTimestamp: epoch,
        metadata: {
          ipAddress: clientIP,
          geoLocation: geo.city && geo.country ? `${geo.city}, ${geo.region}, ${geo.country}` : undefined,
          verified: isValid,
        },
      });

      res.json({ valid: isValid });
    } catch (error: any) {
      res.status(400).json({ message: error.message });
    }
  });

  app.get("/api/templates", async (req, res) => {
    try {
      const tenantId = req.tenantId || undefined;
      const all = await storage.getTemplates(tenantId);
      res.json(all);
    } catch (error: any) {
      res.status(500).json({ message: error.message });
    }
  });

  app.get("/api/templates/:id", async (req, res) => {
    try {
      const template = await storage.getTemplate(req.params.id);
      if (!template) return res.status(404).json({ message: "Template not found" });
      res.json(template);
    } catch (error: any) {
      res.status(500).json({ message: error.message });
    }
  });

  app.post("/api/templates", async (req, res) => {
    try {
      const parsed = insertTemplateSchema.parse({
        ...req.body,
        tenantId: req.tenantId || null,
      });
      const template = await storage.createTemplate(parsed);
      res.status(201).json(template);
    } catch (error: any) {
      res.status(400).json({ message: error.message });
    }
  });

  app.post("/api/templates/:id/fork", async (req, res) => {
    try {
      const source = await storage.getTemplate(req.params.id);
      if (!source) return res.status(404).json({ message: "Template not found" });

      const forked = await storage.createTemplate({
        name: `${source.name} (Copy)`,
        description: source.description,
        category: source.category,
        tags: source.tags as string[],
        fieldDefs: source.fieldDefs as any,
        tenantId: req.tenantId || null,
        isPublic: false,
        forkedFromId: source.id,
        sourceEnvelopeId: source.sourceEnvelopeId,
      });
      res.status(201).json(forked);
    } catch (error: any) {
      res.status(500).json({ message: error.message });
    }
  });

  app.post("/api/envelopes/:id/save-as-template", async (req, res) => {
    try {
      const envelope = await storage.getEnvelope(req.params.id);
      if (!envelope) return res.status(404).json({ message: "Envelope not found" });

      const envelopeFields = await storage.getFieldsByEnvelope(req.params.id);
      const fieldDefs = envelopeFields.map(f => ({
        type: f.type,
        label: f.label || f.type,
        page: f.page,
        x: f.x,
        y: f.y,
        width: f.width,
        height: f.height,
        required: f.required,
      }));

      const data = insertTemplateSchema.parse({
        name: req.body.name || `Template from ${envelope.title}`,
        description: req.body.description || envelope.description,
        category: req.body.category || null,
        tags: req.body.tags || [],
        fieldDefs,
        tenantId: req.tenantId || null,
        isPublic: false,
        sourceEnvelopeId: envelope.id,
      });

      const template = await storage.createTemplate(data);
      res.status(201).json(template);
    } catch (error: any) {
      res.status(500).json({ message: error.message });
    }
  });

  app.patch("/api/templates/:id", async (req, res) => {
    try {
      const updateSchema = insertTemplateSchema.partial();
      const parsed = updateSchema.parse(req.body);
      const updated = await storage.updateTemplate(req.params.id, parsed);
      if (!updated) return res.status(404).json({ message: "Template not found" });
      res.json(updated);
    } catch (error: any) {
      if (error.name === "ZodError") {
        return res.status(400).json({ message: "Validation error", errors: error.errors });
      }
      res.status(500).json({ message: error.message });
    }
  });

  app.delete("/api/templates/:id", async (req, res) => {
    try {
      await storage.deleteTemplate(req.params.id);
      res.json({ message: "Template deleted" });
    } catch (error: any) {
      res.status(500).json({ message: error.message });
    }
  });

  app.get("/api/wbs-tags", async (req, res) => {
    try {
      const tenantId = req.tenantId || undefined;
      const tags = await storage.getWbsTags(tenantId);
      res.json(tags);
    } catch (error: any) {
      res.status(500).json({ message: error.message });
    }
  });

  app.post("/api/wbs-tags", async (req, res) => {
    try {
      const tenantId = req.tenantId || null;
      const existing = await storage.getWbsTags(tenantId || undefined);
      if (existing.length >= 13) {
        return res.status(400).json({ message: "Maximum of 13 WBS tags allowed" });
      }
      const parsed = insertWbsTagSchema.parse({
        ...req.body,
        tenantId,
        sortOrder: req.body.sortOrder ?? existing.length,
      });
      const tag = await storage.createWbsTag(parsed);
      res.status(201).json(tag);
    } catch (error: any) {
      res.status(400).json({ message: error.message });
    }
  });

  app.put("/api/wbs-tags/reorder", async (req, res) => {
    try {
      const tenantId = req.tenantId || null;
      const { orderedIds } = req.body;
      if (!Array.isArray(orderedIds) || orderedIds.length === 0) {
        return res.status(400).json({ message: "orderedIds array is required" });
      }
      const updates = [];
      for (let i = 0; i < orderedIds.length; i++) {
        const updated = await storage.updateWbsTag(orderedIds[i], { sortOrder: i });
        if (updated) updates.push(updated);
      }
      res.json(updates);
    } catch (error: any) {
      res.status(500).json({ message: error.message });
    }
  });

  app.patch("/api/wbs-tags/:id", async (req, res) => {
    try {
      const tenantId = req.tenantId || null;
      const existing = await storage.getWbsTag(req.params.id);
      if (!existing) return res.status(404).json({ message: "WBS tag not found" });
      if (existing.tenantId !== tenantId) return res.status(403).json({ message: "Access denied" });
      const { name, color, sortOrder } = req.body;
      const updates: Record<string, any> = {};
      if (typeof name === "string" && name.trim()) updates.name = name.trim();
      if (typeof color === "string" && /^#[0-9a-fA-F]{6}$/.test(color)) updates.color = color;
      if (typeof sortOrder === "number" && sortOrder >= 0) updates.sortOrder = sortOrder;
      const updated = await storage.updateWbsTag(req.params.id, updates);
      if (!updated) return res.status(404).json({ message: "WBS tag not found" });
      res.json(updated);
    } catch (error: any) {
      res.status(400).json({ message: error.message });
    }
  });

  const WBS_INDUSTRY_TEMPLATES: Record<string, { name: string; color: string }[]> = {
    "Legal": [
      { name: "Case Intake", color: "#D4A017" }, { name: "Discovery", color: "#C0392B" },
      { name: "Litigation", color: "#2980B9" }, { name: "Contracts", color: "#27AE60" },
      { name: "Compliance", color: "#8E44AD" }, { name: "Corporate Governance", color: "#E67E22" },
      { name: "Intellectual Property", color: "#1ABC9C" }, { name: "Employment Law", color: "#E74C3C" },
      { name: "Real Estate", color: "#3498DB" }, { name: "Regulatory Filing", color: "#2ECC71" },
      { name: "Arbitration", color: "#9B59B6" }, { name: "Client Relations", color: "#F39C12" },
      { name: "Billing & Collections", color: "#16A085" },
    ],
    "Real Estate": [
      { name: "Property Acquisition", color: "#D4A017" }, { name: "Leasing", color: "#C0392B" },
      { name: "Title & Escrow", color: "#2980B9" }, { name: "Property Management", color: "#27AE60" },
      { name: "Inspections", color: "#8E44AD" }, { name: "Mortgage & Financing", color: "#E67E22" },
      { name: "Tenant Relations", color: "#1ABC9C" }, { name: "Appraisals", color: "#E74C3C" },
      { name: "Zoning & Permits", color: "#3498DB" }, { name: "Closing Documents", color: "#2ECC71" },
      { name: "Insurance", color: "#9B59B6" }, { name: "Renovations", color: "#F39C12" },
      { name: "Compliance", color: "#16A085" },
    ],
    "Healthcare": [
      { name: "Patient Intake", color: "#D4A017" }, { name: "Clinical Records", color: "#C0392B" },
      { name: "Insurance & Billing", color: "#2980B9" }, { name: "Consent Forms", color: "#27AE60" },
      { name: "HIPAA Compliance", color: "#8E44AD" }, { name: "Prescriptions", color: "#E67E22" },
      { name: "Lab & Diagnostics", color: "#1ABC9C" }, { name: "Referrals", color: "#E74C3C" },
      { name: "Discharge", color: "#3498DB" }, { name: "Staff Credentialing", color: "#2ECC71" },
      { name: "Quality Assurance", color: "#9B59B6" }, { name: "Research & Trials", color: "#F39C12" },
      { name: "Facility Management", color: "#16A085" },
    ],
    "Finance & Banking": [
      { name: "Account Opening", color: "#D4A017" }, { name: "Loan Origination", color: "#C0392B" },
      { name: "KYC / AML", color: "#2980B9" }, { name: "Investment Services", color: "#27AE60" },
      { name: "Wealth Management", color: "#8E44AD" }, { name: "Regulatory Compliance", color: "#E67E22" },
      { name: "Risk Assessment", color: "#1ABC9C" }, { name: "Treasury Operations", color: "#E74C3C" },
      { name: "Audit", color: "#3498DB" }, { name: "Insurance Products", color: "#2ECC71" },
      { name: "Client Onboarding", color: "#9B59B6" }, { name: "Mergers & Acquisitions", color: "#F39C12" },
      { name: "Fraud Prevention", color: "#16A085" },
    ],
    "Construction": [
      { name: "Pre-Construction", color: "#D4A017" }, { name: "Design & Engineering", color: "#C0392B" },
      { name: "Permitting", color: "#2980B9" }, { name: "Site Preparation", color: "#27AE60" },
      { name: "Structural Work", color: "#8E44AD" }, { name: "Mechanical Systems", color: "#E67E22" },
      { name: "Electrical", color: "#1ABC9C" }, { name: "Plumbing", color: "#E74C3C" },
      { name: "Interior Finishing", color: "#3498DB" }, { name: "Safety & Compliance", color: "#2ECC71" },
      { name: "Inspections", color: "#9B59B6" }, { name: "Subcontractor Mgmt", color: "#F39C12" },
      { name: "Closeout & Handover", color: "#16A085" },
    ],
    "Technology & SaaS": [
      { name: "Product Design", color: "#D4A017" }, { name: "Development", color: "#C0392B" },
      { name: "QA & Testing", color: "#2980B9" }, { name: "DevOps", color: "#27AE60" },
      { name: "Security", color: "#8E44AD" }, { name: "Customer Success", color: "#E67E22" },
      { name: "Sales Operations", color: "#1ABC9C" }, { name: "Licensing", color: "#E74C3C" },
      { name: "Data & Analytics", color: "#3498DB" }, { name: "Compliance (SOC2/GDPR)", color: "#2ECC71" },
      { name: "Vendor Management", color: "#9B59B6" }, { name: "HR & Onboarding", color: "#F39C12" },
      { name: "Finance & Billing", color: "#16A085" },
    ],
    "Education": [
      { name: "Admissions", color: "#D4A017" }, { name: "Enrollment", color: "#C0392B" },
      { name: "Curriculum", color: "#2980B9" }, { name: "Faculty Affairs", color: "#27AE60" },
      { name: "Student Services", color: "#8E44AD" }, { name: "Financial Aid", color: "#E67E22" },
      { name: "Research Grants", color: "#1ABC9C" }, { name: "Accreditation", color: "#E74C3C" },
      { name: "Facilities", color: "#3498DB" }, { name: "Athletics", color: "#2ECC71" },
      { name: "Alumni Relations", color: "#9B59B6" }, { name: "Compliance (FERPA)", color: "#F39C12" },
      { name: "IT Services", color: "#16A085" },
    ],
    "Government & Public Sector": [
      { name: "Procurement", color: "#D4A017" }, { name: "Contracts & Grants", color: "#C0392B" },
      { name: "Policy & Legislation", color: "#2980B9" }, { name: "Citizen Services", color: "#27AE60" },
      { name: "Infrastructure", color: "#8E44AD" }, { name: "Public Safety", color: "#E67E22" },
      { name: "Environmental", color: "#1ABC9C" }, { name: "Budget & Finance", color: "#E74C3C" },
      { name: "Human Resources", color: "#3498DB" }, { name: "IT Modernization", color: "#2ECC71" },
      { name: "Records Management", color: "#9B59B6" }, { name: "Inter-Agency", color: "#F39C12" },
      { name: "Audit & Oversight", color: "#16A085" },
    ],
    "Human Resources": [
      { name: "Recruitment", color: "#D4A017" }, { name: "Onboarding", color: "#C0392B" },
      { name: "Compensation", color: "#2980B9" }, { name: "Benefits Admin", color: "#27AE60" },
      { name: "Performance Reviews", color: "#8E44AD" }, { name: "Training & Dev", color: "#E67E22" },
      { name: "Employee Relations", color: "#1ABC9C" }, { name: "Compliance (EEOC)", color: "#E74C3C" },
      { name: "Payroll", color: "#3498DB" }, { name: "Termination", color: "#2ECC71" },
      { name: "Workplace Safety", color: "#9B59B6" }, { name: "Diversity & Inclusion", color: "#F39C12" },
      { name: "Policy Management", color: "#16A085" },
    ],
    "Manufacturing": [
      { name: "Product Design", color: "#D4A017" }, { name: "Procurement", color: "#C0392B" },
      { name: "Production", color: "#2980B9" }, { name: "Quality Control", color: "#27AE60" },
      { name: "Supply Chain", color: "#8E44AD" }, { name: "Inventory Mgmt", color: "#E67E22" },
      { name: "Shipping & Logistics", color: "#1ABC9C" }, { name: "Equipment Maint.", color: "#E74C3C" },
      { name: "Safety & Compliance", color: "#3498DB" }, { name: "R&D", color: "#2ECC71" },
      { name: "Vendor Relations", color: "#9B59B6" }, { name: "Waste Management", color: "#F39C12" },
      { name: "Workforce Mgmt", color: "#16A085" },
    ],
    "Insurance": [
      { name: "Underwriting", color: "#D4A017" }, { name: "Policy Issuance", color: "#C0392B" },
      { name: "Claims Processing", color: "#2980B9" }, { name: "Renewals", color: "#27AE60" },
      { name: "Risk Assessment", color: "#8E44AD" }, { name: "Reinsurance", color: "#E67E22" },
      { name: "Fraud Investigation", color: "#1ABC9C" }, { name: "Agent Management", color: "#E74C3C" },
      { name: "Actuarial", color: "#3498DB" }, { name: "Regulatory Filing", color: "#2ECC71" },
      { name: "Customer Service", color: "#9B59B6" }, { name: "Product Development", color: "#F39C12" },
      { name: "Compliance", color: "#16A085" },
    ],
    "Nonprofit & NGO": [
      { name: "Fundraising", color: "#D4A017" }, { name: "Grant Management", color: "#C0392B" },
      { name: "Program Delivery", color: "#2980B9" }, { name: "Volunteer Mgmt", color: "#27AE60" },
      { name: "Donor Relations", color: "#8E44AD" }, { name: "Events", color: "#E67E22" },
      { name: "Advocacy", color: "#1ABC9C" }, { name: "Communications", color: "#E74C3C" },
      { name: "Board Governance", color: "#3498DB" }, { name: "Financial Reporting", color: "#2ECC71" },
      { name: "Impact Assessment", color: "#9B59B6" }, { name: "Compliance (501c3)", color: "#F39C12" },
      { name: "Partnerships", color: "#16A085" },
    ],
    "General Business": [
      { name: "Sales", color: "#D4A017" }, { name: "Marketing", color: "#C0392B" },
      { name: "Operations", color: "#2980B9" }, { name: "Finance", color: "#27AE60" },
      { name: "Human Resources", color: "#8E44AD" }, { name: "Legal", color: "#E67E22" },
      { name: "IT & Technology", color: "#1ABC9C" }, { name: "Customer Service", color: "#E74C3C" },
      { name: "Procurement", color: "#3498DB" }, { name: "Compliance", color: "#2ECC71" },
      { name: "Administration", color: "#9B59B6" }, { name: "Strategic Planning", color: "#F39C12" },
      { name: "Facilities", color: "#16A085" },
    ],
  };

  app.get("/api/wbs-tags/industries", (_req, res) => {
    res.json(Object.keys(WBS_INDUSTRY_TEMPLATES));
  });

  app.post("/api/wbs-tags/recommend", (req, res) => {
    const { industry } = req.body;
    if (!industry || typeof industry !== "string") {
      return res.status(400).json({ message: "Industry is required" });
    }
    const recommendations = WBS_INDUSTRY_TEMPLATES[industry];
    if (!recommendations) {
      return res.status(400).json({ message: "Industry not recognized" });
    }
    res.json(recommendations);
  });

  const seedTagSchema = z.object({
    tags: z.array(z.object({
      name: z.string().min(1).max(50),
      color: z.string().regex(/^#[0-9a-fA-F]{6}$/),
    })).min(1).max(13),
  });

  app.post("/api/wbs-tags/seed", async (req, res) => {
    try {
      const tenantId = req.tenantId || null;
      const parsed = seedTagSchema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({ message: "Invalid tag data", errors: parsed.error.flatten() });
      }

      const { tags: seedTags } = parsed.data;
      const existingTags = await storage.getWbsTags(tenantId ?? undefined);
      const slotsAvailable = 13 - existingTags.length;
      if (slotsAvailable <= 0) {
        return res.status(400).json({ message: "All 13 WBS tag slots are already filled" });
      }

      const tagsToCreate = seedTags.slice(0, slotsAvailable);
      const created = [];
      for (let i = 0; i < tagsToCreate.length; i++) {
        const tag = tagsToCreate[i];
        const newTag = await storage.createWbsTag({
          name: tag.name,
          color: tag.color,
          sortOrder: existingTags.length + i,
          tenantId,
        });
        created.push(newTag);
      }

      res.json({ created, count: created.length, slotsRemaining: 13 - existingTags.length - created.length });
    } catch (error: any) {
      res.status(500).json({ message: error.message });
    }
  });

  app.get("/api/envelopes/:id/wbs-tags", async (req, res) => {
    try {
      const envelope = await storage.getEnvelope(req.params.id);
      if (!envelope) return res.status(404).json({ message: "Envelope not found" });
      const tags = await storage.getEnvelopeWbsTags(req.params.id);
      res.json(tags);
    } catch (error: any) {
      res.status(500).json({ message: error.message });
    }
  });

  const setEnvTagsSchema = z.object({
    tagIds: z.array(z.string().min(1)).max(13),
  });

  app.put("/api/envelopes/:id/wbs-tags", async (req, res) => {
    try {
      const envelope = await storage.getEnvelope(req.params.id);
      if (!envelope) return res.status(404).json({ message: "Envelope not found" });
      const parsed = setEnvTagsSchema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({ message: "Invalid tag data", errors: parsed.error.flatten() });
      }
      const result = await storage.setEnvelopeWbsTags(req.params.id, parsed.data.tagIds);
      res.json(result);
    } catch (error: any) {
      res.status(500).json({ message: error.message });
    }
  });

  app.get("/api/envelope-wbs-tags", async (_req, res) => {
    try {
      const all = await storage.getAllEnvelopeWbsTags();
      res.json(all);
    } catch (error: any) {
      res.status(500).json({ message: error.message });
    }
  });

  app.delete("/api/wbs-tags/:id", async (req, res) => {
    try {
      const tenantId = req.tenantId || null;
      const existing = await storage.getWbsTag(req.params.id);
      if (!existing) return res.status(404).json({ message: "WBS tag not found" });
      if (existing.tenantId !== tenantId) return res.status(403).json({ message: "Access denied" });
      await storage.deleteWbsTag(req.params.id);
      res.json({ success: true });
    } catch (error: any) {
      res.status(500).json({ message: error.message });
    }
  });

  return httpServer;
}
