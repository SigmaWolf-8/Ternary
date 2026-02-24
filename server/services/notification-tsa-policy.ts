/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * PROPRIETARY AND CONFIDENTIAL
 * All Rights Reserved.
 *
 * NOTIFICATION → TSA POLICY TIER MAPPING
 *
 * Determines which TSA tier (COMPLY, FORENSICS, SENTINEL, etc.) applies
 * to each outbound notification based on channel and content type.
 * Separated from notification-service.ts for clean imports and testability.
 */

import { TSA_POLICIES, TSA_POLICY_METADATA } from './tsa-service';

export const CHANNEL_POLICY: Record<string, string> = {
  email:    TSA_POLICIES.COMPLY,
  sms:      TSA_POLICIES.DEFAULT,
  push:     TSA_POLICIES.DEFAULT,
  webhook:  TSA_POLICIES.SECURE,
  event:    TSA_POLICIES.SENTINEL,
};

export const CONTENT_POLICY: Record<string, string> = {
  'legal-notice':         TSA_POLICIES.FORENSICS,
  'litigation-hold':      TSA_POLICIES.FORENSICS,
  'ediscovery-alert':     TSA_POLICIES.FORENSICS,
  'trade-confirmation':   TSA_POLICIES.COMPLY,
  'regulatory-filing':    TSA_POLICIES.COMPLY,
  'audit-notification':   TSA_POLICIES.COMPLY,
  'security-alert':       TSA_POLICIES.SENTINEL,
  'incident-response':    TSA_POLICIES.SENTINEL,
  'access-violation':     TSA_POLICIES.SENTINEL,
  'document-signed':      TSA_POLICIES.FORENSICS,
  'document-delivered':   TSA_POLICIES.FORENSICS,
  'document-rejected':    TSA_POLICIES.FORENSICS,
};

export function resolveTsaPolicy(
  channel: string,
  contentType?: string,
): string {
  if (contentType && CONTENT_POLICY[contentType]) {
    return CONTENT_POLICY[contentType];
  }
  return CHANNEL_POLICY[channel] || TSA_POLICIES.DEFAULT;
}

export function resolveTierName(
  channel: string,
  contentType?: string,
): string {
  const oid = resolveTsaPolicy(channel, contentType);
  const meta = TSA_POLICY_METADATA[oid];
  return meta?.tier || 'UNKNOWN';
}
