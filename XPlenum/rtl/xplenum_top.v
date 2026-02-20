// =============================================================================
// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
// Patent(s) Pending.
//
// XPLENUM — RISC-V Ternary Security Extension
// Top-Level Module (xplenum_top.v)
// Stage 7 — Pipeline integration, CSR file, instruction decode, result mux
// =============================================================================

`include "xplenum_pkg.vh"

module xplenum_top (
    input  wire        clk,
    input  wire        rst_n,

    // RISC-V pipeline interface
    input  wire [31:0] instruction,    // Full 32-bit instruction word
    input  wire        instr_valid,    // Instruction valid from decode stage
    input  wire [31:0] rs1_data,       // Source register 1 data
    input  wire [31:0] rs2_data,       // Source register 2 data

    // Result interface
    output reg  [31:0] rd_data,        // Destination register data
    output reg         rd_write_en,    // Write-enable to register file
    output reg  [4:0]  rd_addr,        // Destination register address

    // Exception interface
    output wire        xp_exception,   // Exception signal to core
    output wire [3:0]  xp_exc_code,    // Exception cause code

    // External entropy source for CTR_DRBG (optional)
    input  wire [255:0] entropy_i,     // 256-bit entropy from external TRNG
    input  wire         entropy_valid_i, // Entropy valid strobe
    input  wire         reseed_req_i,  // External reseed request

    // DRBG status outputs
    output wire         drbg_health_err_o, // Health test failure
    output wire         drbg_ready_o       // DRBG ready for generation
);

    // -----------------------------------------------------------------------
    // Instruction decode
    // -----------------------------------------------------------------------
    wire [6:0]  opcode  = instruction[6:0];
    wire [4:0]  rd      = instruction[11:7];
    wire [2:0]  funct3  = instruction[14:12];
    wire [4:0]  rs1     = instruction[19:15];
    wire [4:0]  rs2     = instruction[24:20];
    wire [6:0]  funct7  = instruction[31:25];

    wire is_xplenum     = (opcode == `XP_OPCODE) && instr_valid;
    wire is_csr_op      = is_xplenum && (funct3 == `F3_TCSR);

    // -----------------------------------------------------------------------
    // CSR File (0x7C0 – 0x7CB)
    // -----------------------------------------------------------------------
    reg [31:0] csr_xpstatus;       // 0x7C0
    reg [31:0] csr_xpdomid;        // 0x7C1
    reg [31:0] csr_xpcapbase;      // 0x7C2
    reg [31:0] csr_xpcapbound;     // 0x7C3
    reg [31:0] csr_xpmask_seed;    // 0x7C4
    // csr_xpmask_state (0x7C5) — read from mask unit
    reg [31:0] csr_xptrit_mode;    // 0x7C6
    reg [31:0] csr_xpsig_cfg;      // 0x7C7
    reg [31:0] csr_xpexc_cause;    // 0x7C8
    reg [31:0] csr_xpexc_addr;     // 0x7C9
    reg [31:0] csr_xpperf_cnt;     // 0x7CA
    // csr_xpversion (0x7CB) — hardwired

    wire mask_en = csr_xpstatus[`XPSTATUS_MASK_EN];
    wire dom_en  = csr_xpstatus[`XPSTATUS_DOM_EN];
    wire cap_en  = csr_xpstatus[`XPSTATUS_CAP_EN];
    wire sig_en  = csr_xpstatus[`XPSTATUS_SIG_EN];

    // -----------------------------------------------------------------------
    // Subunit dispatch signals
    // -----------------------------------------------------------------------
    wire mask_valid = is_xplenum && (funct3 == `F3_TMASK);
    wire dom_valid  = is_xplenum && (funct3 == `F3_TDOM);
    wire cap_valid  = is_xplenum && (funct3 == `F3_TCAP);
    wire trit_valid = is_xplenum && (funct3 == `F3_TROT ||
                                     funct3 == `F3_TENC ||
                                     funct3 == `F3_TSIG);

    // Seed write pulse
    wire seed_wr = is_csr_op && funct7[6] && (rs2[3:0] == 4'h4);

    // -----------------------------------------------------------------------
    // Subunit instances
    // -----------------------------------------------------------------------

    // Mask Unit (with CTR_DRBG for FIPS 140-3 compliance)
    wire [31:0] mask_result;
    wire        mask_result_valid;
    wire [31:0] mask_state_out;
    wire [3:0]  mask_exc;
    wire        drbg_health_error;
    wire        drbg_ready;

    xplenum_mask_unit u_mask (
        .clk               (clk),
        .rst_n             (rst_n),
        .mask_en           (mask_en),
        .funct7            (funct7),
        .valid             (mask_valid),
        .rs1_data          (rs1_data),
        .rs2_data          (rs2_data),
        .seed_wr           (seed_wr),
        .seed_data         (rs1_data),
        .seed_full_i       (entropy_i),
        .seed_full_valid_i (entropy_valid_i),
        .reseed_i          (reseed_req_i),
        .drbg_health_error_o (drbg_health_error),
        .drbg_ready_o      (drbg_ready),
        .result            (mask_result),
        .result_valid      (mask_result_valid),
        .mask_state        (mask_state_out),
        .exc_code          (mask_exc)
    );

    // Domain Unit
    wire [31:0] dom_result;
    wire        dom_result_valid;
    wire [3:0]  dom_exc;

    xplenum_domain_unit u_domain (
        .clk           (clk),
        .rst_n         (rst_n),
        .dom_en        (dom_en),
        .funct7        (funct7),
        .valid         (dom_valid),
        .current_dom_id(csr_xpdomid[7:0]),
        .rs1_data      (rs1_data),
        .rs2_data      (rs2_data),
        .result        (dom_result),
        .result_valid  (dom_result_valid),
        .exc_code      (dom_exc)
    );

    // Capability Unit
    wire [31:0] cap_result;
    wire        cap_result_valid;
    wire [3:0]  cap_exc;

    xplenum_cap_unit u_cap (
        .clk        (clk),
        .rst_n      (rst_n),
        .cap_en     (cap_en),
        .funct7     (funct7),
        .valid      (cap_valid),
        .rs1_data   (rs1_data),
        .rs2_data   (rs2_data),
        .result     (cap_result),
        .result_valid(cap_result_valid),
        .exc_code   (cap_exc)
    );

    // Trit/Crypto/Signal Unit
    wire [31:0] trit_result;
    wire        trit_result_valid;
    wire [3:0]  trit_exc;

    xplenum_trit_unit u_trit (
        .clk        (clk),
        .rst_n      (rst_n),
        .sig_en     (sig_en),
        .funct3     (funct3),
        .funct7     (funct7),
        .valid      (trit_valid),
        .rs1_data   (rs1_data),
        .rs2_data   (rs2_data),
        .sig_cfg    (csr_xpsig_cfg),
        .result     (trit_result),
        .result_valid(trit_result_valid),
        .exc_code   (trit_exc)
    );

    // -----------------------------------------------------------------------
    // Result multiplexer
    // -----------------------------------------------------------------------
    reg  [31:0] mux_result;
    reg         mux_valid;
    reg  [3:0]  mux_exc;

    always @(*) begin
        mux_result = 32'h0;
        mux_valid  = 1'b0;
        mux_exc    = `XP_EXC_NONE;

        if (mask_result_valid) begin
            mux_result = mask_result;
            mux_valid  = 1'b1;
            mux_exc    = mask_exc;
        end else if (dom_result_valid) begin
            mux_result = dom_result;
            mux_valid  = 1'b1;
            mux_exc    = dom_exc;
        end else if (cap_result_valid) begin
            mux_result = cap_result;
            mux_valid  = 1'b1;
            mux_exc    = cap_exc;
        end else if (trit_result_valid) begin
            mux_result = trit_result;
            mux_valid  = 1'b1;
            mux_exc    = trit_exc;
        end
    end

    // -----------------------------------------------------------------------
    // CSR read logic
    // -----------------------------------------------------------------------
    reg [31:0] csr_read_data;

    always @(*) begin
        case (rs2[3:0])
            4'h0: csr_read_data = csr_xpstatus;
            4'h1: csr_read_data = csr_xpdomid;
            4'h2: csr_read_data = csr_xpcapbase;
            4'h3: csr_read_data = csr_xpcapbound;
            4'h4: csr_read_data = csr_xpmask_seed;
            4'h5: csr_read_data = mask_state_out;     // RO
            4'h6: csr_read_data = csr_xptrit_mode;
            4'h7: csr_read_data = csr_xpsig_cfg;
            4'h8: csr_read_data = csr_xpexc_cause;    // RO
            4'h9: csr_read_data = csr_xpexc_addr;     // RO
            4'hA: csr_read_data = csr_xpperf_cnt;
            4'hB: csr_read_data = `XP_VERSION;         // RO hardwired
            default: csr_read_data = 32'h0;
        endcase
    end

    // -----------------------------------------------------------------------
    // CSR write logic
    // -----------------------------------------------------------------------
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            csr_xpstatus    <= 32'h0;
            csr_xpdomid     <= 32'h0;
            csr_xpcapbase   <= 32'h0;
            csr_xpcapbound  <= 32'h0;
            csr_xpmask_seed <= 32'h0;
            csr_xptrit_mode <= 32'h0;
            csr_xpsig_cfg   <= 32'h0;
            csr_xpexc_cause <= 32'h0;
            csr_xpexc_addr  <= 32'h0;
            csr_xpperf_cnt  <= 32'h0;
        end else begin
            // CSR write (funct7[6] = write enable)
            if (is_csr_op && funct7[6]) begin
                case (rs2[3:0])
                    4'h0: csr_xpstatus    <= rs1_data;
                    4'h1: csr_xpdomid     <= rs1_data;
                    4'h2: csr_xpcapbase   <= rs1_data;
                    4'h3: csr_xpcapbound  <= rs1_data;
                    4'h4: csr_xpmask_seed <= rs1_data;
                    // 0x5: mask_state is RO
                    4'h6: csr_xptrit_mode <= rs1_data;
                    4'h7: csr_xpsig_cfg   <= rs1_data;
                    // 0x8, 0x9: exception regs are RO
                    4'hA: csr_xpperf_cnt  <= rs1_data;
                    // 0xB: version is RO
                    default: ;
                endcase
            end

            // Update exception CSRs when any subunit raises exception
            if (mux_valid && mux_exc != `XP_EXC_NONE) begin
                csr_xpexc_cause <= {28'h0, mux_exc};
                // In real pipeline, this would be the PC
                csr_xpexc_addr  <= 32'hFFFF_FFFF;
            end

            // Performance counter — increment on every valid Xplenum instruction
            if (is_xplenum && (mux_valid || (is_csr_op))) begin
                csr_xpperf_cnt <= csr_xpperf_cnt + 1;
            end
        end
    end

    // -----------------------------------------------------------------------
    // Pipeline registers for output stage alignment
    // -----------------------------------------------------------------------
    reg        is_csr_op_d1;
    reg [31:0] csr_read_data_d1;
    reg [4:0]  rd_d1;

    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            is_csr_op_d1    <= 1'b0;
            csr_read_data_d1 <= 32'h0;
            rd_d1           <= 5'h0;
        end else begin
            is_csr_op_d1    <= is_csr_op;
            csr_read_data_d1 <= csr_read_data;
            rd_d1           <= rd;
        end
    end

    // -----------------------------------------------------------------------
    // Output stage
    // -----------------------------------------------------------------------
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            rd_data     <= 32'h0;
            rd_write_en <= 1'b0;
            rd_addr     <= 5'h0;
        end else if (is_csr_op_d1) begin
            rd_data     <= csr_read_data_d1;
            rd_write_en <= 1'b1;
            rd_addr     <= rd_d1;
        end else if (mux_valid) begin
            rd_data     <= mux_result;
            rd_write_en <= 1'b1;
            rd_addr     <= rd;
        end else begin
            rd_write_en <= 1'b0;
        end
    end

    // -----------------------------------------------------------------------
    // Exception output
    // -----------------------------------------------------------------------
    assign xp_exception = mux_valid && (mux_exc != `XP_EXC_NONE);
    assign xp_exc_code  = mux_exc;

    // DRBG status outputs
    assign drbg_health_err_o = drbg_health_error;
    assign drbg_ready_o      = drbg_ready;

endmodule
