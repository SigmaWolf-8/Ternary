import { Card } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import {
  BookOpen,
  Cpu,
  Lock,
  Shield,
  Network,
  Binary,
  Terminal,
  ExternalLink,
  Rocket,
  Layers,
  HardDrive,
  FolderTree,
  Globe,
  Clock,
  Settings,
  KeyRound,
} from "lucide-react";

const tutorials = [
  {
    title: "Getting Started",
    description: "First ternary program in 15 minutes. Clone, build, and run your first TVM code.",
    time: "15 min",
    icon: Rocket,
    file: "tutorials/GETTING_STARTED.md",
  },
];

const modules = [
  {
    id: "01",
    title: "Kernel Memory",
    description: "Frame allocator, page tables, heap allocator, and memory protection with ternary addressing.",
    priority: "P1",
    tests: "~60",
    icon: Cpu,
    file: "modules/01_KERNEL_MEMORY.md",
  },
  {
    id: "02",
    title: "Sync Primitives",
    description: "Ternary spinlock, security-gated mutex, counting semaphore, PhaseSafeMutex, and RwLock.",
    priority: "P1",
    tests: "~45",
    icon: Lock,
    file: "modules/02_SYNC_PRIMITIVES.md",
  },
  {
    id: "03",
    title: "Process Management",
    description: "8-state process model, 6-level priority scheduler, context switching, and typed IPC.",
    priority: "P1",
    tests: "~55",
    icon: Layers,
    file: "modules/03_PROCESS_MANAGEMENT.md",
  },
  {
    id: "04",
    title: "Modal Security",
    description: "Four-mode security (0/1/phi/phi+), domain isolation, capabilities, audit trail, and policy engine.",
    priority: "P1",
    tests: "~70",
    icon: Shield,
    file: "modules/04_MODAL_SECURITY.md",
  },
  {
    id: "05",
    title: "Cryptography",
    description: "Ternary sponge hash, HMAC, KDF, Lamport one-time signatures, and bijective cipher.",
    priority: "P1",
    tests: "~85",
    icon: KeyRound,
    file: "modules/05_CRYPTOGRAPHY.md",
  },
  {
    id: "06",
    title: "Device Framework",
    description: "Device lifecycle, bus hierarchy (PCI/USB/Ternary), driver registry, IRQ, and DMA engine.",
    priority: "P1.5",
    tests: "~48",
    icon: Settings,
    file: "modules/06_DEVICE_FRAMEWORK.md",
  },
  {
    id: "07",
    title: "I/O Subsystem",
    description: "4-level priority scheduler, LRU buffer cache, block and character device layers.",
    priority: "P1.5",
    tests: "~52",
    icon: HardDrive,
    file: "modules/07_IO_SUBSYSTEM.md",
  },
  {
    id: "08",
    title: "Filesystem",
    description: "Inode management, B-tree directories, TernFS, VFS mount system, and journaling.",
    priority: "P1.5",
    tests: "~56",
    icon: FolderTree,
    file: "modules/08_FILESYSTEM.md",
  },
  {
    id: "09",
    title: "Torsion Network",
    description: "N-dimensional torus topology (7D/10D/13D), geodesic routing, and route caching.",
    priority: "P2",
    tests: "~75",
    icon: Network,
    file: "modules/09_TORSION_NETWORK.md",
  },
  {
    id: "10",
    title: "Network Protocols",
    description: "TTP 11-state transport, T3P application protocol with Witness method, and TDNS resolution.",
    priority: "P2",
    tests: "~85",
    icon: Globe,
    file: "modules/10_NETWORK_PROTOCOLS.md",
  },
  {
    id: "11",
    title: "Ternary Virtual Machine",
    description: "35-opcode ISA, 27 registers, GF(3) ALU, and ternary-aware garbage collector (TAGC).",
    priority: "P3",
    tests: "~95",
    icon: Terminal,
    file: "modules/11_TVM.md",
  },
  {
    id: "12",
    title: "Timing Protocol (HPTP)",
    description: "Femtosecond synchronization, optical clock manager, FINRA 613 and MiFID II compliance.",
    priority: "P3",
    tests: "~55",
    icon: Clock,
    file: "modules/12_TIMING_PROTOCOL.md",
  },
  {
    id: "13",
    title: "Binary-Ternary Gateway",
    description: "Type A/B/C conversions, Universal Adapter, multi-step pipelines, and protocol bridging.",
    priority: "P3",
    tests: "~65",
    icon: Binary,
    file: "modules/13_BTG.md",
  },
];

function PriorityBadge({ priority }: { priority: string }) {
  const variant = priority === "P1" ? "default" : "secondary";
  return <Badge variant={variant} data-testid={`badge-priority-${priority}`}>{priority}</Badge>;
}

export default function Docs() {
  return (
    <div className="min-h-screen bg-background">
      <main className="max-w-6xl mx-auto px-5 py-8">
        <div className="mb-12">
          <h2 className="text-2xl font-bold mb-2" data-testid="text-overview-title">Documentation</h2>
          <p className="text-muted-foreground max-w-2xl">
            Comprehensive guides covering every module of the Salvi Framework. 
            7,316 lines across 15 files — from getting started with your first ternary program to advanced kernel internals, network topology, and regulatory compliance.
          </p>
          <div className="flex flex-wrap gap-4 mt-4">
            <Badge variant="outline">1,011 Tests Passing</Badge>
            <Badge variant="outline">80/80 Roadmap Complete</Badge>
            <Badge variant="outline">13 Module Guides</Badge>
            <Badge variant="outline">1 Tutorial</Badge>
          </div>
        </div>

        <section className="mb-12" data-testid="section-tutorials">
          <h3 className="text-xl font-semibold mb-4 flex items-center gap-2">
            <Rocket className="w-5 h-5 text-primary" />
            Tutorials
          </h3>
          <div className="grid grid-cols-1 gap-4">
            {tutorials.map((tutorial) => (
              <Card key={tutorial.file} className="p-6 hover-elevate" data-testid={`card-tutorial-${tutorial.file}`}>
                <div className="flex items-start gap-4">
                  <div className="p-2 rounded-md bg-primary/10 shrink-0">
                    <tutorial.icon className="w-5 h-5 text-primary" />
                  </div>
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 flex-wrap mb-1">
                      <h4 className="font-semibold">{tutorial.title}</h4>
                      <Badge variant="outline">{tutorial.time}</Badge>
                    </div>
                    <p className="text-sm text-muted-foreground">{tutorial.description}</p>
                  </div>
                  <Button variant="outline" size="sm" asChild data-testid={`button-view-tutorial-${tutorial.file}`}>
                    <a href={`https://github.com/SigmaWolf-8/Ternary/blob/main/salvi_docs/${tutorial.file}`} target="_blank" rel="noopener noreferrer">
                      View
                      <ExternalLink className="w-3 h-3 ml-1" />
                    </a>
                  </Button>
                </div>
              </Card>
            ))}
          </div>
        </section>

        <section className="mb-12" data-testid="section-module-guides">
          <h3 className="text-xl font-semibold mb-4 flex items-center gap-2">
            <BookOpen className="w-5 h-5 text-primary" />
            Module Guides
          </h3>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {modules.map((mod_) => (
              <Card key={mod_.id} className="p-6 hover-elevate" data-testid={`card-module-${mod_.id}`}>
                <div className="flex items-start gap-4">
                  <div className="p-2 rounded-md bg-primary/10 shrink-0">
                    <mod_.icon className="w-5 h-5 text-primary" />
                  </div>
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 flex-wrap mb-1">
                      <h4 className="font-semibold">{mod_.title}</h4>
                      <PriorityBadge priority={mod_.priority} />
                      <Badge variant="outline">{mod_.tests} tests</Badge>
                    </div>
                    <p className="text-sm text-muted-foreground">{mod_.description}</p>
                  </div>
                </div>
                <div className="mt-4 flex justify-end">
                  <Button variant="outline" size="sm" asChild data-testid={`button-view-module-${mod_.id}`}>
                    <a href={`https://github.com/SigmaWolf-8/Ternary/blob/main/salvi_docs/${mod_.file}`} target="_blank" rel="noopener noreferrer">
                      Read Guide
                      <ExternalLink className="w-3 h-3 ml-1" />
                    </a>
                  </Button>
                </div>
              </Card>
            ))}
          </div>
        </section>

        <section data-testid="section-quick-reference">
          <h3 className="text-xl font-semibold mb-4">Quick Reference</h3>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <Card className="p-5">
              <h4 className="font-semibold mb-3">Ternary Computing</h4>
              <div className="text-sm space-y-1 text-muted-foreground">
                <div className="flex justify-between"><span>Digit values</span><span className="font-mono">-1, 0, +1</span></div>
                <div className="flex justify-between"><span>Basic unit</span><span className="font-mono">Trit</span></div>
                <div className="flex justify-between"><span>Byte equivalent</span><span className="font-mono">6 trits (Tryte)</span></div>
                <div className="flex justify-between"><span>Info density</span><span className="font-mono">1.58 bits/digit</span></div>
              </div>
            </Card>
            <Card className="p-5">
              <h4 className="font-semibold mb-3">GF(3) Arithmetic</h4>
              <div className="text-sm space-y-1 text-muted-foreground">
                <div className="flex justify-between"><span>Elements</span><span className="font-mono">0, 1, 2</span></div>
                <div className="flex justify-between"><span>Addition</span><span className="font-mono">(a + b) mod 3</span></div>
                <div className="flex justify-between"><span>Multiplication</span><span className="font-mono">(a * b) mod 3</span></div>
                <div className="flex justify-between"><span>Inverse</span><span className="font-mono">a*a^-1 = 1 mod 3</span></div>
              </div>
            </Card>
            <Card className="p-5">
              <h4 className="font-semibold mb-3">Torsion Topology</h4>
              <div className="text-sm space-y-1 text-muted-foreground">
                <div className="flex justify-between"><span>Dimensions</span><span className="font-mono">7D to 13D</span></div>
                <div className="flex justify-between"><span>7D nodes</span><span className="font-mono">2,187</span></div>
                <div className="flex justify-between"><span>13D nodes</span><span className="font-mono">1,594,323</span></div>
                <div className="flex justify-between"><span>Routing</span><span className="font-mono">Geodesic</span></div>
              </div>
            </Card>
          </div>
        </section>
      </main>

      <footer className="border-t mt-16 py-8 text-center text-sm text-muted-foreground" data-testid="docs-footer">
        <p>Salvi Framework v1.0 - Capomastro Holdings Ltd.</p>
        <p className="mt-1">
          <a href="https://github.com/SigmaWolf-8/Ternary" target="_blank" rel="noopener noreferrer" className="hover:text-primary transition-colors" data-testid="link-docs-github">
            View on GitHub
          </a>
        </p>
      </footer>
    </div>
  );
}
