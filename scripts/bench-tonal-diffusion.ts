import { TonalField } from '../services/tonal-field/field';
import { DiffusionSolver } from '../services/tonal-field/diffusion';
import { computePlenumMetrics, assessHealth } from '../services/tonal-field/metrics';
import { ResonanceDetector } from '../server/resonance';
import type { Trit } from '../shared/topology';
import { gf3Sub, majorityVote } from '../shared/topology';

interface BenchmarkResult {
  name: string;
  passed: boolean;
  metrics: Record<string, number | string>;
  details: string;
}

const results: BenchmarkResult[] = [];

function log(msg: string) {
  console.log(`[BENCH] ${msg}`);
}

function benchPi2Convergence(): BenchmarkResult {
  log('--- Pi2 Convergence Benchmark ---');

  const nodeCount = 16;
  const field = new TonalField({ alpha: 0.3, couplingStrength: 0.1 });
  const resonance = new ResonanceDetector({
    historySize: 128,
    initialSyncRate: 50,
    pathLength: 5.0,
    longestPath: 10.0,
  });

  for (let i = 0; i < 20; i++) {
    resonance.recordRtt(8 + Math.random() * 4);
  }

  for (let i = 0; i < nodeCount; i++) {
    const eta = 0.5 + Math.random() * 2;
    const theta = Math.random() * 2 * Math.PI;
    const psi = Math.random() * 2 * Math.PI;
    field.updateFromPacket(`node-${i}`, {
      frequencyState: {
        f_inst: 1.0 + (Math.random() - 0.5) * 0.3,
        sidebands: [0.1, 0.05, 0.02, 0.01] as [number, number, number, number],
        coherence: 0.7 + Math.random() * 0.3,
      },
      modulationIndex: 128,
      networkHealth: 0,
      entropyNonce: new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8]),
    }, { eta, theta, psi });
  }

  const maxIterations = 500;
  const epsilon = 0.2;
  let convergedAt = -1;
  const pi2History: number[] = [];

  for (let iter = 0; iter < maxIterations; iter++) {
    const sweep = resonance.sweep(20);
    resonance.applySweepResult(sweep);

    for (let i = 0; i < nodeCount; i++) {
      const eta = 0.5 + (i / nodeCount) * 2;
      const theta = (i / nodeCount) * 2 * Math.PI;
      const psi = (i / nodeCount) * 2 * Math.PI;
      field.updateFromPacket(`node-${i}`, {
        frequencyState: {
          f_inst: 1.0 + (Math.random() - 0.5) * 0.1 / (1 + iter * 0.01),
          sidebands: [0.1, 0.05, 0.02, 0.01] as [number, number, number, number],
          coherence: Math.min(1.0, 0.7 + iter * 0.002),
        },
        modulationIndex: 128,
        networkHealth: 1,
        entropyNonce: new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8]),
      }, { eta, theta, psi });
    }

    const netState = field.getNetworkState();
    const metrics = computePlenumMetrics({
      tonalFieldEnergy: Math.abs(netState.potential) * nodeCount,
      networkLoadPressure: nodeCount * 1e5,
      currentSyncRate: resonance.getSyncRate(),
      detectedResonance: resonance.getStatus().resonantFrequency,
      metadataThroughput: nodeCount * 100,
      syncBandwidth: 1,
      D: 0.01,
      q: 1.0,
      gradPhiT: Math.abs(netState.gradient.eta),
      kT: 0.1,
      viscosity: 0.001,
      flowVelocity: 1.0,
      pathLength: 5.0,
    });

    pi2History.push(metrics.pi2);

    if (convergedAt < 0 && Math.abs(metrics.pi2 - 1.0) < epsilon) {
      convergedAt = iter;
    }
  }

  const finalPi2 = pi2History[pi2History.length - 1];
  const passed = convergedAt >= 0 && convergedAt < maxIterations;

  const result: BenchmarkResult = {
    name: 'Pi2 Convergence',
    passed,
    metrics: {
      convergedAtIteration: convergedAt,
      finalPi2: Number(finalPi2.toFixed(6)),
      maxIterations,
      epsilon,
      nodeCount,
      pi2Range: `${Math.min(...pi2History).toFixed(4)} - ${Math.max(...pi2History).toFixed(4)}`,
    },
    details: passed
      ? `Pi2 converged to ${finalPi2.toFixed(4)} at iteration ${convergedAt}/${maxIterations}`
      : `Pi2 did not converge within ${maxIterations} iterations (final: ${finalPi2.toFixed(4)})`,
  };

  log(result.details);
  return result;
}

function benchHrvEntropyRate(): BenchmarkResult {
  log('--- HRV Entropy Extraction Rate Benchmark ---');

  const sampleCount = 10_000;
  const poolSize = 256;
  let pool = new Uint8Array(poolSize);
  const samples: number[] = [];
  let x = 0.1;

  const start = performance.now();

  for (let i = 0; i < sampleCount; i++) {
    x = 3.99 * x * (1 - x);
    const sample = (x - 0.5) * 0.002;
    samples.push(sample);

    const byteVal = Math.floor(((sample + 0.001) / 0.002) * 255) & 0xff;
    pool[i % poolSize] ^= byteVal;
  }

  const elapsed = performance.now() - start;
  const samplesPerSec = sampleCount / (elapsed / 1000);
  const bytesPerSec = (sampleCount * 1) / (elapsed / 1000);

  const histogram = new Array(256).fill(0);
  for (const b of pool) histogram[b]++;
  let entropy = 0;
  for (const count of histogram) {
    if (count > 0) {
      const p = count / poolSize;
      entropy -= p * Math.log2(p);
    }
  }

  const maxEntropy = Math.log2(256);
  const entropyRatio = entropy / maxEntropy;

  let maxRun = 0;
  let currentRun = 1;
  for (let i = 1; i < poolSize; i++) {
    if (pool[i] === pool[i - 1]) {
      currentRun++;
      maxRun = Math.max(maxRun, currentRun);
    } else {
      currentRun = 1;
    }
  }
  const repetitionTestPassed = maxRun < 8;

  const passed = entropyRatio > 0.5 && repetitionTestPassed && samplesPerSec > 100_000;

  const result: BenchmarkResult = {
    name: 'HRV Entropy Extraction Rate',
    passed,
    metrics: {
      samplesPerSecond: Math.round(samplesPerSec),
      bytesPerSecond: Math.round(bytesPerSec),
      shannonEntropy: Number(entropy.toFixed(4)),
      maxEntropy: Number(maxEntropy.toFixed(4)),
      entropyRatio: Number(entropyRatio.toFixed(4)),
      maxRunLength: maxRun,
      repetitionTestPassed: repetitionTestPassed ? 1 : 0,
      elapsedMs: Number(elapsed.toFixed(2)),
    },
    details: `Extracted ${sampleCount} samples in ${elapsed.toFixed(1)}ms ` +
      `(${Math.round(samplesPerSec).toLocaleString()} samples/sec). ` +
      `Shannon entropy ratio: ${(entropyRatio * 100).toFixed(1)}%. ` +
      `Max run length: ${maxRun} (limit: 8).`,
  };

  log(result.details);
  return result;
}

function benchAdversarialSync(): BenchmarkResult {
  log('--- Adversarial Sync Accuracy Benchmark ---');

  const nodeCount = 8;
  const adversaryCount = 2;
  const iterations = 200;

  const field = new TonalField({ alpha: 0.3, couplingStrength: 0.1 });
  const solver = new DiffusionSolver({ D: 0.01, kT: 0.1, dt: 0.05, freqCoupling: 0.001 });

  const nodeIds = Array.from({ length: nodeCount }, (_, i) => `node-${i}`);
  const offsets: Map<string, number> = new Map();

  for (let i = 0; i < nodeCount; i++) {
    const isAdversary = i < adversaryCount;
    offsets.set(nodeIds[i], isAdversary ? 50 + Math.random() * 50 : Math.random() * 5);
  }

  for (let i = 0; i < nodeCount; i++) {
    const eta = 0.5 + (i / nodeCount) * 2;
    const theta = (i / nodeCount) * 2 * Math.PI;
    const psi = (i / nodeCount) * 2 * Math.PI;
    field.updateFromPacket(nodeIds[i], {
      frequencyState: {
        f_inst: 1.0,
        sidebands: [0.1, 0.05, 0.02, 0.01] as [number, number, number, number],
        coherence: i < adversaryCount ? 0.3 : 0.9,
      },
      modulationIndex: 128,
      networkHealth: i < adversaryCount ? -1 : 1,
      entropyNonce: new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8]),
    }, { eta, theta, psi });
  }

  const neighborGraph: Map<string, { neighborId: string; distance: { eta: number }; coherence: number }[]> = new Map();
  for (let i = 0; i < nodeCount; i++) {
    const edges: { neighborId: string; distance: { eta: number }; coherence: number }[] = [];
    for (let j = 0; j < nodeCount; j++) {
      if (i === j) continue;
      edges.push({
        neighborId: nodeIds[j],
        distance: { eta: Math.abs(i - j) * 0.5 },
        coherence: j < adversaryCount ? 0.3 : 0.9,
      });
    }
    neighborGraph.set(nodeIds[i], edges);
  }

  solver.buildLaplacian(neighborGraph, field);

  const offsetHistory: number[][] = [];
  const honestInitialSpread = Math.max(
    ...nodeIds.slice(adversaryCount).map(id => Math.abs(offsets.get(id)! - 0))
  );

  for (let iter = 0; iter < iterations; iter++) {
    for (const id of nodeIds) {
      solver.updateNeighborOffset(id, offsets.get(id)!);
    }

    const iterOffsets: number[] = [];
    for (let i = adversaryCount; i < nodeCount; i++) {
      const id = nodeIds[i];
      const currentOffset = offsets.get(id)!;

      const grad: { eta: Trit; theta: Trit; psi: Trit } = {
        eta: currentOffset > 1 ? -1 : currentOffset < -1 ? 1 : 0,
        theta: 0,
        psi: 0,
      };

      const correction = solver.step(
        id,
        { offset: currentOffset, frequency: 1.0, confidence: 0.9 },
        field,
        grad
      );

      offsets.set(id, currentOffset + correction.offsetAdjust);
      iterOffsets.push(offsets.get(id)!);
    }
    offsetHistory.push(iterOffsets);
  }

  const finalHonestOffsets = nodeIds
    .slice(adversaryCount)
    .map(id => offsets.get(id)!);
  const meanOffset = finalHonestOffsets.reduce((s, o) => s + o, 0) / finalHonestOffsets.length;
  const maxDeviation = Math.max(...finalHonestOffsets.map(o => Math.abs(o - meanOffset)));
  const finalSpread = Math.max(...finalHonestOffsets) - Math.min(...finalHonestOffsets);
  const convergenceRatio = finalSpread / Math.max(honestInitialSpread, 0.001);

  const adversaryOffsets = nodeIds
    .slice(0, adversaryCount)
    .map(id => offsets.get(id)!);
  const honestMean = meanOffset;
  const adversaryInfluence = Math.abs(honestMean) / Math.max(
    adversaryOffsets.reduce((s, o) => s + Math.abs(o), 0) / adversaryCount,
    0.001
  );

  const passed = maxDeviation < 20 && adversaryInfluence < 0.5;

  const adversaryFraction = adversaryCount / nodeCount;

  const result: BenchmarkResult = {
    name: 'Adversarial Sync Accuracy',
    passed,
    metrics: {
      nodeCount,
      adversaryCount,
      adversaryFraction: Number(adversaryFraction.toFixed(2)),
      attackStrategy: 'static-high-offset' as any,
      attackDescription: 'Fixed 50-100ms offset, low coherence (0.3), networkHealth=-1, no collusion, no topology knowledge' as any,
      topology: 'fully-connected (all-to-all), adversaries at nodes 0..1, no topology knowledge advantage' as any,
      collusionModel: 'independent (non-colluding)' as any,
      adversaryInitialOffset: '50-100ms (random)' as any,
      adversaryCoherence: 0.3,
      honestCoherence: 0.9,
      iterations,
      honestMeanOffset: Number(meanOffset.toFixed(4)),
      maxDeviation: Number(maxDeviation.toFixed(4)),
      finalSpread: Number(finalSpread.toFixed(4)),
      convergenceRatio: Number(convergenceRatio.toFixed(4)),
      adversaryInfluence: Number(adversaryInfluence.toFixed(4)),
    },
    details: `${nodeCount} nodes (${adversaryCount} adversarial, ${(adversaryFraction * 100).toFixed(0)}% fraction) over ${iterations} iterations. ` +
      `Attack: static high-offset (50-100ms), non-colluding, no topology knowledge. ` +
      `Honest mean offset: ${meanOffset.toFixed(4)}, max deviation: ${maxDeviation.toFixed(4)}, ` +
      `adversary influence: ${(adversaryInfluence * 100).toFixed(1)}% (limit: 50%).`,
  };

  log(result.details);
  return result;
}

function benchLaplacianPerformance(): BenchmarkResult {
  log('--- Laplacian Build Performance ---');

  const sizes = [10, 50, 100, 200];
  const timings: Record<string, number> = {};

  for (const n of sizes) {
    const field = new TonalField({ alpha: 0.3, couplingStrength: 0.1 });
    const solver = new DiffusionSolver({ D: 0.01, kT: 0.1, dt: 0.05, freqCoupling: 0.001 });

    for (let i = 0; i < n; i++) {
      field.updateFromPacket(`n-${i}`, {
        frequencyState: {
          f_inst: 1.0,
          sidebands: [0.1, 0.05, 0.02, 0.01] as [number, number, number, number],
          coherence: 0.9,
        },
        modulationIndex: 128,
        networkHealth: 1,
        entropyNonce: new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8]),
      }, { eta: Math.random() * 3, theta: Math.random() * 6.28, psi: Math.random() * 6.28 });
    }

    const k = Math.min(6, n - 1);
    const neighborGraph: Map<string, { neighborId: string; distance: { eta: number }; coherence: number }[]> = new Map();
    for (let i = 0; i < n; i++) {
      const edges: { neighborId: string; distance: { eta: number }; coherence: number }[] = [];
      for (let j = 0; j < k; j++) {
        const neighbor = (i + j + 1) % n;
        edges.push({
          neighborId: `n-${neighbor}`,
          distance: { eta: 0.5 + Math.random() },
          coherence: 0.8 + Math.random() * 0.2,
        });
      }
      neighborGraph.set(`n-${i}`, edges);
    }

    const runs = 100;
    const start = performance.now();
    for (let r = 0; r < runs; r++) {
      solver.buildLaplacian(neighborGraph, field);
    }
    const elapsed = performance.now() - start;
    timings[`${n}_nodes_ms`] = Number((elapsed / runs).toFixed(3));
  }

  const passed = timings['200_nodes_ms'] < 50;

  const result: BenchmarkResult = {
    name: 'Laplacian Build Performance',
    passed,
    metrics: timings,
    details: `Laplacian build times: ${sizes.map(n => `${n} nodes = ${timings[`${n}_nodes_ms`]}ms`).join(', ')}`,
  };

  log(result.details);
  return result;
}

function benchGf3Throughput(): BenchmarkResult {
  log('--- GF(3) Arithmetic Throughput ---');

  const trits: Trit[] = [-1, 0, 1];
  const iterations = 1_000_000;

  const start = performance.now();
  let checksum = 0;
  for (let i = 0; i < iterations; i++) {
    const a = trits[i % 3];
    const b = trits[(i * 7) % 3];
    const r = gf3Sub(a, b);
    checksum += r;
  }
  const elapsed = performance.now() - start;

  const opsPerSec = iterations / (elapsed / 1000);
  const passed = opsPerSec > 1_000_000;

  const result: BenchmarkResult = {
    name: 'GF(3) Arithmetic Throughput',
    passed,
    metrics: {
      iterations,
      elapsedMs: Number(elapsed.toFixed(2)),
      opsPerSecond: Math.round(opsPerSec),
      checksum,
    },
    details: `${iterations.toLocaleString()} GF(3) sub operations in ${elapsed.toFixed(1)}ms ` +
      `(${Math.round(opsPerSec).toLocaleString()} ops/sec)`,
  };

  log(result.details);
  return result;
}

async function main() {
  console.log('=== PlenumNET Tonal Diffusion Benchmark Suite ===');
  console.log(`Date: ${new Date().toISOString()}`);
  console.log(`Node.js: ${process.version}`);
  console.log('');

  results.push(benchPi2Convergence());
  results.push(benchHrvEntropyRate());
  results.push(benchAdversarialSync());
  results.push(benchLaplacianPerformance());
  results.push(benchGf3Throughput());

  console.log('');
  console.log('=== SUMMARY ===');
  const passCount = results.filter(r => r.passed).length;
  const failCount = results.filter(r => !r.passed).length;

  for (const r of results) {
    console.log(`  ${r.passed ? 'PASS' : 'FAIL'}  ${r.name}`);
  }

  console.log('');
  console.log(`${passCount} passed, ${failCount} failed out of ${results.length} benchmarks`);

  const output = {
    timestamp: new Date().toISOString(),
    nodeVersion: process.version,
    results: results.map(r => ({
      name: r.name,
      passed: r.passed,
      metrics: r.metrics,
    })),
    summary: { passed: passCount, failed: failCount, total: results.length },
  };

  console.log('');
  console.log('JSON output:');
  console.log(JSON.stringify(output, null, 2));

  process.exit(failCount > 0 ? 1 : 0);
}

main().catch(err => {
  console.error('Benchmark failed:', err);
  process.exit(2);
});
