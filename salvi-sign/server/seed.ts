import { storage } from "./storage";
import { db } from "./db";
import { envelopes } from "@shared/schema";
import { log } from "./index";

export async function seedDatabase() {
  const existing = await storage.getEnvelopes();
  if (existing.length > 0) {
    return;
  }

  log("Seeding database with sample data...", "seed");

  const env1 = await storage.createEnvelope({
    title: "Non-Disclosure Agreement - Acme Corp",
    description: "Standard NDA for the partnership deal with Acme Corporation",
    status: "completed",
  });

  const r1a = await storage.createRecipient({
    envelopeId: env1.id,
    name: "Sarah Chen",
    email: "sarah@acmecorp.com",
    role: "signer",
    status: "signed",
    sortOrder: 0,
  });
  const r1b = await storage.createRecipient({
    envelopeId: env1.id,
    name: "Marcus Johnson",
    email: "marcus@partner.co",
    role: "signer",
    status: "signed",
    sortOrder: 1,
  });

  await storage.createField({
    envelopeId: env1.id,
    recipientId: r1a.id,
    type: "signature",
    label: null,
    page: 1,
    x: 60,
    y: 650,
    width: 200,
    height: 60,
    value: "typed:0:Sarah Chen",
    required: true,
  });
  await storage.createField({
    envelopeId: env1.id,
    recipientId: r1a.id,
    type: "date",
    label: null,
    page: 1,
    x: 300,
    y: 665,
    width: 140,
    height: 36,
    value: "2/10/2026",
    required: true,
  });
  await storage.createField({
    envelopeId: env1.id,
    recipientId: r1b.id,
    type: "signature",
    label: null,
    page: 1,
    x: 60,
    y: 740,
    width: 200,
    height: 60,
    value: "typed:1:Marcus Johnson",
    required: true,
  });

  await storage.createAuditLog({ envelopeId: env1.id, action: "Envelope created", actorName: "System", details: "Created with 2 recipients" });
  await storage.createAuditLog({ envelopeId: env1.id, action: "Envelope sent for signing", actorName: "System", details: "Document sent to all recipients" });
  await storage.createAuditLog({ envelopeId: env1.id, action: "Document signed", actorName: "Sarah Chen", details: "Signed by Sarah Chen (sarah@acmecorp.com)" });
  await storage.createAuditLog({ envelopeId: env1.id, action: "Document signed", actorName: "Marcus Johnson", details: "Signed by Marcus Johnson (marcus@partner.co)" });
  await storage.createAuditLog({ envelopeId: env1.id, action: "Envelope completed", actorName: "System", details: "All signers have signed the document" });

  const env2 = await storage.createEnvelope({
    title: "Software License Agreement",
    description: "Enterprise license for TernarySoft platform access",
    status: "sent",
  });

  const r2a = await storage.createRecipient({
    envelopeId: env2.id,
    name: "Elena Rodriguez",
    email: "elena@ternarysoft.io",
    role: "signer",
    status: "pending",
    sortOrder: 0,
  });
  const r2b = await storage.createRecipient({
    envelopeId: env2.id,
    name: "David Kim",
    email: "david@enterprise.com",
    role: "signer",
    status: "pending",
    sortOrder: 1,
  });

  await storage.createField({
    envelopeId: env2.id,
    recipientId: r2a.id,
    type: "signature",
    label: null,
    page: 1,
    x: 60,
    y: 600,
    width: 200,
    height: 60,
    value: null,
    required: true,
  });
  await storage.createField({
    envelopeId: env2.id,
    recipientId: r2a.id,
    type: "date",
    label: null,
    page: 1,
    x: 300,
    y: 615,
    width: 140,
    height: 36,
    value: null,
    required: true,
  });
  await storage.createField({
    envelopeId: env2.id,
    recipientId: r2b.id,
    type: "signature",
    label: null,
    page: 1,
    x: 60,
    y: 700,
    width: 200,
    height: 60,
    value: null,
    required: true,
  });
  await storage.createField({
    envelopeId: env2.id,
    recipientId: r2b.id,
    type: "text",
    label: null,
    page: 1,
    x: 300,
    y: 715,
    width: 180,
    height: 36,
    value: null,
    required: true,
  });

  await storage.createAuditLog({ envelopeId: env2.id, action: "Envelope created", actorName: "System", details: "Created with 2 recipients" });
  await storage.createAuditLog({ envelopeId: env2.id, action: "Envelope sent for signing", actorName: "System", details: "Document sent to all recipients" });

  const env3 = await storage.createEnvelope({
    title: "Employment Offer Letter - Senior Engineer",
    description: "Full-time offer for the senior engineering position",
    status: "draft",
  });

  await storage.createRecipient({
    envelopeId: env3.id,
    name: "Alex Nakamura",
    email: "alex.n@gmail.com",
    role: "signer",
    status: "pending",
    sortOrder: 0,
  });
  await storage.createRecipient({
    envelopeId: env3.id,
    name: "HR Department",
    email: "hr@salvitech.com",
    role: "witness",
    status: "pending",
    sortOrder: 1,
  });

  await storage.createAuditLog({ envelopeId: env3.id, action: "Envelope created", actorName: "System", details: "Created with 2 recipients" });

  const env4 = await storage.createEnvelope({
    title: "Consulting Services Agreement",
    description: "Q1 2026 consulting engagement terms",
    status: "signing",
  });

  const r4a = await storage.createRecipient({
    envelopeId: env4.id,
    name: "Priya Sharma",
    email: "priya@consultpro.com",
    role: "signer",
    status: "signed",
    sortOrder: 0,
  });
  await storage.createRecipient({
    envelopeId: env4.id,
    name: "James Wright",
    email: "james@clientco.com",
    role: "signer",
    status: "pending",
    sortOrder: 1,
  });

  await storage.createField({
    envelopeId: env4.id,
    recipientId: r4a.id,
    type: "signature",
    label: null,
    page: 1,
    x: 60,
    y: 650,
    width: 200,
    height: 60,
    value: "typed:3:Priya Sharma",
    required: true,
  });

  await storage.createAuditLog({ envelopeId: env4.id, action: "Envelope created", actorName: "System", details: "Created with 2 recipients" });
  await storage.createAuditLog({ envelopeId: env4.id, action: "Envelope sent for signing", actorName: "System", details: "Document sent to all recipients" });
  await storage.createAuditLog({ envelopeId: env4.id, action: "Document signed", actorName: "Priya Sharma", details: "Signed by Priya Sharma (priya@consultpro.com)" });

  log("Database seeded successfully", "seed");
}
