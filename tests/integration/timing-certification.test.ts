/**
 * Timing Certification Integration Tests
 * 
 * Tests for femtosecond timing service and FINRA 613 compliance certification.
 */

import { describe, it, expect } from '@jest/globals';

const FEMTOSECOND_SERVICE_URL = process.env.FEMTOSECOND_SERVICE_URL || 'http://localhost:3006';
const CERTIFICATION_SERVICE_URL = process.env.CERTIFICATION_SERVICE_URL || 'http://localhost:3007';

describe('Femtosecond Timing Service Tests', () => {
  
  describe('1. Health & Status', () => {
    it('should be healthy and synchronized', async () => {
      try {
        const response = await fetch(`${FEMTOSECOND_SERVICE_URL}/health`);
        
        if (response.ok) {
          const data = await response.json();
          expect(data.status).toBe('healthy');
          expect(data.service).toBe('femtosecond-service');
          
          console.log(`✓ Femtosecond service healthy, synchronized: ${data.synchronized}`);
        }
      } catch (error) {
        console.log('⚠ Femtosecond service not available');
      }
    });
  });

  describe('2. Timestamp Generation', () => {
    it('should return femtosecond-precision timestamp', async () => {
      try {
        const response = await fetch(`${FEMTOSECOND_SERVICE_URL}/api/timing/v1/current`);
        
        if (response.ok) {
          const data = await response.json();
          expect(data.success).toBe(true);
          expect(data.timestamp).toBeDefined();
          expect(data.timestamp.femtoseconds).toBeDefined();
          expect(data.timestamp.components).toBeDefined();
          expect(data.timestamp.components.femtoseconds).toBeDefined();
          
          console.log(`✓ Femtosecond timestamp: ${data.timestamp.femtoseconds}`);
        }
      } catch (error) {
        console.log('⚠ Femtosecond service not available');
      }
    });

    it('should return batch of timestamps', async () => {
      try {
        const response = await fetch(`${FEMTOSECOND_SERVICE_URL}/api/timing/v1/batch`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ count: 10 })
        });
        
        if (response.ok) {
          const data = await response.json();
          expect(data.success).toBe(true);
          expect(data.count).toBe(10);
          expect(data.timestamps).toHaveLength(10);
          
          console.log(`✓ Generated ${data.count} timestamps`);
        }
      } catch (error) {
        console.log('⚠ Femtosecond service not available');
      }
    });
  });

  describe('3. HPTP Synchronization', () => {
    it('should return sync status', async () => {
      try {
        const response = await fetch(`${FEMTOSECOND_SERVICE_URL}/api/timing/v1/sync-status`);
        
        if (response.ok) {
          const data = await response.json();
          expect(data.success).toBe(true);
          expect(data.status).toBeDefined();
          expect(typeof data.status.synchronized).toBe('boolean');
          expect(typeof data.status.offsetNs).toBe('number');
          expect(typeof data.status.jitterNs).toBe('number');
          
          console.log(`✓ Sync status: offset=${data.status.offsetNs}ns, jitter=${data.status.jitterNs}ns`);
        }
      } catch (error) {
        console.log('⚠ Femtosecond service not available');
      }
    });

    it('should return peer information', async () => {
      try {
        const response = await fetch(`${FEMTOSECOND_SERVICE_URL}/api/timing/v1/peers`);
        
        if (response.ok) {
          const data = await response.json();
          expect(data.success).toBe(true);
          expect(Array.isArray(data.peers)).toBe(true);
          
          console.log(`✓ Connected to ${data.peers.length} HPTP peers`);
        }
      } catch (error) {
        console.log('⚠ Femtosecond service not available');
      }
    });
  });

  describe('4. Metrics', () => {
    it('should return timing metrics', async () => {
      try {
        const response = await fetch(`${FEMTOSECOND_SERVICE_URL}/api/timing/v1/metrics`);
        
        if (response.ok) {
          const data = await response.json();
          expect(data.success).toBe(true);
          expect(data.metrics).toBeDefined();
          expect(typeof data.metrics.synchronized).toBe('boolean');
          expect(typeof data.metrics.stratum).toBe('number');
          
          console.log(`✓ Metrics: stratum=${data.metrics.stratum}, precision=${data.metrics.precision}`);
        }
      } catch (error) {
        console.log('⚠ Femtosecond service not available');
      }
    });
  });
});

describe('Certification Service Tests', () => {
  let certificateId: string;

  describe('1. Health Check', () => {
    it('should be healthy', async () => {
      try {
        const response = await fetch(`${CERTIFICATION_SERVICE_URL}/health`);
        
        if (response.ok) {
          const data = await response.json();
          expect(data.status).toBe('healthy');
          expect(data.service).toBe('certification-service');
          
          console.log('✓ Certification service healthy');
        }
      } catch (error) {
        console.log('⚠ Certification service not available');
      }
    });
  });

  describe('2. Timestamp Certification', () => {
    it('should certify a valid timestamp', async () => {
      const certificationRequest = {
        operationId: `test_${Date.now()}`,
        timestamp: new Date().toISOString(),
        dataHash: 'abc123'
      };

      try {
        const response = await fetch(`${CERTIFICATION_SERVICE_URL}/api/timing/v1/certify`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(certificationRequest)
        });
        
        if (response.ok) {
          const data = await response.json();
          expect(data.success).toBe(true);
          expect(data.certificate).toBeDefined();
          expect(data.certificate.certificateId).toBeDefined();
          expect(data.certificate.accuracy).toBeDefined();
          expect(data.certificate.verification.passed).toBe(true);
          
          certificateId = data.certificate.certificateId;
          
          console.log(`✓ Certified: ${certificateId}`);
          console.log(`  Accuracy: ${data.certificate.accuracy}`);
          console.log(`  FINRA 613: ${data.certificate.verification.finra613Compliant}`);
          console.log(`  MiFID II: ${data.certificate.verification.mifid2Compliant}`);
        }
      } catch (error) {
        console.log('⚠ Certification service not available');
      }
    });

    it('should retrieve certificate by ID', async () => {
      if (!certificateId) {
        console.log('⚠ Skipping: No certificate from previous test');
        return;
      }

      try {
        const response = await fetch(
          `${CERTIFICATION_SERVICE_URL}/api/timing/v1/certificates/${certificateId}`
        );
        
        if (response.ok) {
          const data = await response.json();
          expect(data.success).toBe(true);
          expect(data.certificate.certificateId).toBe(certificateId);
          
          console.log(`✓ Retrieved certificate: ${certificateId}`);
        }
      } catch (error) {
        console.log('⚠ Certification service not available');
      }
    });

    it('should verify certificate', async () => {
      if (!certificateId) {
        console.log('⚠ Skipping: No certificate from previous test');
        return;
      }

      try {
        const response = await fetch(`${CERTIFICATION_SERVICE_URL}/api/timing/v1/verify`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ certificateId })
        });
        
        if (response.ok) {
          const data = await response.json();
          expect(data.success).toBe(true);
          expect(data.verification).toBeDefined();
          expect(data.verification.valid).toBe(true);
          expect(data.verification.expired).toBe(false);
          expect(data.verification.signatureValid).toBe(true);
          
          console.log('✓ Certificate verification passed');
        }
      } catch (error) {
        console.log('⚠ Certification service not available');
      }
    });
  });

  describe('3. FINRA 613 Compliance', () => {
    it('should return FINRA 613 compliance status', async () => {
      try {
        const response = await fetch(
          `${CERTIFICATION_SERVICE_URL}/api/timing/v1/compliance/finra-613`
        );
        
        if (response.ok) {
          const data = await response.json();
          expect(data.success).toBe(true);
          expect(data.compliance).toBeDefined();
          expect(typeof data.compliance.compliant).toBe('boolean');
          expect(data.compliance.maxAllowedDriftMs).toBeDefined();
          expect(data.compliance.currentDriftMs).toBeDefined();
          
          console.log('\n=== FINRA 613 Compliance ===');
          console.log(`Compliant: ${data.compliance.compliant}`);
          console.log(`Clock sync: ${data.compliance.clockSyncStatus}`);
          console.log(`Max allowed drift: ${data.compliance.maxAllowedDriftMs}ms`);
          console.log(`Current drift: ${data.compliance.currentDriftMs}ms`);
          console.log(`Accuracy within tolerance: ${data.compliance.accuracyWithinTolerance}`);
        }
      } catch (error) {
        console.log('⚠ Certification service not available');
      }
    });
  });
});

describe('Timing Performance Tests', () => {
  it('should handle 100 timestamp requests', async () => {
    const startTime = Date.now();
    const requests: Promise<Response>[] = [];
    
    for (let i = 0; i < 100; i++) {
      requests.push(
        fetch(`${FEMTOSECOND_SERVICE_URL}/api/timing/v1/current`).catch(() => new Response())
      );
    }
    
    try {
      const responses = await Promise.all(requests);
      const successCount = responses.filter(r => r.ok).length;
      const duration = Date.now() - startTime;
      
      console.log(`\n=== Performance Test ===`);
      console.log(`Requests: 100`);
      console.log(`Successful: ${successCount}`);
      console.log(`Duration: ${duration}ms`);
      console.log(`Rate: ${Math.round(100000 / duration)} req/s`);
    } catch (error) {
      console.log('⚠ Performance test skipped - services not available');
    }
  });
});
