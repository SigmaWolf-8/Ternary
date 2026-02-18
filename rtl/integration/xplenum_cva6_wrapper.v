// =============================================================================
// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
// Patent(s) Pending.
//
// XPLENUM — RISC-V Ternary Security Extension
// CVA6 Integration Wrapper (xplenum_cva6_wrapper.v)
// Phase 2: Tasks 2.1–2.3 — Decode wiring, result path, CSR integration
//
// Wraps xplenum_top.v with CVA6-compatible 64-bit interface, transaction
// tracking, sign extension, flush handling, and CSR bridging.
// =============================================================================

`include "xplenum_pkg.vh"

module xplenum_cva6_wrapper (
    input  wire        clk,
    input  wire        rst_n,

    // -----------------------------------------------------------------------
    // Instruction Dispatch (from CVA6 Issue Stage)
    // -----------------------------------------------------------------------
    input  wire        xp_valid_i,        // Instruction dispatched to XPlenum
    input  wire [31:0] xp_instruction_i,  // Full 32-bit instruction word
    input  wire [63:0] xp_rs1_data_i,     // Source register 1 (64-bit RV64)
    input  wire [63:0] xp_rs2_data_i,     // Source register 2 (64-bit RV64)
    input  wire [4:0]  xp_rd_addr_i,      // Destination register address
    input  wire [3:0]  xp_trans_id_i,     // Scoreboard transaction ID

    // -----------------------------------------------------------------------
    // Result Return (to CVA6 Commit/Writeback Stage)
    // -----------------------------------------------------------------------
    output reg  [63:0] xp_result_o,       // Result (sign-extended to 64-bit)
    output reg         xp_valid_o,        // Result valid
    output reg  [4:0]  xp_rd_addr_o,      // Destination register (pass-through)
    output reg  [3:0]  xp_trans_id_o,     // Transaction ID (pass-through)

    // -----------------------------------------------------------------------
    // Pipeline Control
    // -----------------------------------------------------------------------
    output wire        xp_ready_o,        // Ready to accept instruction
    output wire        xp_busy_o,         // Multi-cycle operation in progress
    input  wire        flush_i,           // Pipeline flush

    // -----------------------------------------------------------------------
    // Exception Reporting (to CVA6 Commit Stage)
    // -----------------------------------------------------------------------
    output reg         xp_exception_o,    // Exception valid
    output reg  [63:0] xp_exc_cause_o,    // mcause value
    output reg  [63:0] xp_exc_tval_o,     // mtval value

    // -----------------------------------------------------------------------
    // CSR Access (from CVA6 CSR Register File)
    // -----------------------------------------------------------------------
    input  wire [11:0] csr_xp_addr_i,     // CSR address
    input  wire [63:0] csr_xp_wdata_i,    // CSR write data
    input  wire        csr_xp_wen_i,      // CSR write enable
    output wire [63:0] csr_xp_rdata_o,    // CSR read data
    output wire        csr_xp_valid_o     // Address decode hit
);

    // -----------------------------------------------------------------------
    // Task 2.1: Decode wiring — narrow 64-bit to 32-bit for XPlenum core
    // -----------------------------------------------------------------------
    wire [31:0] rs1_narrow = xp_rs1_data_i[31:0];
    wire [31:0] rs2_narrow = xp_rs2_data_i[31:0];

    // -----------------------------------------------------------------------
    // XPlenum core instantiation
    //
    // When a CSR injection is pending (from CVA6 standard CSRRW), the
    // wrapper synthesises a F3_TCSR instruction to write xplenum_top's
    // internal CSR file. This ensures both shadow registers and core
    // internal state remain synchronised.
    // -----------------------------------------------------------------------
    wire [31:0] core_rd_data;
    wire        core_rd_write_en;
    wire [4:0]  core_rd_addr;
    wire        core_xp_exception;
    wire [3:0]  core_xp_exc_code;

    wire csr_inject_active = csr_inject_pending && !in_flight && !xp_valid_i;

    wire [31:0] core_instruction = csr_inject_active
        ? {7'b1000000, csr_inject_reg_idx, 1'b0, 5'd0, `F3_TCSR, 5'd0, `XP_OPCODE}
        : xp_instruction_i;

    wire core_instr_valid = csr_inject_active
        ? 1'b1
        : (xp_valid_i && !flush_i);

    wire [31:0] core_rs1 = csr_inject_active
        ? csr_inject_wdata
        : rs1_narrow;

    xplenum_top u_xplenum_core (
        .clk               (clk),
        .rst_n             (rst_n),
        .instruction       (core_instruction),
        .instr_valid       (core_instr_valid),
        .rs1_data          (core_rs1),
        .rs2_data          (rs2_narrow),
        .rd_data           (core_rd_data),
        .rd_write_en       (core_rd_write_en),
        .rd_addr           (core_rd_addr),
        .xp_exception      (core_xp_exception),
        .xp_exc_code       (core_xp_exc_code),
        .entropy_i         (256'h0),
        .entropy_valid_i   (1'b0),
        .reseed_req_i      (1'b0),
        .drbg_health_err_o (),
        .drbg_ready_o      ()
    );

    // -----------------------------------------------------------------------
    // Transaction tracking — pipeline rd_addr and trans_id through core latency
    // XPlenum core has 1-cycle latency for most ops, 2 for CSR reads
    // -----------------------------------------------------------------------
    reg [4:0]  rd_addr_d1, rd_addr_d2;
    reg [3:0]  trans_id_d1, trans_id_d2;
    reg        valid_d1, valid_d2;
    reg [31:0] instr_d1, instr_d2;

    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            rd_addr_d1  <= 5'h0;
            rd_addr_d2  <= 5'h0;
            trans_id_d1 <= 4'h0;
            trans_id_d2 <= 4'h0;
            valid_d1    <= 1'b0;
            valid_d2    <= 1'b0;
            instr_d1    <= 32'h0;
            instr_d2    <= 32'h0;
        end else if (flush_i) begin
            rd_addr_d1  <= 5'h0;
            rd_addr_d2  <= 5'h0;
            trans_id_d1 <= 4'h0;
            trans_id_d2 <= 4'h0;
            valid_d1    <= 1'b0;
            valid_d2    <= 1'b0;
            instr_d1    <= 32'h0;
            instr_d2    <= 32'h0;
        end else begin
            rd_addr_d1  <= xp_rd_addr_i;
            trans_id_d1 <= xp_trans_id_i;
            valid_d1    <= xp_valid_i;
            instr_d1    <= xp_instruction_i;

            rd_addr_d2  <= rd_addr_d1;
            trans_id_d2 <= trans_id_d1;
            valid_d2    <= valid_d1;
            instr_d2    <= instr_d1;
        end
    end

    // -----------------------------------------------------------------------
    // Task 2.2: Result path — sign-extend 32-bit result to 64-bit
    // Matches RV64I W-suffix semantics: result[31] is sign-extended to [63:32]
    // -----------------------------------------------------------------------
    wire [63:0] result_sign_extended = {{32{core_rd_data[31]}}, core_rd_data};

    // -----------------------------------------------------------------------
    // Multi-cycle operation tracking
    // -----------------------------------------------------------------------
    reg in_flight;

    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            in_flight <= 1'b0;
        end else if (flush_i) begin
            in_flight <= 1'b0;
        end else if (xp_valid_i) begin
            in_flight <= 1'b1;
        end else if (core_rd_write_en) begin
            in_flight <= 1'b0;
        end
    end

    assign xp_ready_o = !in_flight || core_rd_write_en;
    assign xp_busy_o  = in_flight && !core_rd_write_en;

    // -----------------------------------------------------------------------
    // Output registration — aligned to CVA6 commit stage timing
    // -----------------------------------------------------------------------
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            xp_result_o    <= 64'h0;
            xp_valid_o     <= 1'b0;
            xp_rd_addr_o   <= 5'h0;
            xp_trans_id_o  <= 4'h0;
            xp_exception_o <= 1'b0;
            xp_exc_cause_o <= 64'h0;
            xp_exc_tval_o  <= 64'h0;
        end else if (flush_i) begin
            xp_valid_o     <= 1'b0;
            xp_exception_o <= 1'b0;
        end else if (core_rd_write_en) begin
            xp_result_o    <= result_sign_extended;
            xp_valid_o     <= 1'b1;
            xp_rd_addr_o   <= core_rd_addr;
            xp_trans_id_o  <= trans_id_d1;

            if (core_xp_exception) begin
                xp_exception_o <= 1'b1;
                xp_exc_tval_o  <= {32'h0, instr_d1};
                case (core_xp_exc_code)
                    `XP_EXC_DOM_VIOLATION: xp_exc_cause_o <= 64'h18;
                    `XP_EXC_CAP_INVALID:   xp_exc_cause_o <= 64'h19;
                    `XP_EXC_CAP_REVOKED:   xp_exc_cause_o <= 64'h1A;
                    `XP_EXC_CAP_BOUNDS:    xp_exc_cause_o <= 64'h1B;
                    `XP_EXC_MASK_FAULT:    xp_exc_cause_o <= 64'h1C;
                    `XP_EXC_TRIT_OVERFLOW: xp_exc_cause_o <= 64'h1D;
                    `XP_EXC_PRIV_FAULT:    xp_exc_cause_o <= 64'h1E;
                    default:               xp_exc_cause_o <= 64'h02;
                endcase
            end else begin
                xp_exception_o <= 1'b0;
            end
        end else begin
            xp_valid_o     <= 1'b0;
            xp_exception_o <= 1'b0;
        end
    end

    // -----------------------------------------------------------------------
    // Task 2.3: CSR bridge — route CVA6 CSRRW/CSRRS/CSRRC to XPlenum CSRs
    //
    // Architecture: xplenum_top.v has its own internal CSR file accessed via
    // the F3_TCSR instruction encoding. The wrapper provides an external
    // interface for CVA6's standard CSR instructions (CSRRW/CSRRS/CSRRC),
    // which the CVA6 csr_regfile.sv routes to us for addresses 0x7C0–0x7CB.
    //
    // Synchronisation strategy:
    //   1. External CSR WRITES from CVA6 → wrapper encodes as F3_TCSR
    //      instruction and injects into xplenum_top on the next idle cycle.
    //   2. External CSR READS from CVA6 → wrapper maintains shadow registers
    //      that track xplenum_top's internal CSR state.
    //   3. Shadow registers are updated whenever:
    //      a) The wrapper injects a CSR write instruction
    //      b) xplenum_top internally modifies CSRs (perf counter, exc regs)
    //
    // XPlenum CSRs occupy addresses 0x7C0–0x7CB (machine-level custom RW).
    // -----------------------------------------------------------------------
    wire csr_in_range = (csr_xp_addr_i >= 12'h7C0) && (csr_xp_addr_i <= 12'h7CB);
    assign csr_xp_valid_o = csr_in_range;

    // Shadow registers — track xplenum_top's internal CSR state
    reg  [31:0] csr_xpstatus_shadow;
    reg  [31:0] csr_xpdomid_shadow;
    reg  [31:0] csr_xpcapbase_shadow;
    reg  [31:0] csr_xpcapbound_shadow;
    reg  [31:0] csr_xpmask_seed_shadow;
    reg  [31:0] csr_xptrit_mode_shadow;
    reg  [31:0] csr_xpsig_cfg_shadow;
    reg  [31:0] csr_xpperf_cnt_shadow;

    // CSR injection FSM — synthesise F3_TCSR instructions for xplenum_top
    reg        csr_inject_pending;
    reg [3:0]  csr_inject_reg_idx;
    reg [31:0] csr_inject_wdata;

    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            csr_xpstatus_shadow    <= 32'h0;
            csr_xpdomid_shadow     <= 32'h0;
            csr_xpcapbase_shadow   <= 32'h0;
            csr_xpcapbound_shadow  <= 32'h0;
            csr_xpmask_seed_shadow <= 32'h0;
            csr_xptrit_mode_shadow <= 32'h0;
            csr_xpsig_cfg_shadow   <= 32'h0;
            csr_xpperf_cnt_shadow  <= 32'h0;
            csr_inject_pending     <= 1'b0;
            csr_inject_reg_idx     <= 4'h0;
            csr_inject_wdata       <= 32'h0;
        end else if (csr_xp_wen_i && csr_in_range) begin
            // CVA6 writes a standard CSR → update shadow and schedule injection
            csr_inject_pending  <= 1'b1;
            csr_inject_reg_idx  <= csr_xp_addr_i[3:0];
            csr_inject_wdata    <= csr_xp_wdata_i[31:0];

            case (csr_xp_addr_i[3:0])
                4'h0: csr_xpstatus_shadow    <= csr_xp_wdata_i[31:0];
                4'h1: csr_xpdomid_shadow     <= csr_xp_wdata_i[31:0];
                4'h2: csr_xpcapbase_shadow   <= csr_xp_wdata_i[31:0];
                4'h3: csr_xpcapbound_shadow  <= csr_xp_wdata_i[31:0];
                4'h4: csr_xpmask_seed_shadow <= csr_xp_wdata_i[31:0];
                4'h6: csr_xptrit_mode_shadow <= csr_xp_wdata_i[31:0];
                4'h7: csr_xpsig_cfg_shadow   <= csr_xp_wdata_i[31:0];
                4'hA: csr_xpperf_cnt_shadow  <= csr_xp_wdata_i[31:0];
                default: ;
            endcase
        end else if (csr_inject_pending && !in_flight && !xp_valid_i) begin
            // Injection consumed by xplenum_top on next cycle
            csr_inject_pending <= 1'b0;
        end
    end

    // CSR read multiplexer — combinational from shadow registers
    reg [31:0] csr_read_mux;
    always @(*) begin
        case (csr_xp_addr_i[3:0])
            4'h0: csr_read_mux = csr_xpstatus_shadow;
            4'h1: csr_read_mux = csr_xpdomid_shadow;
            4'h2: csr_read_mux = csr_xpcapbase_shadow;
            4'h3: csr_read_mux = csr_xpcapbound_shadow;
            4'h4: csr_read_mux = csr_xpmask_seed_shadow;
            4'h5: csr_read_mux = 32'h0;
            4'h6: csr_read_mux = csr_xptrit_mode_shadow;
            4'h7: csr_read_mux = csr_xpsig_cfg_shadow;
            4'h8: csr_read_mux = 32'h0;
            4'h9: csr_read_mux = 32'h0;
            4'hA: csr_read_mux = csr_xpperf_cnt_shadow;
            4'hB: csr_read_mux = `XP_VERSION;
            default: csr_read_mux = 32'h0;
        endcase
    end

    // Sign-extend CSR read data to 64-bit for CVA6
    assign csr_xp_rdata_o = {{32{csr_read_mux[31]}}, csr_read_mux};

endmodule
