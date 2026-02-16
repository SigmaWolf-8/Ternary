import type { Express } from "express";
import { createServer, type Server } from "http";
import { storage } from "./storage";
import { insertEnvelopeSchema, insertRecipientSchema } from "@shared/schema";
import { z } from "zod";

export async function registerRoutes(
  httpServer: Server,
  app: Express
): Promise<Server> {

  app.get("/api/envelopes", async (_req, res) => {
    const envelopes = await storage.getEnvelopes();
    res.json(envelopes);
  });

  app.get("/api/envelopes/:id", async (req, res) => {
    const envelope = await storage.getEnvelope(req.params.id);
    if (!envelope) return res.status(404).json({ message: "Not found" });
    res.json(envelope);
  });

  app.post("/api/envelopes", async (req, res) => {
    try {
      const { title, description, recipients: recipientList } = req.body;

      const parsed = insertEnvelopeSchema.parse({ title, description, status: "draft" });
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

      await storage.createAuditLog({
        envelopeId: envelope.id,
        action: "Envelope created",
        actorName: "System",
        details: `Created with ${recipientList?.length || 0} recipient(s)`,
      });

      res.json(envelope);
    } catch (error: any) {
      res.status(400).json({ message: error.message });
    }
  });

  app.patch("/api/envelopes/:id", async (req, res) => {
    try {
      const envelope = await storage.updateEnvelope(req.params.id, req.body);
      if (!envelope) return res.status(404).json({ message: "Not found" });

      if (req.body.status === "sent") {
        await storage.createAuditLog({
          envelopeId: envelope.id,
          action: "Envelope sent for signing",
          actorName: "System",
          details: "Document sent to all recipients",
        });
      }

      res.json(envelope);
    } catch (error: any) {
      res.status(400).json({ message: error.message });
    }
  });

  app.delete("/api/envelopes/:id", async (req, res) => {
    await storage.deleteEnvelope(req.params.id);
    res.json({ success: true });
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

      if (fieldValues && typeof fieldValues === "object") {
        for (const [fieldId, value] of Object.entries(fieldValues)) {
          const existingFields = await storage.getFieldsByEnvelope(envelopeId);
          const field = existingFields.find((f) => f.id === fieldId);
          if (field) {
            await storage.updateField(fieldId, { value: value as string });
          }
        }
      }

      await storage.updateRecipient(recipientId, {
        status: "signed",
        signedAt: new Date(),
      });

      await storage.createAuditLog({
        envelopeId,
        action: "Document signed",
        actorName: recipient.name,
        details: `Signed by ${recipient.name} (${recipient.email})`,
      });

      const allRecipients = await storage.getRecipientsByEnvelope(envelopeId);
      const signers = allRecipients.filter((r) => r.role === "signer");
      const allSigned = signers.every((r) => r.status === "signed" || r.id === recipientId);

      if (allSigned) {
        await storage.updateEnvelope(envelopeId, { status: "completed" });
        await storage.createAuditLog({
          envelopeId,
          action: "Envelope completed",
          actorName: "System",
          details: "All signers have signed the document",
        });
      } else {
        await storage.updateEnvelope(envelopeId, { status: "signing" });
      }

      res.json({ success: true });
    } catch (error: any) {
      res.status(400).json({ message: error.message });
    }
  });

  app.get("/api/envelopes/:id/audit", async (req, res) => {
    const logs = await storage.getAuditLogsByEnvelope(req.params.id);
    res.json(logs);
  });

  return httpServer;
}
