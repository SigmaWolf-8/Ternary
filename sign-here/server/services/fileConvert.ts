import { execSync } from "child_process";
import { writeFileSync, readFileSync, mkdirSync, unlinkSync, readdirSync } from "fs";
import { join } from "path";
import { randomBytes } from "crypto";

const TMP_DIR = "/tmp/signhere-convert";

function ensureTmpDir() {
  try {
    mkdirSync(TMP_DIR, { recursive: true });
  } catch {}
}

function cleanup(files: string[]) {
  for (const f of files) {
    try { unlinkSync(f); } catch {}
  }
}

export function getFileExtension(fileName: string): string {
  const dot = fileName.lastIndexOf(".");
  return dot >= 0 ? fileName.slice(dot).toLowerCase() : "";
}

export function isPdfFile(fileName: string, mimeType?: string): boolean {
  if (mimeType === "application/pdf") return true;
  return getFileExtension(fileName) === ".pdf";
}

export function isConvertible(fileName: string): boolean {
  const ext = getFileExtension(fileName);
  return [".docx", ".xlsx", ".csv"].includes(ext);
}

export async function convertToPdf(base64Data: string, fileName: string): Promise<{ pdfBase64: string; pageCount: number }> {
  ensureTmpDir();

  const ext = getFileExtension(fileName);
  const uniqueId = randomBytes(8).toString("hex");
  const inputPath = join(TMP_DIR, `${uniqueId}${ext}`);
  const expectedPdfName = `${uniqueId}.pdf`;
  const outputPath = join(TMP_DIR, expectedPdfName);

  const filesToClean: string[] = [inputPath, outputPath];

  try {
    const buffer = Buffer.from(base64Data, "base64");
    writeFileSync(inputPath, buffer);

    execSync(
      `libreoffice --headless --norestore --nologo --convert-to pdf --outdir "${TMP_DIR}" "${inputPath}"`,
      { timeout: 60000, stdio: "pipe" }
    );

    const pdfFiles = readdirSync(TMP_DIR).filter(
      (f) => f.startsWith(uniqueId) && f.endsWith(".pdf")
    );

    if (pdfFiles.length === 0) {
      throw new Error(`Conversion failed: no PDF output for ${fileName}`);
    }

    for (const pf of pdfFiles) {
      filesToClean.push(join(TMP_DIR, pf));
    }

    const actualPdfPath = join(TMP_DIR, pdfFiles[0]);
    const pdfBuffer = readFileSync(actualPdfPath);
    const pdfBase64 = pdfBuffer.toString("base64");

    let pageCount = 1;
    const pdfStr = pdfBuffer.toString("latin1");
    const pageMatches = pdfStr.match(/\/Type\s*\/Page[^s]/g);
    if (pageMatches) {
      pageCount = pageMatches.length;
    }

    return { pdfBase64, pageCount };
  } finally {
    cleanup(filesToClean);
  }
}
