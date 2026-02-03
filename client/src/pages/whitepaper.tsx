import { Link } from "wouter";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Card } from "@/components/ui/card";
import { 
  ArrowLeft,
  FileText,
  Download,
  Calendar,
  User,
  Clock,
  ChevronDown,
  ChevronUp,
  BookOpen,
  Layers,
  Shield,
  Cpu,
  Network,
  Zap,
  Github
} from "lucide-react";
import { useState, useEffect } from "react";
import { motion } from "framer-motion";

interface Whitepaper {
  id: number;
  version: string;
  title: string;
  content: string;
  summary: string | null;
  author: string | null;
  isActive: number;
  createdAt: string;
  updatedAt: string;
}

interface TableOfContentsItem {
  id: string;
  title: string;
  level: number;
}

function slugify(text: string): string {
  return text.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/(^-|-$)/g, '');
}

function parseMarkdownToHTML(markdown: string): string {
  let html = markdown.replace(/\r\n/g, '\n').replace(/\r/g, '\n');
  
  html = html.replace(/^### \*\*(.+?)\*\*$/gm, (_, title) => `<h3 id="${slugify(title)}" class="text-xl font-bold text-foreground mt-8 mb-4 scroll-mt-24">${title}</h3>`);
  html = html.replace(/^## \*\*(.+?)\*\*$/gm, (_, title) => `<h2 id="${slugify(title)}" class="text-2xl font-bold text-foreground mt-10 mb-6 pb-2 border-b border-primary/20 scroll-mt-24">${title}</h2>`);
  html = html.replace(/^# \*\*(.+?)\*\*$/gm, (_, title) => `<h1 id="${slugify(title)}" class="text-3xl font-bold text-foreground mt-12 mb-8 scroll-mt-24">${title}</h1>`);
  html = html.replace(/^### (.+)$/gm, (_, title) => `<h3 id="${slugify(title)}" class="text-xl font-semibold text-foreground mt-8 mb-4 scroll-mt-24">${title}</h3>`);
  html = html.replace(/^## (.+)$/gm, (_, title) => `<h2 id="${slugify(title)}" class="text-2xl font-bold text-foreground mt-10 mb-6 pb-2 border-b border-primary/20 scroll-mt-24">${title}</h2>`);
  html = html.replace(/^# (.+)$/gm, (_, title) => `<h1 id="${slugify(title)}" class="text-3xl font-bold text-foreground mt-12 mb-8 scroll-mt-24">${title}</h1>`);
  html = html.replace(/^#### (.+)$/gm, (_, title) => `<h4 id="${slugify(title)}" class="text-lg font-semibold text-foreground mt-6 mb-3 scroll-mt-24">${title}</h4>`);
  
  html = html.replace(/\*\*(.+?)\*\*/g, '<strong class="font-semibold text-foreground">$1</strong>');
  html = html.replace(/\*(.+?)\*/g, '<em>$1</em>');
  html = html.replace(/`([^`]+)`/g, '<code class="bg-secondary px-1.5 py-0.5 rounded text-primary text-sm font-mono">$1</code>');
  
  html = html.replace(/```(\w+)?\n([\s\S]*?)```/g, (_, lang, code) => {
    return `<pre class="bg-secondary/50 border border-primary/10 rounded-lg p-4 overflow-x-auto my-4"><code class="text-sm font-mono text-muted-foreground">${code.trim()}</code></pre>`;
  });
  
  html = html.replace(/^  • (.+)$/gm, '<li class="ml-6 text-muted-foreground">$1</li>');
  html = html.replace(/^- (.+)$/gm, '<li class="ml-4 text-muted-foreground list-disc">$1</li>');
  html = html.replace(/^\d+\. (.+)$/gm, '<li class="ml-4 text-muted-foreground list-decimal">$1</li>');
  
  html = html.replace(/^---$/gm, '<hr class="my-8 border-primary/20" />');
  
  html = html.replace(/^(?!<[h|l|p|u|o|d|c|b|t|s|a]|$)(.+)$/gm, '<p class="text-muted-foreground leading-relaxed mb-4">$1</p>');
  
  return html;
}

function extractTableOfContents(content: string): TableOfContentsItem[] {
  const items: TableOfContentsItem[] = [];
  const normalizedContent = content.replace(/\r\n/g, '\n').replace(/\r/g, '\n');
  const lines = normalizedContent.split('\n');
  
  lines.forEach((line) => {
    const trimmedLine = line.trim();
    const h1Match = trimmedLine.match(/^# \*\*(.+?)\*\*$/) || trimmedLine.match(/^# (.+)$/);
    const h2Match = trimmedLine.match(/^## \*\*(.+?)\*\*$/) || trimmedLine.match(/^## (.+)$/);
    const h3Match = trimmedLine.match(/^### \*\*(.+?)\*\*$/) || trimmedLine.match(/^### (.+)$/);
    
    if (h1Match) {
      items.push({ id: slugify(h1Match[1]), title: h1Match[1], level: 1 });
    } else if (h2Match) {
      items.push({ id: slugify(h2Match[1]), title: h2Match[1], level: 2 });
    } else if (h3Match) {
      items.push({ id: slugify(h3Match[1]), title: h3Match[1], level: 3 });
    }
  });
  
  return items;
}

export default function WhitepaperPage() {
  const [whitepaper, setWhitepaper] = useState<Whitepaper | null>(null);
  const [allVersions, setAllVersions] = useState<Whitepaper[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [showToc, setShowToc] = useState(true);
  const [tableOfContents, setTableOfContents] = useState<TableOfContentsItem[]>([]);

  useEffect(() => {
    const fetchWhitepaper = async () => {
      try {
        const [activeRes, allRes] = await Promise.all([
          fetch('/api/whitepapers/active'),
          fetch('/api/whitepapers')
        ]);
        
        if (activeRes.ok) {
          const data = await activeRes.json();
          setWhitepaper(data.whitepaper);
          setTableOfContents(extractTableOfContents(data.whitepaper.content));
        }
        
        if (allRes.ok) {
          const data = await allRes.json();
          setAllVersions(data.whitepapers || []);
        }
      } catch (error) {
        console.error('Failed to fetch whitepaper:', error);
      } finally {
        setIsLoading(false);
      }
    };
    
    fetchWhitepaper();
  }, []);

  const scrollToSection = (id: string) => {
    const element = document.getElementById(id);
    if (element) {
      element.scrollIntoView({ behavior: 'smooth', block: 'start' });
    }
  };

  return (
    <div className="min-h-screen bg-background">
      <header className="fixed top-0 left-0 right-0 z-50 bg-background/80 backdrop-blur-lg border-b border-primary/10">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex items-center justify-between h-16">
            <div className="flex items-center gap-4">
              <Link href="/">
                <Button variant="ghost" size="sm" className="gap-2" data-testid="button-back-home">
                  <ArrowLeft className="w-4 h-4" />
                  Home
                </Button>
              </Link>
              <div className="h-6 w-px bg-primary/20" />
              <div className="flex items-center gap-2">
                <FileText className="w-5 h-5 text-primary" />
                <span className="font-semibold text-foreground">Whitepaper</span>
              </div>
            </div>
            <div className="flex items-center gap-3">
              <Link href="/ternarydb">
                <Button variant="outline" size="sm" data-testid="link-ternarydb">
                  PlenumDB
                </Button>
              </Link>
              <a href="https://github.com" target="_blank" rel="noopener noreferrer">
                <Button variant="ghost" size="icon" data-testid="link-github">
                  <Github className="w-5 h-5" />
                </Button>
              </a>
            </div>
          </div>
        </div>
      </header>

      <main className="pt-24 pb-16">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          {isLoading ? (
            <div className="flex items-center justify-center py-24">
              <div className="w-8 h-8 border-2 border-primary border-t-transparent rounded-full animate-spin" />
            </div>
          ) : !whitepaper ? (
            <div className="text-center py-24">
              <FileText className="w-16 h-16 mx-auto text-muted-foreground mb-4" />
              <h2 className="text-2xl font-bold text-foreground mb-2">No Whitepaper Available</h2>
              <p className="text-muted-foreground">The whitepaper is being prepared and will be available soon.</p>
            </div>
          ) : (
            <>
            <div className="lg:hidden mb-6">
              <Card className="p-4 border-primary/10 bg-card/70 backdrop-blur-sm">
                <button
                  onClick={() => setShowToc(!showToc)}
                  className="w-full flex items-center justify-between"
                  data-testid="button-mobile-toc"
                >
                  <h3 className="font-semibold text-foreground flex items-center gap-2">
                    <BookOpen className="w-4 h-4 text-primary" />
                    Table of Contents
                  </h3>
                  {showToc ? <ChevronUp className="w-5 h-5 text-primary" /> : <ChevronDown className="w-5 h-5 text-primary" />}
                </button>
                
                {showToc && (
                  <nav className="mt-4 space-y-1 max-h-[50vh] overflow-y-auto">
                    {tableOfContents.filter(item => item.level <= 2).map((item, index) => (
                      <button
                        key={`mobile-${item.id}-${index}`}
                        onClick={() => {
                          scrollToSection(item.id);
                          setShowToc(false);
                        }}
                        className={`block w-full text-left py-1.5 text-sm hover:text-primary transition-colors ${
                          item.level === 1 ? 'font-semibold text-foreground border-l-2 border-primary pl-3' :
                          'pl-6 text-muted-foreground'
                        }`}
                      >
                        {item.title}
                      </button>
                    ))}
                  </nav>
                )}
              </Card>
            </div>

            <div className="flex gap-8">
              <aside className="hidden lg:block w-72 flex-shrink-0">
                <div className="sticky top-24">
                  <Card className="p-4 border-primary/10 bg-card/70 backdrop-blur-sm" data-testid="toc-sidebar">
                    <h3 className="font-semibold text-foreground flex items-center gap-2 mb-4">
                      <BookOpen className="w-4 h-4 text-primary" />
                      Table of Contents
                    </h3>
                    
                    <nav className="space-y-0.5 max-h-[65vh] overflow-y-auto pr-2 scrollbar-thin">
                      {tableOfContents.map((item, index) => (
                        <button
                          key={`${item.id}-${index}`}
                          onClick={() => scrollToSection(item.id)}
                          className={`block w-full text-left py-1 hover:text-primary transition-colors ${
                            item.level === 1 
                              ? 'font-semibold text-foreground text-sm border-l-2 border-primary pl-3 mt-3 first:mt-0' 
                              : item.level === 2 
                              ? 'pl-4 text-muted-foreground text-sm' 
                              : 'pl-7 text-muted-foreground/80 text-xs'
                          }`}
                          data-testid={`toc-item-${item.id}`}
                        >
                          {item.title.length > 35 ? item.title.slice(0, 35) + '...' : item.title}
                        </button>
                      ))}
                    </nav>
                  </Card>

                  {allVersions.length > 1 && (
                    <Card className="p-4 border-primary/10 bg-card/70 backdrop-blur-sm mt-4">
                      <h3 className="font-semibold text-foreground mb-3 flex items-center gap-2">
                        <Layers className="w-4 h-4 text-primary" />
                        Versions
                      </h3>
                      <div className="space-y-2">
                        {allVersions.map((v) => (
                          <button
                            key={v.id}
                            className={`w-full text-left p-2 rounded text-sm transition-colors ${
                              v.id === whitepaper.id
                                ? 'bg-primary/10 text-primary'
                                : 'text-muted-foreground hover:bg-secondary'
                            }`}
                            data-testid={`version-${v.id}`}
                          >
                            v{v.version}
                            {v.id === whitepaper.id && <Badge variant="outline" className="ml-2 text-xs">Active</Badge>}
                          </button>
                        ))}
                      </div>
                    </Card>
                  )}
                </div>
              </aside>

              <article className="flex-1 min-w-0">
                <motion.div
                  initial={{ opacity: 0, y: 20 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ duration: 0.5 }}
                >
                  <div className="mb-8">
                    <div className="flex flex-wrap items-center gap-3 mb-4">
                      <Badge variant="outline" className="border-primary/30 text-primary">
                        v{whitepaper.version}
                      </Badge>
                      <Badge variant="secondary" className="gap-1">
                        <Calendar className="w-3 h-3" />
                        {new Date(whitepaper.createdAt).toLocaleDateString()}
                      </Badge>
                      {whitepaper.author && (
                        <Badge variant="secondary" className="gap-1">
                          <User className="w-3 h-3" />
                          {whitepaper.author}
                        </Badge>
                      )}
                    </div>
                    
                    <h1 className="text-3xl md:text-4xl font-bold text-foreground mb-4" data-testid="whitepaper-title">
                      {whitepaper.title}
                    </h1>
                    
                    {whitepaper.summary && (
                      <p className="text-lg text-muted-foreground leading-relaxed">
                        {whitepaper.summary}
                      </p>
                    )}
                  </div>

                  <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-8">
                    <Card className="p-4 border-primary/10 bg-card/50 text-center">
                      <Shield className="w-6 h-6 text-primary mx-auto mb-2" />
                      <div className="text-sm font-medium text-foreground">Post-Quantum</div>
                      <div className="text-xs text-muted-foreground">Security</div>
                    </Card>
                    <Card className="p-4 border-primary/10 bg-card/50 text-center">
                      <Cpu className="w-6 h-6 text-primary mx-auto mb-2" />
                      <div className="text-sm font-medium text-foreground">Ternary Logic</div>
                      <div className="text-xs text-muted-foreground">59% Efficiency</div>
                    </Card>
                    <Card className="p-4 border-primary/10 bg-card/50 text-center">
                      <Network className="w-6 h-6 text-primary mx-auto mb-2" />
                      <div className="text-sm font-medium text-foreground">13D Network</div>
                      <div className="text-xs text-muted-foreground">Torsion Routing</div>
                    </Card>
                    <Card className="p-4 border-primary/10 bg-card/50 text-center">
                      <Zap className="w-6 h-6 text-primary mx-auto mb-2" />
                      <div className="text-sm font-medium text-foreground">Femtosecond</div>
                      <div className="text-xs text-muted-foreground">Precision</div>
                    </Card>
                  </div>

                  <Card className="p-6 md:p-8 border-primary/10 bg-card/70 backdrop-blur-sm">
                    <div 
                      className="prose prose-invert max-w-none whitepaper-content"
                      dangerouslySetInnerHTML={{ __html: parseMarkdownToHTML(whitepaper.content) }}
                      data-testid="whitepaper-content"
                    />
                  </Card>
                </motion.div>
              </article>
            </div>
            </>
          )}
        </div>
      </main>

      <footer className="border-t border-primary/10 py-8 bg-card/50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 text-center">
          <p className="text-muted-foreground text-sm">
            All Rights Reserved and Preserved; Capomastro Holdings Ltd.
          </p>
        </div>
      </footer>
    </div>
  );
}
