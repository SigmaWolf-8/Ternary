/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 */

import type { Express } from "express";
import type { IStorage } from "../storage";
import { z } from "zod";
import { createRequireAdmin, resolveGitHubToken, sanitizePath } from "./middleware";
import { createLogger, toErrorMessage } from "../logger";
import { githubTokenLimiter, authLimiter } from "../middleware/rate-limiter";

const log = createLogger("github");

export function registerGitHubRoutes(app: Express, storage: IStorage): void {
  const requireAdmin = createRequireAdmin(storage);

  app.post("/api/github/token", githubTokenLimiter, requireAdmin, async (req: any, res) => {
    try {
      const schema = z.object({
        token: z.string().min(1)
      });
      const parsed = schema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({ error: "Invalid token" });
      }
      
      await storage.updateUserGithubToken(req.adminUser.id, parsed.data.token);
      res.json({ success: true, message: "GitHub token saved" });
    } catch (error: unknown) {
      log.error("GitHub token save error:", error);
      res.status(500).json({ error: "Failed to save token" });
    }
  });

  app.get("/api/github/status", requireAdmin, async (req: any, res) => {
    try {
      const token = resolveGitHubToken(req.adminUser);
      const tokenSource = req.adminUser?.githubToken ? "user" : (process.env.GITHUB_TOKEN ? "env" : "none");
      res.json({ success: true, hasToken: !!token, tokenSource });
    } catch (error: unknown) {
      res.status(500).json({ error: "Failed to check status" });
    }
  });

  app.get("/api/github/repos/:owner/:repo/branches", requireAdmin, async (req: any, res) => {
    try {
      const { owner, repo } = req.params;
      const token = resolveGitHubToken(req.adminUser);
      
      if (!token) {
        return res.status(400).json({ error: "GitHub token not configured" });
      }

      const response = await fetch(
        `https://api.github.com/repos/${owner}/${repo}/branches`,
        {
          headers: {
            Authorization: `Bearer ${token}`,
            Accept: "application/vnd.github.v3+json",
            "User-Agent": "Salvi-Framework"
          }
        }
      );

      if (!response.ok) {
        const error = await response.json();
        return res.status(response.status).json({ error: error.message || "GitHub API error" });
      }

      const data = await response.json();
      res.json({ success: true, branches: data.map((b: any) => ({ name: b.name, protected: b.protected })) });
    } catch (error: unknown) {
      log.error("GitHub branches error:", error);
      res.status(500).json({ error: "Failed to fetch branches" });
    }
  });

  app.get("/api/github/repos/:owner/:repo/contents", requireAdmin, async (req: any, res) => {
    try {
      const { owner, repo } = req.params;
      const path = sanitizePath((req.query.path as string) || "");
      const branch = (req.query.branch as string) || "";
      const token = resolveGitHubToken(req.adminUser);
      
      if (!token) {
        return res.status(400).json({ error: "GitHub token not configured" });
      }

      const url = new URL(`https://api.github.com/repos/${owner}/${repo}/contents/${path}`);
      if (branch) {
        url.searchParams.set("ref", branch);
      }

      const response = await fetch(
        url.toString(),
        {
          headers: {
            Authorization: `Bearer ${token}`,
            Accept: "application/vnd.github.v3+json",
            "User-Agent": "Salvi-Framework"
          }
        }
      );

      if (!response.ok) {
        const error = await response.json();
        return res.status(response.status).json({ error: error.message || "GitHub API error" });
      }

      const data = await response.json();
      res.json({ success: true, data });
    } catch (error: unknown) {
      log.error("GitHub contents error:", error);
      res.status(500).json({ error: "Failed to fetch contents" });
    }
  });

  app.get("/api/github/file/:owner/:repo", requireAdmin, async (req: any, res) => {
    try {
      const { owner, repo } = req.params;
      const path = sanitizePath((req.query.path as string) || "");
      const branch = (req.query.branch as string) || "";
      const token = resolveGitHubToken(req.adminUser);
      
      if (!token) {
        return res.status(400).json({ error: "GitHub token not configured" });
      }

      const url = new URL(`https://api.github.com/repos/${owner}/${repo}/contents/${path}`);
      if (branch) {
        url.searchParams.set("ref", branch);
      }

      const response = await fetch(
        url.toString(),
        {
          headers: {
            Authorization: `Bearer ${token}`,
            Accept: "application/vnd.github.v3+json",
            "User-Agent": "Salvi-Framework"
          }
        }
      );

      if (!response.ok) {
        const error = await response.json();
        return res.status(response.status).json({ error: error.message || "GitHub API error" });
      }

      const data = await response.json();
      
      if (data.content) {
        const content = Buffer.from(data.content, "base64").toString("utf-8");
        res.json({ 
          success: true, 
          file: {
            name: data.name,
            path: data.path,
            sha: data.sha,
            size: data.size,
            content
          }
        });
      } else {
        res.status(400).json({ error: "Not a file" });
      }
    } catch (error: unknown) {
      log.error("GitHub file error:", error);
      res.status(500).json({ error: "Failed to fetch file" });
    }
  });

  app.put("/api/github/file/:owner/:repo", requireAdmin, async (req: any, res) => {
    try {
      const { owner, repo } = req.params;
      const token = resolveGitHubToken(req.adminUser);
      
      if (!token) {
        return res.status(400).json({ error: "GitHub token not configured" });
      }

      const schema = z.object({
        path: z.string().min(1),
        content: z.string(),
        message: z.string().min(1),
        sha: z.string().optional(),
        branch: z.string().optional()
      });
      
      const parsed = schema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({ error: "Invalid request", details: parsed.error.errors });
      }

      const { path: rawPath, content, message, sha, branch } = parsed.data;
      const filePath = sanitizePath(rawPath);
      const encodedContent = Buffer.from(content).toString("base64");

      const body: any = {
        message,
        content: encodedContent
      };
      if (sha) {
        body.sha = sha;
      }
      if (branch) {
        body.branch = branch;
      }

      const response = await fetch(
        `https://api.github.com/repos/${owner}/${repo}/contents/${filePath}`,
        {
          method: "PUT",
          headers: {
            Authorization: `Bearer ${token}`,
            Accept: "application/vnd.github.v3+json",
            "Content-Type": "application/json",
            "User-Agent": "Salvi-Framework"
          },
          body: JSON.stringify(body)
        }
      );

      if (!response.ok) {
        const error = await response.json();
        return res.status(response.status).json({ error: error.message || "GitHub API error" });
      }

      const data = await response.json();
      res.json({ 
        success: true, 
        message: sha ? "File updated" : "File created",
        commit: data.commit
      });
    } catch (error: unknown) {
      log.error("GitHub file create/update error:", error);
      res.status(500).json({ error: "Failed to save file" });
    }
  });

  app.delete("/api/github/file/:owner/:repo", requireAdmin, async (req: any, res) => {
    try {
      const { owner, repo } = req.params;
      const token = resolveGitHubToken(req.adminUser);
      
      if (!token) {
        return res.status(400).json({ error: "GitHub token not configured" });
      }

      const schema = z.object({
        path: z.string().min(1),
        message: z.string().min(1),
        sha: z.string(),
        branch: z.string().optional()
      });
      
      const parsed = schema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({ error: "Invalid request", details: parsed.error.errors });
      }

      const { path: rawPath, message, sha, branch } = parsed.data;
      const filePath = sanitizePath(rawPath);

      const deleteBody: any = { message, sha };
      if (branch) {
        deleteBody.branch = branch;
      }

      const response = await fetch(
        `https://api.github.com/repos/${owner}/${repo}/contents/${filePath}`,
        {
          method: "DELETE",
          headers: {
            Authorization: `Bearer ${token}`,
            Accept: "application/vnd.github.v3+json",
            "Content-Type": "application/json",
            "User-Agent": "Salvi-Framework"
          },
          body: JSON.stringify(deleteBody)
        }
      );

      if (!response.ok) {
        const error = await response.json();
        return res.status(response.status).json({ error: error.message || "GitHub API error" });
      }

      const data = await response.json();
      res.json({ 
        success: true, 
        message: "File deleted",
        commit: data.commit
      });
    } catch (error: unknown) {
      log.error("GitHub file delete error:", error);
      res.status(500).json({ error: "Failed to delete file" });
    }
  });

  app.post("/api/github/push-workflows/:owner/:repo", requireAdmin, async (req: any, res) => {
    try {
      const { owner, repo } = req.params;
      const token = resolveGitHubToken(req.adminUser);

      if (!token) {
        return res.status(400).json({ error: "GitHub token not configured" });
      }

      const fs = await import('fs/promises');
      const pathModule = await import('path');

      const workflowDir = pathModule.join(process.cwd(), '.github', 'workflows');

      try {
        await fs.access(workflowDir);
      } catch {
        return res.status(404).json({ error: "No .github/workflows/ directory found locally" });
      }

      const workflowFiles = await fs.readdir(workflowDir);
      const ymlFiles = workflowFiles.filter(f => f.endsWith('.yml') || f.endsWith('.yaml'));

      if (ymlFiles.length === 0) {
        return res.status(404).json({ error: "No workflow files (.yml) found in .github/workflows/" });
      }

      const results: { file: string; status: string; error?: string }[] = [];

      for (const fileName of ymlFiles) {
        try {
          const filePath = pathModule.join(workflowDir, fileName);
          const content = await fs.readFile(filePath, 'utf-8');
          const encodedContent = Buffer.from(content).toString('base64');
          const githubPath = `.github/workflows/${fileName}`;

          const existingResponse = await fetch(
            `https://api.github.com/repos/${owner}/${repo}/contents/${githubPath}`,
            {
              headers: {
                Authorization: `Bearer ${token}`,
                Accept: "application/vnd.github.v3+json",
                "User-Agent": "Salvi-Framework"
              }
            }
          );

          let sha: string | undefined;
          if (existingResponse.ok) {
            const existingFile = await existingResponse.json();
            sha = existingFile.sha;
          }

          const body: any = {
            message: `CI/CD: Update ${fileName}`,
            content: encodedContent
          };
          if (sha) body.sha = sha;

          const pushResponse = await fetch(
            `https://api.github.com/repos/${owner}/${repo}/contents/${githubPath}`,
            {
              method: "PUT",
              headers: {
                Authorization: `Bearer ${token}`,
                Accept: "application/vnd.github.v3+json",
                "Content-Type": "application/json",
                "User-Agent": "Salvi-Framework"
              },
              body: JSON.stringify(body)
            }
          );

          if (pushResponse.ok) {
            results.push({ file: fileName, status: "success" });
          } else {
            const errorData = await pushResponse.json().catch(() => ({}));
            const httpStatus = pushResponse.status;
            let errorMsg = (errorData as any).message || `HTTP ${httpStatus}`;
            if (httpStatus === 403 || httpStatus === 422) {
              errorMsg += " — Your GitHub token likely needs the 'workflow' scope to push to .github/workflows/. Regenerate your PAT with 'workflow' scope enabled.";
            }
            results.push({ file: fileName, status: "error", error: errorMsg });
          }
        } catch (fileError: unknown) {
          results.push({ file: fileName, status: "error", error: toErrorMessage(fileError) });
        }
      }

      const succeeded = results.filter(r => r.status === "success").length;
      const failed = results.filter(r => r.status === "error").length;
      const workflowScopeHint = failed > 0 ? " If workflow pushes failed with 403/422, your token needs the 'workflow' scope." : "";

      res.json({
        success: failed === 0,
        message: `Pushed ${succeeded}/${ymlFiles.length} workflow files.${workflowScopeHint}`,
        results
      });
    } catch (error: unknown) {
      log.error("GitHub workflow push error:", error);
      res.status(500).json({ error: "Failed to push workflow files" });
    }
  });

  const ALLOWED_PUSH_PREFIXES = [
    "src/kernel/",
    "src/tsl/",
    "src/thdl/",
    "src/timing-api/",
    "libternary/",
    "ternary-math/",
    "salvi_docs/",
    ".github/",
    "kong/",
    "scripts/",
    "docs/",
    "tests/",
    "keys/",
    "server/",
    "shared/",
    "client/",
    "rtl/",
    "PQTI-P0-STATUS.md",
    "PQTI-REMAINING-WORK.md",
    "GITHUB-REPOSITORY-ARCHITECTURE.md",
    "README.md",
    "CONTRIBUTING.md",
    "SECURITY.md",
    "CODE_OF_CONDUCT.md",
    "CHANGELOG.md",
    "ROADMAP.md",
    "Makefile",
    "Cargo.toml",
  ];

  const isPathAllowed = (filePath: string): boolean => {
    const normalized = filePath.replace(/\.\./g, "").replace(/^\/+/, "");
    if (normalized !== filePath) return false;
    return ALLOWED_PUSH_PREFIXES.some(prefix => normalized.startsWith(prefix) || normalized === prefix);
  };

  app.post("/api/github/push-env/:owner/:repo", authLimiter, async (req: any, res) => {
    try {
      const { owner, repo } = req.params;
      const token = process.env.GITHUB_TOKEN;

      if (!token) {
        return res.status(400).json({ error: "No GITHUB_TOKEN environment variable" });
      }

      const schema = z.object({
        files: z.array(z.object({
          localPath: z.string().min(1),
          githubPath: z.string().min(1),
        })),
        message: z.string().min(1),
      });

      const parsed = schema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({ error: "Invalid request" });
      }

      const { files, message } = parsed.data;
      const fs = await import('fs/promises');
      const pathModule = await import('path');

      const rejectedFiles = files.filter(f => !isPathAllowed(f.localPath));
      if (rejectedFiles.length > 0) {
        return res.status(403).json({ error: "Path not allowed", rejected: rejectedFiles.map(f => f.localPath) });
      }

      const results: { file: string; status: string; error?: string }[] = [];

      for (const file of files) {
        try {
          const localFilePath = pathModule.resolve(process.cwd(), file.localPath);
          if (!localFilePath.startsWith(process.cwd())) {
            results.push({ file: file.githubPath, status: "error", error: "Path traversal blocked" });
            continue;
          }
          const content = await fs.readFile(localFilePath, 'utf-8');
          const encodedContent = Buffer.from(content).toString('base64');
          const ghPath = sanitizePath(file.githubPath);

          const existingResponse = await fetch(
            `https://api.github.com/repos/${owner}/${repo}/contents/${ghPath}`,
            { headers: { Authorization: `Bearer ${token}`, Accept: "application/vnd.github.v3+json", "User-Agent": "Salvi-Framework" } }
          );

          let sha: string | undefined;
          if (existingResponse.ok) {
            const existingFile = await existingResponse.json();
            sha = (existingFile as any).sha;
          }

          const body: any = { message: `${message} - ${file.githubPath}`, content: encodedContent, branch: "main" };
          if (sha) body.sha = sha;

          const pushResponse = await fetch(
            `https://api.github.com/repos/${owner}/${repo}/contents/${ghPath}`,
            {
              method: "PUT",
              headers: { Authorization: `Bearer ${token}`, Accept: "application/vnd.github.v3+json", "Content-Type": "application/json", "User-Agent": "Salvi-Framework" },
              body: JSON.stringify(body)
            }
          );

          if (pushResponse.ok) {
            results.push({ file: file.githubPath, status: "success" });
          } else {
            const errorData = await pushResponse.json().catch(() => ({}));
            results.push({ file: file.githubPath, status: "error", error: (errorData as any).message || `HTTP ${pushResponse.status}` });
          }
        } catch (fileError: unknown) {
          results.push({ file: file.githubPath, status: "error", error: toErrorMessage(fileError) });
        }
      }

      const succeeded = results.filter(r => r.status === "success").length;
      res.json({ success: results.every(r => r.status === "success"), message: `Pushed ${succeeded}/${files.length} files`, results });
    } catch (error: unknown) {
      log.error("GitHub env push error:", error);
      res.status(500).json({ error: "Failed to push files" });
    }
  });

  app.post("/api/github/push-batch/:owner/:repo", authLimiter, requireAdmin, async (req: any, res) => {
    try {
      const { owner, repo } = req.params;
      const token = resolveGitHubToken(req.adminUser);

      if (!token) {
        return res.status(400).json({ error: "GitHub token not configured" });
      }

      const schema = z.object({
        files: z.array(z.object({
          localPath: z.string().min(1),
          githubPath: z.string().min(1),
        })),
        message: z.string().min(1),
      });

      const parsed = schema.safeParse(req.body);
      if (!parsed.success) {
        return res.status(400).json({ error: "Invalid request", details: parsed.error.errors });
      }

      const { files, message } = parsed.data;
      const fs = await import('fs/promises');
      const pathModule = await import('path');

      const rejectedFiles = files.filter(f => !isPathAllowed(f.localPath));
      if (rejectedFiles.length > 0) {
        return res.status(403).json({
          error: "Path not allowed",
          rejected: rejectedFiles.map(f => f.localPath),
        });
      }

      const results: { file: string; status: string; error?: string }[] = [];

      for (const file of files) {
        try {
          const localFilePath = pathModule.resolve(process.cwd(), file.localPath);
          if (!localFilePath.startsWith(process.cwd())) {
            results.push({ file: file.githubPath, status: "error", error: "Path traversal blocked" });
            continue;
          }
          const content = await fs.readFile(localFilePath, 'utf-8');
          const encodedContent = Buffer.from(content).toString('base64');
          const ghPath = sanitizePath(file.githubPath);

          const existingResponse = await fetch(
            `https://api.github.com/repos/${owner}/${repo}/contents/${ghPath}`,
            {
              headers: {
                Authorization: `Bearer ${token}`,
                Accept: "application/vnd.github.v3+json",
                "User-Agent": "Salvi-Framework"
              }
            }
          );

          let sha: string | undefined;
          if (existingResponse.ok) {
            const existingFile = await existingResponse.json();
            sha = existingFile.sha;
          }

          const body: any = { message: `${message} - ${file.githubPath}`, content: encodedContent };
          if (sha) body.sha = sha;

          const pushResponse = await fetch(
            `https://api.github.com/repos/${owner}/${repo}/contents/${ghPath}`,
            {
              method: "PUT",
              headers: {
                Authorization: `Bearer ${token}`,
                Accept: "application/vnd.github.v3+json",
                "Content-Type": "application/json",
                "User-Agent": "Salvi-Framework"
              },
              body: JSON.stringify(body)
            }
          );

          if (pushResponse.ok) {
            results.push({ file: file.githubPath, status: "success" });
          } else {
            const errorData = await pushResponse.json().catch(() => ({}));
            const httpStatus = pushResponse.status;
            let errorMsg = (errorData as any).message || `HTTP ${httpStatus}`;
            if ((httpStatus === 403 || httpStatus === 422) && file.githubPath.startsWith(".github/workflows/")) {
              errorMsg += " — Token needs 'workflow' scope to push to .github/workflows/. Regenerate PAT with 'workflow' scope.";
            }
            results.push({ file: file.githubPath, status: "error", error: errorMsg });
          }
        } catch (fileError: unknown) {
          results.push({ file: file.githubPath, status: "error", error: toErrorMessage(fileError) });
        }
      }

      const succeeded = results.filter(r => r.status === "success").length;
      res.json({
        success: results.every(r => r.status === "success"),
        message: `Pushed ${succeeded}/${files.length} files`,
        results
      });
    } catch (error: unknown) {
      log.error("GitHub batch push error:", error);
      res.status(500).json({ error: "Failed to push files" });
    }
  });
}
