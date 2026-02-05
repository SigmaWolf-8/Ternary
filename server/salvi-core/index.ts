/**
 * Salvi Framework Core API
 * 
 * Implements the Unified Ternary Logic System from the whitepaper:
 * - Three Bijective Representations (A, B, C)
 * - Ternary Operations (addition, multiplication, rotation)
 * - Femtosecond Timestamp Generation
 * - Phase-aware operations
 * 
 * Payment & Witnessing Architecture v1.0:
 * - Payment Listener API for webhook handling
 * - SFK Operations API for kernel management
 * - Blockchain Integrations (Hedera, XRPL, Algorand)
 * - Femtosecond Timing Service
 * - Unified Metadata Schema
 * - Error Handling & Retry Logic
 */

// Core Ternary Types and Operations
export * from './ternary-types';
export * from './ternary-operations';
export * from './femtosecond-timing';
export * from './phase-encryption';
export * from './libternary/aspect-api';

// Payment & Witnessing Architecture APIs
// Note: These exports exclude duplicates (SecurityMode from unified-metadata-schema, SALVI_EPOCH from timing-service)
export { 
  TernaryContext, PaymentContext, ClockSource, TimingContext,
  HederaReference, XRPLReference, AlgorandReference, BlockchainRefs,
  OperationMetrics, OperationResults, UnifiedOperationMetadata,
  WebhookValidation, EventTypeMapping, MetadataMapping, GatewaySchema,
  WEBHOOK_SCHEMA_REGISTRY, RATE_LIMITS, BURST_LIMITS,
  createOperationMetadata, validateOperationMetadata
} from './unified-metadata-schema';

export type { PaymentGateway, TorsionDimensions } from './unified-metadata-schema';

export * from './payment-listener-api';
export * from './sfk-operations-api';
export * from './blockchain-integrations';

export { 
  ClockSourceType, SyncStatus, CertificationLevel,
  TimestampComponents, ClockSourceInfo, SynchronizationInfo,
  LeapSecondInfo, TimingProtocolMetadata, CurrentTimestampResponse,
  TimingEvent, CertificationRequirements, TimingCertificationRequest,
  CertificationStatus, CertificationMetrics, TimestampProof,
  CertificationProof, CertificationSignature, TimingCertificationResponse,
  TIMING_CONSTANTS, getCurrentFemtosecondTimestamp, calculateTotalUncertainty,
  meetsRequirements, createCurrentTimestampResponse, createCertificationResponse,
  ITimingService
} from './timing-service';

export * from './error-handling';
