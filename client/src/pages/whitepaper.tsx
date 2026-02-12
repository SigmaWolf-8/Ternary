/**
 * Copyright (c) 2025–2026 Capomastro Holdings Ltd. (Canada)
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

import { Link } from "wouter";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Card } from "@/components/ui/card";
import { 
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
} from "lucide-react";
import { useState, useEffect } from "react";
import { motion } from "framer-motion";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";

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
      <main className="pb-16">
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
                      className="prose max-w-none whitepaper-content"
                      data-testid="whitepaper-content"
                    >
                      <ReactMarkdown 
                        remarkPlugins={[remarkGfm]}
                        components={{
                          h1: ({children}) => <h1 className="text-3xl font-bold mt-8 mb-4 text-foreground">{children}</h1>,
                          h2: ({children}) => <h2 className="text-2xl font-semibold mt-6 mb-3 text-foreground">{children}</h2>,
                          h3: ({children}) => <h3 className="text-xl font-semibold mt-5 mb-2 text-foreground">{children}</h3>,
                          p: ({children}) => <p className="mb-4 text-base leading-relaxed text-foreground/90">{children}</p>,
                          ul: ({children}) => <ul className="list-disc list-inside mb-4 space-y-1">{children}</ul>,
                          ol: ({children}) => <ol className="list-decimal list-inside mb-4 space-y-1">{children}</ol>,
                          li: ({children}) => <li className="text-foreground/90">{children}</li>,
                          a: ({href, children}) => <a href={href} className="text-primary hover:underline" target="_blank" rel="noopener noreferrer">{children}</a>,
                          code: ({children, className}) => {
                            const isBlock = className?.includes("language-");
                            if (isBlock) return <pre className="bg-muted rounded-md p-4 mb-4 overflow-x-auto"><code className="text-sm font-mono">{children}</code></pre>;
                            return <code className="bg-muted rounded px-1.5 py-0.5 text-sm font-mono">{children}</code>;
                          },
                          pre: ({children}) => <>{children}</>,
                          table: ({children}) => <div className="overflow-x-auto mb-4"><table className="min-w-full border-collapse border border-border">{children}</table></div>,
                          th: ({children}) => <th className="border border-border bg-muted px-3 py-2 text-left text-sm font-semibold">{children}</th>,
                          td: ({children}) => <td className="border border-border px-3 py-2 text-sm">{children}</td>,
                          hr: () => <hr className="my-6 border-t border-muted-foreground/20" />,
                          blockquote: ({children}) => <blockquote className="border-l-4 border-primary/30 pl-4 my-4 italic text-muted-foreground">{children}</blockquote>,
                        }}
                      >
                        {whitepaper.content}
                      </ReactMarkdown>
                    </div>
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
