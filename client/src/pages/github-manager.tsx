import { useState, useMemo } from "react";
import { useQuery, useMutation } from "@tanstack/react-query";
import { Link, useLocation } from "wouter";
import { ArrowLeft, FolderOpen, File, Plus, Save, Trash2, RefreshCw, Key, Github, ChevronRight, Home, AlertTriangle, GitBranch, Search, ArrowUpDown, Filter, BarChart3, FileCode, FileText, Image, Archive, Settings, List, Grid } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Badge } from "@/components/ui/badge";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { useToast } from "@/hooks/use-toast";
import { queryClient, apiRequest } from "@/lib/queryClient";

type SortField = "name" | "size" | "type";
type SortOrder = "asc" | "desc";
type FileTypeFilter = "all" | "code" | "docs" | "config" | "other";

const FILE_TYPE_PATTERNS: Record<string, FileTypeFilter> = {
  ts: "code", tsx: "code", js: "code", jsx: "code", rs: "code", py: "code", go: "code",
  md: "docs", txt: "docs", pdf: "docs", rst: "docs",
  json: "config", yaml: "config", yml: "config", toml: "config", lock: "config",
  png: "other", jpg: "other", svg: "other", gif: "other",
};

const getFileType = (filename: string): FileTypeFilter => {
  const ext = filename.split(".").pop()?.toLowerCase() || "";
  return FILE_TYPE_PATTERNS[ext] || "other";
};

const getFileIcon = (filename: string) => {
  const ext = filename.split(".").pop()?.toLowerCase() || "";
  const type = FILE_TYPE_PATTERNS[ext];
  if (type === "code") return <FileCode className="w-4 h-4 text-blue-500" />;
  if (type === "docs") return <FileText className="w-4 h-4 text-green-500" />;
  if (type === "config") return <Settings className="w-4 h-4 text-orange-500" />;
  if (["png", "jpg", "svg", "gif"].includes(ext)) return <Image className="w-4 h-4 text-purple-500" />;
  if (["zip", "tar", "gz"].includes(ext)) return <Archive className="w-4 h-4 text-yellow-500" />;
  return <File className="w-4 h-4 text-muted-foreground" />;
};

const formatFileSize = (bytes: number): string => {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
};

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

interface Branch {
  name: string;
  protected: boolean;
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
  const [selectedBranch, setSelectedBranch] = useState("main");
  const [showTokenUpdate, setShowTokenUpdate] = useState(false);
  
  // Sorting and filtering state
  const [searchQuery, setSearchQuery] = useState("");
  const [sortField, setSortField] = useState<SortField>("name");
  const [sortOrder, setSortOrder] = useState<SortOrder>("asc");
  const [fileTypeFilter, setFileTypeFilter] = useState<FileTypeFilter>("all");
  const [viewMode, setViewMode] = useState<"list" | "grid">("list");

  const { data: adminStatus, isLoading: statusLoading, isError: statusError } = useQuery<{
    isAdmin: boolean;
    authenticated: boolean;
    hasGithubToken: boolean;
  }>({
    queryKey: ["/api/user/admin-status"],
  });

  const { data: branches, isLoading: branchesLoading } = useQuery<{
    success: boolean;
    branches: Branch[];
  }>({
    queryKey: ["/api/github/repos", REPO_OWNER, REPO_NAME, "branches"],
    queryFn: async () => {
      const res = await fetch(`/api/github/repos/${REPO_OWNER}/${REPO_NAME}/branches`);
      if (!res.ok) {
        const error = await res.json();
        throw new Error(error.error || "Failed to fetch branches");
      }
      return res.json();
    },
    enabled: adminStatus?.isAdmin && adminStatus?.hasGithubToken,
  });

  const { data: contents, isLoading: contentsLoading, refetch: refetchContents } = useQuery<{
    success: boolean;
    data: GitHubFile[];
  }>({
    queryKey: ["/api/github/repos", REPO_OWNER, REPO_NAME, "contents", currentPath, selectedBranch],
    queryFn: async () => {
      const res = await fetch(`/api/github/repos/${REPO_OWNER}/${REPO_NAME}/contents?path=${encodeURIComponent(currentPath)}&branch=${encodeURIComponent(selectedBranch)}`);
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
      setShowTokenUpdate(false);
    },
    onError: (error: Error) => {
      toast({ title: "Error", description: error.message, variant: "destructive" });
    },
  });

  const saveFileMutation = useMutation({
    mutationFn: async (data: { path: string; content: string; message: string; sha?: string; branch?: string }) => {
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
    mutationFn: async (data: { path: string; sha: string; message: string; branch?: string }) => {
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
      const res = await fetch(`/api/github/file/${REPO_OWNER}/${REPO_NAME}?path=${encodeURIComponent(path)}&branch=${encodeURIComponent(selectedBranch)}`);
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
      branch: selectedBranch,
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
      branch: selectedBranch,
    });
  };

  const handleDeleteFile = () => {
    if (!selectedFile) return;
    if (!confirm(`Delete ${selectedFile.name}?`)) return;
    deleteFileMutation.mutate({
      path: selectedFile.path,
      sha: selectedFile.sha,
      message: `Delete ${selectedFile.name}`,
      branch: selectedBranch,
    });
  };

  const handleBranchChange = (branch: string) => {
    setSelectedBranch(branch);
    setCurrentPath("");
    setSelectedFile(null);
  };

  // Filtered and sorted files
  const processedFiles = useMemo(() => {
    if (!contents?.data) return [];
    
    let files = [...contents.data];
    
    // Filter by search query
    if (searchQuery) {
      const query = searchQuery.toLowerCase();
      files = files.filter(f => f.name.toLowerCase().includes(query));
    }
    
    // Filter by file type
    if (fileTypeFilter !== "all") {
      files = files.filter(f => {
        if (f.type === "dir") return true; // Always show directories
        return getFileType(f.name) === fileTypeFilter;
      });
    }
    
    // Sort files (directories first, then by selected criteria)
    files.sort((a, b) => {
      // Directories always first
      if (a.type === "dir" && b.type !== "dir") return -1;
      if (a.type !== "dir" && b.type === "dir") return 1;
      
      let comparison = 0;
      switch (sortField) {
        case "name":
          comparison = a.name.toLowerCase().localeCompare(b.name.toLowerCase());
          break;
        case "size":
          comparison = (a.size || 0) - (b.size || 0);
          break;
        case "type":
          const typeA = a.name.split(".").pop()?.toLowerCase() || "";
          const typeB = b.name.split(".").pop()?.toLowerCase() || "";
          comparison = typeA.localeCompare(typeB);
          break;
      }
      
      return sortOrder === "asc" ? comparison : -comparison;
    });
    
    return files;
  }, [contents?.data, searchQuery, fileTypeFilter, sortField, sortOrder]);

  // Calculate metrics
  const metrics = useMemo(() => {
    if (!contents?.data) return null;
    
    const files = contents.data.filter(f => f.type === "file");
    const dirs = contents.data.filter(f => f.type === "dir");
    const totalSize = files.reduce((sum, f) => sum + (f.size || 0), 0);
    
    const typeCounts = files.reduce((acc, f) => {
      const type = getFileType(f.name);
      acc[type] = (acc[type] || 0) + 1;
      return acc;
    }, {} as Record<string, number>);
    
    return {
      totalFiles: files.length,
      totalDirs: dirs.length,
      totalSize,
      typeCounts,
    };
  }, [contents?.data]);

  const toggleSortOrder = () => {
    setSortOrder(prev => prev === "asc" ? "desc" : "asc");
  };

  const handleSortFieldChange = (field: SortField) => {
    if (field === sortField) {
      // Clicking same field toggles direction
      toggleSortOrder();
    } else {
      setSortField(field);
      setSortOrder("asc"); // Reset to ascending when changing field
    }
  };

  if (statusLoading) {
    return (
      <div className="min-h-screen bg-background flex items-center justify-center">
        <RefreshCw className="w-8 h-8 animate-spin text-primary" />
      </div>
    );
  }

  if (statusError) {
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
                Error
              </CardTitle>
              <CardDescription>Failed to check access status. Please try again.</CardDescription>
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
            {adminStatus?.hasGithubToken && (
              <Button
                variant="outline"
                size="sm"
                onClick={() => setShowTokenUpdate(!showTokenUpdate)}
                data-testid="button-update-token"
              >
                <Key className="w-4 h-4 mr-1" />
                {showTokenUpdate ? "Cancel" : "Update Token"}
              </Button>
            )}
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
        {(!adminStatus?.hasGithubToken || showTokenUpdate) ? (
          <Card className="max-w-lg mx-auto">
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Key className="w-5 h-5" />
                {showTokenUpdate ? "Update GitHub Token" : "Configure GitHub Access"}
              </CardTitle>
              <CardDescription>
                {showTokenUpdate 
                  ? "Enter a new GitHub Personal Access Token. Include 'workflow' scope to push GitHub Actions."
                  : "Enter your GitHub Personal Access Token to manage files."
                }
                <br />
                <a
                  href="https://github.com/settings/tokens/new?scopes=repo,workflow&description=PlenumNET-Framework"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-primary hover:underline"
                >
                  Create a new token with repo + workflow scope
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
          <div className="space-y-4">
            {/* Metrics Bar */}
            {metrics && (
              <Card className="bg-gradient-to-r from-slate-50 to-blue-50 border-slate-200">
                <CardContent className="py-3">
                  <div className="flex flex-wrap items-center gap-4 text-sm">
                    <div className="flex items-center gap-2">
                      <BarChart3 className="w-4 h-4 text-blue-500" />
                      <span className="font-medium">Repository Metrics</span>
                    </div>
                    <Badge variant="outline" className="bg-white" data-testid="metric-files">
                      <File className="w-3 h-3 mr-1" />
                      {metrics.totalFiles} files
                    </Badge>
                    <Badge variant="outline" className="bg-white" data-testid="metric-dirs">
                      <FolderOpen className="w-3 h-3 mr-1" />
                      {metrics.totalDirs} folders
                    </Badge>
                    <Badge variant="outline" className="bg-white" data-testid="metric-size">
                      {formatFileSize(metrics.totalSize)}
                    </Badge>
                    {metrics.typeCounts.code > 0 && (
                      <Badge variant="secondary" className="bg-blue-100 text-blue-700">
                        <FileCode className="w-3 h-3 mr-1" />
                        {metrics.typeCounts.code} code
                      </Badge>
                    )}
                    {metrics.typeCounts.docs > 0 && (
                      <Badge variant="secondary" className="bg-green-100 text-green-700">
                        <FileText className="w-3 h-3 mr-1" />
                        {metrics.typeCounts.docs} docs
                      </Badge>
                    )}
                    {metrics.typeCounts.config > 0 && (
                      <Badge variant="secondary" className="bg-orange-100 text-orange-700">
                        <Settings className="w-3 h-3 mr-1" />
                        {metrics.typeCounts.config} config
                      </Badge>
                    )}
                  </div>
                </CardContent>
              </Card>
            )}

            <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            <Card>
              <CardHeader className="pb-3">
                <div className="flex items-center justify-between gap-2 flex-wrap">
                  <div className="flex items-center gap-3">
                    <CardTitle className="text-lg">Files</CardTitle>
                    <div className="flex items-center gap-2">
                      <GitBranch className="w-4 h-4 text-muted-foreground" />
                      <Select value={selectedBranch} onValueChange={handleBranchChange}>
                        <SelectTrigger className="w-[140px] h-8" data-testid="select-branch">
                          <SelectValue placeholder="Branch" />
                        </SelectTrigger>
                        <SelectContent>
                          {branches?.branches?.map((branch) => (
                            <SelectItem key={branch.name} value={branch.name}>
                              {branch.name}
                            </SelectItem>
                          ))}
                          {(!branches?.branches || branches.branches.length === 0) && (
                            <SelectItem value="main">main</SelectItem>
                          )}
                        </SelectContent>
                      </Select>
                    </div>
                  </div>
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
              <CardContent className="space-y-3">
                {/* Search and Filter Toolbar */}
                <div className="flex flex-wrap gap-2">
                  <div className="relative flex-1 min-w-[200px]">
                    <Search className="absolute left-2 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
                    <Input
                      placeholder="Search files..."
                      value={searchQuery}
                      onChange={(e) => setSearchQuery(e.target.value)}
                      className="pl-8 h-8"
                      data-testid="input-search-files"
                    />
                  </div>
                  
                  <Select value={fileTypeFilter} onValueChange={(v) => setFileTypeFilter(v as FileTypeFilter)}>
                    <SelectTrigger className="w-[120px] h-8" data-testid="select-file-type">
                      <Filter className="w-3 h-3 mr-1" />
                      <SelectValue placeholder="Type" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="all">All Types</SelectItem>
                      <SelectItem value="code">Code</SelectItem>
                      <SelectItem value="docs">Docs</SelectItem>
                      <SelectItem value="config">Config</SelectItem>
                      <SelectItem value="other">Other</SelectItem>
                    </SelectContent>
                  </Select>
                  
                  <Select value={sortField} onValueChange={(v) => handleSortFieldChange(v as SortField)}>
                    <SelectTrigger className="w-[100px] h-8" data-testid="select-sort-field">
                      <SelectValue placeholder="Sort" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="name">Name</SelectItem>
                      <SelectItem value="size">Size</SelectItem>
                      <SelectItem value="type">Type</SelectItem>
                    </SelectContent>
                  </Select>
                  
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={toggleSortOrder}
                    className="h-8 px-2"
                    data-testid="button-toggle-sort"
                  >
                    <ArrowUpDown className="w-4 h-4" />
                    <span className="ml-1 text-xs">{sortOrder === "asc" ? "A-Z" : "Z-A"}</span>
                  </Button>
                  
                  <div className="flex border rounded-md">
                    <Button
                      variant={viewMode === "list" ? "secondary" : "ghost"}
                      size="sm"
                      onClick={() => setViewMode("list")}
                      className="h-8 px-2 rounded-r-none"
                      data-testid="button-view-list"
                    >
                      <List className="w-4 h-4" />
                    </Button>
                    <Button
                      variant={viewMode === "grid" ? "secondary" : "ghost"}
                      size="sm"
                      onClick={() => setViewMode("grid")}
                      className="h-8 px-2 rounded-l-none"
                      data-testid="button-view-grid"
                    >
                      <Grid className="w-4 h-4" />
                    </Button>
                  </div>
                </div>

                {contentsLoading ? (
                  <div className="flex items-center justify-center py-8">
                    <RefreshCw className="w-6 h-6 animate-spin text-muted-foreground" />
                  </div>
                ) : viewMode === "list" ? (
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
                    {processedFiles.map((item) => (
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
                          getFileIcon(item.name)
                        )}
                        <span className="flex-1 truncate">{item.name}</span>
                        {item.type === "file" && (
                          <span className="text-xs text-muted-foreground">
                            {(item.size / 1024).toFixed(1)}KB
                          </span>
                        )}
                      </button>
                    ))}
                    {processedFiles.length === 0 && (
                      <p className="text-muted-foreground text-center py-4">
                        {searchQuery || fileTypeFilter !== "all" ? "No matching files" : "Empty directory"}
                      </p>
                    )}
                  </div>
                ) : (
                  /* Grid View */
                  <div className="grid grid-cols-3 sm:grid-cols-4 md:grid-cols-5 gap-2">
                    {currentPath && (
                      <button
                        onClick={navigateUp}
                        className="flex flex-col items-center justify-center gap-1 p-3 rounded-md hover:bg-accent text-center"
                        data-testid="button-navigate-up-grid"
                      >
                        <FolderOpen className="w-8 h-8 text-muted-foreground" />
                        <span className="text-xs">..</span>
                      </button>
                    )}
                    {processedFiles.map((item) => (
                      <button
                        key={item.sha}
                        onClick={() => handleFileClick(item)}
                        className={`flex flex-col items-center justify-center gap-1 p-3 rounded-md hover:bg-accent text-center ${
                          selectedFile?.path === item.path ? "bg-accent" : ""
                        }`}
                        data-testid={`file-grid-${item.name}`}
                      >
                        {item.type === "dir" ? (
                          <FolderOpen className="w-8 h-8 text-primary" />
                        ) : (
                          <div className="w-8 h-8 flex items-center justify-center">
                            {getFileIcon(item.name)}
                          </div>
                        )}
                        <span className="text-xs truncate w-full">{item.name}</span>
                        {item.type === "file" && (
                          <span className="text-[10px] text-muted-foreground">
                            {formatFileSize(item.size)}
                          </span>
                        )}
                      </button>
                    ))}
                    {processedFiles.length === 0 && (
                      <p className="col-span-full text-muted-foreground text-center py-4">
                        {searchQuery || fileTypeFilter !== "all" ? "No matching files" : "Empty directory"}
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
          </div>
        )}
      </main>
    </div>
  );
}