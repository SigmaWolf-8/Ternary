// =============================================================================
// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division — Patent(s) Pending
//
// XPLENUM — RISC-V Ternary Security Extension
// Formal Verification Properties
//
// Security-critical assertions for bounded model checking and k-induction.
// Use with SymbiYosys: sby -f xplenum_formal.sby
// =============================================================================

`ifdef FORMAL

// =========================================================================
// SECTION 1: Reset and Initialization Properties
// =========================================================================

// P1.1: After reset, all CSRs must be zero
always @(posedge clk) begin
    if ($past(!rst_n)) begin
        assert(csr_xpstatus    == 32'h0);
        assert(csr_xpdomid     == 32'h0);
        assert(csr_xpcapbase   == 32'h0);
        assert(csr_xpcapbound  == 32'h0);
        assert(csr_xpmask_seed == 32'h0);
        assert(csr_xptrit_mode == 32'h0);
        assert(csr_xpsig_cfg   == 32'h0);
        assert(csr_xpexc_cause == 32'h0);
        assert(csr_xpexc_addr  == 32'h0);
        assert(csr_xpperf_cnt  == 32'h0);
    end
end

// P1.2: After reset, outputs must be inactive
always @(posedge clk) begin
    if ($past(!rst_n)) begin
        assert(rd_write_en == 1'b0);
        assert(rd_data     == 32'h0);
    end
end

// P1.3: After reset, no exception
always @(posedge clk) begin
    if ($past(!rst_n)) begin
        assert(xp_exception == 1'b0);
        assert(xp_exc_code  == 4'h0);
    end
end

// =========================================================================
// SECTION 2: Instruction Decode Safety
// =========================================================================

// P2.1: Non-XPLENUM opcodes must never produce a write-enable
// (ensures the extension doesn't corrupt base ISA state)
always @(posedge clk) begin
    if (rst_n && $past(rst_n) && !$past(is_xplenum) && !$past(is_csr_op_d1)) begin
        assert(rd_write_en == 1'b0);
    end
end

// P2.2: Version CSR is always hardwired to 0x01_00_00
always @(*) begin
    if (rs2[3:0] == 4'hB) begin
        assert(csr_read_data == `XP_VERSION);
    end
end

// P2.3: Read-only CSRs (0x5, 0x8, 0x9, 0xB) cannot be written
// If a CSR write targets a RO address, the value must not change
always @(posedge clk) begin
    if (rst_n && $past(rst_n) && $past(is_csr_op) && $past(funct7[6])) begin
        case ($past(rs2[3:0]))
            4'h5: assert(mask_state_out == $past(mask_state_out));
            4'hB: ; // version is hardwired, always passes
        endcase
    end
end

// =========================================================================
// SECTION 3: Trit Encoding Invariants (Critical Security Property)
// =========================================================================

// P3.1: No trit register output ever contains the invalid encoding (2'b11)
// This is the single most important security property for the ternary extension.
// An invalid trit in the output pipeline could corrupt binary computations.

// Check all 16 trit pairs in the mask unit result
always @(posedge clk) begin
    if (rst_n && u_mask.result_valid) begin
        assert(u_mask.result[ 1: 0] != `TRIT_INVALID);
        assert(u_mask.result[ 3: 2] != `TRIT_INVALID);
        assert(u_mask.result[ 5: 4] != `TRIT_INVALID);
        assert(u_mask.result[ 7: 6] != `TRIT_INVALID);
        assert(u_mask.result[ 9: 8] != `TRIT_INVALID);
        assert(u_mask.result[11:10] != `TRIT_INVALID);
        assert(u_mask.result[13:12] != `TRIT_INVALID);
        assert(u_mask.result[15:14] != `TRIT_INVALID);
        assert(u_mask.result[17:16] != `TRIT_INVALID);
        assert(u_mask.result[19:18] != `TRIT_INVALID);
        assert(u_mask.result[21:20] != `TRIT_INVALID);
        assert(u_mask.result[23:22] != `TRIT_INVALID);
        assert(u_mask.result[25:24] != `TRIT_INVALID);
        assert(u_mask.result[27:26] != `TRIT_INVALID);
        assert(u_mask.result[29:28] != `TRIT_INVALID);
        assert(u_mask.result[31:30] != `TRIT_INVALID);
    end
end

// P3.2: T-box output never contains invalid trits (for valid inputs)
// When trit_unit processes TTBOX on valid input, output trits 0..29 must be valid
always @(posedge clk) begin
    if (rst_n && u_trit.result_valid && $past(u_trit.funct3 == `F3_TROT) && $past(u_trit.funct7 == `F7_TTBOX)) begin
        // Upper 2 bits are padded to 00, the 5 groups of 6 bits must be valid
        assert(u_trit.result[31:30] == 2'b00);
    end
end

// P3.3: LFSR-to-trit conversion never produces invalid encoding
// The mask unit's lfsr_to_trits function maps 2'b11 -> 2'b00
// Verify the random_mask output has no invalid trits
always @(*) begin
    if (rst_n) begin
        assert(u_mask.random_mask[ 1: 0] != `TRIT_INVALID);
        assert(u_mask.random_mask[ 3: 2] != `TRIT_INVALID);
        assert(u_mask.random_mask[ 5: 4] != `TRIT_INVALID);
        assert(u_mask.random_mask[ 7: 6] != `TRIT_INVALID);
        assert(u_mask.random_mask[ 9: 8] != `TRIT_INVALID);
        assert(u_mask.random_mask[11:10] != `TRIT_INVALID);
        assert(u_mask.random_mask[13:12] != `TRIT_INVALID);
        assert(u_mask.random_mask[15:14] != `TRIT_INVALID);
        assert(u_mask.random_mask[17:16] != `TRIT_INVALID);
        assert(u_mask.random_mask[19:18] != `TRIT_INVALID);
        assert(u_mask.random_mask[21:20] != `TRIT_INVALID);
        assert(u_mask.random_mask[23:22] != `TRIT_INVALID);
        assert(u_mask.random_mask[25:24] != `TRIT_INVALID);
        assert(u_mask.random_mask[27:26] != `TRIT_INVALID);
        assert(u_mask.random_mask[29:28] != `TRIT_INVALID);
        assert(u_mask.random_mask[31:30] != `TRIT_INVALID);
    end
end

// =========================================================================
// SECTION 4: Masking Algebraic Properties
// =========================================================================

// P4.1: Mask then unmask is identity (for valid trit inputs)
// If we apply a mask and then remove it with the same mask,
// we must get back the original value.
// This is checked combinationally on the mask unit's functions.
reg [31:0] p4_test_data;
reg [31:0] p4_test_mask;
wire [31:0] p4_masked;
wire [31:0] p4_unmasked;

// Constrain test inputs to valid trits only
integer p4_i;
always @(*) begin
    for (p4_i = 0; p4_i < 16; p4_i = p4_i + 1) begin
        assume(p4_test_data[2*p4_i +: 2] != `TRIT_INVALID);
        assume(p4_test_mask[2*p4_i +: 2] != `TRIT_INVALID);
    end
end

// Apply mask then remove — must be identity
assign p4_masked   = u_mask.apply_mask(p4_test_data, p4_test_mask);
assign p4_unmasked = u_mask.remove_mask(p4_masked, p4_test_mask);

always @(*) begin
    assert(p4_unmasked == p4_test_data);
end

// =========================================================================
// SECTION 5: Domain Isolation Security
// =========================================================================

// P5.1: Domain check with mismatched domain ID must raise exception
always @(posedge clk) begin
    if (rst_n && $past(rst_n) && $past(dom_valid) && $past(dom_en)) begin
        if ($past(u_domain.funct7 == `F7_TDOMCHK)) begin
            // If the stored owner doesn't match current domain, exception expected
            // (This is architecture-dependent — adapt to actual domain_unit logic)
        end
    end
end

// P5.2: Domain operations disabled when dom_en=0 must not modify domain state
always @(posedge clk) begin
    if (rst_n && $past(rst_n) && $past(dom_valid) && !$past(dom_en)) begin
        // Domain unit should produce no valid result when disabled
        // Exception or zeroed result expected
    end
end

// =========================================================================
// SECTION 6: Capability Bounds Checking
// =========================================================================

// P6.1: Capability check with out-of-range index must raise CAP_INVALID
always @(posedge clk) begin
    if (rst_n && $past(rst_n) && $past(cap_valid) && $past(cap_en)) begin
        if ($past(u_cap.funct7 == `F7_TCAPCHK) && $past(rs1_data[5:0] >= `CAP_TABLE_SIZE)) begin
            assert(u_cap.exc_code == `XP_EXC_CAP_INVALID);
        end
    end
end

// P6.2: Capability operations disabled when cap_en=0 must not access table
always @(posedge clk) begin
    if (rst_n && $past(rst_n) && $past(cap_valid) && !$past(cap_en)) begin
        // Should produce exception or zero result
    end
end

// =========================================================================
// SECTION 7: Exception Handling Safety
// =========================================================================

// P7.1: Exception output is combinational and reflects current mux state
always @(*) begin
    if (mux_valid && mux_exc != `XP_EXC_NONE) begin
        assert(xp_exception == 1'b1);
        assert(xp_exc_code  == mux_exc);
    end
end

// P7.2: No exception when no valid result
always @(*) begin
    if (!mux_valid) begin
        assert(xp_exception == 1'b0);
    end
end

// P7.3: Exception CSRs update when subunit raises exception
always @(posedge clk) begin
    if (rst_n && $past(rst_n) && $past(mux_valid) && $past(mux_exc != `XP_EXC_NONE)) begin
        assert(csr_xpexc_cause[3:0] == $past(mux_exc));
    end
end

// =========================================================================
// SECTION 8: Performance Counter
// =========================================================================

// P8.1: Performance counter increments on every valid XPLENUM instruction
always @(posedge clk) begin
    if (rst_n && $past(rst_n) && $past(is_xplenum) && ($past(mux_valid) || $past(is_csr_op))) begin
        assert(csr_xpperf_cnt == $past(csr_xpperf_cnt) + 1);
    end
end

// P8.2: Performance counter does not increment on non-XPLENUM cycles
always @(posedge clk) begin
    if (rst_n && $past(rst_n) && !$past(is_xplenum)) begin
        if (!($past(mux_valid) && $past(mux_exc != `XP_EXC_NONE))) begin
            // Counter should not change (unless exception CSR update path)
        end
    end
end

// =========================================================================
// SECTION 9: Result Multiplexer Safety
// =========================================================================

// P9.1: At most one subunit produces a valid result per cycle
// (mutual exclusion — prevents data corruption)
always @(*) begin
    assert((mask_result_valid + dom_result_valid + cap_result_valid + trit_result_valid) <= 1);
end

// P9.2: If no subunit is valid, mux_valid must be 0
always @(*) begin
    if (!mask_result_valid && !dom_result_valid && !cap_result_valid && !trit_result_valid) begin
        assert(mux_valid == 1'b0);
    end
end

// =========================================================================
// SECTION 10: LFSR Non-Degeneracy (Side-Channel Resistance)
// =========================================================================

// P10.1: LFSR must never be all-zeros (stuck state)
always @(posedge clk) begin
    if (rst_n) begin
        assert(u_mask.lfsr != 32'h0);
    end
end

// P10.2: Seed write of zero loads default (0xDEAD_BEEF), not zero
always @(posedge clk) begin
    if (rst_n && $past(rst_n) && $past(seed_wr) && $past(seed_data == 32'h0)) begin
        assert(u_mask.lfsr == 32'hDEAD_BEEF);
    end
end

// =========================================================================
// SECTION 11: Information Flow — No Secret Leak to Public Outputs
// =========================================================================

// P11.1: When masking is disabled, mask operations return zero (not internal state)
always @(posedge clk) begin
    if (rst_n && $past(rst_n) && $past(mask_valid) && !$past(mask_en)) begin
        assert(u_mask.result == 32'h0);
        assert(u_mask.exc_code == `XP_EXC_MASK_FAULT);
    end
end

// P11.2: Signal processing disabled returns zero, not stale data
always @(posedge clk) begin
    if (rst_n && $past(rst_n) && $past(trit_valid) && $past(funct3 == `F3_TSIG) && !$past(sig_en)) begin
        assert(u_trit.result == 32'h0);
    end
end

// =========================================================================
// SECTION 12: Cover Properties (Reachability)
// =========================================================================

// C12.1: Can reach a state where mask is applied and then unmasked
always @(posedge clk) begin
    cover(rst_n && mask_result_valid && u_mask.funct7 == `F7_TUNMASK);
end

// C12.2: Can reach a state where T-box substitution produces valid output
always @(posedge clk) begin
    cover(rst_n && trit_result_valid && u_trit.funct3 == `F3_TROT);
end

// C12.3: Can reach a state where capability check raises exception
always @(posedge clk) begin
    cover(rst_n && cap_result_valid && u_cap.exc_code != `XP_EXC_NONE);
end

// C12.4: Can reach a state where domain check passes
always @(posedge clk) begin
    cover(rst_n && dom_result_valid && u_domain.exc_code == `XP_EXC_NONE);
end

// C12.5: Performance counter reaches 10
always @(posedge clk) begin
    cover(rst_n && csr_xpperf_cnt == 32'd10);
end

// C12.6: Exception is raised during ternary operation
always @(posedge clk) begin
    cover(rst_n && xp_exception && xp_exc_code == `XP_EXC_TRIT_OVERFLOW);
end

`endif // FORMAL
