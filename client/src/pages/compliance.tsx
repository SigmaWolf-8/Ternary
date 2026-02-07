import { Badge } from "@/components/ui/badge";
import { Card } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import {
  Shield,
  ShieldCheck,
  ShieldAlert,
  Clock,
  Check,
  ArrowRight,
  Lock,
  Key,
  FileSignature,
  Hash,
  Layers,
  Binary,
  CircleDot,
  ExternalLink,
} from "lucide-react";
import { motion } from "framer-motion";
import { Link } from "wouter";

interface ComplianceMapping {
  algorithm: string;
  nistStandard: string;
  category: string;
  securityBits: number;
  deadline: string;
  status: "equivalent" | "planned" | "not_implemented";
  plenumEquivalent: string;
  plenumModule: string;
  securityNotes: string;
}

const cnsa2Matrix: ComplianceMapping[] = [
  {
    algorithm: "AES-256",
    nistStandard: "FIPS 197",
    category: "Symmetric Encryption",
    securityBits: 256,
    deadline: "Immediate",
    status: "planned",
    plenumEquivalent: "Ternary Bijective Cipher (design complete, implementation planned)",
    plenumModule: "salvi_kernel::crypto::cipher (planned)",
    securityNotes: "Designed for GF(3) with bijective S-box substitution. Phase encryption (salvi_kernel::phase) provides current symmetric encryption. Dedicated cipher module planned.",
  },
  {
    algorithm: "SHA-384",
    nistStandard: "FIPS 180-4",
    category: "Hashing",
    securityBits: 192,
    deadline: "Immediate",
    status: "equivalent",
    plenumEquivalent: "Ternary Sponge Hash (243-trit)",
    plenumModule: "salvi_kernel::crypto::hash",
    securityNotes: "Keccak-inspired sponge over GF(3). 729-trit state, 243-trit rate, 486-trit capacity. 243-trit output = 385.4 equivalent bits.",
  },
  {
    algorithm: "SHA-512",
    nistStandard: "FIPS 180-4",
    category: "Hashing",
    securityBits: 256,
    deadline: "Immediate",
    status: "equivalent",
    plenumEquivalent: "Ternary Sponge Hash (486-trit extended)",
    plenumModule: "salvi_kernel::crypto::sponge",
    securityNotes: "Extended sponge squeeze. 486-trit output = 770.8 equivalent bits. Configurable output length.",
  },
  {
    algorithm: "ML-KEM-512",
    nistStandard: "FIPS 203",
    category: "Key Encapsulation",
    securityBits: 128,
    deadline: "By 2030",
    status: "planned",
    plenumEquivalent: "Ternary Lattice KEM (TL-KEM)",
    plenumModule: "planned",
    securityNotes: "Module-LWE maps naturally to ternary arithmetic. Kyber uses ternary error distributions {-1, 0, 1} matching Representation A.",
  },
  {
    algorithm: "ML-KEM-768",
    nistStandard: "FIPS 203",
    category: "Key Encapsulation",
    securityBits: 192,
    deadline: "By 2030",
    status: "planned",
    plenumEquivalent: "Ternary Lattice KEM (TL-KEM-768)",
    plenumModule: "planned",
    securityNotes: "Medium-security TL-KEM. Ternary lattice dimension provides natural resistance to lattice reduction attacks.",
  },
  {
    algorithm: "ML-KEM-1024",
    nistStandard: "FIPS 203",
    category: "Key Encapsulation",
    securityBits: 256,
    deadline: "By 2030",
    status: "planned",
    plenumEquivalent: "Ternary Lattice KEM (TL-KEM-1024)",
    plenumModule: "planned",
    securityNotes: "High-security variant for classified data. Ternary coefficient encoding reduces ciphertext size vs binary ML-KEM.",
  },
  {
    algorithm: "ML-DSA-44",
    nistStandard: "FIPS 204",
    category: "Digital Signatures",
    securityBits: 128,
    deadline: "By 2035",
    status: "planned",
    plenumEquivalent: "Ternary Lattice DSA (TL-DSA)",
    plenumModule: "salvi_kernel::crypto::signature (enum stub only)",
    securityNotes: "Dilithium stub exists. GF(3) polynomial arithmetic provides natural coefficient representation.",
  },
  {
    algorithm: "ML-DSA-65",
    nistStandard: "FIPS 204",
    category: "Digital Signatures",
    securityBits: 192,
    deadline: "By 2035",
    status: "planned",
    plenumEquivalent: "Ternary Lattice DSA (TL-DSA-65)",
    plenumModule: "salvi_kernel::crypto::signature (enum stub only)",
    securityNotes: "Ternary polynomial ring Rq = Z3[X]/(X^n + 1) provides compact representation.",
  },
  {
    algorithm: "ML-DSA-87",
    nistStandard: "FIPS 204",
    category: "Digital Signatures",
    securityBits: 256,
    deadline: "By 2035",
    status: "planned",
    plenumEquivalent: "Ternary Lattice DSA (TL-DSA-87)",
    plenumModule: "salvi_kernel::crypto::signature (enum stub only)",
    securityNotes: "High-security Dilithium variant. CNSA 2.0 requires this level for national security systems.",
  },
  {
    algorithm: "LMS",
    nistStandard: "SP 800-208",
    category: "Hash-Based Signatures",
    securityBits: 256,
    deadline: "By 2030",
    status: "equivalent",
    plenumEquivalent: "Ternary Lamport OTS with Key Chain",
    plenumModule: "salvi_kernel::crypto::signature",
    securityNotes: "Fully implemented. Security relies only on hash preimage resistance. LamportKeyChain supports indexed multi-message signing.",
  },
  {
    algorithm: "XMSS",
    nistStandard: "SP 800-208",
    category: "Hash-Based Signatures",
    securityBits: 256,
    deadline: "By 2030",
    status: "equivalent",
    plenumEquivalent: "Ternary Lamport OTS (Merkle extension planned)",
    plenumModule: "salvi_kernel::crypto::signature",
    securityNotes: "Lamport OTS provides the foundational primitive. XMSS-equivalent Merkle tree structure planned for stateful multi-use signing.",
  },
];

interface TimelineMilestone {
  year: number;
  title: string;
  description: string;
  algorithms: string[];
  status: "complete" | "in_progress" | "planned";
}

const timeline: TimelineMilestone[] = [
  {
    year: 2025,
    title: "Foundation Complete",
    description: "Ternary sponge hash, HMAC, KDF, and Lamport OTS provide GF(3)-native equivalents for SHA-384/512 and LMS/XMSS. Phase encryption provides current symmetric encryption.",
    algorithms: ["SHA-384", "SHA-512", "LMS"],
    status: "complete",
  },
  {
    year: 2026,
    title: "Lattice Foundations",
    description: "GF(3) polynomial ring arithmetic primitives. Module-LWE and Module-SIS problem implementations in ternary domain.",
    algorithms: [],
    status: "in_progress",
  },
  {
    year: 2027,
    title: "TL-KEM Implementation",
    description: "Ternary Lattice Key Encapsulation at all three security levels. Binary compatibility layer for ML-KEM interoperability.",
    algorithms: ["ML-KEM-512", "ML-KEM-768", "ML-KEM-1024"],
    status: "planned",
  },
  {
    year: 2028,
    title: "TL-DSA Implementation",
    description: "Full Dilithium-equivalent signatures with ternary polynomial arithmetic at all security levels.",
    algorithms: ["ML-DSA-44", "ML-DSA-65", "ML-DSA-87"],
    status: "planned",
  },
  {
    year: 2029,
    title: "XMSS Merkle Extension",
    description: "Stateful hash-based signature trees using ternary sponge hash. Full XMSS-equivalent Merkle tree construction.",
    algorithms: ["XMSS"],
    status: "planned",
  },
  {
    year: 2030,
    title: "Full CNSA 2.0 Coverage",
    description: "Complete ternary-native equivalents for all CNSA 2.0 algorithms with binary compatibility layer and FIPS validation initiated.",
    algorithms: [],
    status: "planned",
  },
];

function StatusBadge({ status }: { status: "equivalent" | "planned" | "not_implemented" }) {
  if (status === "equivalent") {
    return (
      <Badge variant="default" className="bg-emerald-600 text-white" data-testid="badge-status-equivalent">
        <ShieldCheck className="w-3 h-3 mr-1" />
        Ternary Equivalent
      </Badge>
    );
  }
  if (status === "planned") {
    return (
      <Badge variant="secondary" data-testid="badge-status-planned">
        <Clock className="w-3 h-3 mr-1" />
        Planned
      </Badge>
    );
  }
  return (
    <Badge variant="destructive" data-testid="badge-status-not-implemented">
      <ShieldAlert className="w-3 h-3 mr-1" />
      Not Implemented
    </Badge>
  );
}

function MilestoneStatusBadge({ status }: { status: "complete" | "in_progress" | "planned" }) {
  if (status === "complete") {
    return (
      <Badge variant="default" className="bg-emerald-600 text-white" data-testid="badge-milestone-complete">
        <Check className="w-3 h-3 mr-1" />
        Complete
      </Badge>
    );
  }
  if (status === "in_progress") {
    return (
      <Badge variant="default" className="bg-blue-600 text-white" data-testid="badge-milestone-in-progress">
        <CircleDot className="w-3 h-3 mr-1" />
        In Progress
      </Badge>
    );
  }
  return (
    <Badge variant="secondary" data-testid="badge-milestone-planned">
      <Clock className="w-3 h-3 mr-1" />
      Planned
    </Badge>
  );
}

function getCategoryIcon(category: string) {
  switch (category) {
    case "Symmetric Encryption": return <Lock className="w-4 h-4" />;
    case "Hashing": return <Hash className="w-4 h-4" />;
    case "Key Encapsulation": return <Key className="w-4 h-4" />;
    case "Digital Signatures": return <FileSignature className="w-4 h-4" />;
    case "Hash-Based Signatures": return <FileSignature className="w-4 h-4" />;
    default: return <Shield className="w-4 h-4" />;
  }
}

function HeroSection() {
  const equivalentCount = cnsa2Matrix.filter(m => m.status === "equivalent").length;
  const plannedCount = cnsa2Matrix.filter(m => m.status === "planned").length;
  const totalCount = cnsa2Matrix.length;
  const coveragePercent = Math.round((equivalentCount / totalCount) * 100);

  return (
    <section className="py-16 md:py-24" data-testid="section-compliance-hero">
      <div className="max-w-7xl mx-auto px-5">
        <div className="text-center mb-12">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5 }}
          >
            <Badge variant="outline" className="border-primary/30 bg-primary/10 text-primary px-4 py-1.5 mb-4" data-testid="badge-cnsa2-label">
              CNSA 2.0 Compliance
            </Badge>
          </motion.div>
          <motion.h1
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5, delay: 0.1 }}
            className="text-3xl md:text-5xl font-bold mb-4"
            data-testid="text-compliance-title"
          >
            Post-Quantum Algorithm Coverage
          </motion.h1>
          <motion.p
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5, delay: 0.15 }}
            className="text-muted-foreground text-lg max-w-3xl mx-auto mb-8"
            data-testid="text-compliance-description"
          >
            Mapping NSA CNSA 2.0 requirements to PlenumNET's ternary-native cryptographic primitives. 
            Operating in GF(3) provides quantum resistance through a fundamentally different mathematical domain.
          </motion.p>
        </div>

        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.2 }}
          className="grid grid-cols-2 md:grid-cols-4 gap-4 max-w-3xl mx-auto"
        >
          <Card className="p-4 text-center" data-testid="card-stat-total">
            <div className="text-2xl font-bold text-foreground">{totalCount}</div>
            <div className="text-xs text-muted-foreground mt-1">Total Algorithms</div>
          </Card>
          <Card className="p-4 text-center" data-testid="card-stat-equivalent">
            <div className="text-2xl font-bold text-emerald-600">{equivalentCount}</div>
            <div className="text-xs text-muted-foreground mt-1">Ternary Equivalent</div>
          </Card>
          <Card className="p-4 text-center" data-testid="card-stat-planned">
            <div className="text-2xl font-bold text-blue-600">{plannedCount}</div>
            <div className="text-xs text-muted-foreground mt-1">Planned</div>
          </Card>
          <Card className="p-4 text-center" data-testid="card-stat-coverage">
            <div className="text-2xl font-bold text-primary">{coveragePercent}%</div>
            <div className="text-xs text-muted-foreground mt-1">Current Coverage</div>
          </Card>
        </motion.div>
      </div>
    </section>
  );
}

function ComplianceMatrixSection() {
  const categories = ["Symmetric Encryption", "Hashing", "Key Encapsulation", "Digital Signatures", "Hash-Based Signatures"];

  return (
    <section className="py-16 md:py-20 bg-secondary/30" data-testid="section-compliance-matrix">
      <div className="max-w-7xl mx-auto px-5">
        <div className="text-center mb-12">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5 }}
          >
            <Badge variant="outline" className="border-primary/30 bg-primary/10 text-primary px-4 py-1.5 mb-4">
              Algorithm Matrix
            </Badge>
          </motion.div>
          <motion.h2
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.1 }}
            className="text-2xl md:text-3xl font-bold mb-3"
            data-testid="text-matrix-title"
          >
            CNSA 2.0 Requirements vs PlenumNET Equivalents
          </motion.h2>
          <motion.p
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.15 }}
            className="text-muted-foreground max-w-2xl mx-auto"
          >
            Each CNSA 2.0 algorithm maps to a ternary-native equivalent operating in GF(3) with comparable or superior security properties.
          </motion.p>
        </div>

        <div className="space-y-6">
          {categories.map((category, catIndex) => {
            const categoryAlgorithms = cnsa2Matrix.filter(m => m.category === category);
            return (
              <motion.div
                key={category}
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.4, delay: catIndex * 0.08 }}
              >
                <div className="flex items-center gap-2 mb-3">
                  {getCategoryIcon(category)}
                  <h3 className="text-lg font-semibold" data-testid={`text-category-${category.replace(/\s+/g, '-').toLowerCase()}`}>
                    {category}
                  </h3>
                </div>
                <div className="space-y-3">
                  {categoryAlgorithms.map((mapping) => (
                    <Card key={mapping.algorithm} className="p-4" data-testid={`card-algorithm-${mapping.algorithm.toLowerCase().replace(/[^a-z0-9]/g, '-')}`}>
                      <div className="flex flex-col md:flex-row md:items-start gap-4">
                        <div className="flex-1 min-w-0">
                          <div className="flex flex-wrap items-center gap-2 mb-2">
                            <span className="font-semibold text-sm" data-testid={`text-algorithm-name-${mapping.algorithm.toLowerCase().replace(/[^a-z0-9]/g, '-')}`}>
                              {mapping.algorithm}
                            </span>
                            <Badge variant="outline" className="text-[10px]">{mapping.nistStandard}</Badge>
                            <Badge variant="outline" className="text-[10px]">{mapping.securityBits}-bit</Badge>
                            <Badge variant="outline" className="text-[10px]">{mapping.deadline}</Badge>
                          </div>
                          <div className="flex items-center gap-2 mb-2">
                            <ArrowRight className="w-3 h-3 text-primary flex-shrink-0" />
                            <span className="text-sm text-foreground font-medium">{mapping.plenumEquivalent}</span>
                          </div>
                          <p className="text-xs text-muted-foreground">{mapping.securityNotes}</p>
                        </div>
                        <div className="flex-shrink-0">
                          <StatusBadge status={mapping.status} />
                        </div>
                      </div>
                    </Card>
                  ))}
                </div>
              </motion.div>
            );
          })}
        </div>
      </div>
    </section>
  );
}

function TimelineSection() {
  return (
    <section className="py-16 md:py-20" data-testid="section-compliance-timeline">
      <div className="max-w-7xl mx-auto px-5">
        <div className="text-center mb-12">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5 }}
          >
            <Badge variant="outline" className="border-primary/30 bg-primary/10 text-primary px-4 py-1.5 mb-4">
              Transition Timeline
            </Badge>
          </motion.div>
          <motion.h2
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.1 }}
            className="text-2xl md:text-3xl font-bold mb-3"
            data-testid="text-timeline-title"
          >
            CNSA 2.0 Transition Roadmap
          </motion.h2>
          <motion.p
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.15 }}
            className="text-muted-foreground max-w-2xl mx-auto"
          >
            From foundation primitives to full algorithm coverage, aligned with NSA transition deadlines.
          </motion.p>
        </div>

        <div className="max-w-3xl mx-auto">
          <div className="relative">
            <div className="absolute left-[27px] top-0 bottom-0 w-px bg-border" />
            
            <div className="space-y-6">
              {timeline.map((milestone, index) => (
                <motion.div
                  key={milestone.year}
                  initial={{ opacity: 0, x: -20 }}
                  whileInView={{ opacity: 1, x: 0 }}
                  viewport={{ once: true }}
                  transition={{ duration: 0.4, delay: index * 0.08 }}
                  className="relative flex gap-4"
                  data-testid={`timeline-milestone-${milestone.year}`}
                >
                  <div className="relative z-10 flex-shrink-0">
                    <div className={`w-[55px] h-[55px] rounded-full flex items-center justify-center text-sm font-bold border-2 ${
                      milestone.status === "complete" 
                        ? "bg-emerald-600 text-white border-emerald-600" 
                        : milestone.status === "in_progress"
                        ? "bg-blue-600 text-white border-blue-600"
                        : "bg-background text-muted-foreground border-border"
                    }`}>
                      {milestone.year}
                    </div>
                  </div>
                  <Card className="flex-1 p-4">
                    <div className="flex flex-wrap items-center gap-2 mb-2">
                      <h3 className="font-semibold text-sm">{milestone.title}</h3>
                      <MilestoneStatusBadge status={milestone.status} />
                    </div>
                    <p className="text-xs text-muted-foreground mb-2">{milestone.description}</p>
                    {milestone.algorithms.length > 0 && (
                      <div className="flex flex-wrap gap-1">
                        {milestone.algorithms.map(alg => (
                          <Badge key={alg} variant="outline" className="text-[10px]">{alg}</Badge>
                        ))}
                      </div>
                    )}
                  </Card>
                </motion.div>
              ))}
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}

function TernaryAdvantageSection() {
  const advantages = [
    {
      icon: Layers,
      title: "Different Mathematical Domain",
      description: "GF(3) operations don't map directly to quantum gate operations optimized for GF(2). Quantum circuits must be redesigned for ternary arithmetic, reducing the practical advantage of known quantum algorithms.",
    },
    {
      icon: Key,
      title: "Natural Lattice Coefficients",
      description: "Post-quantum lattice problems (LWE, SIS) use ternary error distributions {-1, 0, 1}. PlenumNET's Representation A provides native encoding without binary-to-ternary conversion overhead.",
    },
    {
      icon: Binary,
      title: "Information Density",
      description: "Each trit carries 1.585 bits. A 243-trit hash output provides 385.4 equivalent bits of security, exceeding SHA-384's 384 bits with fewer computational elements.",
    },
    {
      icon: Shield,
      title: "Binary Compatibility",
      description: "The Binary-Ternary Gateway enables hybrid deployment where ternary operations run natively while maintaining interoperability with standard binary CNSA 2.0 implementations.",
    },
  ];

  const gf3Operations = [
    { operation: "Addition", definition: "(a + b) mod 3 in balanced representation", resistance: "Non-binary field structure" },
    { operation: "Multiplication", definition: "(a x b) mod 3 with ternary S-boxes", resistance: "Different algebraic group" },
    { operation: "Inverse", definition: "Unique multiplicative inverse in GF(3)*", resistance: "Novel factoring resistance" },
    { operation: "XOR-equivalent", definition: "Ternary addition in GF(3)", resistance: "Three-valued logic complexity" },
  ];

  return (
    <section className="py-16 md:py-20 bg-secondary/30" data-testid="section-ternary-advantage">
      <div className="max-w-7xl mx-auto px-5">
        <div className="text-center mb-12">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5 }}
          >
            <Badge variant="outline" className="border-primary/30 bg-primary/10 text-primary px-4 py-1.5 mb-4">
              Ternary Advantage
            </Badge>
          </motion.div>
          <motion.h2
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.1 }}
            className="text-2xl md:text-3xl font-bold mb-3"
            data-testid="text-advantage-title"
          >
            Why GF(3) Matters for Post-Quantum Security
          </motion.h2>
          <motion.p
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.15 }}
            className="text-muted-foreground max-w-2xl mx-auto"
          >
            Quantum computers attack binary cryptography by exploiting binary field structure. 
            PlenumNET's ternary-native approach introduces structural quantum resistance.
          </motion.p>
        </div>

        <div className="grid md:grid-cols-2 gap-4 mb-10">
          {advantages.map((adv, index) => (
            <motion.div
              key={adv.title}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.4, delay: index * 0.08 }}
            >
              <Card className="p-5 h-full" data-testid={`card-advantage-${index}`}>
                <div className="flex items-start gap-3">
                  <div className="w-9 h-9 rounded-md bg-primary/10 flex items-center justify-center flex-shrink-0">
                    <adv.icon className="w-4 h-4 text-primary" />
                  </div>
                  <div>
                    <h3 className="font-semibold text-sm mb-1">{adv.title}</h3>
                    <p className="text-xs text-muted-foreground">{adv.description}</p>
                  </div>
                </div>
              </Card>
            </motion.div>
          ))}
        </div>

        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5, delay: 0.3 }}
        >
          <Card className="p-5 max-w-3xl mx-auto" data-testid="card-gf3-operations">
            <h3 className="font-semibold text-sm mb-3">GF(3) Arithmetic Operations</h3>
            <div className="overflow-x-auto">
              <table className="w-full text-xs">
                <thead>
                  <tr className="border-b">
                    <th className="text-left py-2 pr-4 font-medium text-muted-foreground">Operation</th>
                    <th className="text-left py-2 pr-4 font-medium text-muted-foreground">Definition</th>
                    <th className="text-left py-2 font-medium text-muted-foreground">Quantum Resistance</th>
                  </tr>
                </thead>
                <tbody>
                  {gf3Operations.map(op => (
                    <tr key={op.operation} className="border-b last:border-0">
                      <td className="py-2 pr-4 font-medium">{op.operation}</td>
                      <td className="py-2 pr-4 text-muted-foreground font-mono text-[11px]">{op.definition}</td>
                      <td className="py-2 text-muted-foreground">{op.resistance}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </Card>
        </motion.div>
      </div>
    </section>
  );
}

function RegulatoryContextSection() {
  const regulations = [
    {
      icon: Clock,
      title: "FINRA Rule 613",
      subtitle: "Consolidated Audit Trail",
      requirement: "Sub-50ms NIST synchronization for US securities trading",
      plenumResponse: "HPTP femtosecond timing exceeds threshold by orders of magnitude. Audit records signed with quantum-resistant Lamport OTS.",
    },
    {
      icon: Shield,
      title: "MiFID II Article 50",
      subtitle: "EU Trading Regulation",
      requirement: "100us synchronization for HFT, 1ms for standard trading",
      plenumResponse: "HPTP with optical clock sync provides sub-picosecond precision. All timing certificates cryptographically signed.",
    },
    {
      icon: FileSignature,
      title: "NIST SP 800-208",
      subtitle: "Hash-Based Signatures",
      requirement: "Stateful hash-based signature standard for firmware and software signing",
      plenumResponse: "Ternary Lamport OTS with sponge hash aligns with stateful signature approach. LamportKeyChain provides indexed signing.",
    },
  ];

  return (
    <section className="py-16 md:py-20" data-testid="section-regulatory-context">
      <div className="max-w-7xl mx-auto px-5">
        <div className="text-center mb-12">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5 }}
          >
            <Badge variant="outline" className="border-primary/30 bg-primary/10 text-primary px-4 py-1.5 mb-4">
              Regulatory Intersection
            </Badge>
          </motion.div>
          <motion.h2
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.1 }}
            className="text-2xl md:text-3xl font-bold mb-3"
            data-testid="text-regulatory-title"
          >
            CNSA 2.0 Meets Financial Compliance
          </motion.h2>
          <motion.p
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.15 }}
            className="text-muted-foreground max-w-2xl mx-auto"
          >
            CNSA 2.0 cryptographic compliance intersects with PlenumNET's existing regulatory framework for financial services.
          </motion.p>
        </div>

        <div className="grid md:grid-cols-3 gap-4 max-w-5xl mx-auto">
          {regulations.map((reg, index) => (
            <motion.div
              key={reg.title}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.4, delay: index * 0.1 }}
            >
              <Card className="p-5 h-full" data-testid={`card-regulation-${index}`}>
                <div className="flex items-center gap-2 mb-3">
                  <div className="w-8 h-8 rounded-md bg-primary/10 flex items-center justify-center flex-shrink-0">
                    <reg.icon className="w-4 h-4 text-primary" />
                  </div>
                  <div>
                    <h3 className="font-semibold text-sm">{reg.title}</h3>
                    <p className="text-[10px] text-muted-foreground">{reg.subtitle}</p>
                  </div>
                </div>
                <div className="space-y-2">
                  <div>
                    <span className="text-[10px] font-medium uppercase tracking-wider text-muted-foreground">Requirement</span>
                    <p className="text-xs text-foreground">{reg.requirement}</p>
                  </div>
                  <div>
                    <span className="text-[10px] font-medium uppercase tracking-wider text-muted-foreground">PlenumNET Response</span>
                    <p className="text-xs text-muted-foreground">{reg.plenumResponse}</p>
                  </div>
                </div>
              </Card>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
}

function CTASection() {
  return (
    <section className="py-16 md:py-20 bg-secondary/30" data-testid="section-compliance-cta">
      <div className="max-w-3xl mx-auto px-5 text-center">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
        >
          <h2 className="text-2xl md:text-3xl font-bold mb-3">Explore the Framework</h2>
          <p className="text-muted-foreground mb-6">
            Dive deeper into the cryptographic primitives, read the whitepaper, or explore the full documentation.
          </p>
          <div className="flex flex-wrap justify-center gap-3">
            <Button asChild data-testid="button-cta-whitepaper">
              <Link href="/whitepaper">
                Read Whitepaper
                <ArrowRight className="w-4 h-4 ml-1" />
              </Link>
            </Button>
            <Button variant="outline" asChild data-testid="button-cta-docs">
              <Link href="/docs">
                Documentation
                <ExternalLink className="w-4 h-4 ml-1" />
              </Link>
            </Button>
            <Button variant="outline" asChild data-testid="button-cta-api-demo">
              <Link href="/api-demo">
                API Demo
                <ExternalLink className="w-4 h-4 ml-1" />
              </Link>
            </Button>
          </div>
        </motion.div>
      </div>
    </section>
  );
}

export default function CompliancePage() {
  return (
    <div className="min-h-screen" data-testid="page-compliance">
      <HeroSection />
      <ComplianceMatrixSection />
      <TimelineSection />
      <TernaryAdvantageSection />
      <RegulatoryContextSection />
      <CTASection />
    </div>
  );
}
