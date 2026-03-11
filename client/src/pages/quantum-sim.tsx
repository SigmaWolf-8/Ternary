/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * PROPRIETARY AND CONFIDENTIAL
 * All Rights Reserved.
 *
 * This file is part of the Salvi Framework / PlenumNET platform.
 * Unauthorized copying, modification, distribution, or use of this file,
 * via any medium, is strictly prohibited without the prior written
 * permission of Capomastro Holdings Ltd.
 *
 * See LICENSE in the repository root for full terms.
 */

import { useState, useCallback } from "react";
import { Link } from "wouter";
import {
  Atom, ChevronRight, Shield, Zap, Activity,
  Play, RotateCcw, CheckCircle2, XCircle, AlertTriangle,
  FlaskConical, BarChart3, Lock, FileCheck,
} from "lucide-react";
import { Card } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { PLATFORM } from "@shared/constants";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Progress } from "@/components/ui/progress";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

import {
  type QutritState,
  type ErrorType,
  applyQutritError,
  encodeQutritStabilizer,
  measureSyndrome,
  correctQutritStabilizer,
  simulateTriorthogonalDistillation,
  qutritFidelity,
  detectError,
  qutritDepolarizingChannel,
  codeDistance,
} from "@shared/qutrit-fault-tolerance";

import {
  suftBasisState,
  qutritFromAmplitudes,
  normalizeQutrit,
  bornProbabilities,
  suftPhaseGate,
  applyGate,
  isUnitaryQutrit,
} from "@shared/qutrit-basics";

import { cx, cxMagSq } from "@shared/complex-utils";

import {
  tribonacciActionDeviation,
  generateTribonacci,
  tribonacciResiduals,
  tribonacciRatioConvergence,
  variationalFitness,
  verifyCanonicalTribonacci,
} from "@shared/tribonacci-variational";

import {
  computeHamiltonianState,
  checkHamiltonianConstraint,
  validateOpcodeSequence,
  computeBankTernaryParity,
} from "@shared/hamiltonian-constraints";

import {
  SUFT_RADIUS,
  SUFT_LUNAR_HARMONIC,
  SUFT_COSMIC_CIRCUMFERENCE,
  MASS_SHELL_RATIO,
} from "@shared/plenum-square";

function formatComplex(c: { re: number; im: number }): string {
  const r = c.re.toFixed(4);
  const i = c.im.toFixed(4);
  if (Math.abs(c.im) < 1e-10) return r;
  if (Math.abs(c.re) < 1e-10) return `${i}i`;
  return `${r}${c.im >= 0 ? "+" : ""}${i}i`;
}

function formatProb(p: number): string {
  return `${(p * 100).toFixed(2)}%`;
}

interface FTSimState {
  basisChoice: "-1" | "0" | "+1" | "superposition";
  errorType: ErrorType;
  distillationM: number;
  depolarizeRate: number;
  originalState: QutritState | null;
  errorState: QutritState | null;
  encodedBlocks: [QutritState, QutritState, QutritState] | null;
  syndrome: [number, number] | null;
  correctedState: QutritState | null;
  errorPosition: number | null;
  fidelityBeforeCorrection: number | null;
  fidelityAfterCorrection: number | null;
  distillationResult: {
    distilled: QutritState;
    yieldGamma: number;
    codeParams: { n: number; k: number; d: number };
  } | null;
  phaseGateResult: { unitary: boolean; appliedState: QutritState } | null;
  step: number;
}

function QutritFTTab() {
  const [sim, setSim] = useState<FTSimState>({
    basisChoice: "0",
    errorType: "phase",
    distillationM: 2,
    depolarizeRate: 0.1,
    originalState: null,
    errorState: null,
    encodedBlocks: null,
    syndrome: null,
    correctedState: null,
    errorPosition: null,
    fidelityBeforeCorrection: null,
    fidelityAfterCorrection: null,
    distillationResult: null,
    phaseGateResult: null,
    step: 0,
  });

  const resetSim = useCallback(() => {
    setSim(prev => ({
      ...prev,
      originalState: null,
      errorState: null,
      encodedBlocks: null,
      syndrome: null,
      correctedState: null,
      errorPosition: null,
      fidelityBeforeCorrection: null,
      fidelityAfterCorrection: null,
      distillationResult: null,
      phaseGateResult: null,
      step: 0,
    }));
  }, []);

  const prepareState = useCallback(() => {
    let state: QutritState;
    if (sim.basisChoice === "superposition") {
      state = normalizeQutrit(qutritFromAmplitudes(
        cx(1 / Math.sqrt(3)),
        cx(1 / Math.sqrt(3)),
        cx(1 / Math.sqrt(3))
      ));
    } else {
      state = suftBasisState(parseInt(sim.basisChoice) as -1 | 0 | 1);
    }
    setSim(prev => ({ ...prev, originalState: state, step: 1 }));
  }, [sim.basisChoice]);

  const applyError = useCallback(() => {
    if (!sim.originalState) return;
    const errored = applyQutritError(sim.originalState, sim.errorType);
    const fid = qutritFidelity(errored, sim.originalState);
    setSim(prev => ({
      ...prev,
      errorState: errored,
      fidelityBeforeCorrection: fid,
      step: 2,
    }));
  }, [sim.originalState, sim.errorType]);

  const encodeAndCorrect = useCallback(() => {
    if (!sim.originalState) return;
    const encoded = encodeQutritStabilizer(sim.originalState);
    if (sim.errorState) {
      encoded[1] = sim.errorState;
    }
    const synd = measureSyndrome(encoded);
    const { corrected, errorPosition } = correctQutritStabilizer(encoded);
    const fidAfter = qutritFidelity(corrected, sim.originalState);
    setSim(prev => ({
      ...prev,
      encodedBlocks: encoded,
      syndrome: synd,
      correctedState: corrected,
      errorPosition: errorPosition,
      fidelityAfterCorrection: fidAfter,
      step: 3,
    }));
  }, [sim.originalState, sim.errorState]);

  const runDistillation = useCallback(() => {
    if (!sim.originalState) return;
    const inputs = Array(sim.distillationM * 3).fill(null).map(() => {
      const noisy = qutritDepolarizingChannel(sim.originalState!, sim.depolarizeRate);
      return noisy;
    });
    const result = simulateTriorthogonalDistillation(sim.distillationM, inputs);
    setSim(prev => ({ ...prev, distillationResult: result, step: 4 }));
  }, [sim.originalState, sim.distillationM, sim.depolarizeRate]);

  const runPhaseGate = useCallback(() => {
    if (!sim.originalState) return;
    const gate = suftPhaseGate(Math.PI / 4);
    const unitary = isUnitaryQutrit(gate);
    const applied = applyGate(gate, sim.originalState);
    setSim(prev => ({ ...prev, phaseGateResult: { unitary, appliedState: applied }, step: 5 }));
  }, [sim.originalState]);

  const codeParams = codeDistance(sim.distillationM);

  return (
    <div className="space-y-6">
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <Card className="p-4">
          <h3 className="text-sm font-semibold mb-3 flex items-center gap-2">
            <Atom className="w-4 h-4 text-primary" />
            State Preparation
          </h3>
          <div className="space-y-3">
            <div>
              <label className="text-xs text-muted-foreground mb-1 block">Basis State</label>
              <Select
                value={sim.basisChoice}
                onValueChange={(v) => { resetSim(); setSim(prev => ({ ...prev, basisChoice: v as any })); }}
              >
                <SelectTrigger data-testid="select-basis-state" className="text-xs">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="-1">|-1&#x27E9; Moon.Ra (past)</SelectItem>
                  <SelectItem value="0">|0&#x27E9; Amun.Ra (present)</SelectItem>
                  <SelectItem value="+1">|+1&#x27E9; SUN.Ra (future)</SelectItem>
                  <SelectItem value="superposition">Equal Superposition</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div>
              <label className="text-xs text-muted-foreground mb-1 block">Error Channel</label>
              <Select
                value={sim.errorType}
                onValueChange={(v) => setSim(prev => ({ ...prev, errorType: v as ErrorType }))}
              >
                <SelectTrigger data-testid="select-error-type" className="text-xs">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="none">No Error</SelectItem>
                  <SelectItem value="phase">Phase Flip (Z-type)</SelectItem>
                  <SelectItem value="leak">Leakage</SelectItem>
                  <SelectItem value="depolarize">Depolarizing</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="flex flex-wrap gap-2">
              <Button size="sm" onClick={prepareState} data-testid="button-prepare-state">
                <FlaskConical className="w-3 h-3 mr-1" />
                Prepare
              </Button>
              <Button size="sm" onClick={applyError} disabled={!sim.originalState} variant="outline" data-testid="button-apply-error">
                <AlertTriangle className="w-3 h-3 mr-1" />
                Inject Error
              </Button>
              <Button size="sm" onClick={encodeAndCorrect} disabled={!sim.originalState} variant="outline" data-testid="button-encode-correct">
                <Shield className="w-3 h-3 mr-1" />
                Encode & Correct
              </Button>
              <Button size="sm" onClick={runPhaseGate} disabled={!sim.originalState} variant="outline" data-testid="button-phase-gate">
                <Zap className="w-3 h-3 mr-1" />
                SUFT Phase Gate
              </Button>
              <Button size="sm" onClick={resetSim} variant="ghost" data-testid="button-reset-sim">
                <RotateCcw className="w-3 h-3 mr-1" />
                Reset
              </Button>
            </div>
          </div>
        </Card>

        <Card className="p-4">
          <h3 className="text-sm font-semibold mb-3 flex items-center gap-2">
            <Activity className="w-4 h-4 text-primary" />
            State Vector
          </h3>
          {sim.originalState ? (
            <div className="space-y-2">
              <div className="font-mono text-xs space-y-1">
                <div className="flex items-center gap-2" data-testid="text-amplitude-0">
                  <Badge variant="outline" className="text-[10px]">|-1&#x27E9;</Badge>
                  <span>{formatComplex(sim.originalState[0])}</span>
                  <span className="text-muted-foreground">P={formatProb(cxMagSq(sim.originalState[0]))}</span>
                </div>
                <div className="flex items-center gap-2" data-testid="text-amplitude-1">
                  <Badge variant="outline" className="text-[10px]">|0&#x27E9;</Badge>
                  <span>{formatComplex(sim.originalState[1])}</span>
                  <span className="text-muted-foreground">P={formatProb(cxMagSq(sim.originalState[1]))}</span>
                </div>
                <div className="flex items-center gap-2" data-testid="text-amplitude-2">
                  <Badge variant="outline" className="text-[10px]">|+1&#x27E9;</Badge>
                  <span>{formatComplex(sim.originalState[2])}</span>
                  <span className="text-muted-foreground">P={formatProb(cxMagSq(sim.originalState[2]))}</span>
                </div>
              </div>
              {sim.errorState && (
                <div className="border-t pt-2 mt-2">
                  <p className="text-xs text-muted-foreground mb-1">After {sim.errorType} error:</p>
                  <div className="font-mono text-xs space-y-1">
                    {sim.errorState.map((a: { re: number; im: number }, i: number) => (
                      <div key={i} className="flex items-center gap-2">
                        <Badge variant="secondary" className="text-[10px]">{["|−1⟩", "|0⟩", "|+1⟩"][i]}</Badge>
                        <span>{formatComplex(a)}</span>
                      </div>
                    ))}
                  </div>
                  <p className="text-xs mt-1" data-testid="text-fidelity-before">
                    Fidelity: <span className={sim.fidelityBeforeCorrection! > 0.99 ? "text-green-600 dark:text-green-400" : "text-amber-600 dark:text-amber-400"}>{(sim.fidelityBeforeCorrection! * 100).toFixed(4)}%</span>
                  </p>
                </div>
              )}
            </div>
          ) : (
            <p className="text-xs text-muted-foreground">Select a basis state and click Prepare</p>
          )}
        </Card>
      </div>

      {sim.syndrome !== null && (
        <Card className="p-4">
          <h3 className="text-sm font-semibold mb-3 flex items-center gap-2">
            <Shield className="w-4 h-4 text-primary" />
            [[3,1,2]]&#x2083; Stabilizer Correction
          </h3>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div>
              <p className="text-xs text-muted-foreground mb-1">Syndrome Measurement</p>
              <div className="font-mono text-xs" data-testid="text-syndrome">
                <div>s&#x2081; = {sim.syndrome[0].toFixed(6)}</div>
                <div>s&#x2082; = {sim.syndrome[1].toFixed(6)}</div>
              </div>
            </div>
            <div>
              <p className="text-xs text-muted-foreground mb-1">Error Detection</p>
              <div className="flex items-center gap-2" data-testid="text-error-position">
                {sim.errorPosition !== null ? (
                  <>
                    <XCircle className="w-4 h-4 text-amber-500" />
                    <span className="text-xs">Error at block {sim.errorPosition}</span>
                  </>
                ) : (
                  <>
                    <CheckCircle2 className="w-4 h-4 text-green-500" />
                    <span className="text-xs">No error detected</span>
                  </>
                )}
              </div>
            </div>
            <div>
              <p className="text-xs text-muted-foreground mb-1">Post-Correction Fidelity</p>
              <div className="flex items-center gap-2" data-testid="text-fidelity-after">
                <span className="font-mono text-xs font-semibold">
                  {(sim.fidelityAfterCorrection! * 100).toFixed(4)}%
                </span>
                {sim.fidelityAfterCorrection! > 0.99 && (
                  <Badge variant="outline" className="text-[10px] text-green-600 dark:text-green-400 border-green-600/30">Recovered</Badge>
                )}
              </div>
            </div>
          </div>
        </Card>
      )}

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <Card className="p-4">
          <h3 className="text-sm font-semibold mb-3 flex items-center gap-2">
            <Zap className="w-4 h-4 text-primary" />
            Magic State Distillation
          </h3>
          <div className="space-y-3">
            <div className="flex flex-wrap items-end gap-3">
              <div>
                <label className="text-xs text-muted-foreground mb-1 block">Code parameter m</label>
                <Select
                  value={String(sim.distillationM)}
                  onValueChange={(v) => setSim(prev => ({ ...prev, distillationM: parseInt(v) }))}
                >
                  <SelectTrigger className="w-20 text-xs" data-testid="select-distillation-m">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {[1, 2, 3, 4, 5].map(m => (
                      <SelectItem key={m} value={String(m)}>{m}</SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <div>
                <label className="text-xs text-muted-foreground mb-1 block">Noise rate</label>
                <Select
                  value={String(sim.depolarizeRate)}
                  onValueChange={(v) => setSim(prev => ({ ...prev, depolarizeRate: parseFloat(v) }))}
                >
                  <SelectTrigger className="w-24 text-xs" data-testid="select-noise-rate">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {[0.01, 0.05, 0.1, 0.2, 0.3].map(r => (
                      <SelectItem key={r} value={String(r)}>{(r * 100).toFixed(0)}%</SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <Button size="sm" onClick={runDistillation} disabled={!sim.originalState} data-testid="button-run-distillation">
                <Play className="w-3 h-3 mr-1" />
                Distill
              </Button>
            </div>
            <div className="text-xs text-muted-foreground">
              Code: [[{codeParams.n},{codeParams.k},{codeParams.d}]]&#x2083; | Overhead: {codeParams.overhead.toFixed(2)}x | SUFT scale: {SUFT_RADIUS}
            </div>
            {sim.distillationResult && (
              <div className="border-t pt-2 space-y-1" data-testid="text-distillation-result">
                <div className="font-mono text-xs">
                  Yield &#x3B3;: {sim.distillationResult.yieldGamma.toFixed(4)} bits
                </div>
                <div className="font-mono text-xs">
                  Distilled state: [{sim.distillationResult.distilled.map((a: { re: number; im: number }) => formatComplex(a)).join(", ")}]
                </div>
                <div className="font-mono text-xs">
                  Fidelity vs original: {sim.originalState ? (qutritFidelity(sim.distillationResult.distilled, sim.originalState) * 100).toFixed(4) : "N/A"}%
                </div>
              </div>
            )}
          </div>
        </Card>

        {sim.phaseGateResult && (
          <Card className="p-4">
            <h3 className="text-sm font-semibold mb-3 flex items-center gap-2">
              <Atom className="w-4 h-4 text-primary" />
              SUFT Phase Gate (&#x03C6;/13 coupling)
            </h3>
            <div className="space-y-2">
              <div className="flex items-center gap-2" data-testid="text-gate-unitarity">
                {sim.phaseGateResult.unitary ? (
                  <>
                    <CheckCircle2 className="w-4 h-4 text-green-500" />
                    <span className="text-xs">Gate is unitary (verified)</span>
                  </>
                ) : (
                  <>
                    <XCircle className="w-4 h-4 text-red-500" />
                    <span className="text-xs">Unitarity check failed</span>
                  </>
                )}
              </div>
              <div className="font-mono text-xs space-y-1">
                <p className="text-muted-foreground mb-1">Post-gate state:</p>
                {sim.phaseGateResult.appliedState.map((a: { re: number; im: number }, i: number) => (
                  <div key={i} className="flex items-center gap-2">
                    <Badge variant="outline" className="text-[10px]">{["|−1⟩", "|0⟩", "|+1⟩"][i]}</Badge>
                    <span>{formatComplex(a)}</span>
                    <span className="text-muted-foreground">P={formatProb(cxMagSq(a))}</span>
                  </div>
                ))}
              </div>
              <div className="text-xs text-muted-foreground">
                Gate: e^(i&#x03C6;/{SUFT_RADIUS} &#x03B8;) phase on SUFT branches
              </div>
            </div>
          </Card>
        )}
      </div>
    </div>
  );
}

interface FIPSCheckItem {
  id: string;
  category: string;
  requirement: string;
  status: "implemented" | "prototype" | "planned";
  detail: string;
}

const fipsChecklist: FIPSCheckItem[] = [
  { id: "cmvp-1", category: "Cryptographic Boundary", requirement: "CMVP Module Boundary Definition", status: "implemented", detail: "AES-256-GCM + SHA-384 boundary defined in kernel crypto module. Hardware/software distinction documented." },
  { id: "cmvp-2", category: "Cryptographic Boundary", requirement: "Approved Algorithms (CNSA 2.0)", status: "implemented", detail: "ML-KEM-1024, ML-DSA-87, AES-256-GCM, SHA-384, XMSS/LMS all implemented in Rust kernel." },
  { id: "cmvp-3", category: "Cryptographic Boundary", requirement: "Key Management Lifecycle", status: "implemented", detail: "TL-KEM key generation, encapsulation, decapsulation with zeroization. HKDF-based key derivation." },
  { id: "ft-1", category: "Fault Tolerance", requirement: "Error Detection via Stabilizer Codes", status: "implemented", detail: "[[3,1,2]]_3 stabilizer code with operational syndrome extraction (QSyndrome 0xA4) and correction (QCorrect 0xA5) on VM registers. 14 kernel tests passing." },
  { id: "ft-2", category: "Fault Tolerance", requirement: "Magic State Distillation Protocol", status: "implemented", detail: "Triorthogonal [[9m-k,k,2]]_3 codes with SUFT-scaled distillation. QDistill (0xA6) opcode and kernel benchmark binaries implemented." },
  { id: "ft-3", category: "Fault Tolerance", requirement: "Qutrit Depolarizing Channel Simulation", status: "implemented", detail: "Configurable error rates with normalization-preserving noise. QErrInject (0xAC) opcode for controlled error injection on register state." },
  { id: "ft-4", category: "Fault Tolerance", requirement: "HPTP Qutrit Jitter Correction", status: "implemented", detail: "Treats 3 consecutive femtosecond timestamps as qutrit-encoded triple. Syndrome-based median correction for outlier detection. Windowed processing support." },
  { id: "qr-1", category: "Quantum Resistance", requirement: "Post-Quantum Key Exchange (TL-KEM)", status: "implemented", detail: "Ternary Lattice KEM with CNSA 2.0 parameter sets. Hardware opcode support in VM ISA v2.1." },
  { id: "qr-2", category: "Quantum Resistance", requirement: "Post-Quantum Signatures (TL-DSA)", status: "implemented", detail: "Ternary Lattice Digital Signature Algorithm. Deterministic signing with SHA-384 internal hash." },
  { id: "qr-3", category: "Quantum Resistance", requirement: "Hash-Based Signatures (XMSS/LMS)", status: "implemented", detail: "Stateful hash-based signatures for firmware attestation. Forward-secure key management." },
  { id: "inv-1", category: "Invariant Enforcement", requirement: "Hamiltonian Energy Conservation", status: "implemented", detail: "Register state energy H = sum(reg_i^2) mod 312 preserved across opcode transitions. Drift tolerance T(7)=13. Validated in hamiltonian-constraints module." },
  { id: "inv-2", category: "Invariant Enforcement", requirement: "Noether Symmetry Checks", status: "implemented", detail: "Three operational kernel checks: ternary gauge sum invariant, reparametrization energy (SUFT phi ratio), periodicity (mod 364). Post-correction verification in noether_checks.rs. 6 kernel tests passing." },
  { id: "inv-3", category: "Invariant Enforcement", requirement: "GF(3) Parity Conservation", status: "implemented", detail: "Per-bank ternary parity (sum mod 3) conservation across opcode execution chains. Enforced via computeBankTernaryParity." },
  { id: "inv-4", category: "Invariant Enforcement", requirement: "Qudit Generalized Correction", status: "implemented", detail: "QUDIT_CORRECT_D opcode extends [[3,1,2]]_3 to arbitrary d>=3 (up to d=13). Syndrome-based averaging with per-block normalization. Noether invariants verified post-correction." },
  { id: "exp-1", category: "Export Control", requirement: "ECCN 5D002 Classification", status: "implemented", detail: "Documented in EXPORT-CONTROL.md. Wassenaar Category 5 Part 2 with Canadian ITAR compliance." },
  { id: "exp-2", category: "Export Control", requirement: "GDPR/PIPEDA Data Handling", status: "implemented", detail: "Cookie consent, data minimization, right-to-erasure via API endpoints. Privacy policy dynamic rendering." },
];

function FIPSPathTab() {
  const statusColors = {
    implemented: "text-green-600 dark:text-green-400",
    prototype: "text-amber-600 dark:text-amber-400",
    planned: "text-muted-foreground",
  };
  const statusIcons = {
    implemented: <CheckCircle2 className="w-3.5 h-3.5 text-green-500" />,
    prototype: <FlaskConical className="w-3.5 h-3.5 text-amber-500" />,
    planned: <AlertTriangle className="w-3.5 h-3.5 text-muted-foreground" />,
  };

  const categories = [...new Set(fipsChecklist.map(c => c.category))];
  const implementedCount = fipsChecklist.filter(c => c.status === "implemented").length;
  const prototypeCount = fipsChecklist.filter(c => c.status === "prototype").length;
  const progress = ((implementedCount + prototypeCount * 0.5) / fipsChecklist.length) * 100;

  return (
    <div className="space-y-6">
      <Card className="p-4">
        <div className="flex flex-wrap items-center justify-between gap-4 mb-4">
          <div>
            <h3 className="text-sm font-semibold flex items-center gap-2">
              <Lock className="w-4 h-4 text-primary" />
              FIPS 140-3 Compliance Readiness
            </h3>
            <p className="text-xs text-muted-foreground mt-1">
              Mapping stabilizer codes and quantum-resistant primitives to CMVP boundary requirements
            </p>
          </div>
          <div className="flex items-center gap-3">
            <div className="text-right">
              <span className="text-lg font-bold" data-testid="text-fips-progress">{progress.toFixed(0)}%</span>
              <p className="text-[10px] text-muted-foreground">readiness</p>
            </div>
            <div className="w-32">
              <Progress value={progress} className="h-2" />
            </div>
          </div>
        </div>
        <div className="flex flex-wrap gap-4 text-xs">
          <div className="flex items-center gap-1.5">
            <CheckCircle2 className="w-3 h-3 text-green-500" />
            <span>{implementedCount} Implemented</span>
          </div>
          <div className="flex items-center gap-1.5">
            <FlaskConical className="w-3 h-3 text-amber-500" />
            <span>{prototypeCount} Prototype</span>
          </div>
          <div className="flex items-center gap-1.5">
            <AlertTriangle className="w-3 h-3 text-muted-foreground" />
            <span>{fipsChecklist.length - implementedCount - prototypeCount} Planned</span>
          </div>
        </div>
      </Card>

      {categories.map(category => (
        <Card key={category} className="p-4">
          <h4 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground mb-3">{category}</h4>
          <div className="space-y-2">
            {fipsChecklist.filter(c => c.category === category).map(item => (
              <div key={item.id} className="flex items-start gap-3 p-2 rounded-md bg-muted/30" data-testid={`fips-item-${item.id}`}>
                <div className="mt-0.5 flex-shrink-0">{statusIcons[item.status]}</div>
                <div className="min-w-0 flex-1">
                  <div className="flex flex-wrap items-center gap-2 mb-0.5">
                    <span className="text-xs font-medium">{item.requirement}</span>
                    <Badge variant="outline" className={`text-[10px] ${statusColors[item.status]}`}>{item.status}</Badge>
                  </div>
                  <p className="text-[11px] text-muted-foreground">{item.detail}</p>
                </div>
              </div>
            ))}
          </div>
        </Card>
      ))}

      <Card className="p-4">
        <h4 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground mb-3">CMVP Module Boundary</h4>
        <div className="font-mono text-[11px] leading-relaxed bg-muted/30 rounded-md p-3 overflow-x-auto">
          <pre className="whitespace-pre">{`┌─────────────────────────────────────────────────────────┐
│                    FIPS 140-3 Module                     │
│                   (Security Level 1)                     │
├─────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌───────────────┐  │
│  │  AES-256-GCM │  │   SHA-384    │  │   HKDF-384    │  │
│  │  (Approved)  │  │  (Approved)  │  │  (Approved)   │  │
│  └──────────────┘  └──────────────┘  └───────────────┘  │
│  ┌──────────────┐  ┌──────────────┐  ┌───────────────┐  │
│  │  ML-KEM-1024 │  │  ML-DSA-87   │  │   XMSS/LMS   │  │
│  │  (TL-KEM)    │  │  (TL-DSA)    │  │ (Hash-based)  │  │
│  └──────────────┘  └──────────────┘  └───────────────┘  │
├─────────────────────────────────────────────────────────┤
│  Quantum Resilience Layer (Operational — 29 tests)       │
│  ┌──────────────┐  ┌──────────────┐  ┌───────────────┐  │
│  │ [[3,1,2]]_3  │  │ Magic State  │  │  Hamiltonian  │  │
│  │ Stabilizer   │  │ Distillation │  │  Constraints  │  │
│  │ QCorrect     │  │ QDistill     │  │  Noether Chk  │  │
│  └──────────────┘  └──────────────┘  └───────────────┘  │
│  ┌──────────────┐  ┌──────────────┐  ┌───────────────┐  │
│  │ HPTP Jitter  │  │ Qudit d≥3   │  │ QFT Bench    │  │
│  │ Correction   │  │ Correction   │  │ (0xAF)       │  │
│  └──────────────┘  └──────────────┘  └───────────────┘  │
├─────────────────────────────────────────────────────────┤
│  Key: T(7)=13 | T(8)=24 | Φ = 13/28 | mod 312 | mod 3 │
└─────────────────────────────────────────────────────────┘`}</pre>
        </div>
      </Card>
    </div>
  );
}

interface BenchmarkResult {
  name: string;
  time: number;
  result: Record<string, string | number | boolean>;
}

function VariationalBenchmarksTab() {
  const [results, setResults] = useState<BenchmarkResult[]>([]);
  const [running, setRunning] = useState(false);

  const runBenchmarks = useCallback(() => {
    setRunning(true);
    setResults([]);

    const benchmarks: BenchmarkResult[] = [];

    {
      const t0 = performance.now();
      const seq = generateTribonacci(30);
      const action = tribonacciActionDeviation(seq);
      const fitness = variationalFitness(seq);
      const canonical = verifyCanonicalTribonacci();
      const ratios = tribonacciRatioConvergence(seq);
      const lastRatio = ratios[ratios.length - 1];
      const t1 = performance.now();
      benchmarks.push({
        name: "Tribonacci Variational Action",
        time: t1 - t0,
        result: {
          "Sequence length": 30,
          "Action deviation": action,
          "Variational fitness": fitness.toFixed(6),
          "Canonical valid": canonical.valid,
          "Ratio convergence (n=29)": lastRatio?.ratio.toFixed(10) ?? "N/A",
          "Tau error": lastRatio?.error.toExponential(4) ?? "N/A",
        },
      });
    }

    {
      const t0 = performance.now();
      const perturbed = generateTribonacci(20);
      perturbed[10] += 5;
      perturbed[15] -= 3;
      const residuals = tribonacciResiduals(perturbed);
      const perturbedAction = tribonacciActionDeviation(perturbed);
      const perturbedFitness = variationalFitness(perturbed);
      const t1 = performance.now();
      benchmarks.push({
        name: "Perturbed Sequence Recovery",
        time: t1 - t0,
        result: {
          "Original action": 0,
          "Perturbed action": perturbedAction,
          "Fitness drop": perturbedFitness.toFixed(6),
          "Violations detected": residuals.filter((r: { penalty: number }) => r.penalty > 0).length,
          "Max residual": Math.max(...residuals.map((r: { residual: number }) => Math.abs(r.residual))),
        },
      });
    }

    {
      const t0 = performance.now();
      const registers = Array(27).fill(0).map((_, i) => ((i * 7 + 3) % 9) - 4);
      const state0 = computeHamiltonianState(registers);
      const mutated = [...registers];
      mutated[5] += 1;
      mutated[14] -= 1;
      const check = checkHamiltonianConstraint(registers, mutated);
      const snapshots = [registers];
      for (let i = 0; i < 20; i++) {
        const prev = snapshots[snapshots.length - 1];
        const next = prev.map((v, j) => v + (j % 3 === 0 ? 1 : j % 3 === 1 ? -1 : 0));
        snapshots.push(next);
      }
      const seqValidation = validateOpcodeSequence(snapshots);
      const parities = [0, 1, 2].map(b => computeBankTernaryParity(registers, b as 0 | 1 | 2));
      const t1 = performance.now();
      benchmarks.push({
        name: "Hamiltonian Constraint Chain",
        time: t1 - t0,
        result: {
          "Initial energy H": state0.energy,
          "Constraint surface": state0.constraintValue.toFixed(4),
          "Single-step drift": check.drift,
          "Single-step valid": check.valid,
          "Chain steps": seqValidation.totalSteps,
          "Chain violations": seqValidation.violations,
          "Max chain drift": seqValidation.maxDrift,
          "Bank parities (GF3)": `[${parities.join(", ")}]`,
        },
      });
    }

    {
      const t0 = performance.now();
      const qFT_trials = 100;
      let totalFidelity = 0;
      let corrections = 0;
      for (let i = 0; i < qFT_trials; i++) {
        const basis = suftBasisState(([- 1, 0, 1] as const)[i % 3]);
        const noisy = qutritDepolarizingChannel(basis, 0.1);
        const encoded = encodeQutritStabilizer(basis);
        encoded[1] = noisy;
        const { corrected, errorPosition } = correctQutritStabilizer(encoded);
        const fid = qutritFidelity(corrected, basis);
        totalFidelity += fid;
        if (errorPosition !== null) corrections++;
      }
      const t1 = performance.now();
      benchmarks.push({
        name: "Stabilizer Code Throughput",
        time: t1 - t0,
        result: {
          "Trials": qFT_trials,
          "Avg fidelity": ((totalFidelity / qFT_trials) * 100).toFixed(4) + "%",
          "Corrections applied": corrections,
          "Error rate": "10%",
          "Code": "[[3,1,2]]_3",
          "Throughput": (qFT_trials / ((t1 - t0) / 1000)).toFixed(0) + " ops/sec",
        },
      });
    }

    {
      const t0 = performance.now();
      const mValues = [1, 2, 3, 4, 5];
      const distillResults = mValues.map(m => {
        const params = codeDistance(m);
        const inputs = Array(m * 3).fill(null).map(() =>
          qutritDepolarizingChannel(suftBasisState(0), 0.15)
        );
        const { yieldGamma } = simulateTriorthogonalDistillation(m, inputs);
        return { m, ...params, yieldGamma: yieldGamma.toFixed(4) };
      });
      const t1 = performance.now();
      benchmarks.push({
        name: "Distillation Scaling (QVQE)",
        time: t1 - t0,
        result: Object.fromEntries(
          distillResults.map(d => [`m=${d.m}`, `[[${d.n},${d.k},${d.d}]] gamma=${d.yieldGamma} overhead=${d.overhead.toFixed(2)}x`])
        ),
      });
    }

    {
      const t0 = performance.now();
      const gate = suftPhaseGate(Math.PI / 4);
      const unitaryCheck = isUnitaryQutrit(gate);
      let state: QutritState = suftBasisState(0);
      const traceNorms: number[] = [];
      for (let step = 0; step < 50; step++) {
        state = applyGate(gate, state);
        const probs = bornProbabilities(state);
        traceNorms.push(probs[0] + probs[1] + probs[2]);
      }
      const maxDeviation = Math.max(...traceNorms.map(n => Math.abs(n - 1)));
      const t1 = performance.now();
      benchmarks.push({
        name: "SUFT Phase Gate Stability (QAOA analog)",
        time: t1 - t0,
        result: {
          "Gate unitary": unitaryCheck,
          "Evolution steps": 50,
          "Max trace deviation": maxDeviation.toExponential(4),
          "SUFT coupling": `phi/${SUFT_RADIUS}`,
          "Mass-shell ratio": MASS_SHELL_RATIO.toFixed(6),
          "Cosmic circumference": SUFT_COSMIC_CIRCUMFERENCE,
        },
      });
    }

    {
      const ftCycles = 1000;
      const ftTrials = 5;
      const trialTimes: number[] = [];
      let totalCorrections = 0;
      let totalFid = 0;

      for (let trial = 0; trial < ftTrials; trial++) {
        const t0 = performance.now();
        for (let c = 0; c < ftCycles; c++) {
          const basisIdx = ([-1, 0, 1] as const)[c % 3];
          const logical = suftBasisState(basisIdx);
          const noisy = qutritDepolarizingChannel(logical, 0.1);
          const encoded = encodeQutritStabilizer(logical);
          encoded[Math.floor(Math.random() * 3)] = noisy;
          const { corrected, errorPosition } = correctQutritStabilizer(encoded);
          const fid = qutritFidelity(corrected, logical);
          totalFid += fid;
          if (errorPosition !== null) totalCorrections++;
        }
        const t1 = performance.now();
        trialTimes.push(t1 - t0);
      }

      const meanMs = trialTimes.reduce((a, b) => a + b, 0) / ftTrials;
      const stdMs = Math.sqrt(trialTimes.map((t: number) => (t - meanMs) ** 2).reduce((a: number, b: number) => a + b, 0) / ftTrials);
      const opsPerSec = Math.round((ftCycles * ftTrials) / (trialTimes.reduce((a: number, b: number) => a + b, 0) / 1000));
      const avgFid = totalFid / (ftCycles * ftTrials);

      benchmarks.push({
        name: "Qutrit FT Cycle Benchmark (QFTB 0xAF)",
        time: meanMs,
        result: {
          "Opcode": "QFTB (0xAF)",
          "Cycles/trial": ftCycles,
          "Trials": ftTrials,
          "Mean time": meanMs.toFixed(2) + " ms",
          "Std dev": stdMs.toFixed(2) + " ms",
          "Ops/sec": opsPerSec.toLocaleString(),
          "Avg fidelity": (avgFid * 100).toFixed(4) + "%",
          "Corrections": totalCorrections,
          "Code": "[[3,1,2]]_3 stabilizer",
          "Cycle": "Encode\u2192Error\u2192Syndrome\u2192Correct",
        },
      });
    }

    setResults(benchmarks);
    setRunning(false);
  }, []);

  const totalTime = results.reduce((s, r) => s + r.time, 0);

  return (
    <div className="space-y-6">
      <Card className="p-4">
        <div className="flex flex-wrap items-center justify-between gap-4 mb-4">
          <div>
            <h3 className="text-sm font-semibold flex items-center gap-2">
              <BarChart3 className="w-4 h-4 text-primary" />
              Variational Quantum Optimization Benchmarks
            </h3>
            <p className="text-xs text-muted-foreground mt-1">
              QVQE/QAOA-inspired benchmarks on the ternary VM using SUFT variational mechanics
            </p>
          </div>
          <div className="flex items-center gap-3">
            <Button onClick={runBenchmarks} disabled={running} data-testid="button-run-benchmarks">
              <Play className="w-3 h-3 mr-1" />
              {running ? "Running..." : "Run All Benchmarks"}
            </Button>
          </div>
        </div>
        {results.length > 0 && (
          <div className="flex flex-wrap gap-4 text-xs">
            <div data-testid="text-benchmark-summary">
              <span className="text-muted-foreground">Total: </span>
              <span className="font-mono font-semibold">{totalTime.toFixed(2)}ms</span>
            </div>
            <div>
              <span className="text-muted-foreground">Benchmarks: </span>
              <span className="font-mono font-semibold">{results.length}</span>
            </div>
            <div>
              <span className="text-muted-foreground">SUFT Constants: </span>
              <span className="font-mono">T(7)={SUFT_RADIUS} | {SUFT_LUNAR_HARMONIC} | {SUFT_COSMIC_CIRCUMFERENCE}</span>
            </div>
          </div>
        )}
      </Card>

      {results.map((bench, idx) => (
        <Card key={idx} className="p-4" data-testid={`benchmark-result-${idx}`}>
          <div className="flex flex-wrap items-center justify-between gap-2 mb-3">
            <h4 className="text-xs font-semibold">{bench.name}</h4>
            <Badge variant="outline" className="text-[10px] font-mono">{bench.time.toFixed(2)}ms</Badge>
          </div>
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-x-4 gap-y-1">
            {Object.entries(bench.result).map(([key, value]) => (
              <div key={key} className="flex items-baseline gap-1.5 text-xs py-0.5">
                <span className="text-muted-foreground whitespace-nowrap">{key}:</span>
                <span className="font-mono font-medium truncate" title={String(value)}>
                  {typeof value === "boolean" ? (
                    value ? <CheckCircle2 className="w-3 h-3 text-green-500 inline" /> : <XCircle className="w-3 h-3 text-red-500 inline" />
                  ) : String(value)}
                </span>
              </div>
            ))}
          </div>
        </Card>
      ))}

      {results.length === 0 && !running && (
        <Card className="p-8">
          <div className="text-center text-muted-foreground">
            <BarChart3 className="w-8 h-8 mx-auto mb-2 opacity-50" />
            <p className="text-sm">Click "Run All Benchmarks" to execute QVQE/QAOA-inspired tests</p>
            <p className="text-xs mt-1">
              Tests Tribonacci variational action, Hamiltonian constraints, stabilizer throughput, distillation scaling, and SUFT phase gate stability
            </p>
          </div>
        </Card>
      )}
    </div>
  );
}

export default function QuantumSim() {
  return (
    <div className="min-h-screen bg-background">
      <div className="bg-gradient-to-b from-blue-950 to-slate-900 text-white py-12 px-4">
        <div className="max-w-6xl mx-auto">
          <div className="flex items-center gap-2 text-sm text-slate-400 mb-4" data-testid="text-breadcrumb">
            <Link href="/" className="hover-elevate rounded px-1">Home</Link>
            <ChevronRight className="w-3 h-3" />
            <span>Quantum Simulator</span>
          </div>

          <div className="flex flex-wrap items-start gap-4 justify-between">
            <div>
              <div className="flex items-center gap-3 mb-2">
                <Atom className="w-8 h-8 text-blue-400" />
                <h1 className="text-3xl font-bold tracking-tight" data-testid="text-quantum-title">
                  Quantum-Ternary Simulator
                </h1>
              </div>
              <p className="text-slate-300 max-w-2xl text-sm leading-relaxed">
                Interactive qutrit fault-tolerance simulation, FIPS 140-3 compliance mapping,
                and variational quantum optimization benchmarks — powered by the {PLATFORM.VM_OPCODES}-opcode ISA {PLATFORM.VM_ISA_VERSION}
                Quantum-Ternary category (0xA0-0xAF) on the 27-register ternary VM.
              </p>
              <div className="flex flex-wrap gap-2 mt-4">
                <Badge variant="outline" className="text-[10px] border-blue-400/40 text-blue-300">ISA v2.1 (0xA0-0xAF)</Badge>
                <Badge variant="outline" className="text-[10px] border-blue-400/40 text-blue-300">SUFT-Coupled</Badge>
                <Badge variant="outline" className="text-[10px] border-blue-400/40 text-blue-300">[[3,1,2]]&#x2083; Stabilizer</Badge>
                <Badge variant="outline" className="text-[10px] border-green-400/40 text-green-300">Operational Kernel</Badge>
                <Badge variant="outline" className="text-[10px] border-blue-400/40 text-blue-300">Noether Invariants</Badge>
                <Badge variant="outline" className="text-[10px] border-blue-400/40 text-blue-300">CNSA 2.0</Badge>
                <Badge variant="outline" className="text-[10px] border-blue-400/40 text-blue-300">QVQE/QAOA</Badge>
                <Badge variant="outline" className="text-[10px] border-blue-400/40 text-blue-300">Patent Pending</Badge>
              </div>
            </div>
            <Link href="/vm-demo">
              <Button variant="outline" className="bg-white/5 backdrop-blur-sm border-white/20 text-white" data-testid="link-vm-demo">
                <Atom className="w-3 h-3 mr-1" />
                Ternary VM Demo
              </Button>
            </Link>
          </div>
        </div>
      </div>

      <div className="max-w-6xl mx-auto px-4 py-8">
        <Tabs defaultValue="ft-mode" className="w-full">
          <TabsList className="w-full justify-start mb-6 flex-wrap">
            <TabsTrigger value="ft-mode" data-testid="tab-ft-mode" className="text-xs gap-1.5">
              <Shield className="w-3 h-3" />
              Qutrit FT Mode
            </TabsTrigger>
            <TabsTrigger value="fips-path" data-testid="tab-fips-path" className="text-xs gap-1.5">
              <FileCheck className="w-3 h-3" />
              FIPS 140-3 Path
            </TabsTrigger>
            <TabsTrigger value="benchmarks" data-testid="tab-benchmarks" className="text-xs gap-1.5">
              <BarChart3 className="w-3 h-3" />
              Variational Benchmarks
            </TabsTrigger>
          </TabsList>

          <TabsContent value="ft-mode">
            <QutritFTTab />
          </TabsContent>

          <TabsContent value="fips-path">
            <FIPSPathTab />
          </TabsContent>

          <TabsContent value="benchmarks">
            <VariationalBenchmarksTab />
          </TabsContent>
        </Tabs>
      </div>

      <div className="border-t bg-muted/30 py-6 px-4">
        <div className="max-w-6xl mx-auto text-center text-xs text-muted-foreground">
          <p>GEOMETRIA PRIMUS. TEMPORIS ARCHITECTURA ABSOLUTA.</p>
          <p className="mt-1">Patent(s) Pending. All Rights Reserved and Preserved. Capomastro Holdings Ltd. 2026</p>
        </div>
      </div>
    </div>
  );
}
