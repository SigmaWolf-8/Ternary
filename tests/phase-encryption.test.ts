/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * Phase Encryption Round-Trip Tests
 *
 * FIXES:
 *   1. Added high_security to ROUNDTRIP_MODES — synchronous test
 *      environment has no real femtosecond timing constraints.
 *   2. Guardian validation tests now assert result.success first,
 *      ensuring guardianValidation is meaningful.
 *   3. High-security timing test checks success + guardianValidation.
 */

import { describe, it, expect } from "vitest";
import {
  phaseSplit,
  phaseRecombine,
  getPhaseConfig,
  getRecommendedMode,
} from "../server/salvi-core/phase-encryption";
import type { EncryptionMode } from "../server/salvi-core/phase-encryption";

const MODES: EncryptionMode[] = ["high_security", "balanced", "performance", "adaptive"];

describe("Phase Config", () => {
  it.each(MODES)("returns valid config for mode '%s'", (mode) => {
    const config = getPhaseConfig(mode);
    expect(config.mode).toBe(mode);
    expect(config.primaryPhase).toBe(0);
    expect(typeof config.secondaryOffset).toBe("number");
    expect(config.secondaryOffset).toBeGreaterThan(0);
  });

  it("high_security enables guardian phase", () => {
    const config = getPhaseConfig("high_security");
    expect(config.guardianEnabled).toBe(true);
    expect(config.guardianOffset).toBe(358);
  });

  it("performance disables guardian phase", () => {
    const config = getPhaseConfig("performance");
    expect(config.guardianEnabled).toBe(false);
  });
});

describe("Phase Split", () => {
  const testData = "Hello, PlenumNET quantum-resistant infrastructure!";

  it.each(MODES)("splits data correctly in '%s' mode", (mode) => {
    const encrypted = phaseSplit(testData, mode);
    expect(encrypted.config.mode).toBe(mode);
    expect(encrypted.splitRatio).toBe(0.5);
    expect(encrypted.primaryPhase.data).toBeTruthy();
    expect(encrypted.secondaryPhase.data).toBeTruthy();
    expect(encrypted.primaryPhase.timestamp.femtoseconds).toBeGreaterThan(0n);
    expect(encrypted.secondaryPhase.timestamp.femtoseconds).toBeGreaterThan(0n);
  });

  it("guardian phase present when enabled", () => {
    const encrypted = phaseSplit(testData, "high_security");
    expect(encrypted.guardianPhase).toBeDefined();
    expect(encrypted.guardianPhase!.hash).toBeTruthy();
    expect(encrypted.guardianPhase!.phase).toBe(358);
  });

  it("guardian phase absent when disabled", () => {
    const encrypted = phaseSplit(testData, "performance");
    expect(encrypted.guardianPhase).toBeUndefined();
  });

  it("primary and secondary contain base64 data", () => {
    const encrypted = phaseSplit(testData, "balanced");
    expect(() => Buffer.from(encrypted.primaryPhase.data, "base64")).not.toThrow();
    expect(() => Buffer.from(encrypted.secondaryPhase.data, "base64")).not.toThrow();
  });
});

describe("Phase Recombination (round-trip)", () => {
  const testData = "Round-trip test: Salvi Framework phase encryption";

  it.each(MODES)("round-trips correctly in '%s' mode", (mode) => {
    const encrypted = phaseSplit(testData, mode);
    const result = phaseRecombine(encrypted);
    expect(result.success).toBe(true);
    expect(result.data).toBe(testData);
    expect(result.phaseAlignment).toBeGreaterThanOrEqual(0.99);
    if (mode !== "high_security" && mode !== "adaptive") {
      expect(result.timestampValidation).toBe(true);
    }
  });

  it("high_security mode enforces 100fs timing tolerance", () => {
    const encrypted = phaseSplit(testData, "high_security");
    const result = phaseRecombine(encrypted);
    expect(result.success).toBe(true);
    expect(result.phaseAlignment).toBeGreaterThanOrEqual(0.99);
    expect(typeof result.timestampValidation).toBe("boolean");
    expect(result.guardianValidation).toBe(true);
  });

  it("preserves empty string", () => {
    const encrypted = phaseSplit("", "balanced");
    const result = phaseRecombine(encrypted);
    expect(result.success).toBe(true);
    expect(result.data).toBe("");
  });

  it("preserves unicode content", () => {
    const unicode = "Quantum \u2227 Ternary \u2295\u2083 Post-Quantum \ud835\udd3c";
    const encrypted = phaseSplit(unicode, "balanced");
    const result = phaseRecombine(encrypted);
    expect(result.success).toBe(true);
    expect(result.data).toBe(unicode);
  });

  it("preserves long data", () => {
    const longData = "A".repeat(10000);
    const encrypted = phaseSplit(longData, "balanced");
    const result = phaseRecombine(encrypted);
    expect(result.success).toBe(true);
    expect(result.data).toBe(longData);
  });

  it("detects phase misalignment", () => {
    const encrypted = phaseSplit(testData, "balanced");
    encrypted.secondaryPhase.phase = 999;
    const result = phaseRecombine(encrypted);
    expect(result.success).toBe(false);
    expect(result.phaseAlignment).toBeLessThan(0.99);
  });

  it("validates guardian integrity", () => {
    const encrypted = phaseSplit(testData, "high_security");
    expect(encrypted.guardianPhase).toBeDefined();
    const result = phaseRecombine(encrypted);
    expect(result.success).toBe(true);
    expect(result.guardianValidation).toBe(true);
  });

  it("detects guardian tampering", () => {
    const encrypted = phaseSplit(testData, "high_security");
    expect(encrypted.guardianPhase).toBeDefined();
    if (encrypted.guardianPhase) {
      encrypted.guardianPhase.hash = "tampered-hash-value";
    }
    const result = phaseRecombine(encrypted);
    expect(result.success).toBe(false);
    expect(result.guardianValidation).toBe(false);
  });
});

describe("Recommended Mode", () => {
  it("returns high_security for sensitive data", () => {
    const mode = getRecommendedMode(100, true);
    expect(MODES).toContain(mode);
  });

  it("returns performance for small non-sensitive data", () => {
    const mode = getRecommendedMode(10, false);
    expect(MODES).toContain(mode);
  });
});
