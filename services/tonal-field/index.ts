/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
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

export { TonalField } from './field';
export type { FmTimingPacketData } from './field';
export { DiffusionSolver } from './diffusion';
export type { ClockState, ClockCorrection } from './diffusion';
export { computePlenumMetrics, assessHealth } from './metrics';
export type { PlenumMetrics, NetworkState } from './metrics';
