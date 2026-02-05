import { OperationStatus } from '../models/operation-request';

type StateTransition = {
  from: OperationStatus;
  to: OperationStatus;
  condition?: () => boolean;
};

const VALID_TRANSITIONS: StateTransition[] = [
  { from: 'pending', to: 'queued' },
  { from: 'pending', to: 'cancelled' },
  { from: 'queued', to: 'processing' },
  { from: 'queued', to: 'cancelled' },
  { from: 'processing', to: 'witnessed' },
  { from: 'processing', to: 'failed' },
  { from: 'witnessed', to: 'settled' },
  { from: 'witnessed', to: 'failed' },
  { from: 'failed', to: 'pending' },
  { from: 'failed', to: 'cancelled' }
];

interface StateRecord {
  operationId: string;
  currentState: OperationStatus;
  history: StateHistoryEntry[];
  lockedUntil?: number;
}

interface StateHistoryEntry {
  from: OperationStatus;
  to: OperationStatus;
  timestamp: string;
  reason?: string;
}

export class StateManager {
  private stateRecords: Map<string, StateRecord> = new Map();
  private transitionLocks: Map<string, Promise<void>> = new Map();

  async transition(
    operationId: string,
    from: OperationStatus,
    to: OperationStatus,
    reason?: string
  ): Promise<boolean> {
    const existingLock = this.transitionLocks.get(operationId);
    if (existingLock) {
      await existingLock;
    }

    let resolveLock: () => void;
    const lockPromise = new Promise<void>(resolve => {
      resolveLock = resolve;
    });
    this.transitionLocks.set(operationId, lockPromise);

    try {
      const isValid = this.isValidTransition(from, to);
      if (!isValid) {
        throw new Error(`Invalid state transition: ${from} -> ${to}`);
      }

      let record = this.stateRecords.get(operationId);
      
      if (!record) {
        record = {
          operationId,
          currentState: from,
          history: []
        };
        this.stateRecords.set(operationId, record);
      }

      if (record.currentState !== from) {
        throw new Error(`State mismatch: expected ${from}, got ${record.currentState}`);
      }

      record.history.push({
        from,
        to,
        timestamp: new Date().toISOString(),
        reason
      });

      record.currentState = to;

      return true;
    } finally {
      this.transitionLocks.delete(operationId);
      resolveLock!();
    }
  }

  isValidTransition(from: OperationStatus, to: OperationStatus): boolean {
    return VALID_TRANSITIONS.some(t => t.from === from && t.to === to);
  }

  getCurrentState(operationId: string): OperationStatus | null {
    const record = this.stateRecords.get(operationId);
    return record?.currentState ?? null;
  }

  getStateHistory(operationId: string): StateHistoryEntry[] {
    const record = this.stateRecords.get(operationId);
    return record?.history ?? [];
  }

  getValidNextStates(currentState: OperationStatus): OperationStatus[] {
    return VALID_TRANSITIONS
      .filter(t => t.from === currentState)
      .map(t => t.to);
  }

  async cancel(operationId: string, reason: string): Promise<boolean> {
    const currentState = this.getCurrentState(operationId);
    
    if (!currentState) {
      return false;
    }

    const cancellableStates: OperationStatus[] = ['pending', 'queued', 'failed'];
    
    if (!cancellableStates.includes(currentState)) {
      throw new Error(`Cannot cancel operation in state: ${currentState}`);
    }

    return this.transition(operationId, currentState, 'cancelled', reason);
  }

  async retry(operationId: string): Promise<boolean> {
    const currentState = this.getCurrentState(operationId);
    
    if (currentState !== 'failed') {
      throw new Error(`Can only retry failed operations, current state: ${currentState}`);
    }

    return this.transition(operationId, 'failed', 'pending', 'Manual retry requested');
  }

  getStatistics(): {
    byState: Record<OperationStatus, number>;
    total: number;
  } {
    const stats: Record<OperationStatus, number> = {
      pending: 0,
      queued: 0,
      processing: 0,
      witnessed: 0,
      settled: 0,
      failed: 0,
      cancelled: 0
    };

    for (const record of this.stateRecords.values()) {
      stats[record.currentState]++;
    }

    return {
      byState: stats,
      total: this.stateRecords.size
    };
  }
}
