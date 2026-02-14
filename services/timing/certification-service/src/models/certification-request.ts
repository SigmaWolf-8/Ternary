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

import { z } from 'zod';

export const CertificationRequestSchema = z.object({
  operationId: z.string().min(1, 'Operation ID is required'),
  timestamp: z.string().datetime('Invalid timestamp format'),
  dataHash: z.string().optional(),
  securityMode: z.enum(['mode_zero', 'mode_one', 'phi_plus']).default('phi_plus'),
  metadata: z.record(z.string()).optional()
});

export type CertificationRequestInput = z.infer<typeof CertificationRequestSchema>;

export const CertificateQuerySchema = z.object({
  certificateId: z.string().min(1, 'Certificate ID is required')
});

export type CertificateQuery = z.infer<typeof CertificateQuerySchema>;

export const VerificationRequestSchema = z.object({
  certificateId: z.string().min(1, 'Certificate ID is required'),
  includeDetails: z.boolean().default(false)
});

export type VerificationRequest = z.infer<typeof VerificationRequestSchema>;

export interface CertificationResponse {
  success: boolean;
  certificate?: {
    certificateId: string;
    operationId: string;
    timestamp: string;
    certifiedAt: string;
    validUntil: string;
    accuracy: string;
    accuracyNs: number;
    clockSource: string;
    synchronized: boolean;
    verification: {
      passed: boolean;
      checks: string[];
      finra613Compliant: boolean;
      mifid2Compliant: boolean;
    };
    signature: string;
    issuer: string;
    version: string;
  };
  error?: string;
}

export interface VerificationResponse {
  success: boolean;
  verification?: {
    valid: boolean;
    expired: boolean;
    signatureValid: boolean;
  };
  certificate?: CertificationResponse['certificate'];
  error?: string;
}
