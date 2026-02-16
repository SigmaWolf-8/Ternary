import type { Express } from "express";
import { createServer, type Server } from "http";
import { storage } from "./storage";
import { insertEnvelopeSchema, insertRecipientSchema } from "@shared/schema";
import { z } from "zod";
import { tenantMiddleware } from "./middleware/tenant";
import { healthCheck as plenumHealthCheck } from "./services/plenum";
import { saveEnvelope as hybridSave } from "./services/saveCopy";
import { getHPTP, witnessSign, mlDsaSign } from "./services/plenum";
import { bakeFillablePdf, type CertificationData } from "./services/pdfForms";
import { generateZKProof, verifyZKProofServer } from "./services/zk";
import { encryptPdf, decryptPdf } from "./services/pdfCrypto";
import { sendEnvelopeEmails } from "./services/email";

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
        const emailResult = await sendEnvelopeEmails(
          envelope.id,
          envelope.title,
          "SalviSign User",
          allRecipients.map((r) => ({ id: r.id, email: r.email, name: r.name, role: r.role }))
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

      const { pdfData, pageCount } = req.body;
      if (!pdfData) return res.status(400).json({ message: "pdfData (base64) is required" });

      const encryptedPdf = encryptPdf(pdfData);
      const updated = await storage.updateEnvelope(req.params.id, {
        pdfData: encryptedPdf,
        pageCount: pageCount || 1,
      });

      const epoch = await getHPTP();
      await storage.createAuditLog({
        envelopeId: req.params.id,
        tenantId: req.tenantId || null,
        action: "PDF uploaded",
        actorName: "System",
        details: `PDF with ${pageCount || 1} page(s) attached`,
        hpTpTimestamp: epoch,
      });

      res.json({ success: true, pageCount: updated?.pageCount });
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

      const signEpoch = await getHPTP();

      await witnessSign(recipientId, "recipient_sign", req.tenantId || "default");

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
        metadata: { recipientId, fieldCount: Object.keys(fieldValues || {}).length },
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
    res.json(logs);
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

  app.post("/api/tenants", async (req, res) => {
    try {
      const tenant = await storage.createTenant(req.body);
      res.status(201).json(tenant);
    } catch (error: any) {
      res.status(400).json({ message: error.message });
    }
  });

  app.post("/api/envelopes/:id/share-proof", async (req, res) => {
    try {
      const envelopeId = req.params.id;
      const tenantId = req.tenantId || "default";
      const clientIP = req.headers["x-forwarded-for"] || req.socket.remoteAddress || "unknown";

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
          ipAddress: typeof clientIP === "string" ? clientIP : clientIP[0],
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

      const clientIP = req.headers["x-forwarded-for"] || req.socket.remoteAddress || "unknown";
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
          ipAddress: typeof clientIP === "string" ? clientIP : clientIP[0],
          verified: isValid,
        },
      });

      res.json({ valid: isValid });
    } catch (error: any) {
      res.status(400).json({ message: error.message });
    }
  });

  return httpServer;
}
