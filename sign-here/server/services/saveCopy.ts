/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Applied Physics Division
 *
 * PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
 * Patent(s) Pending.
 *
 * This file is part of the Salvi Framework / PlenumNET platform.
 * Unauthorized copying, modification, distribution, or use of this file,
 * via any medium, is strictly prohibited without the prior written
 * permission of Capomastro Holdings Ltd.
 *
 * See LICENSE in the repository root for full terms.
 */
import { db } from "../db";
import { envelopes, auditLogs } from "@shared/schema";
import { secureDoc, witnessSign, getHPTP } from "./plenum";

interface SaveEnvelopeInput {
  pdfBytes: Buffer;
  fields: any[];
  tenantId: string;
  title?: string;
  description?: string;
}

export async function saveEnvelope({ pdfBytes, fields, tenantId, title, description }: SaveEnvelopeInput) {
  const { id: plenumDocId } = await secureDoc(pdfBytes, tenantId);

  const witnessedFields = await Promise.all(
    fields.map(async (f) => {
      if (f.type === "signature" && f.value) {
        try {
          const witness = await witnessSign(f.id, f.value, tenantId);
          return { ...f, witnessProof: witness };
        } catch (err) {
          console.warn(`Witness failed for field ${f.id}:`, err);
          return f;
        }
      }
      return f;
    })
  );

  const [newEnvelope] = await db
    .insert(envelopes)
    .values({
      tenantId,
      plenumDocId,
      title: title || "Untitled Document",
      description: description || null,
      status: "draft",
    })
    .returning();

  const epoch = await getHPTP();
  await db.insert(auditLogs).values({
    envelopeId: newEnvelope.id,
    tenantId,
    action: "Envelope created via PlenumDB",
    actorName: "System",
    hpTpTimestamp: epoch,
    metadata: { fieldCount: fields.length, plenumDocId, witnessedCount: witnessedFields.filter((f: any) => f.witnessProof).length },
  });

  return {
    envelopeId: newEnvelope.id,
    plenumDocId,
    status: newEnvelope.status,
  };
}
