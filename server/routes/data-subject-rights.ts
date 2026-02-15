/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * PROPRIETARY AND CONFIDENTIAL
 * All Rights Reserved.
 *
 * This file is part of the Salvi Framework / PlenumNET platform.
 * Unauthorized copying, modification, distribution, or use of this file,
 * via any medium, is strictly prohibited without the prior written
 * permission of Capomastro Holdings Ltd.
 *
 * See LICENSE in the repository root for full terms.
 */

import { type Express } from "express";
import { z } from "zod";
import { type IStorage } from "../storage";
import { isAuthenticated } from "../replit_integrations/auth";
import { createLogger } from "../logger";

const log = createLogger("gdpr");

export function registerDataSubjectRightsRoutes(app: Express, storage: IStorage): void {

  app.get("/api/gdpr/data-export", isAuthenticated, async (req: any, res) => {
    try {
      const userId = req.user?.claims?.sub;
      if (!userId) {
        return res.status(401).json({ error: "Authentication required" });
      }

      const dsrRequest = await storage.createDataSubjectRequest({
        userId,
        requestType: "access",
        status: "completed",
        responseData: null,
      });

      const userData = await storage.getUserData(userId);

      await storage.updateDataSubjectRequest(dsrRequest.id, "completed", { exported: true });

      log.info(`GDPR data export completed for user ${userId}`);
      res.json({
        requestId: dsrRequest.id,
        exportDate: new Date().toISOString(),
        dataController: "Capomastro Holdings Ltd.",
        dataControllerContact: "privacy@plenumnet.com",
        legalBasis: "GDPR Article 15 - Right of Access",
        data: userData,
      });
    } catch (error: unknown) {
      log.error("GDPR data export error:", error);
      res.status(500).json({ error: "Failed to export user data" });
    }
  });

  app.delete("/api/gdpr/delete-account", isAuthenticated, async (req: any, res) => {
    try {
      const userId = req.user?.claims?.sub;
      if (!userId) {
        return res.status(401).json({ error: "Authentication required" });
      }

      const confirmSchema = z.object({
        confirmation: z.literal("DELETE_MY_ACCOUNT"),
      });
      const parsed = confirmSchema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({
          error: "Confirmation required. Send { \"confirmation\": \"DELETE_MY_ACCOUNT\" } to proceed.",
        });
      }

      await storage.createDataSubjectRequest({
        userId,
        requestType: "erasure",
        status: "completed",
        responseData: { erasedAt: new Date().toISOString() },
      });

      await storage.deleteUserData(userId);

      log.info(`GDPR account deletion completed for user ${userId}`);
      res.json({
        message: "Account and associated data have been deleted",
        deletionDate: new Date().toISOString(),
        legalBasis: "GDPR Article 17 - Right to Erasure",
        retentionNote: "Audit logs may be retained for up to 90 days for legal compliance.",
      });
    } catch (error: unknown) {
      log.error("GDPR account deletion error:", error);
      res.status(500).json({ error: "Failed to delete account" });
    }
  });

  app.get("/api/gdpr/requests", isAuthenticated, async (req: any, res) => {
    try {
      const userId = req.user?.claims?.sub;
      if (!userId) {
        return res.status(401).json({ error: "Authentication required" });
      }

      const requests = await storage.getDataSubjectRequests(userId);
      res.json({ requests });
    } catch (error: unknown) {
      log.error("GDPR request history error:", error);
      res.status(500).json({ error: "Failed to retrieve request history" });
    }
  });

  app.get("/api/gdpr/policy", (_req, res) => {
    res.json({
      dataController: "Capomastro Holdings Ltd.",
      jurisdiction: "Canada",
      privacyOfficerContact: "privacy@plenumnet.com",
      applicableLaws: ["PIPEDA", "GDPR", "CCPA"],
      dataSubjectRights: [
        "Right of access (Art. 15)",
        "Right to rectification (Art. 16)",
        "Right to erasure (Art. 17)",
        "Right to data portability (Art. 20)",
        "Right to object (Art. 21)",
      ],
      responseTimeframe: "30 days from request",
      crossBorderTransfers: "EU/UK → Canada/US via Standard Contractual Clauses",
      retentionPolicy: {
        accountData: "Duration of account plus 30 days",
        securityLogs: "90 days",
        auditTrail: "7 years (regulatory requirement)",
      },
    });
  });
}
