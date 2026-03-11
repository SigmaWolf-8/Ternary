/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * WASM Bridge for TL-Sponge-385
 *
 * Loads the WASM sponge module compiled from ternary-math/src/sponge.rs
 * and provides TypeScript bindings. Falls back to the pure TypeScript
 * implementation if WASM is unavailable.
 *
 * Build WASM:
 *   cd ternary-math && wasm-pack build --target nodejs --out-dir pkg
 *
 * The WASM module is expected at ternary-math/pkg/ternary_math.js
 */

import {
  spongeHash as tsSpongeHash,
  spongeHashV1 as tsSpongeHashV1,
  spongeKeystream as tsSpongeKeystream,
  spongeKeystreamV1 as tsSpongeKeystreamV1,
  SpongeDuplex,
  tritsToHex,
  SPONGE_VERSION,
} from './sponge-hash';

let wasmModule: any = null;
let useWasm = false;

export async function initWasmSponge(): Promise<boolean> {
  if (useWasm) return true;
  try {
    wasmModule = await import('../../ternary-math/pkg/ternary_math');
    useWasm = true;
    console.log('[sponge] WASM backend loaded — TL-Sponge-385');
    return true;
  } catch {
    console.log('[sponge] WASM not available — using TypeScript backend');
    return false;
  }
}

export function isWasmAvailable(): boolean {
  return useWasm;
}

export function spongeHash(input: Buffer): string {
  if (useWasm && wasmModule) {
    const result = wasmModule.sponge_hash(new Uint8Array(input), 49);
    return Buffer.from(result).toString('hex');
  }
  return tsSpongeHash(input);
}

export function spongeHashV1(input: Buffer): string {
  return tsSpongeHashV1(input);
}

export function spongeKeystream(domain: Buffer, tritCount: number): Int8Array {
  if (useWasm && wasmModule) {
    const result = wasmModule.sponge_keystream(new Uint8Array(domain), tritCount);
    return new Int8Array(result);
  }
  return tsSpongeKeystream(domain, tritCount);
}

export function spongeKeystreamV1(domain: Buffer, tritCount: number): Int8Array {
  return tsSpongeKeystreamV1(domain, tritCount);
}

export function createDuplex(version: number = 2): SpongeDuplex {
  return new SpongeDuplex(version);
}

export { SPONGE_VERSION };

try {
  initWasmSponge().catch(() => {});
} catch {}
