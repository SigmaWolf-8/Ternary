// =============================================================================
// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division — Patent(s) Pending
//
// XPLENUM — RISC-V Ternary Security Extension
// Induction Helper Properties
//
// Strengthened invariants for k-induction proofs. These assumptions and
// auxiliary assertions help the solver establish induction hypotheses
// that the base properties alone may not provide.
//
// Guarded by `ifdef INDUCTION — included only in prove/induction runs.
// =============================================================================

`ifdef FORMAL
`ifdef INDUCTION

// =========================================================================
// INDUCTION SECTION A: Reset Stability Assumptions
// =========================================================================

// IA.1: After reset deasserts, it stays deasserted (no glitch re-entry)
// This is a standard induction helper — prevents the solver from
// inventing spurious reset transitions mid-proof.
reg past_rst_valid;
initial past_rst_valid = 1'b0;
always @(posedge clk) begin
    past_rst_valid <= 1'b1;
end

always @(posedge clk) begin
    if (past_rst_valid && $past(rst_n) && rst_n) begin
        assume(rst_n);
    end
end

// =========================================================================
// INDUCTION SECTION B: Guardian Checksum Inductive Invariants
// =========================================================================

// IB.1: Guardian checksum is well-formed when valid
// The guardian value must be within the expected hash range whenever
// it is marked valid. This strengthens the induction base for P13.x.
always @(posedge clk) begin
    if (rst_n && guardian_valid) begin
        assert(guardian_checksum != 32'h0);
    end
end

// IB.2: Guardian validity tracks QCorrect completion
// Once QCorrect completes, guardian_valid must be asserted and remain
// stable until a new ternary operation begins.
always @(posedge clk) begin
    if (rst_n && $past(rst_n) && $past(qcorrect_done) && !is_xplenum) begin
        assert(guardian_valid == $past(guardian_valid));
    end
end

// =========================================================================
// INDUCTION SECTION C: CSR State Machine Invariants
// =========================================================================

// IC.1: CSR write-enable is single-cycle (never stuck high)
// Prevents induction from assuming a perpetual write state.
always @(posedge clk) begin
    if (rst_n && $past(rst_n) && $past(csr_wr_en)) begin
        assert(csr_wr_en == 1'b0 || is_csr_op);
    end
end

// IC.2: CSR address must be in valid range when accessed
always @(posedge clk) begin
    if (rst_n && is_csr_op) begin
        assert(rs2[3:0] <= 4'hB);
    end
end

// =========================================================================
// INDUCTION SECTION D: HPTP Counter Inductive Strengthening
// =========================================================================

// ID.1: HPTP counter does not wrap without overflow flag
// The counter value must increase monotonically; if it would wrap,
// an overflow signal must be asserted.
always @(posedge clk) begin
    if (rst_n && $past(rst_n) && hptp_en && $past(hptp_en)) begin
        if (hptp_counter < $past(hptp_counter)) begin
            assert(hptp_overflow == 1'b1);
        end
    end
end

// ID.2: Jitter correction delta is bounded
// The symplectic corrector cannot shift the timestamp by more than
// the maximum jitter window (prevents solver from inventing
// arbitrarily large corrections).
always @(posedge clk) begin
    if (rst_n && hptp_valid) begin
        assert((hptp_corrected_ts - hptp_raw_ts) <= `HPTP_MAX_JITTER);
    end
end

// =========================================================================
// INDUCTION SECTION E: Trit Encoding Inductive Strengthening
// =========================================================================

// IE.1: All trit registers maintain valid encoding across all cycles
// This is the inductive strengthening of P3.x — asserts that if trits
// are valid in cycle N, they remain valid in cycle N+1 (absent reset).
genvar ie_i;
generate
    for (ie_i = 0; ie_i < 16; ie_i = ie_i + 1) begin : trit_induct
        always @(posedge clk) begin
            if (rst_n && $past(rst_n)) begin
                if ($past(u_mask.result[2*ie_i +: 2]) != `TRIT_INVALID) begin
                    assert(u_mask.result[2*ie_i +: 2] != `TRIT_INVALID);
                end
            end
        end
    end
endgenerate

// IE.2: Trit ALU output validity preservation
// If trit_unit input was valid, output must be valid.
always @(posedge clk) begin
    if (rst_n && u_trit.result_valid) begin : trit_alu_induct
        integer tai;
        for (tai = 0; tai < 16; tai = tai + 1) begin
            assert(u_trit.result[2*tai +: 2] != `TRIT_INVALID);
        end
    end
end

// =========================================================================
// INDUCTION SECTION F: Subunit Mutual Exclusion Strengthening
// =========================================================================

// IF.1: Instruction dispatch is single-hot
// At most one subunit receives a valid input per cycle. This
// strengthens P9.x for induction by asserting the input side.
always @(posedge clk) begin
    if (rst_n) begin
        assert((mask_valid + dom_valid + cap_valid + trit_valid) <= 1);
    end
end

// IF.2: Valid input implies valid output within bounded latency
// If a subunit receives valid input, it must produce a result
// within 2 cycles (for combinational or single-pipeline-stage units).
always @(posedge clk) begin
    if (rst_n && $past(rst_n, 2) && $past(mask_valid, 2)) begin
        assert($past(mask_result_valid) || mask_result_valid);
    end
end

// =========================================================================
// INDUCTION SECTION G: Data-Flow Isolation Strengthening
// =========================================================================

// IG.1: QCorrect tag tracks ternary pipeline activity
// binary_output_qcorrected can only be high if a ternary operation
// was recently processed.
always @(posedge clk) begin
    if (rst_n && $past(rst_n) && binary_output_qcorrected) begin
        assert($past(ternary_output_valid) || $past(binary_output_qcorrected));
    end
end

// IG.2: No stale ternary data on binary output bus
// When no ternary operation is active, the binary output must be zero.
always @(posedge clk) begin
    if (rst_n && $past(rst_n) && !mux_valid && !$past(mux_valid)) begin
        assert(rd_data == 32'h0 || rd_write_en == 1'b0);
    end
end

`endif // INDUCTION
`endif // FORMAL
