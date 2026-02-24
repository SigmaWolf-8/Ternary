/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * SINGLE SOURCE OF TRUTH — Platform-wide constants.
 * Update numbers here and they propagate to frontend, backend, and docs endpoints.
 */

export const PLATFORM = {
  VM_OPCODES: 176,
  VM_ISA_VERSION: "v2.1",
  VM_ISA_PREV_VERSION: "v2.0",
  VM_ISA_PREV_OPCODES: 160,
  VM_ISA_V1_OPCODES: 62,
  VM_REGISTERS: 27,
  API_ENDPOINTS: 202,
  API_SERVICES: 21,
  KERNEL_LOC: "47,000+",
  KERNEL_SUBSYSTEMS: 14,
  KERNEL_BINARY_SIZE: "33 MB",
  TESTS_PASSING: "1,040",
  MILESTONES: "80/80",
  DENSITY_ADVANTAGE: 59,
  PLATFORM_VERSION: "2.3.0",
} as const;

export type Platform = typeof PLATFORM;
