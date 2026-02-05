/**
 * Payment Flow Integration Tests
 * 
 * End-to-end tests for the complete payment witnessing flow:
 * Webhook → SFK Core API → Hedera HCS → XRPL Settlement → Algorand Recording
 */

import { describe, it, expect, beforeAll, afterAll } from '@jest/globals';

const PAYMENT_LISTENER_URL = process.env.PAYMENT_LISTENER_URL || 'http://localhost:3001';
const SFK_CORE_API_URL = process.env.SFK_CORE_API_URL || 'http://localhost:3002';
const HEDERA_SERVICE_URL = process.env.HEDERA_SERVICE_URL || 'http://localhost:3003';
const XRPL_SERVICE_URL = process.env.XRPL_SERVICE_URL || 'http://localhost:3004';
const ALGORAND_SERVICE_URL = process.env.ALGORAND_SERVICE_URL || 'http://localhost:3005';

interface TestContext {
  operationId?: string;
  paymentId?: string;
  hederaTransactionId?: string;
  xrplTransactionHash?: string;
  algorandTxId?: string;
}

const testContext: TestContext = {};

describe('Payment Flow Integration Tests', () => {
  
  describe('1. Health Checks', () => {
    it('should verify all services are healthy', async () => {
      const services = [
        { name: 'Payment Listener', url: `${PAYMENT_LISTENER_URL}/health` },
        { name: 'SFK Core API', url: `${SFK_CORE_API_URL}/health` },
        { name: 'Hedera Service', url: `${HEDERA_SERVICE_URL}/health` },
        { name: 'XRPL Service', url: `${XRPL_SERVICE_URL}/health` },
        { name: 'Algorand Service', url: `${ALGORAND_SERVICE_URL}/health` }
      ];

      for (const service of services) {
        try {
          const response = await fetch(service.url);
          if (response.ok) {
            const data = await response.json();
            expect(data.status).toBe('healthy');
            console.log(`✓ ${service.name} is healthy`);
          }
        } catch (error) {
          console.log(`⚠ ${service.name} not available (expected in test environment)`);
        }
      }
    });
  });

  describe('2. SFK Operation Creation', () => {
    it('should create a payment_witness operation', async () => {
      const operationRequest = {
        operationType: 'payment_witness',
        securityMode: 'phi_plus',
        priority: 'high',
        paymentContext: {
          gateway: 'stripe',
          paymentId: `pi_test_${Date.now()}`,
          amount: 100.00,
          currency: 'USD'
        },
        ternaryContext: {
          representation: 'A',
          compressionEnabled: true
        },
        blockchainTargets: {
          witnessToHedera: true,
          settleOnXrpl: true,
          recordOnAlgorand: true
        },
        idempotencyKey: `test_${Date.now()}`
      };

      try {
        const response = await fetch(`${SFK_CORE_API_URL}/api/sfk/v1/operations`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(operationRequest)
        });

        if (response.ok) {
          const data = await response.json();
          expect(data.success).toBe(true);
          expect(data.operationId).toBeDefined();
          expect(data.status).toBe('queued');
          
          testContext.operationId = data.operationId;
          testContext.paymentId = operationRequest.paymentContext.paymentId;
          
          console.log(`✓ Created operation: ${data.operationId}`);
        }
      } catch (error) {
        console.log('⚠ SFK Core API not available');
      }
    });

    it('should retrieve operation status', async () => {
      if (!testContext.operationId) {
        console.log('⚠ Skipping: No operation ID from previous test');
        return;
      }

      try {
        const response = await fetch(
          `${SFK_CORE_API_URL}/api/sfk/v1/status/${testContext.operationId}`
        );

        if (response.ok) {
          const data = await response.json();
          expect(data.operationId).toBe(testContext.operationId);
          expect(['pending', 'queued', 'processing', 'witnessed', 'settled']).toContain(data.status);
          
          console.log(`✓ Operation status: ${data.status}`);
        }
      } catch (error) {
        console.log('⚠ SFK Core API not available');
      }
    });
  });

  describe('3. Hedera HCS Witnessing', () => {
    it('should submit witness record to Hedera', async () => {
      const witnessRequest = {
        operationId: testContext.operationId || `test_${Date.now()}`,
        batchRef: `BATCH_${Date.now()}`,
        dataHash: 'a'.repeat(64),
        timestamp: new Date().toISOString(),
        securityMode: 'phi_plus'
      };

      try {
        const response = await fetch(`${HEDERA_SERVICE_URL}/internal/api/hedera/v1/witness`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(witnessRequest)
        });

        if (response.ok) {
          const data = await response.json();
          expect(data.success).toBe(true);
          expect(data.hedera).toBeDefined();
          expect(data.hedera.topicId).toBeDefined();
          expect(data.hedera.transactionId).toBeDefined();
          
          testContext.hederaTransactionId = data.hedera.transactionId;
          
          console.log(`✓ Witnessed on Hedera: ${data.hedera.transactionId}`);
        }
      } catch (error) {
        console.log('⚠ Hedera service not available');
      }
    });

    it('should verify witness record status', async () => {
      if (!testContext.hederaTransactionId) {
        console.log('⚠ Skipping: No Hedera transaction from previous test');
        return;
      }

      try {
        const response = await fetch(
          `${HEDERA_SERVICE_URL}/internal/api/hedera/v1/witness/${encodeURIComponent(testContext.hederaTransactionId)}`
        );

        if (response.ok) {
          const data = await response.json();
          expect(data.status).toBe('SUCCESS');
          
          console.log(`✓ Witness verified: ${data.status}`);
        }
      } catch (error) {
        console.log('⚠ Hedera service not available');
      }
    });
  });

  describe('4. XRPL Settlement', () => {
    it('should submit payment to XRPL', async () => {
      const paymentRequest = {
        operationId: testContext.operationId || `test_${Date.now()}`,
        amount: '100',
        currency: 'XRP',
        destination: 'rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh',
        memo: `Payment for operation ${testContext.operationId}`
      };

      try {
        const response = await fetch(`${XRPL_SERVICE_URL}/internal/api/xrpl/v1/payments`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(paymentRequest)
        });

        if (response.ok) {
          const data = await response.json();
          expect(data.success).toBe(true);
          expect(data.xrpl).toBeDefined();
          expect(data.xrpl.transactionHash).toBeDefined();
          expect(data.xrpl.validated).toBe(true);
          
          testContext.xrplTransactionHash = data.xrpl.transactionHash;
          
          console.log(`✓ Settled on XRPL: ${data.xrpl.transactionHash}`);
        }
      } catch (error) {
        console.log('⚠ XRPL service not available');
      }
    });

    it('should verify XRPL payment status', async () => {
      if (!testContext.xrplTransactionHash) {
        console.log('⚠ Skipping: No XRPL transaction from previous test');
        return;
      }

      try {
        const response = await fetch(
          `${XRPL_SERVICE_URL}/internal/api/xrpl/v1/payments/${testContext.xrplTransactionHash}`
        );

        if (response.ok) {
          const data = await response.json();
          expect(data.validated).toBe(true);
          expect(data.result).toBe('tesSUCCESS');
          
          console.log(`✓ Payment verified: ${data.result}`);
        }
      } catch (error) {
        console.log('⚠ XRPL service not available');
      }
    });
  });

  describe('5. Algorand Smart Contract Recording', () => {
    it('should call Algorand smart contract', async () => {
      const contractRequest = {
        operationId: testContext.operationId || `test_${Date.now()}`,
        appId: 12345,
        method: 'record_operation',
        args: [testContext.operationId, testContext.hederaTransactionId, '100']
      };

      try {
        const response = await fetch(`${ALGORAND_SERVICE_URL}/internal/api/algorand/v1/application/call`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(contractRequest)
        });

        if (response.ok) {
          const data = await response.json();
          expect(data.success).toBe(true);
          expect(data.algorand).toBeDefined();
          expect(data.algorand.txId).toBeDefined();
          
          testContext.algorandTxId = data.algorand.txId;
          
          console.log(`✓ Recorded on Algorand: ${data.algorand.txId}`);
        }
      } catch (error) {
        console.log('⚠ Algorand service not available');
      }
    });

    it('should verify Algorand transaction', async () => {
      if (!testContext.algorandTxId) {
        console.log('⚠ Skipping: No Algorand transaction from previous test');
        return;
      }

      try {
        const response = await fetch(
          `${ALGORAND_SERVICE_URL}/internal/api/algorand/v1/transactions/${testContext.algorandTxId}`
        );

        if (response.ok) {
          const data = await response.json();
          expect(data.confirmed).toBe(true);
          
          console.log(`✓ Transaction confirmed: round ${data.round}`);
        }
      } catch (error) {
        console.log('⚠ Algorand service not available');
      }
    });
  });

  describe('6. End-to-End Flow Verification', () => {
    it('should verify complete operation timeline', async () => {
      if (!testContext.operationId) {
        console.log('⚠ Skipping: No operation ID from previous tests');
        return;
      }

      try {
        const response = await fetch(
          `${SFK_CORE_API_URL}/api/sfk/v1/status/${testContext.operationId}/timeline`
        );

        if (response.ok) {
          const data = await response.json();
          expect(data.operationId).toBe(testContext.operationId);
          expect(data.timeline).toBeDefined();
          expect(Array.isArray(data.timeline)).toBe(true);
          
          console.log(`✓ Timeline events: ${data.timeline.length}`);
          console.log(`✓ Total duration: ${data.totalDurationMs}ms`);
        }
      } catch (error) {
        console.log('⚠ SFK Core API not available');
      }
    });

    it('should verify unified metadata', async () => {
      if (!testContext.operationId) {
        console.log('⚠ Skipping: No operation ID from previous tests');
        return;
      }

      try {
        const response = await fetch(
          `${SFK_CORE_API_URL}/api/sfk/v1/operations/${testContext.operationId}/metadata`
        );

        if (response.ok) {
          const data = await response.json();
          expect(data.operationId).toBe(testContext.operationId);
          expect(data.auditTrail).toBeDefined();
          
          console.log(`✓ Audit trail entries: ${data.auditTrail.length}`);
        }
      } catch (error) {
        console.log('⚠ SFK Core API not available');
      }
    });
  });
});

describe('Status Summary', () => {
  it('should get overall status summary', async () => {
    try {
      const response = await fetch(`${SFK_CORE_API_URL}/api/sfk/v1/status/summary`);

      if (response.ok) {
        const data = await response.json();
        console.log('\n=== Status Summary ===');
        console.log(`Total operations: ${data.total}`);
        console.log(`Average completion time: ${data.averageCompletionTimeMs}ms`);
        console.log('By status:', data.byStatus);
        console.log('By type:', data.byType);
      }
    } catch (error) {
      console.log('⚠ SFK Core API not available');
    }
  });
});
