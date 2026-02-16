import { PDFDocument, rgb, StandardFonts } from "pdf-lib";

interface BakeField {
  id: string;
  type: string;
  page: number;
  x: number;
  y: number;
  width: number;
  height: number;
  value?: string | null;
}

export async function bakeFillablePdf(
  originalPdfBytes: Buffer,
  fieldsList: BakeField[]
): Promise<Uint8Array> {
  const pdfDoc = await PDFDocument.load(originalPdfBytes);
  const helvetica = await pdfDoc.embedFont(StandardFonts.Helvetica);
  const goldColor = rgb(0.78, 0.68, 0.0);

  for (const f of fieldsList) {
    if (!f.value) continue;

    const pageIndex = Math.max(0, (f.page || 1) - 1);
    if (pageIndex >= pdfDoc.getPageCount()) continue;

    const page = pdfDoc.getPage(pageIndex);
    const pageHeight = page.getHeight();
    const pageWidth = page.getWidth();

    const scaleX = pageWidth / 800;
    const scaleY = pageHeight / 1131;

    const pdfX = f.x * scaleX;
    const pdfY = pageHeight - (f.y * scaleY) - (f.height * scaleY);

    switch (f.type) {
      case "signature":
      case "initials": {
        if (f.value.startsWith("typed:")) {
          const parts = f.value.split(":");
          const text = parts.slice(2).join(":");
          const fontSize = f.type === "initials" ? 10 : 14;
          page.drawText(text, {
            x: pdfX + 4,
            y: pdfY + (f.height * scaleY) / 2 - fontSize / 2,
            size: fontSize * scaleX,
            font: helvetica,
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
        page.drawText(f.value, {
          x: pdfX + 4,
          y: pdfY + (f.height * scaleY) / 2 - 5,
          size: 10 * scaleX,
          font: helvetica,
          color: rgb(0, 0, 0),
        });
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

  return await pdfDoc.save();
}
