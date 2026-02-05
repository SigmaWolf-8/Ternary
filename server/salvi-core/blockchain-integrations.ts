/**
 * Blockchain Integration APIs
 * Hedera, XRPL, and Algorand Service Interfaces
 * 
 * @author Capomastro Holdings Ltd.
 * @license Proprietary - All Rights Reserved
 */

import type { SecurityMode, TorsionDimensions } from './unified-metadata-schema';

// ============================================================================
// HEDERA CONSENSUS SERVICE (HCS) WITNESSING
// ============================================================================

/**
 * Witness Type Enumeration
 */
export type WitnessType = 'MERKLE_ROOT_BATCH' | 'SINGLE_HASH' | 'AGGREGATE_PROOF';

/**
 * Witness Payload
 */
export interface WitnessPayload {
  hash: string;
  hash_algorithm: 'SHA256' | 'SHA384' | 'SHA512' | 'KECCAK256';
  encoding: 'hex' | 'base64';
}

/**
 * Ternary Context for Witnessing
 */
export interface WitnessTernaryContext {
  security_mode: SecurityMode;
  phase_offset: number;
  torsion_dimensions: TorsionDimensions;
  batch_size: number;
  operation_count: number;
}

/**
 * Payment Context for Witnessing
 */
export interface WitnessPaymentContext {
  gateway: string;
  payment_id: string;
  amount: number;
  currency: string;
}

/**
 * Timing Information for Witnessing
 */
export interface WitnessTimingInfo {
  batch_start_ts: string;
  batch_end_ts: string;
  duration_ns: number;
  femtosecond_sync_accuracy: number;
}

/**
 * Witness Metadata
 */
export interface WitnessMetadata {
  salvi_batch_ref: string;
  kernel_op_id: string;
  ternary_context: WitnessTernaryContext;
  payment_context: WitnessPaymentContext;
  timing: WitnessTimingInfo;
}

/**
 * Hedera Topic Configuration
 */
export interface HederaTopic {
  id: string;
  memo: string;
}

/**
 * Witness Submission Configuration
 */
export interface WitnessSubmission {
  max_fee_hbar: number;
  submit_key: string;
  require_consensus: boolean;
}

/**
 * Hedera Witness Request
 * POST /internal/api/hedera/v1/witness
 */
export interface HederaWitnessRequest {
  operation_id: string;
  witness_type: WitnessType;
  payload: WitnessPayload;
  metadata: WitnessMetadata;
  topic: HederaTopic;
  submission: WitnessSubmission;
}

/**
 * Chunk Information
 */
export interface ChunkInfo {
  total: number;
  number: number;
  initial_transaction_id: string;
}

/**
 * Transaction Details
 */
export interface HederaTransactionDetails {
  id: string;
  status: 'SUCCESS' | 'PENDING' | 'FAILED';
  consensus_timestamp: string;
  topic_id: string;
  sequence_number: number;
  running_hash: string;
  chunk_info: ChunkInfo;
}

/**
 * Transaction Costs
 */
export interface HederaCosts {
  fee_hbar: number;
  fee_usd: number;
  exchange_rate: number;
}

/**
 * Verification Information
 */
export interface HederaVerification {
  verifiable: boolean;
  proof_available: boolean;
  query_endpoints: string[];
}

/**
 * Hedera Timing Details
 */
export interface HederaTiming {
  submitted_at: string;
  consensus_at: string;
  latency_ns: number;
}

/**
 * Hedera Witness Response
 */
export interface HederaWitnessResponse {
  success: boolean;
  transaction: HederaTransactionDetails;
  costs: HederaCosts;
  verification: HederaVerification;
  timing: HederaTiming;
}

// ============================================================================
// XRPL PAYMENT SERVICE
// ============================================================================

/**
 * XRPL Amount
 */
export interface XRPLAmount {
  value: string;
  currency: string;
  issuer: string;
}

/**
 * XRPL Transaction Definition
 */
export interface XRPLTransaction {
  type: 'Payment' | 'TrustSet' | 'EscrowCreate' | 'EscrowFinish';
  account: string;
  destination: string;
  amount: XRPLAmount;
  fee: string;
  sequence: number;
  last_ledger_sequence: number;
}

/**
 * XRPL Memo Data
 */
export interface XRPLMemoData {
  salvi_batch_ref: string;
  hedera_tx_id: string;
  purpose: string;
  ternary_metrics: {
    operations_count: number;
    success_rate: number;
    average_latency_ns: number;
  };
}

/**
 * XRPL Memo
 */
export interface XRPLMemo {
  type: string;
  format: string;
  data: XRPLMemoData;
}

/**
 * XRPL Signing Configuration
 */
export interface XRPLSigning {
  key_type: 'family_seed' | 'ed25519' | 'secp256k1';
  key_reference: string;
  multi_sig: boolean;
}

/**
 * XRPL Submission Configuration
 */
export interface XRPLSubmission {
  max_retries: number;
  timeout_ms: number;
  require_validation: boolean;
}

/**
 * XRPL Payment Request
 * POST /internal/api/xrpl/v1/payments
 */
export interface XRPLPaymentRequest {
  operation_id: string;
  transaction: XRPLTransaction;
  memos: XRPLMemo[];
  signing: XRPLSigning;
  submission: XRPLSubmission;
}

/**
 * XRPL Transaction Meta
 */
export interface XRPLTransactionMeta {
  transaction_result: 'tesSUCCESS' | 'tecPATH_DRY' | 'tecUNFUNDED_PAYMENT' | string;
  delivered_amount: string;
  balance_changes: {
    sender: string;
    receiver: string;
  };
}

/**
 * XRPL Transaction Response
 */
export interface XRPLTransactionResponse {
  hash: string;
  ledger_index: number;
  fee: string;
  validated: boolean;
  timestamp: number;
  meta: XRPLTransactionMeta;
}

/**
 * XRPL Costs
 */
export interface XRPLCosts {
  fee_xrp: string;
  fee_usd: string;
  network_load: number;
}

/**
 * XRPL Verification
 */
export interface XRPLVerification {
  explorer_url: string;
  api_endpoint: string;
}

/**
 * XRPL Timing
 */
export interface XRPLTiming {
  submitted_at: string;
  validated_at: string;
  confirmation_latency_ns: number;
}

/**
 * XRPL Payment Response
 */
export interface XRPLPaymentResponse {
  success: boolean;
  transaction: XRPLTransactionResponse;
  costs: XRPLCosts;
  verification: XRPLVerification;
  timing: XRPLTiming;
}

// ============================================================================
// ALGORAND SMART CONTRACT SERVICE
// ============================================================================

/**
 * Algorand Application Definition
 */
export interface AlgorandApplication {
  id: number;
  type: 'stateful' | 'stateless';
  name: string;
}

/**
 * Algorand Application Call
 */
export interface AlgorandCall {
  method: string;
  sender: string;
  app_args: string[];
  accounts: string[];
  foreign_apps: number[];
  foreign_assets: number[];
}

/**
 * Algorand Payment Requirement
 */
export interface AlgorandPayment {
  required: boolean;
  amount: number;
  receiver: string;
}

/**
 * Algorand Signing Configuration
 */
export interface AlgorandSigning {
  key_type: 'mnemonic' | 'ledger' | 'multisig';
  key_reference: string;
}

/**
 * Algorand Submission Configuration
 */
export interface AlgorandSubmission {
  wait_for_confirmation: boolean;
  confirmation_wait_rounds: number;
  timeout_ms: number;
}

/**
 * Algorand Application Call Request
 * POST /internal/api/algorand/v1/application/call
 */
export interface AlgorandApplicationCallRequest {
  operation_id: string;
  application: AlgorandApplication;
  call: AlgorandCall;
  payment: AlgorandPayment;
  signing: AlgorandSigning;
  submission: AlgorandSubmission;
}

/**
 * Algorand Inner Transaction
 */
export interface AlgorandInnerTransaction {
  type: 'axfer' | 'pay' | 'appl';
  asset_id?: number;
  amount: number;
  receiver: string;
}

/**
 * Algorand Transaction Response
 */
export interface AlgorandTransactionResponse {
  id: string;
  confirmed_round: number;
  sender: string;
  application_index: number;
  inner_txns: AlgorandInnerTransaction[];
  logs: string[];
}

/**
 * Algorand Global State
 */
export interface AlgorandGlobalState {
  total_rewards_distributed: string;
  last_batch_processed: string;
}

/**
 * Algorand Local State
 */
export interface AlgorandLocalState {
  user_claimed: boolean;
  claim_timestamp: number;
  amount_claimed: number;
}

/**
 * Algorand Application State
 */
export interface AlgorandApplicationState {
  global_state: AlgorandGlobalState;
  local_state: AlgorandLocalState;
}

/**
 * Algorand Costs
 */
export interface AlgorandCosts {
  fee_microalgos: number;
  fee_usd: string;
  min_balance_increase: number;
}

/**
 * Algorand Timing
 */
export interface AlgorandTiming {
  submitted_at: string;
  confirmed_at: string;
  confirmation_latency_ns: number;
}

/**
 * Algorand Application Call Response
 */
export interface AlgorandApplicationCallResponse {
  success: boolean;
  transaction: AlgorandTransactionResponse;
  application_state: AlgorandApplicationState;
  costs: AlgorandCosts;
  timing: AlgorandTiming;
}

// ============================================================================
// SERVICE INTERFACES
// ============================================================================

/**
 * Hedera Witnessing Service Interface
 */
export interface IHederaWitnessingService {
  submitWitness(request: HederaWitnessRequest): Promise<HederaWitnessResponse>;
  getWitnessStatus(transactionId: string): Promise<HederaWitnessResponse | null>;
  verifyWitness(topicId: string, sequenceNumber: number): Promise<boolean>;
}

/**
 * XRPL Payment Service Interface
 */
export interface IXRPLPaymentService {
  submitPayment(request: XRPLPaymentRequest): Promise<XRPLPaymentResponse>;
  getTransactionStatus(txHash: string): Promise<XRPLPaymentResponse | null>;
  getAccountBalance(account: string): Promise<{ xrp: string; tokens: XRPLAmount[] }>;
}

/**
 * Algorand Smart Contract Service Interface
 */
export interface IAlgorandContractService {
  callApplication(request: AlgorandApplicationCallRequest): Promise<AlgorandApplicationCallResponse>;
  getApplicationState(appId: number, address?: string): Promise<AlgorandApplicationState>;
  getTransactionStatus(txId: string): Promise<AlgorandApplicationCallResponse | null>;
}
