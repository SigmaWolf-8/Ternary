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
interface SuggestedField {
  type: "signature" | "date" | "text" | "checkbox" | "initials";
  label: string;
  page: number;
  x: number;
  y: number;
  width: number;
  height: number;
  confidence: number;
  reason: string;
}

const FIELD_PATTERNS: Array<{
  regex: RegExp;
  type: SuggestedField["type"];
  label: string;
  width: number;
  height: number;
}> = [
  { regex: /\b(signature|sign\s+here|authorized\s+signature|signatory|signer)\b/i, type: "signature", label: "Signature", width: 200, height: 60 },
  { regex: /\b(date|dated|effective\s+date|signed?\s+date)\b/i, type: "date", label: "Date", width: 140, height: 36 },
  { regex: /\b(print\s+name|full\s+name|printed\s+name|name\s*:)\b/i, type: "text", label: "Full Name", width: 180, height: 36 },
  { regex: /\b(title|position|job\s+title)\b/i, type: "text", label: "Title", width: 180, height: 36 },
  { regex: /\b(email|e-mail)\s*(address)?\s*:?\b/i, type: "text", label: "Email", width: 180, height: 36 },
  { regex: /\b(phone|telephone|tel)\s*(number)?\s*:?\b/i, type: "text", label: "Phone", width: 140, height: 36 },
  { regex: /\b(address|street|city|state|zip)\s*:?\b/i, type: "text", label: "Address", width: 200, height: 36 },
  { regex: /\b(initial|initials)\b/i, type: "initials", label: "Initials", width: 80, height: 40 },
  { regex: /\b(agree|i\s+agree|accept|acknowledge|consent|confirm)\b/i, type: "checkbox", label: "Agreement", width: 28, height: 28 },
  { regex: /\b(witness|witnessed\s+by)\b/i, type: "signature", label: "Witness Signature", width: 200, height: 60 },
  { regex: /\b(company|organization|employer)\s*(name)?\s*:?\b/i, type: "text", label: "Company", width: 180, height: 36 },
  { regex: /\b(ssn|social\s+security|tax\s+id|ein)\b/i, type: "text", label: "ID Number", width: 140, height: 36 },
];

const DOC_TYPE_PATTERNS: Array<{
  regex: RegExp;
  docType: string;
  fields: Array<{ type: SuggestedField["type"]; label: string; width: number; height: number }>;
}> = [
  {
    regex: /\b(non[\s-]?disclosure|nda|confidentiality)\b/i,
    docType: "NDA",
    fields: [
      { type: "signature", label: "Party 1 Signature", width: 200, height: 60 },
      { type: "date", label: "Date", width: 140, height: 36 },
      { type: "text", label: "Print Name", width: 180, height: 36 },
      { type: "signature", label: "Party 2 Signature", width: 200, height: 60 },
      { type: "date", label: "Date", width: 140, height: 36 },
      { type: "text", label: "Print Name", width: 180, height: 36 },
    ],
  },
  {
    regex: /\b(employment|offer\s+letter|employment\s+agreement)\b/i,
    docType: "Employment",
    fields: [
      { type: "signature", label: "Employee Signature", width: 200, height: 60 },
      { type: "date", label: "Date", width: 140, height: 36 },
      { type: "text", label: "Employee Name", width: 180, height: 36 },
      { type: "signature", label: "Employer Signature", width: 200, height: 60 },
      { type: "text", label: "Title", width: 180, height: 36 },
    ],
  },
  {
    regex: /\b(lease|rental|tenancy)\b/i,
    docType: "Lease",
    fields: [
      { type: "signature", label: "Landlord Signature", width: 200, height: 60 },
      { type: "signature", label: "Tenant Signature", width: 200, height: 60 },
      { type: "date", label: "Date", width: 140, height: 36 },
      { type: "text", label: "Print Name", width: 180, height: 36 },
      { type: "initials", label: "Initials", width: 80, height: 40 },
    ],
  },
  {
    regex: /\b(purchase|sales|sale\s+agreement|bill\s+of\s+sale)\b/i,
    docType: "Purchase Agreement",
    fields: [
      { type: "signature", label: "Buyer Signature", width: 200, height: 60 },
      { type: "signature", label: "Seller Signature", width: 200, height: 60 },
      { type: "date", label: "Date", width: 140, height: 36 },
      { type: "text", label: "Print Name", width: 180, height: 36 },
    ],
  },
  {
    regex: /\b(consent|authorization|waiver|release)\b/i,
    docType: "Consent Form",
    fields: [
      { type: "checkbox", label: "I Agree", width: 28, height: 28 },
      { type: "signature", label: "Signature", width: 200, height: 60 },
      { type: "date", label: "Date", width: 140, height: 36 },
      { type: "text", label: "Print Name", width: 180, height: 36 },
    ],
  },
];

export function detectFields(text: string, pageCount: number): { docType: string; fields: SuggestedField[] } {
  const suggestions: SuggestedField[] = [];
  let detectedDocType = "General Document";

  for (const dp of DOC_TYPE_PATTERNS) {
    if (dp.regex.test(text)) {
      detectedDocType = dp.docType;
      const lastPage = pageCount;
      let yPos = 500;
      for (const df of dp.fields) {
        suggestions.push({
          type: df.type,
          label: df.label,
          page: lastPage,
          x: 50,
          y: yPos,
          width: df.width,
          height: df.height,
          confidence: 0.85,
          reason: `Standard ${dp.docType} field`,
        });
        yPos += df.height + 15;
      }
      break;
    }
  }

  const lines = text.split("\n");
  for (let lineIdx = 0; lineIdx < lines.length; lineIdx++) {
    const line = lines[lineIdx];
    for (const pattern of FIELD_PATTERNS) {
      if (pattern.regex.test(line)) {
        const alreadyHas = suggestions.some(
          (s) => s.type === pattern.type && s.label === pattern.label
        );
        if (!alreadyHas) {
          const page = Math.min(Math.ceil((lineIdx / lines.length) * pageCount) || 1, pageCount);
          const yEstimate = Math.round(((lineIdx % Math.ceil(lines.length / pageCount)) / Math.ceil(lines.length / pageCount)) * 800) + 50;
          suggestions.push({
            type: pattern.type,
            label: pattern.label,
            page,
            x: 50,
            y: Math.min(yEstimate, 900),
            width: pattern.width,
            height: pattern.height,
            confidence: 0.7,
            reason: `Keyword match: "${line.trim().substring(0, 50)}"`,
          });
        }
      }
    }
  }

  if (suggestions.length === 0) {
    const lastPage = pageCount;
    suggestions.push(
      { type: "signature", label: "Signature", page: lastPage, x: 50, y: 600, width: 200, height: 60, confidence: 0.5, reason: "Default signature field" },
      { type: "date", label: "Date", page: lastPage, x: 50, y: 670, width: 140, height: 36, confidence: 0.5, reason: "Default date field" },
      { type: "text", label: "Print Name", page: lastPage, x: 50, y: 716, width: 180, height: 36, confidence: 0.5, reason: "Default name field" },
    );
  }

  return { docType: detectedDocType, fields: suggestions };
}
