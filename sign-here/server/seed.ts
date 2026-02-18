import { storage } from "./storage";
import { db } from "./db";
import { envelopes, templates } from "@shared/schema";
import { log } from "./index";
import { eq } from "drizzle-orm";

async function seedTemplates() {
  const existing = await storage.getTemplates();
  if (existing.length > 0) return;

  log("Seeding built-in templates...", "seed");

  await storage.createTemplate({
    name: "Non-Disclosure Agreement (NDA)",
    description: "Standard mutual NDA for business partnerships, vendor relationships, and confidential discussions",
    category: "Legal",
    tags: ["NDA", "confidential", "partnership"],
    isPublic: true,
    fieldDefs: [
      { type: "text", label: "Party A Name", page: 1, x: 60, y: 200, width: 220, height: 36, required: true },
      { type: "text", label: "Party B Name", page: 1, x: 320, y: 200, width: 220, height: 36, required: true },
      { type: "date", label: "Effective Date", page: 1, x: 60, y: 260, width: 150, height: 36, required: true },
      { type: "signature", label: "Party A Signature", page: 1, x: 60, y: 500, width: 200, height: 50, required: true },
      { type: "date", label: "Date Signed", page: 1, x: 280, y: 510, width: 140, height: 36, required: true },
      { type: "signature", label: "Party B Signature", page: 1, x: 60, y: 580, width: 200, height: 50, required: true },
      { type: "date", label: "Date Signed", page: 1, x: 280, y: 590, width: 140, height: 36, required: true },
    ],
  });

  await storage.createTemplate({
    name: "Employment Offer Letter",
    description: "Standard employment offer with position details, compensation, and start date",
    category: "HR",
    tags: ["employment", "offer", "hiring"],
    isPublic: true,
    fieldDefs: [
      { type: "text", label: "Candidate Name", page: 1, x: 60, y: 180, width: 250, height: 36, required: true },
      { type: "text", label: "Position Title", page: 1, x: 60, y: 240, width: 250, height: 36, required: true },
      { type: "text", label: "Annual Salary", page: 1, x: 60, y: 300, width: 180, height: 36, required: true },
      { type: "date", label: "Start Date", page: 1, x: 280, y: 300, width: 150, height: 36, required: true },
      { type: "signature", label: "Candidate Signature", page: 1, x: 60, y: 500, width: 200, height: 50, required: true },
      { type: "date", label: "Date", page: 1, x: 280, y: 510, width: 140, height: 36, required: true },
      { type: "signature", label: "Employer Signature", page: 1, x: 60, y: 580, width: 200, height: 50, required: true },
    ],
  });

  await storage.createTemplate({
    name: "Service Agreement",
    description: "Professional services contract covering scope, payment terms, and deliverables",
    category: "Legal",
    tags: ["services", "contract", "consulting"],
    isPublic: true,
    fieldDefs: [
      { type: "text", label: "Service Provider", page: 1, x: 60, y: 180, width: 220, height: 36, required: true },
      { type: "text", label: "Client Name", page: 1, x: 320, y: 180, width: 220, height: 36, required: true },
      { type: "text", label: "Scope of Work", page: 1, x: 60, y: 300, width: 480, height: 60, required: true },
      { type: "text", label: "Total Fee", page: 1, x: 60, y: 400, width: 180, height: 36, required: true },
      { type: "signature", label: "Provider Signature", page: 1, x: 60, y: 500, width: 200, height: 50, required: true },
      { type: "signature", label: "Client Signature", page: 1, x: 60, y: 580, width: 200, height: 50, required: true },
    ],
  });

  await storage.createTemplate({
    name: "Lease Agreement",
    description: "Residential or commercial property lease agreement with terms and conditions",
    category: "Real Estate",
    tags: ["lease", "property", "rental"],
    isPublic: true,
    fieldDefs: [
      { type: "text", label: "Landlord Name", page: 1, x: 60, y: 180, width: 220, height: 36, required: true },
      { type: "text", label: "Tenant Name", page: 1, x: 320, y: 180, width: 220, height: 36, required: true },
      { type: "text", label: "Property Address", page: 1, x: 60, y: 240, width: 480, height: 36, required: true },
      { type: "text", label: "Monthly Rent", page: 1, x: 60, y: 300, width: 180, height: 36, required: true },
      { type: "date", label: "Lease Start", page: 1, x: 280, y: 300, width: 130, height: 36, required: true },
      { type: "date", label: "Lease End", page: 1, x: 430, y: 300, width: 130, height: 36, required: true },
      { type: "initials", label: "Tenant Initials", page: 1, x: 60, y: 460, width: 100, height: 40, required: true },
      { type: "signature", label: "Landlord Signature", page: 1, x: 60, y: 520, width: 200, height: 50, required: true },
      { type: "signature", label: "Tenant Signature", page: 1, x: 60, y: 600, width: 200, height: 50, required: true },
    ],
  });

  await storage.createTemplate({
    name: "Consent Form",
    description: "General consent form for data processing, medical procedures, or research participation",
    category: "Healthcare",
    tags: ["consent", "HIPAA", "authorization"],
    isPublic: true,
    fieldDefs: [
      { type: "text", label: "Patient/Participant Name", page: 1, x: 60, y: 200, width: 280, height: 36, required: true },
      { type: "date", label: "Date of Birth", page: 1, x: 380, y: 200, width: 150, height: 36, required: true },
      { type: "checkbox", label: "I consent to the terms", page: 1, x: 60, y: 420, width: 28, height: 28, required: true },
      { type: "signature", label: "Signature", page: 1, x: 60, y: 500, width: 200, height: 50, required: true },
      { type: "date", label: "Date", page: 1, x: 280, y: 510, width: 140, height: 36, required: true },
      { type: "signature", label: "Witness Signature", page: 1, x: 60, y: 580, width: 200, height: 50, required: false },
    ],
  });

  await storage.createTemplate({
    name: "Invoice",
    description: "Standard invoice template for billing clients with itemized charges",
    category: "Finance",
    tags: ["invoice", "billing", "payment"],
    isPublic: true,
    fieldDefs: [
      { type: "text", label: "Invoice Number", page: 1, x: 350, y: 100, width: 160, height: 36, required: true },
      { type: "date", label: "Invoice Date", page: 1, x: 350, y: 150, width: 160, height: 36, required: true },
      { type: "text", label: "Bill To", page: 1, x: 60, y: 200, width: 250, height: 36, required: true },
      { type: "text", label: "Amount Due", page: 1, x: 350, y: 400, width: 160, height: 36, required: true },
      { type: "signature", label: "Authorized Signature", page: 1, x: 60, y: 540, width: 200, height: 50, required: true },
    ],
  });

  log("Built-in templates seeded", "seed");
}

async function seedSAdmin() {
  const allUsers = await storage.getAllUsers();
  const hasSAdmin = allUsers.some((u: any) => u.role === "sadmin" && u.isPlatformCreator);
  if (hasSAdmin) return;

  log("Seeding SAdmin (platform creator)...", "seed");
  await storage.createUser({
    username: "sadmin",
    password: "admin",
    email: "admin@signhere.io",
    role: "sadmin",
    isPlatformCreator: true,
    tenantId: "default",
  });
  log("SAdmin user created (username: sadmin)", "seed");
}

export async function seedDatabase() {
  await seedSAdmin();

  const existing = await storage.getEnvelopes();
  if (existing.length > 0) {
    await seedTemplates();
    await seedWbsTags();
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

  await seedTemplates();
  await seedWbsTags();

  log("Database seeded successfully", "seed");
}

async function seedWbsTags() {
  const existing = await storage.getWbsTags();
  if (existing.length > 0) return;

  log("Seeding WBS tags...", "seed");

  const tags = [
    { name: "Construction Agreement", color: "#C0392B", sortOrder: 0 },
    { name: "Lease Agreement", color: "#C0392B", sortOrder: 1 },
    { name: "Change Order", color: "#C0392B", sortOrder: 2 },
    { name: "Pre Occupancy Closeout", color: "#2980B9", sortOrder: 3 },
    { name: "Subcontract Agreement", color: "#2980B9", sortOrder: 4 },
    { name: "Consulting Agreement", color: "#2980B9", sortOrder: 5 },
    { name: "Employment Offer", color: "#3498DB", sortOrder: 6 },
    { name: "Human Resources", color: "#8E44AD", sortOrder: 7 },
    { name: "NDA - Non Disclosure Agreement", color: "#27AE60", sortOrder: 8 },
    { name: "Safety & Compliance", color: "#F39C12", sortOrder: 9 },
    { name: "Legal", color: "#E67E22", sortOrder: 10 },
  ];

  for (const tag of tags) {
    await storage.createWbsTag(tag);
  }

  log("WBS tags seeded successfully", "seed");
}
