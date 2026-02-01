import { useState } from "react";
import { useQuery, useMutation } from "@tanstack/react-query";
import { Link, useLocation } from "wouter";
import { ArrowLeft, FolderOpen, File, Plus, Save, Trash2, RefreshCw, Key, Github, ChevronRight, Home, AlertTriangle } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Badge } from "@/components/ui/badge";
import { useToast } from "@/hooks/use-toast";
import { queryClient, apiRequest } from "@/lib/queryClient";

const REPO_OWNER = "SigmaWolf-8";
const REPO_NAME = "Ternary";

interface GitHubFile {
  name: string;
  path: string;
  sha: string;
  size: number;
  type: "file" | "dir";
  download_url?: string;
}

interface FileContent {
  name: string;
  path: string;
  sha: string;
  size: number;
  content: string;
}

export default function GitHubManager() {
  const [, setLocation] = useLocation();
  const { toast } = useToast();
  const [currentPath, setCurrentPath] = useState("");
  const [selectedFile, setSelectedFile] = useState<FileContent | null>(null);
  const [editedContent, setEditedContent] = useState("");
  const [newFileName, setNewFileName] = useState("");
  const [newFileContent, setNewFileContent] = useState("");
  const [commitMessage, setCommitMessage] = useState("");
  const [showNewFile, setShowNewFile] = useState(false);
  const [tokenInput, setTokenInput] = useState("");

  const { data: adminStatus, isLoading: statusLoading } = useQuery<{
    isAdmin: boolean;
    authenticated: boolean;
    hasGithubToken: boolean;
  }>({
    queryKey: ["/api/user/admin-status"],
  });

  const { data: contents, isLoading: contentsLoading, refetch: refetchContents } = useQuery<{
    success: boolean;
    data: GitHubFile[];
  }>({
    queryKey: ["/api/github/repos", REPO_OWNER, REPO_NAME, "contents", currentPath],
    queryFn: async () => {
      const res = await fetch(`/api/github/repos/${REPO_OWNER}/${REPO_NAME}/contents?path=${encodeURIComponent(currentPath)}`);
      if (!res.ok) {
        const error = await res.json();
        throw new Error(error.error || "Failed to fetch contents");
      }
      return res.json();
    },
    enabled: adminStatus?.isAdmin && adminStatus?.hasGithubToken,
  });

  const saveTokenMutation = useMutation({
    mutationFn: async (token: string) => {
      return apiRequest("POST", "/api/github/token", { token });
    },
    onSuccess: () => {
      toast({ title: "Success", description: "GitHub token saved" });
      queryClient.invalidateQueries({ queryKey: ["/api/user/admin-status"] });
      setTokenInput("");
    },
    onError: (error: Error) => {
      toast({ title: "Error", description: error.message, variant: "destructive" });
    },
  });

  const saveFileMutation = useMutation({
    mutationFn: async (data: { path: string; content: string; message: string; sha?: string }) => {
      return apiRequest("PUT", `/api/github/file/${REPO_OWNER}/${REPO_NAME}`, data);
    },
    onSuccess: () => {
      toast({ title: "Success", description: "File saved to GitHub" });
      refetchContents();
      setSelectedFile(null);
      setShowNewFile(false);
      setNewFileName("");
      setNewFileContent("");
      setCommitMessage("");
    },
    onError: (error: Error) => {
      toast({ title: "Error", description: error.message, variant: "destructive" });
    },
  });

  const deleteFileMutation = useMutation({
    mutationFn: async (data: { path: string; sha: string; message: string }) => {
      return apiRequest("DELETE", `/api/github/file/${REPO_OWNER}/${REPO_NAME}`, data);
    },
    onSuccess: () => {
      toast({ title: "Success", description: "File deleted" });
      refetchContents();
      setSelectedFile(null);
    },
    onError: (error: Error) => {
      toast({ title: "Error", description: error.message, variant: "destructive" });
    },
  });

  const fetchFileContent = async (path: string) => {
    try {
      const res = await fetch(`/api/github/file/${REPO_OWNER}/${REPO_NAME}?path=${encodeURIComponent(path)}`);
      if (!res.ok) throw new Error("Failed to fetch file");
      const data = await res.json();
      setSelectedFile(data.file);
      setEditedContent(data.file.content);
      setCommitMessage(`Update ${data.file.name}`);
    } catch (error) {
      toast({ title: "Error", description: "Failed to load file", variant: "destructive" });
    }
  };

  const handleFileClick = (file: GitHubFile) => {
    if (file.type === "dir") {
      setCurrentPath(file.path);
      setSelectedFile(null);
    } else {
      fetchFileContent(file.path);
    }
  };

  const navigateUp = () => {
    const parts = currentPath.split("/");
    parts.pop();
    setCurrentPath(parts.join("/"));
    setSelectedFile(null);
  };

  const handleSaveFile = () => {
    if (!selectedFile || !commitMessage.trim()) {
      toast({ title: "Error", description: "Please enter a commit message", variant: "destructive" });
      return;
    }
    saveFileMutation.mutate({
      path: selectedFile.path,
      content: editedContent,
      message: commitMessage,
      sha: selectedFile.sha,
    });
  };

  const handleCreateFile = () => {
    if (!newFileName.trim() || !commitMessage.trim()) {
      toast({ title: "Error", description: "Please enter file name and commit message", variant: "destructive" });
      return;
    }
    const filePath = currentPath ? `${currentPath}/${newFileName}` : newFileName;
    saveFileMutation.mutate({
      path: filePath,
      content: newFileContent,
      message: commitMessage,
    });
  };

  const handleDeleteFile = () => {
    if (!selectedFile) return;
    if (!confirm(`Delete ${selectedFile.name}?`)) return;
    deleteFileMutation.mutate({
      path: selectedFile.path,
      sha: selectedFile.sha,
      message: `Delete ${selectedFile.name}`,
    });
  };

  if (statusLoading) {
    return (
      <div className="min-h-screen bg-background flex items-center justify-center">
        <RefreshCw className="w-8 h-8 animate-spin text-primary" />
      </div>
    );
  }

  if (!adminStatus?.authenticated) {
    return (
      <div className="min-h-screen bg-background">
        <header className="border-b bg-card">
          <div className="container mx-auto px-4 py-4 flex items-center gap-4">
            <Link href="/">
              <Button variant="ghost" size="sm" data-testid="link-home">
                <ArrowLeft className="w-4 h-4 mr-2" />
                Back
              </Button>
            </Link>
            <h1 className="text-xl font-semibold">GitHub Manager</h1>
          </div>
        </header>
        <main className="container mx-auto px-4 py-12">
          <Card className="max-w-md mx-auto">
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <AlertTriangle className="w-5 h-5 text-yellow-500" />
                Authentication Required
              </CardTitle>
              <CardDescription>Please log in to access the GitHub Manager.</CardDescription>
            </CardHeader>
            <CardContent>
              <Link href="/">
                <Button data-testid="button-login-redirect">Go to Home</Button>
              </Link>
            </CardContent>
          </Card>
        </main>
      </div>
    );
  }

  if (!adminStatus?.isAdmin) {
    return (
      <div className="min-h-screen bg-background">
        <header className="border-b bg-card">
          <div className="container mx-auto px-4 py-4 flex items-center gap-4">
            <Link href="/">
              <Button variant="ghost" size="sm" data-testid="link-home">
                <ArrowLeft className="w-4 h-4 mr-2" />
                Back
              </Button>
            </Link>
            <h1 className="text-xl font-semibold">GitHub Manager</h1>
          </div>
        </header>
        <main className="container mx-auto px-4 py-12">
          <Card className="max-w-md mx-auto">
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <AlertTriangle className="w-5 h-5 text-red-500" />
                Access Denied
              </CardTitle>
              <CardDescription>This page is restricted to administrators only.</CardDescription>
            </CardHeader>
            <CardContent>
              <Link href="/">
                <Button data-testid="button-go-home">Go to Home</Button>
              </Link>
            </CardContent>
          </Card>
        </main>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-background">
      <header className="border-b bg-card sticky top-0 z-50">
        <div className="container mx-auto px-4 py-4 flex items-center justify-between gap-4">
          <div className="flex items-center gap-4">
            <Link href="/">
              <Button variant="ghost" size="sm" data-testid="link-home">
                <ArrowLeft className="w-4 h-4 mr-2" />
                Back
              </Button>
            </Link>
            <div className="flex items-center gap-2">
              <Github className="w-5 h-5" />
              <h1 className="text-xl font-semibold">GitHub Manager</h1>
            </div>
            <Badge variant="secondary">{REPO_OWNER}/{REPO_NAME}</Badge>
          </div>
          <div className="flex items-center gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={() => refetchContents()}
              disabled={!adminStatus?.hasGithubToken}
              data-testid="button-refresh"
            >
              <RefreshCw className="w-4 h-4" />
            </Button>
          </div>
        </div>
      </header>

      <main className="container mx-auto px-4 py-6">
        {!adminStatus?.hasGithubToken ? (
          <Card className="max-w-lg mx-auto">
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Key className="w-5 h-5" />
                Configure GitHub Access
              </CardTitle>
              <CardDescription>
                Enter your GitHub Personal Access Token to manage files.
                <br />
                <a
                  href="https://github.com/settings/tokens/new?scopes=repo&description=Salvi-Framework"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-primary hover:underline"
                >
                  Create a new token with repo scope
                </a>
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <Input
                type="password"
                placeholder="ghp_xxxxxxxxxxxxxxxx"
                value={tokenInput}
                onChange={(e) => setTokenInput(e.target.value)}
                data-testid="input-github-token"
              />
              <Button
                onClick={() => saveTokenMutation.mutate(tokenInput)}
                disabled={!tokenInput || saveTokenMutation.isPending}
                data-testid="button-save-token"
              >
                {saveTokenMutation.isPending ? "Saving..." : "Save Token"}
              </Button>
            </CardContent>
          </Card>
        ) : (
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            <Card>
              <CardHeader className="pb-3">
                <div className="flex items-center justify-between">
                  <CardTitle className="text-lg">Files</CardTitle>
                  <Button
                    size="sm"
                    onClick={() => {
                      setShowNewFile(true);
                      setSelectedFile(null);
                      setCommitMessage(`Add new file`);
                    }}
                    data-testid="button-new-file"
                  >
                    <Plus className="w-4 h-4 mr-1" />
                    New File
                  </Button>
                </div>
                <div className="flex items-center gap-1 text-sm text-muted-foreground flex-wrap">
                  <button
                    onClick={() => {
                      setCurrentPath("");
                      setSelectedFile(null);
                    }}
                    className="hover:text-foreground flex items-center"
                    data-testid="breadcrumb-root"
                  >
                    <Home className="w-3 h-3" />
                  </button>
                  {currentPath && (
                    <>
                      <ChevronRight className="w-3 h-3" />
                      {currentPath.split("/").map((part, i, arr) => (
                        <span key={i} className="flex items-center">
                          <button
                            onClick={() => {
                              setCurrentPath(arr.slice(0, i + 1).join("/"));
                              setSelectedFile(null);
                            }}
                            className="hover:text-foreground"
                            data-testid={`breadcrumb-${part}`}
                          >
                            {part}
                          </button>
                          {i < arr.length - 1 && <ChevronRight className="w-3 h-3 mx-1" />}
                        </span>
                      ))}
                    </>
                  )}
                </div>
              </CardHeader>
              <CardContent>
                {contentsLoading ? (
                  <div className="flex items-center justify-center py-8">
                    <RefreshCw className="w-6 h-6 animate-spin text-muted-foreground" />
                  </div>
                ) : (
                  <div className="space-y-1">
                    {currentPath && (
                      <button
                        onClick={navigateUp}
                        className="w-full flex items-center gap-2 p-2 rounded-md hover:bg-accent text-left"
                        data-testid="button-navigate-up"
                      >
                        <FolderOpen className="w-4 h-4 text-muted-foreground" />
                        <span>..</span>
                      </button>
                    )}
                    {contents?.data?.map((item) => (
                      <button
                        key={item.sha}
                        onClick={() => handleFileClick(item)}
                        className={`w-full flex items-center gap-2 p-2 rounded-md hover:bg-accent text-left ${
                          selectedFile?.path === item.path ? "bg-accent" : ""
                        }`}
                        data-testid={`file-${item.name}`}
                      >
                        {item.type === "dir" ? (
                          <FolderOpen className="w-4 h-4 text-primary" />
                        ) : (
                          <File className="w-4 h-4 text-muted-foreground" />
                        )}
                        <span className="flex-1 truncate">{item.name}</span>
                        {item.type === "file" && (
                          <span className="text-xs text-muted-foreground">
                            {(item.size / 1024).toFixed(1)}KB
                          </span>
                        )}
                      </button>
                    ))}
                    {contents?.data?.length === 0 && (
                      <p className="text-muted-foreground text-center py-4">
                        Empty directory
                      </p>
                    )}
                  </div>
                )}
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="pb-3">
                <CardTitle className="text-lg">
                  {showNewFile ? "New File" : selectedFile ? selectedFile.name : "Editor"}
                </CardTitle>
                {selectedFile && (
                  <CardDescription className="font-mono text-xs">
                    {selectedFile.path}
                  </CardDescription>
                )}
              </CardHeader>
              <CardContent className="space-y-4">
                {showNewFile ? (
                  <>
                    <Input
                      placeholder="filename.ts"
                      value={newFileName}
                      onChange={(e) => setNewFileName(e.target.value)}
                      data-testid="input-new-filename"
                    />
                    <Textarea
                      placeholder="File content..."
                      value={newFileContent}
                      onChange={(e) => setNewFileContent(e.target.value)}
                      className="font-mono text-sm min-h-[300px]"
                      data-testid="textarea-new-content"
                    />
                    <Input
                      placeholder="Commit message"
                      value={commitMessage}
                      onChange={(e) => setCommitMessage(e.target.value)}
                      data-testid="input-commit-message"
                    />
                    <div className="flex gap-2">
                      <Button
                        onClick={handleCreateFile}
                        disabled={saveFileMutation.isPending}
                        data-testid="button-create-file"
                      >
                        <Save className="w-4 h-4 mr-2" />
                        {saveFileMutation.isPending ? "Creating..." : "Create File"}
                      </Button>
                      <Button
                        variant="outline"
                        onClick={() => {
                          setShowNewFile(false);
                          setNewFileName("");
                          setNewFileContent("");
                          setCommitMessage("");
                        }}
                        data-testid="button-cancel-new"
                      >
                        Cancel
                      </Button>
                    </div>
                  </>
                ) : selectedFile ? (
                  <>
                    <Textarea
                      value={editedContent}
                      onChange={(e) => setEditedContent(e.target.value)}
                      className="font-mono text-sm min-h-[300px]"
                      data-testid="textarea-edit-content"
                    />
                    <Input
                      placeholder="Commit message"
                      value={commitMessage}
                      onChange={(e) => setCommitMessage(e.target.value)}
                      data-testid="input-edit-commit-message"
                    />
                    <div className="flex gap-2">
                      <Button
                        onClick={handleSaveFile}
                        disabled={saveFileMutation.isPending}
                        data-testid="button-save-file"
                      >
                        <Save className="w-4 h-4 mr-2" />
                        {saveFileMutation.isPending ? "Saving..." : "Save"}
                      </Button>
                      <Button
                        variant="destructive"
                        onClick={handleDeleteFile}
                        disabled={deleteFileMutation.isPending}
                        data-testid="button-delete-file"
                      >
                        <Trash2 className="w-4 h-4 mr-2" />
                        Delete
                      </Button>
                    </div>
                  </>
                ) : (
                  <div className="text-center py-12 text-muted-foreground">
                    <File className="w-12 h-12 mx-auto mb-4 opacity-50" />
                    <p>Select a file to edit or create a new one</p>
                  </div>
                )}
              </CardContent>
            </Card>
          </div>
        )}
      </main>
    </div>
  );
}