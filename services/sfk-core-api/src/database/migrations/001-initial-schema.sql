-- SFK Core API Database Schema
-- Version: 1.0.0
-- Description: Initial schema for SFK operations and blockchain references

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Operations table
CREATE TABLE IF NOT EXISTS sfk_operations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    batch_ref VARCHAR(64) NOT NULL,
    kernel_op_id VARCHAR(64) NOT NULL,
    operation_type VARCHAR(32) NOT NULL CHECK (operation_type IN ('payment_witness', 'data_attest', 'state_transition', 'consensus_round')),
    security_mode VARCHAR(16) NOT NULL DEFAULT 'phi_plus' CHECK (security_mode IN ('mode_zero', 'mode_one', 'phi_plus')),
    status VARCHAR(16) NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'queued', 'processing', 'witnessed', 'settled', 'failed', 'cancelled')),
    priority VARCHAR(8) NOT NULL DEFAULT 'normal' CHECK (priority IN ('low', 'normal', 'high', 'critical')),
    
    -- Timing metadata
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    started_at TIMESTAMP WITH TIME ZONE,
    completed_at TIMESTAMP WITH TIME ZONE,
    queued_at TIMESTAMP WITH TIME ZONE,
    witnessed_at TIMESTAMP WITH TIME ZONE,
    settled_at TIMESTAMP WITH TIME ZONE,
    total_latency_ns BIGINT,
    
    -- Payment context (nullable)
    payment_gateway VARCHAR(16) CHECK (payment_gateway IN ('stripe', 'interac', 'crypto')),
    payment_id VARCHAR(128),
    payment_amount DECIMAL(20, 8),
    payment_currency VARCHAR(3),
    payment_metadata JSONB,
    
    -- Ternary context (nullable)
    ternary_representation CHAR(1) CHECK (ternary_representation IN ('A', 'B', 'C')),
    ternary_data_hash VARCHAR(128),
    ternary_compression_enabled BOOLEAN DEFAULT true,
    
    -- Configuration
    callback_url TEXT,
    idempotency_key VARCHAR(128) UNIQUE,
    ttl_seconds INTEGER DEFAULT 3600,
    
    -- Error info (nullable)
    error_code VARCHAR(64),
    error_message TEXT,
    error_retryable BOOLEAN,
    retry_count INTEGER DEFAULT 0,
    max_retries INTEGER DEFAULT 3,
    
    -- Indexes for common queries
    CONSTRAINT unique_batch_kernel UNIQUE (batch_ref, kernel_op_id)
);

-- Blockchain references table
CREATE TABLE IF NOT EXISTS blockchain_refs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    operation_id UUID NOT NULL REFERENCES sfk_operations(id) ON DELETE CASCADE,
    blockchain_type VARCHAR(16) NOT NULL CHECK (blockchain_type IN ('hedera', 'xrpl', 'algorand')),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    
    -- Hedera fields
    hedera_topic_id VARCHAR(32),
    hedera_sequence_number BIGINT,
    hedera_transaction_id VARCHAR(128),
    hedera_consensus_timestamp TIMESTAMP WITH TIME ZONE,
    hedera_running_hash VARCHAR(128),
    
    -- XRPL fields
    xrpl_ledger_index BIGINT,
    xrpl_transaction_hash VARCHAR(128),
    xrpl_validated BOOLEAN,
    xrpl_fee VARCHAR(32),
    xrpl_destination_tag INTEGER,
    
    -- Algorand fields
    algorand_app_id BIGINT,
    algorand_round BIGINT,
    algorand_tx_id VARCHAR(64),
    algorand_group_id VARCHAR(64),
    algorand_inner_tx_count INTEGER,
    
    CONSTRAINT unique_operation_blockchain UNIQUE (operation_id, blockchain_type)
);

-- Audit trail table
CREATE TABLE IF NOT EXISTS audit_entries (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    operation_id UUID NOT NULL REFERENCES sfk_operations(id) ON DELETE CASCADE,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    event VARCHAR(64) NOT NULL,
    actor VARCHAR(64) NOT NULL,
    details JSONB NOT NULL DEFAULT '{}',
    signature VARCHAR(256),
    
    -- Index for time-based queries
    CONSTRAINT audit_operation_time UNIQUE (operation_id, timestamp, event)
);

-- Timing certifications table
CREATE TABLE IF NOT EXISTS timing_certifications (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    operation_id UUID NOT NULL REFERENCES sfk_operations(id) ON DELETE CASCADE,
    certificate_id VARCHAR(64) NOT NULL UNIQUE,
    certified_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    valid_until TIMESTAMP WITH TIME ZONE NOT NULL,
    accuracy VARCHAR(32) NOT NULL,
    clock_source VARCHAR(32) NOT NULL,
    synchronized BOOLEAN NOT NULL,
    offset_ns BIGINT,
    jitter_ns BIGINT
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_operations_status ON sfk_operations(status);
CREATE INDEX IF NOT EXISTS idx_operations_type ON sfk_operations(operation_type);
CREATE INDEX IF NOT EXISTS idx_operations_created ON sfk_operations(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_operations_batch_ref ON sfk_operations(batch_ref);
CREATE INDEX IF NOT EXISTS idx_operations_payment_gateway ON sfk_operations(payment_gateway) WHERE payment_gateway IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_blockchain_refs_operation ON blockchain_refs(operation_id);
CREATE INDEX IF NOT EXISTS idx_audit_entries_operation ON audit_entries(operation_id);
CREATE INDEX IF NOT EXISTS idx_timing_certs_operation ON timing_certifications(operation_id);

-- Trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_sfk_operations_updated_at
    BEFORE UPDATE ON sfk_operations
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- View for operation status summary
CREATE OR REPLACE VIEW operation_status_summary AS
SELECT 
    status,
    operation_type,
    COUNT(*) as count,
    AVG(EXTRACT(EPOCH FROM (completed_at - created_at)) * 1000) as avg_completion_ms
FROM sfk_operations
GROUP BY status, operation_type;

-- View for recent operations
CREATE OR REPLACE VIEW recent_operations AS
SELECT 
    o.*,
    (SELECT jsonb_agg(jsonb_build_object(
        'blockchain_type', br.blockchain_type,
        'hedera_transaction_id', br.hedera_transaction_id,
        'xrpl_transaction_hash', br.xrpl_transaction_hash,
        'algorand_tx_id', br.algorand_tx_id
    )) FROM blockchain_refs br WHERE br.operation_id = o.id) as blockchain_refs
FROM sfk_operations o
WHERE o.created_at > NOW() - INTERVAL '24 hours'
ORDER BY o.created_at DESC
LIMIT 100;

COMMENT ON TABLE sfk_operations IS 'Main table for SFK kernel operations';
COMMENT ON TABLE blockchain_refs IS 'Blockchain references for witnessed/settled operations';
COMMENT ON TABLE audit_entries IS 'Immutable audit trail for all operation events';
COMMENT ON TABLE timing_certifications IS 'Femtosecond timing certifications for regulatory compliance';
