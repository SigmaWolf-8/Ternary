/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * DAEMON REMOTE OPERATIONS CHANNEL — Message Protocol
 * Defines all operations message types, error codes, and operator access control
 * for the WebSocket relay operations channel.
 *
 * @version 1.0.0
 */

export type OpsPermissionScope = "full" | "exec-only" | "read-only";

export interface OperatorEntry {
  name: string;
  keyFingerprint: string;
  publicKey: string;
  scope: OpsPermissionScope;
  registeredAt: string;
}

export interface OpsConfig {
  ops_enabled: boolean;
  operators: OperatorEntry[];
  exec_timeout_seconds: number;
  file_size_limit_bytes: number;
  whitelisted_directories: string[];
  blocked_extensions: string[];
  chunk_size_bytes: number;
  telemetry_interval_seconds: number;
  audit_log_path: string;
  audit_log_max_size_mb: number;
}

export const DEFAULT_OPS_CONFIG: OpsConfig = {
  ops_enabled: false,
  operators: [],
  exec_timeout_seconds: 120,
  file_size_limit_bytes: 5 * 1024 * 1024,
  whitelisted_directories: [
    ".plenumnet/ops/",
    ".plenumnet/logs/",
    ".plenumnet/configs/",
    ".plenumnet/transfers/",
  ],
  blocked_extensions: [
    ".exe", ".dll", ".sys", ".bat", ".cmd", ".com", ".scr",
    ".vbs", ".vbe", ".js", ".jse", ".wsf", ".wsh", ".msi",
  ],
  chunk_size_bytes: 512 * 1024,
  telemetry_interval_seconds: 60,
  audit_log_path: ".plenumnet/ops-audit.jsonl",
  audit_log_max_size_mb: 50,
};

export const OPS_ERROR_CODES = {
  SIGNATURE_INVALID: "SIGNATURE_INVALID",
  SIGNATURE_MISSING: "SIGNATURE_MISSING",
  SCOPE_VIOLATION: "SCOPE_VIOLATION",
  PATH_NOT_WHITELISTED: "PATH_NOT_WHITELISTED",
  EXTENSION_BLOCKED: "EXTENSION_BLOCKED",
  FILE_TOO_LARGE: "FILE_TOO_LARGE",
  FILE_READ_FAILED: "FILE_READ_FAILED",
  FILE_WRITE_FAILED: "FILE_WRITE_FAILED",
  OVERWRITE_REQUIRED: "OVERWRITE_REQUIRED",
  EXEC_TIMEOUT: "EXEC_TIMEOUT",
  EXEC_FAILED: "EXEC_FAILED",
  TAIL_FAILED: "TAIL_FAILED",
  CHUNK_WRITE_FAILED: "CHUNK_WRITE_FAILED",
  TRANSFER_HASH_MISMATCH: "TRANSFER_HASH_MISMATCH",
  TRANSFER_HASH_MISSING: "TRANSFER_HASH_MISSING",
  TRANSFER_STALE: "TRANSFER_STALE",
  MODEL_NOT_FOUND: "MODEL_NOT_FOUND",
  MODEL_SWAP_FAILED: "MODEL_SWAP_FAILED",
  OPS_DISABLED: "OPS_DISABLED",
  NODE_NOT_FOUND: "NODE_NOT_FOUND",
  NODE_DISCONNECTED: "NODE_DISCONNECTED",
} as const;

export const OPS_ERROR_DISPLAY_MESSAGES: Record<OpsErrorCode, string> = {
  SIGNATURE_INVALID: "The operation signature could not be verified. Re-sign the request in NinjaExec.",
  SIGNATURE_MISSING: "This operation requires a NinjaExec signature. Open NinjaExec and sign the request.",
  SCOPE_VIOLATION: "Your operator key does not have permission for this operation. Contact the cluster administrator.",
  PATH_NOT_WHITELISTED: "The target path is outside the allowed directories. Check the node's whitelisted_directories configuration.",
  EXTENSION_BLOCKED: "Files with this extension are not allowed. Check the node's blocked_extensions list.",
  FILE_TOO_LARGE: "The file exceeds the maximum allowed size. Use chunked transfer for files over 5 MB.",
  FILE_READ_FAILED: "The file could not be read from the target node. Verify the file exists and permissions are correct.",
  FILE_WRITE_FAILED: "The file could not be written to the target node. Verify directory permissions.",
  OVERWRITE_REQUIRED: "A file already exists at this path. Enable the overwrite option to replace it.",
  EXEC_TIMEOUT: "The script exceeded the maximum execution time and was terminated.",
  EXEC_FAILED: "Script execution failed. Check the error output for details.",
  TAIL_FAILED: "Could not start log tailing. Verify the log file path exists on the target node.",
  CHUNK_WRITE_FAILED: "A chunk could not be written during transfer. The transfer can be resumed.",
  TRANSFER_HASH_MISMATCH: "The completed file hash does not match the expected hash. The transfer is corrupt — re-upload the file.",
  TRANSFER_HASH_MISSING: "No hash was provided for verification. Include a tis27_hash_full in the chunk-init request.",
  TRANSFER_STALE: "The transfer session has expired. Start a new transfer or resume with the transfer ID.",
  MODEL_NOT_FOUND: "The model file was not found on the target node. Verify the path under .plenumnet/models/.",
  MODEL_SWAP_FAILED: "The model could not be loaded by the inference engine. Check model format and engine compatibility.",
  OPS_DISABLED: "The operations channel is inactive on this node. Enable it in the node's ops configuration.",
  NODE_NOT_FOUND: "The target node is not registered in the cluster.",
  NODE_DISCONNECTED: "The target node is currently offline. Wait for reconnection or check the node's network status.",
};

export type OpsErrorCode = keyof typeof OPS_ERROR_CODES;

export type OpsMessageType =
  | "exec"
  | "exec-result"
  | "tail"
  | "tail-data"
  | "tail-stop"
  | "telemetry"
  | "file-push"
  | "file-push-ack"
  | "file-pull"
  | "file-data"
  | "chunk-init"
  | "chunk-data"
  | "chunk-ack"
  | "chunk-complete"
  | "transfer-cancel"
  | "model-swap"
  | "model-swap-result"
  | "ops-error";

export const OPS_MESSAGE_TYPES: readonly OpsMessageType[] = [
  "exec", "exec-result", "tail", "tail-data", "tail-stop",
  "telemetry", "file-push", "file-push-ack", "file-pull", "file-data",
  "chunk-init", "chunk-data", "chunk-ack", "chunk-complete",
  "transfer-cancel", "model-swap", "model-swap-result", "ops-error",
];

export const AUTHENTICATED_OPS_TYPES: readonly OpsMessageType[] = [
  "exec", "tail", "tail-stop", "file-push", "file-pull",
  "chunk-init", "chunk-data", "chunk-complete", "transfer-cancel", "model-swap",
];

export const SCOPE_PERMISSIONS: Record<OpsPermissionScope, readonly OpsMessageType[]> = {
  full: [
    "exec", "tail", "tail-stop", "file-push", "file-pull",
    "chunk-init", "chunk-data", "chunk-complete", "transfer-cancel", "model-swap",
  ],
  "exec-only": ["exec", "tail", "tail-stop"],
  "read-only": ["tail", "tail-stop", "file-pull"],
};

interface OpsMessageBase {
  type: OpsMessageType;
  node_id: string;
  request_id: string;
  signature?: string;
  operator_fingerprint?: string;
  timestamp?: string;
}

export interface ExecMessage extends OpsMessageBase {
  type: "exec";
  script: string;
  language: "powershell" | "cmd";
  timeout_seconds?: number;
  signature: string;
  operator_fingerprint: string;
}

export interface ExecResultMessage extends OpsMessageBase {
  type: "exec-result";
  exit_code: number;
  stdout: string;
  stderr: string;
  duration_ms: number;
  timed_out: boolean;
}

export interface TailMessage extends OpsMessageBase {
  type: "tail";
  file_path: string;
  lines: number;
  follow: boolean;
  signature: string;
  operator_fingerprint: string;
}

export interface TailDataMessage extends OpsMessageBase {
  type: "tail-data";
  file_path: string;
  data: string;
  line_count: number;
  eof: boolean;
}

export interface TailStopMessage extends OpsMessageBase {
  type: "tail-stop";
  original_request_id?: string;
  signature: string;
  operator_fingerprint: string;
}

export interface TelemetryMessage extends OpsMessageBase {
  type: "telemetry";
  cpu_pct: number;
  ram_pct: number;
  ram_used_mb: number;
  ram_total_mb: number;
  disk_pct: number;
  disk_used_gb: number;
  disk_total_gb: number;
  gpu_pct: number | null;
  gpu_name: string | null;
  gpu_vram_used_mb: number | null;
  gpu_vram_total_mb: number | null;
  process_uptime_seconds: number;
  active_model: string | null;
  llm_engine_status: "running" | "stopped" | "error" | "unknown" | "loaded" | "idle";
  os_version: string;
}

export interface FilePushMessage extends OpsMessageBase {
  type: "file-push";
  file_path: string;
  data_base64: string;
  size_bytes: number;
  overwrite: boolean;
  tis27_hash: string;
  signature: string;
  operator_fingerprint: string;
}

export interface FilePushAckMessage extends OpsMessageBase {
  type: "file-push-ack";
  file_path: string;
  success: boolean;
  bytes_written: number;
}

export interface FilePullMessage extends OpsMessageBase {
  type: "file-pull";
  file_path: string;
  signature: string;
  operator_fingerprint: string;
}

export interface FileDataMessage extends OpsMessageBase {
  type: "file-data";
  file_path: string;
  data_base64: string;
  size_bytes: number;
  tis27_hash: string;
}

export interface ChunkInitMessage extends OpsMessageBase {
  type: "chunk-init";
  file_path: string;
  total_size_bytes: number;
  chunk_count: number;
  chunk_size_bytes: number;
  tis27_hash_full: string;
  resume_from_chunk?: number;
  signature: string;
  operator_fingerprint: string;
}

export interface ChunkDataMessage extends OpsMessageBase {
  type: "chunk-data";
  transfer_id: string;
  chunk_index: number;
  data_base64: string;
  tis27_hash_chunk: string;
  signature: string;
  operator_fingerprint: string;
}

export interface ChunkAckMessage extends OpsMessageBase {
  type: "chunk-ack";
  transfer_id: string;
  chunk_index: number;
  all_chunks_received?: boolean;
  success: boolean;
  error_code?: OpsErrorCode;
  error_message?: string;
}

export interface ChunkCompleteRequestMessage extends OpsMessageBase {
  type: "chunk-complete";
  transfer_id: string;
  full_hash: string;
  signature: string;
  operator_fingerprint: string;
}

export interface ChunkCompleteMessage extends OpsMessageBase {
  type: "chunk-complete";
  transfer_id: string;
  file_path: string;
  total_bytes: number;
  tis27_hash_verified: boolean;
  success: boolean;
  error_message?: string;
}

export interface TransferCancelMessage extends OpsMessageBase {
  type: "transfer-cancel";
  transfer_id: string;
  signature: string;
  operator_fingerprint: string;
}

export interface ModelSwapMessage extends OpsMessageBase {
  type: "model-swap";
  model_path: string;
  model_name: string;
  engine_params?: Record<string, string | number>;
  signature: string;
  operator_fingerprint: string;
}

export type EngineStatus =
  | "running"
  | "stopped"
  | "swapping"
  | "error"
  | "degraded"
  | "unreachable"
  | "running_rollback"
  | "degraded_rollback"
  | "running_restarted"
  | "recovery_failed"
  | "pre-validation-failed";

export const OPS_STATUS_COLORS = {
  success:  { hex: "#4ade80", tailwind: "text-green-400",  label: "Success" },
  error:    { hex: "#f87171", tailwind: "text-red-400",    label: "Error" },
  warning:  { hex: "#fbbf24", tailwind: "text-amber-400",  label: "Warning" },
  pending:  { hex: "#60a5fa", tailwind: "text-blue-400",   label: "Pending" },
  timeout:  { hex: "#fb923c", tailwind: "text-orange-400", label: "Timeout" },
} as const;

export const ENGINE_STATUS_COLORS: Record<EngineStatus, { hex: string; tailwind: string; label: string }> = {
  running:              { hex: "#4ade80", tailwind: "text-green-400",  label: "Running" },
  stopped:              { hex: "#9ca3af", tailwind: "text-gray-400",   label: "Stopped" },
  swapping:             { hex: "#a78bfa", tailwind: "text-violet-400", label: "Swapping model…" },
  error:                { hex: "#f87171", tailwind: "text-red-400",    label: "Error" },
  degraded:             { hex: "#fbbf24", tailwind: "text-amber-400",  label: "Degraded" },
  unreachable:          { hex: "#ef4444", tailwind: "text-red-500",    label: "Unreachable" },
  running_rollback:     { hex: "#fbbf24", tailwind: "text-amber-400",  label: "Rolled back" },
  degraded_rollback:    { hex: "#fb923c", tailwind: "text-orange-400", label: "Rollback (degraded)" },
  running_restarted:    { hex: "#facc15", tailwind: "text-yellow-400", label: "Restarted" },
  recovery_failed:      { hex: "#ef4444", tailwind: "text-red-500",    label: "Recovery failed" },
  "pre-validation-failed": { hex: "#ef4444", tailwind: "text-red-500", label: "Validation failed" },
};

export interface ModelSwapResultMessage extends OpsMessageBase {
  type: "model-swap-result";
  success: boolean;
  previous_model: string | null;
  new_model: string;
  engine_status: EngineStatus;
  rollback_performed: boolean;
  rollback_verified?: boolean;
  model_size_mb?: number;
  error_message?: string;
}

export interface OpsErrorMessage extends OpsMessageBase {
  type: "ops-error";
  error_code: OpsErrorCode;
  message: string;
  original_request_id?: string;
  original_type?: OpsMessageType;
}

export type OpsMessage =
  | ExecMessage
  | ExecResultMessage
  | TailMessage
  | TailDataMessage
  | TailStopMessage
  | TelemetryMessage
  | FilePushMessage
  | FilePushAckMessage
  | FilePullMessage
  | FileDataMessage
  | ChunkInitMessage
  | ChunkDataMessage
  | ChunkAckMessage
  | ChunkCompleteRequestMessage
  | ChunkCompleteMessage
  | TransferCancelMessage
  | ModelSwapMessage
  | ModelSwapResultMessage
  | OpsErrorMessage;

export type OpsResponseMessage =
  | ExecResultMessage
  | TailDataMessage
  | TelemetryMessage
  | FilePushAckMessage
  | FileDataMessage
  | ChunkAckMessage
  | ChunkCompleteMessage
  | ModelSwapResultMessage
  | OpsErrorMessage;

export interface OpsAuditEntry {
  timestamp: string;
  operation: OpsMessageType;
  operator_name: string;
  operator_fingerprint: string;
  node_id: string;
  request_id: string;
  payload_tis27_hash: string;
  script_text?: string;
  exit_code?: number;
  stdout_truncated?: string;
  stderr_truncated?: string;
  duration_ms?: number;
  file_path?: string;
  file_size?: number;
  result: "success" | "failure" | "timeout" | "rejected";
  error_code?: OpsErrorCode;
  error_message?: string;
}

export interface NodeTelemetrySnapshot {
  node_id: string;
  address: string;
  last_seen: string;
  last_telemetry: TelemetryMessage | null;
  connection_state: "connected" | "disconnected" | "suspect";
  ops_enabled: boolean;
}

export interface OpsStatusResponse {
  nodes: NodeTelemetrySnapshot[];
  relay_uptime_seconds: number;
  ops_version: string;
}

export function isOpsMessageType(type: string): type is OpsMessageType {
  return OPS_MESSAGE_TYPES.includes(type as OpsMessageType);
}

export function requiresSignature(type: OpsMessageType): boolean {
  return AUTHENTICATED_OPS_TYPES.includes(type);
}

export function isScopeAuthorized(scope: OpsPermissionScope, messageType: OpsMessageType): boolean {
  return SCOPE_PERMISSIONS[scope].includes(messageType);
}

export interface AiProposedExec {
  source: "yoda-ai";
  proposed_script: string;
  rationale: string;
  target_node_id: string;
  proposal_id: string;
  proposed_at: string;
}

export const OPS_PROTOCOL_VERSION = "1.0.0";
export const MAX_STDOUT_BYTES = 10 * 1024;
export const MAX_STDERR_BYTES = 10 * 1024;
export const MAX_FILE_SIZE_BYTES = 5 * 1024 * 1024;
export const CHUNK_STALE_TIMEOUT_MS = 60 * 60 * 1000;
export const NINJAEXEC_PORT = 21027;
export const NINJAEXEC_SIGN_URL = `http://localhost:${NINJAEXEC_PORT}/sign`;
