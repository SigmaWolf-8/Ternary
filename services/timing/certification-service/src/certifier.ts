/**
 * Timing Certificate Authority
 * 
 * Provides cryptographically signed timing certificates for FINRA 613 and MiFID II compliance.
 * 
 * PRODUCTION REQUIREMENTS:
 * - Set SIGNING_SECRET_KEY environment variable with a secure 256-bit key
 * - Configure persistent storage for certificate retention (default: in-memory)
 * - Connect to real HPTP peers for accurate timing synchronization
 * - For hardware timestamping, integrate with GPS/PTP clock sources
 */

import { v4 as uuidv4 } from 'uuid';
import * as crypto from 'crypto';
import { TimingVerifier, VerificationResult } from './verifiers/timing-verifier';

const IS_PRODUCTION = process.env.NODE_ENV === 'production';

function getSigningKey(): string {
  const key = process.env.SIGNING_SECRET_KEY;
  
  if (IS_PRODUCTION && !key) {
    console.error('CRITICAL: SIGNING_SECRET_KEY must be set in production mode');
    process.exit(1);
  }
  
  if (!key) {
    console.warn('WARNING: Using development signing key. Set SIGNING_SECRET_KEY for production.');
    return 'dev-signing-key-not-for-production';
  }
  
  return key;
}

const SIGNING_KEY = getSigningKey();

export interface CertificationRequest {
  operationId: string;
  timestamp: string;
  dataHash: string;
  securityMode?: string;
}

export interface TimingCertificate {
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
}

export interface CertificateVerification {
  valid: boolean;
  expired: boolean;
  signatureValid: boolean;
  certificate?: TimingCertificate;
}

export interface FINRA613ComplianceStatus {
  compliant: boolean;
  clockSyncStatus: 'synchronized' | 'degraded' | 'unsynchronized';
  accuracyWithinTolerance: boolean;
  maxAllowedDriftMs: number;
  currentDriftMs: number;
  lastSyncCheck: string;
  certificationCapable: boolean;
  auditTrailEnabled: boolean;
}

export class Certifier {
  private timingVerifier: TimingVerifier;
  private certificates: Map<string, TimingCertificate> = new Map();
  private readonly CERTIFICATE_VALIDITY_HOURS = 24;
  private readonly ISSUER = 'Salvi Framework Timing Authority';
  private readonly VERSION = '1.0';

  constructor(timingVerifier: TimingVerifier) {
    this.timingVerifier = timingVerifier;
  }

  async certifyTimestamp(request: CertificationRequest): Promise<TimingCertificate> {
    const verification = await this.timingVerifier.verifyTimestamp(request.timestamp);
    
    if (!verification.passed) {
      throw new Error(`Timestamp verification failed: ${verification.failureReasons.join(', ')}`);
    }

    const certificateId = `CERT_${uuidv4().replace(/-/g, '').slice(0, 16).toUpperCase()}`;
    const certifiedAt = new Date();
    const validUntil = new Date(certifiedAt.getTime() + this.CERTIFICATE_VALIDITY_HOURS * 60 * 60 * 1000);

    const certificate: TimingCertificate = {
      certificateId,
      operationId: request.operationId,
      timestamp: request.timestamp,
      certifiedAt: certifiedAt.toISOString(),
      validUntil: validUntil.toISOString(),
      accuracy: verification.accuracy,
      accuracyNs: verification.accuracyNs,
      clockSource: verification.clockSource,
      synchronized: verification.synchronized,
      verification: {
        passed: verification.passed,
        checks: verification.checksPerformed,
        finra613Compliant: verification.finra613Compliant,
        mifid2Compliant: verification.mifid2Compliant
      },
      signature: this.generateSignature(certificateId, request.timestamp, certifiedAt.toISOString()),
      issuer: this.ISSUER,
      version: this.VERSION
    };

    this.certificates.set(certificateId, certificate);

    return certificate;
  }

  getCertificate(certificateId: string): TimingCertificate | undefined {
    return this.certificates.get(certificateId);
  }

  async verifyCertificate(certificateId: string): Promise<CertificateVerification> {
    const certificate = this.certificates.get(certificateId);
    
    if (!certificate) {
      return {
        valid: false,
        expired: false,
        signatureValid: false
      };
    }

    const now = new Date();
    const validUntil = new Date(certificate.validUntil);
    const expired = now > validUntil;

    const signatureValid = this.verifySignature(certificate);

    return {
      valid: !expired && signatureValid && certificate.verification.passed,
      expired,
      signatureValid,
      certificate
    };
  }

  getFINRA613ComplianceStatus(): FINRA613ComplianceStatus {
    const syncStatus = this.timingVerifier.getSyncStatus();
    
    const maxAllowedDriftMs = 50;
    const currentDriftMs = syncStatus.offsetNs / 1_000_000;
    
    return {
      compliant: syncStatus.synchronized && Math.abs(currentDriftMs) <= maxAllowedDriftMs,
      clockSyncStatus: syncStatus.synchronized ? 'synchronized' : 'unsynchronized',
      accuracyWithinTolerance: Math.abs(currentDriftMs) <= maxAllowedDriftMs,
      maxAllowedDriftMs,
      currentDriftMs: Math.abs(currentDriftMs),
      lastSyncCheck: syncStatus.lastSyncAt,
      certificationCapable: true,
      auditTrailEnabled: true
    };
  }

  private generateSignature(certificateId: string, timestamp: string, certifiedAt: string): string {
    const data = `${certificateId}:${timestamp}:${certifiedAt}:${this.ISSUER}:${this.VERSION}`;
    
    const hmac = crypto.createHmac('sha256', SIGNING_KEY);
    hmac.update(data);
    return hmac.digest('hex');
  }

  private verifySignature(certificate: TimingCertificate): boolean {
    const expectedSignature = this.generateSignature(
      certificate.certificateId,
      certificate.timestamp,
      certificate.certifiedAt
    );
    
    try {
      return crypto.timingSafeEqual(
        Buffer.from(certificate.signature, 'hex'),
        Buffer.from(expectedSignature, 'hex')
      );
    } catch {
      return false;
    }
  }

  listCertificates(operationId?: string): TimingCertificate[] {
    const certs = Array.from(this.certificates.values());
    
    if (operationId) {
      return certs.filter(c => c.operationId === operationId);
    }
    
    return certs;
  }

  async revokeCertificate(certificateId: string): Promise<boolean> {
    return this.certificates.delete(certificateId);
  }
}
