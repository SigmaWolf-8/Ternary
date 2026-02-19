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
import { PDFDocument, rgb, StandardFonts, PDFFont, PDFPage } from "pdf-lib";
import fontkit from "@pdf-lib/fontkit";
import fs from "fs";
import path from "path";

interface BakeField {
  id: string;
  type: string;
  label?: string | null;
  page: number;
  x: number;
  y: number;
  width: number;
  height: number;
  value?: string | null;
}

export interface CertificationData {
  title: string;
  certifiedAt: string;
  signerCount: number;
  signers: { name: string; email: string; signedAt: string | null }[];
  auditTrail: { action: string; actorName: string; details: string; hpTpTimestamp: string | null; createdAt: string }[];
  mlDsaSignature?: string;
}

const FONT_FILE_MAP: Record<number, string> = {
  0: "ArchitectsDaughter-Regular.ttf",
  1: "LibreBaskerville-Regular.ttf",
  2: "Lora-Regular.ttf",
  3: "GreatVibes-Regular.ttf",
  4: "DancingScript-Regular.ttf",
  5: "Pacifico-Regular.ttf",
  6: "Sacramento-Regular.ttf",
  7: "AlexBrush-Regular.ttf",
};

const fontCache = new Map<string, Uint8Array>();

function loadFontBytes(filename: string): Uint8Array {
  if (fontCache.has(filename)) return fontCache.get(filename)!;
  const fontPath = path.join(__dirname, "..", "fonts", filename);
  const bytes = new Uint8Array(fs.readFileSync(fontPath));
  fontCache.set(filename, bytes);
  return bytes;
}

function formatCertDate(isoStr: string): string {
  try {
    const d = new Date(isoStr);
    return d.toLocaleString("en-US", {
      year: "numeric",
      month: "short",
      day: "numeric",
      hour: "numeric",
      minute: "2-digit",
      hour12: true,
    });
  } catch {
    return isoStr;
  }
}

function drawWrappedText(
  page: PDFPage,
  text: string,
  x: number,
  y: number,
  maxWidth: number,
  font: PDFFont,
  size: number,
  color: ReturnType<typeof rgb>,
  lineHeight: number = size * 1.4
): number {
  const words = text.split(" ");
  let line = "";
  let currentY = y;

  for (const word of words) {
    const testLine = line ? `${line} ${word}` : word;
    const testWidth = font.widthOfTextAtSize(testLine, size);
    if (testWidth > maxWidth && line) {
      page.drawText(line, { x, y: currentY, size, font, color });
      currentY -= lineHeight;
      line = word;
    } else {
      line = testLine;
    }
  }
  if (line) {
    page.drawText(line, { x, y: currentY, size, font, color });
    currentY -= lineHeight;
  }
  return currentY;
}

export async function bakeFillablePdf(
  originalPdfBytes: Buffer,
  fieldsList: BakeField[],
  certData?: CertificationData
): Promise<Uint8Array> {
  const pdfDoc = await PDFDocument.load(originalPdfBytes);
  pdfDoc.registerFontkit(fontkit);

  const helvetica = await pdfDoc.embedFont(StandardFonts.Helvetica);
  const helveticaBold = await pdfDoc.embedFont(StandardFonts.HelveticaBold);
  const goldColor = rgb(0.78, 0.68, 0.0);

  const embeddedFonts = new Map<number, Awaited<ReturnType<typeof pdfDoc.embedFont>>>();

  async function getSignatureFont(fontIndex: number) {
    if (embeddedFonts.has(fontIndex)) return embeddedFonts.get(fontIndex)!;
    const filename = FONT_FILE_MAP[fontIndex];
    if (filename) {
      try {
        const fontBytes = loadFontBytes(filename);
        const font = await pdfDoc.embedFont(fontBytes);
        embeddedFonts.set(fontIndex, font);
        return font;
      } catch (e) {
        console.warn(`Failed to embed font ${filename}, falling back to Helvetica:`, e);
      }
    }
    return helvetica;
  }

  for (const f of fieldsList) {
    if (!f.value) continue;

    const pageIndex = Math.max(0, (f.page || 1) - 1);
    if (pageIndex >= pdfDoc.getPageCount()) continue;

    const page = pdfDoc.getPage(pageIndex);
    const pageHeight = page.getHeight();
    const pageWidth = page.getWidth();

    const scaleX = pageWidth / 800;
    const renderedHeight = (pageHeight / pageWidth) * 800;
    const scaleY = pageHeight / renderedHeight;

    const pdfX = f.x * scaleX;
    const pdfY = pageHeight - (f.y * scaleY) - (f.height * scaleY);

    switch (f.type) {
      case "signature":
      case "initials": {
        if (f.value.startsWith("typed:")) {
          const parts = f.value.split(":");
          const fontIndex = parseInt(parts[1]) || 0;
          const text = parts.slice(2).join(":");
          const sigFont = await getSignatureFont(fontIndex);
          const fontSize = f.type === "initials" ? 10 : 14;
          const scaledSize = fontSize * scaleX;

          let safeText = text;
          try {
            sigFont.encodeText(text);
          } catch {
            safeText = text.replace(/[^\x20-\x7E]/g, "");
          }

          page.drawText(safeText, {
            x: pdfX + 4,
            y: pdfY + (f.height * scaleY) / 2 - scaledSize / 2,
            size: scaledSize,
            font: sigFont,
            color: rgb(0, 0, 0),
          });
        } else if (f.value.startsWith("drawn:")) {
          try {
            const dataUrl = f.value.replace("drawn:", "");
            const base64 = dataUrl.split(",")[1];
            if (base64) {
              const imageBytes = Buffer.from(base64, "base64");
              const image = await pdfDoc.embedPng(imageBytes);
              const dims = image.scale(1);
              const fitW = f.width * scaleX;
              const fitH = f.height * scaleY;
              const aspectRatio = dims.width / dims.height;
              let drawW = fitW;
              let drawH = fitW / aspectRatio;
              if (drawH > fitH) {
                drawH = fitH;
                drawW = fitH * aspectRatio;
              }
              page.drawImage(image, {
                x: pdfX + (fitW - drawW) / 2,
                y: pdfY + (fitH - drawH) / 2,
                width: drawW,
                height: drawH,
              });
            }
          } catch {}
        }
        break;
      }
      case "date": {
        page.drawText(f.value, {
          x: pdfX + 4,
          y: pdfY + (f.height * scaleY) / 2 - 5,
          size: 10 * scaleX,
          font: helvetica,
          color: rgb(0, 0, 0),
        });
        break;
      }
      case "text": {
        if (f.label === "footer-line") {
          const lineY = pdfY + (f.height * scaleY) / 2;
          page.drawLine({
            start: { x: pdfX, y: lineY },
            end: { x: pdfX + f.width * scaleX, y: lineY },
            thickness: 1,
            color: rgb(0.3, 0.3, 0.3),
          });
        } else if (f.label === "seal") {
          const sealFontSize = 6.5 * scaleX;
          const fieldW = f.width * scaleX;
          const fieldH = f.height * scaleY;
          const sealColor = rgb(0.35, 0.35, 0.35);
          drawWrappedText(
            page,
            f.value,
            pdfX + 4,
            pdfY + fieldH - sealFontSize * 0.8,
            fieldW - 8,
            helvetica,
            sealFontSize,
            sealColor,
            sealFontSize * 1.3
          );
        } else {
          page.drawText(f.value, {
            x: pdfX + 4,
            y: pdfY + (f.height * scaleY) / 2 - 5,
            size: 10 * scaleX,
            font: helvetica,
            color: rgb(0, 0, 0),
          });
        }
        break;
      }
      case "checkbox": {
        if (f.value === "checked") {
          page.drawRectangle({
            x: pdfX + 2,
            y: pdfY + 2,
            width: f.width * scaleX - 4,
            height: f.height * scaleY - 4,
            color: goldColor,
            opacity: 0.3,
          });
          page.drawText("X", {
            x: pdfX + (f.width * scaleX) / 2 - 5,
            y: pdfY + (f.height * scaleY) / 2 - 5,
            size: 12 * scaleX,
            font: helvetica,
            color: rgb(0, 0, 0),
          });
        }
        break;
      }
    }
  }

  if (certData) {
    await appendCertificationPage(pdfDoc, certData, helvetica, helveticaBold, goldColor);
  }

  return await pdfDoc.save();
}

async function appendCertificationPage(
  pdfDoc: PDFDocument,
  cert: CertificationData,
  helvetica: PDFFont,
  helveticaBold: PDFFont,
  goldColor: ReturnType<typeof rgb>
) {
  const pageWidth = 612;
  const pageHeight = 792;
  const margin = 60;
  const contentWidth = pageWidth - margin * 2;
  const black = rgb(0, 0, 0);
  const darkGray = rgb(0.3, 0.3, 0.3);
  const medGray = rgb(0.5, 0.5, 0.5);
  const lightGray = rgb(0.85, 0.85, 0.85);

  let currentPage = pdfDoc.addPage([pageWidth, pageHeight]);
  let y = pageHeight - margin;

  function addGoldBar(pg: PDFPage) {
    pg.drawRectangle({ x: 0, y: pageHeight - 4, width: pageWidth, height: 4, color: goldColor });
  }

  let currentSection = "";

  function ensureSpace(needed: number): PDFPage {
    if (y < margin + needed) {
      currentPage = pdfDoc.addPage([pageWidth, pageHeight]);
      addGoldBar(currentPage);
      y = pageHeight - margin;
      if (currentSection) {
        currentPage.drawText(`${currentSection} (continued)`, {
          x: margin, y, size: 10, font: helveticaBold, color: goldColor,
        });
        y -= 18;
      }
    }
    return currentPage;
  }

  addGoldBar(currentPage);

  currentPage.drawText("DOCUMENT CERTIFIED", {
    x: margin, y, size: 22, font: helveticaBold, color: goldColor,
  });
  y -= 30;

  currentPage.drawRectangle({
    x: margin, y: y + 4, width: contentWidth, height: 1.5, color: goldColor,
  });
  y -= 10;

  currentPage.drawText(cert.title, {
    x: margin, y, size: 12, font: helveticaBold, color: black,
  });
  y -= 24;

  currentPage.drawText("HPTP Certification Engine", {
    x: margin, y, size: 10, font: helveticaBold, color: darkGray,
  });
  y -= 16;

  currentPage.drawText(formatCertDate(cert.certifiedAt), {
    x: margin, y, size: 10, font: helvetica, color: darkGray,
  });
  y -= 16;

  currentPage.drawText(`HPTP: ${cert.certifiedAt}`, {
    x: margin, y, size: 9, font: helvetica, color: medGray,
  });
  y -= 18;

  const summaryText = `All ${cert.signerCount}/${cert.signerCount} signers completed - document certified with femtosecond HPTP timestamp and ML-DSA signature`;
  y = drawWrappedText(currentPage, summaryText, margin, y, contentWidth, helvetica, 9, darkGray);
  y -= 16;

  if (cert.mlDsaSignature) {
    currentPage.drawText("ML-DSA Signature:", {
      x: margin, y, size: 8, font: helveticaBold, color: medGray,
    });
    y -= 12;
    const sigDisplay = cert.mlDsaSignature.length > 80
      ? cert.mlDsaSignature.substring(0, 80) + "..."
      : cert.mlDsaSignature;
    y = drawWrappedText(currentPage, sigDisplay, margin, y, contentWidth, helvetica, 7, medGray);
    y -= 10;
  }

  currentPage.drawRectangle({
    x: margin, y: y + 4, width: contentWidth, height: 1, color: lightGray,
  });
  y -= 20;

  currentSection = "SIGNERS";
  currentPage.drawText("SIGNERS", {
    x: margin, y, size: 11, font: helveticaBold, color: goldColor,
  });
  y -= 18;

  if (cert.signers.length === 0) {
    currentPage.drawText("No signers recorded", {
      x: margin + 4, y, size: 9, font: helvetica, color: medGray,
    });
    y -= 16;
  }

  for (const signer of cert.signers) {
    currentPage = ensureSpace(40);

    currentPage.drawText(signer.name, {
      x: margin + 4, y, size: 10, font: helveticaBold, color: black,
    });

    const emailX = margin + 4 + helveticaBold.widthOfTextAtSize(signer.name, 10) + 8;
    currentPage.drawText(`<${signer.email}>`, {
      x: emailX, y, size: 9, font: helvetica, color: medGray,
    });
    y -= 14;

    if (signer.signedAt) {
      currentPage.drawText(`Signed: ${formatCertDate(signer.signedAt)}`, {
        x: margin + 12, y, size: 8, font: helvetica, color: darkGray,
      });
    } else {
      currentPage.drawText("Status: Pending", {
        x: margin + 12, y, size: 8, font: helvetica, color: medGray,
      });
    }
    y -= 20;
  }

  currentPage.drawRectangle({
    x: margin, y: y + 8, width: contentWidth, height: 1, color: lightGray,
  });
  y -= 14;

  currentSection = "AUDIT TRAIL";
  currentPage.drawText("AUDIT TRAIL", {
    x: margin, y, size: 11, font: helveticaBold, color: goldColor,
  });
  y -= 18;

  if (cert.auditTrail.length === 0) {
    currentPage.drawText("No audit entries recorded", {
      x: margin + 4, y, size: 9, font: helvetica, color: medGray,
    });
    y -= 16;
  }

  for (const entry of cert.auditTrail) {
    currentPage = ensureSpace(60);
    y = drawAuditEntry(currentPage, entry, margin, y, contentWidth, helvetica, helveticaBold, black, darkGray, medGray);
  }

  y -= 10;
  currentPage = ensureSpace(30);
  currentPage.drawRectangle({
    x: margin, y: y + 8, width: contentWidth, height: 1, color: lightGray,
  });
  y -= 16;

  currentPage.drawText("Sign Here", {
    x: margin, y, size: 9, font: helveticaBold, color: goldColor,
  });
  const brandWidth = helveticaBold.widthOfTextAtSize("Sign Here", 9);
  currentPage.drawText("  |  Powered by HPTP Certification Engine  |  Ternary", {
    x: margin + brandWidth, y, size: 8, font: helvetica, color: medGray,
  });
}

function drawAuditEntry(
  page: PDFPage,
  entry: { action: string; actorName: string; details: string; hpTpTimestamp: string | null; createdAt: string },
  margin: number,
  y: number,
  contentWidth: number,
  helvetica: PDFFont,
  helveticaBold: PDFFont,
  black: ReturnType<typeof rgb>,
  darkGray: ReturnType<typeof rgb>,
  medGray: ReturnType<typeof rgb>
): number {
  page.drawText(entry.action, {
    x: margin + 4,
    y,
    size: 9,
    font: helveticaBold,
    color: black,
  });
  y -= 13;

  page.drawText(entry.actorName, {
    x: margin + 12,
    y,
    size: 8,
    font: helvetica,
    color: darkGray,
  });

  const timestamp = entry.hpTpTimestamp || entry.createdAt;
  const timeStr = formatCertDate(timestamp);
  const timeWidth = helvetica.widthOfTextAtSize(timeStr, 8);
  page.drawText(timeStr, {
    x: margin + contentWidth - timeWidth,
    y,
    size: 8,
    font: helvetica,
    color: medGray,
  });
  y -= 13;

  if (entry.details) {
    y = drawWrappedText(page, entry.details, margin + 12, y, contentWidth - 16, helvetica, 7.5, medGray, 10);
  }

  if (entry.hpTpTimestamp) {
    page.drawText(`HPTP: ${entry.hpTpTimestamp}`, {
      x: margin + 12,
      y,
      size: 7,
      font: helvetica,
      color: medGray,
    });
    y -= 10;
  }

  y -= 8;
  return y;
}
