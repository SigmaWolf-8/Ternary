/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * Security Infrastructure Database Performance Indexes
 * Applied: February 17, 2026
 * 
 * These indexes optimize query performance for the 4 security infrastructure tables.
 * All indexes are aligned with the architecture specification in
 * BACKEND_INFRASTRUCTURE_DOCS.md, Section: Database Schema.
 */

-- =============================================================================
-- Table 1: security_audit_log (6 indexes)
-- =============================================================================

-- Single-column indexes for filtered queries
CREATE INDEX IF NOT EXISTS idx_audit_log_severity ON security_audit_log(severity);
CREATE INDEX IF NOT EXISTS idx_audit_log_category ON security_audit_log(category);
CREATE INDEX IF NOT EXISTS idx_audit_log_timestamp ON security_audit_log(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_audit_log_resolution ON security_audit_log(resolution_status);
CREATE INDEX IF NOT EXISTS idx_audit_log_event_type ON security_audit_log(event_type);

-- Composite index for dashboard aggregation queries (severity + category + time window)
CREATE INDEX IF NOT EXISTS idx_audit_log_severity_category_ts ON security_audit_log(severity, category, created_at DESC);

-- =============================================================================
-- Table 2: hptp_anomaly_events (5 indexes)
-- =============================================================================

CREATE INDEX IF NOT EXISTS idx_hptp_anomaly_severity ON hptp_anomaly_events(severity_score DESC);
CREATE INDEX IF NOT EXISTS idx_hptp_anomaly_timestamp ON hptp_anomaly_events(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_hptp_anomaly_active_tier ON hptp_anomaly_events(active_tier);
CREATE INDEX IF NOT EXISTS idx_hptp_anomaly_escalation ON hptp_anomaly_events(escalation_triggered);
CREATE INDEX IF NOT EXISTS idx_hptp_anomaly_type ON hptp_anomaly_events(anomaly_type);

-- =============================================================================
-- Table 3: threat_model_entries (3 indexes)
-- =============================================================================

CREATE INDEX IF NOT EXISTS idx_threat_model_category ON threat_model_entries(category);
CREATE INDEX IF NOT EXISTS idx_threat_model_risk_score ON threat_model_entries(risk_score DESC);
CREATE INDEX IF NOT EXISTS idx_threat_model_mitigation ON threat_model_entries(mitigation_status);

-- =============================================================================
-- Table 4: implementation_status (3 indexes)
-- =============================================================================

CREATE INDEX IF NOT EXISTS idx_implementation_status_col ON implementation_status(status);
CREATE INDEX IF NOT EXISTS idx_implementation_category ON implementation_status(category);
CREATE INDEX IF NOT EXISTS idx_implementation_completion ON implementation_status(completion_percentage DESC);

-- =============================================================================
-- Total: 17 indexes across 4 tables
-- =============================================================================
