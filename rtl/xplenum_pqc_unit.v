// ===================================================================
// XPlenum Post-Quantum Cryptography Unit (Tasks 8C.2, 8C.3)
//
// Implements hardware-accelerated primitives for:
//   - ML-KEM (Kyber): NTT, inverse NTT, modular arithmetic
//   - ML-DSA (Dilithium): NTT, modular reduction, sampling
//
// New instructions (Custom-1 encoding space, 0x2B):
//   XPQC.NTT_BF   -- NTT butterfly (forward)
//   XPQC.INTT_BF  -- Inverse NTT butterfly
//   XPQC.MOD_RED  -- Configurable modular reduction
//   XPQC.MOD_MUL  -- Modular multiplication
//   XPQC.MOD_ADD  -- Modular addition
//   XPQC.CBD_SAMP -- Centered binomial distribution sampling
//   XPQC.REJ_SAMP -- Rejection sampling
//   XPQC.POLY_MAC -- Polynomial multiply-accumulate
//   XPQC.COMPRESS -- Polynomial coefficient compression
//   XPQC.DECOMP   -- Polynomial coefficient decompression
// ===================================================================
`timescale 1ns/1ps

module xplenum_pqc_unit (
    input             clk,
    input             rst_n,

    // Instruction interface
    input  [2:0]      funct3,
    input  [6:0]      funct7,
    input             valid_in,
    input  [63:0]     rs1_data,
    input  [63:0]     rs2_data,

    // Configuration CSR (sets active parameter set)
    input  [63:0]     pqc_config_csr,

    // Result
    output reg [63:0] rd_data,
    output reg        rd_wen,
    output reg        busy
);

    // -- Parameter set extraction from CSR --
    wire [15:0] q_modulus    = pqc_config_csr[15:0];
    wire [7:0]  param_set    = pqc_config_csr[23:16];
    wire        is_dilithium = param_set >= 8'd3;

    // -- Instruction opcodes (funct7 within Custom-1 funct3=0x4) --
    localparam PQC_NTT_BF   = 7'h20;
    localparam PQC_INTT_BF  = 7'h21;
    localparam PQC_MOD_RED  = 7'h22;
    localparam PQC_MOD_MUL  = 7'h23;
    localparam PQC_MOD_ADD  = 7'h24;
    localparam PQC_CBD_SAMP = 7'h25;
    localparam PQC_REJ_SAMP = 7'h26;
    localparam PQC_POLY_MAC = 7'h27;
    localparam PQC_COMPRESS = 7'h28;
    localparam PQC_DECOMP   = 7'h29;

    // -- Modular arithmetic primitives --

    // Barrett reduction: a mod q
    function [31:0] barrett_reduce;
        input [63:0] a;
        input [15:0] q;
        reg [63:0] t;
        reg [31:0] result;
        begin
            if (q == 16'd3329) begin
                t = (a * 64'd1290167) >> 32;
                result = a[31:0] - t[31:0] * 32'd3329;
                if (result >= 32'd3329)
                    result = result - 32'd3329;
            end else begin
                result = a % {48'd0, q};
            end
            barrett_reduce = result;
        end
    endfunction

    // Montgomery multiplication: (a * b * R^-1) mod q
    function [31:0] mont_mul;
        input [31:0] a;
        input [31:0] b;
        input [15:0] q;
        reg [63:0] product;
        reg [31:0] t;
        begin
            product = {32'd0, a} * {32'd0, b};
            if (q == 16'd3329) begin
                t = product[15:0] * 16'd3327;
                product = (product + {32'd0, t[15:0]} * {48'd0, q}) >> 16;
                mont_mul = (product >= {48'd0, q}) ?
                           product[31:0] - {16'd0, q} : product[31:0];
            end else begin
                mont_mul = product % {48'd0, q};
            end
        end
    endfunction

    // -- NTT Butterfly --
    // Cooley-Tukey butterfly: (a + w*b, a - w*b) mod q
    reg [31:0] ntt_a, ntt_b, ntt_w;
    reg [31:0] ntt_wb;
    reg [31:0] ntt_out_lo;
    reg [31:0] ntt_out_hi;

    always @(*) begin
        ntt_a  = rs1_data[31:0];
        ntt_b  = rs2_data[31:0];
        ntt_w  = rs2_data[63:32];

        ntt_wb = mont_mul(ntt_w, ntt_b, q_modulus);

        ntt_out_lo = barrett_reduce({32'd0, ntt_a} + {32'd0, ntt_wb}, q_modulus);
        ntt_out_hi = barrett_reduce({32'd0, ntt_a} + {48'd0, q_modulus} - {32'd0, ntt_wb}, q_modulus);
    end

    // -- Inverse NTT Butterfly --
    // Gentleman-Sande: (a + b, w*(a - b)) mod q
    reg [31:0] intt_sum, intt_diff, intt_out_lo, intt_out_hi;

    always @(*) begin
        intt_sum  = barrett_reduce({32'd0, ntt_a} + {32'd0, ntt_b}, q_modulus);
        intt_diff = barrett_reduce({32'd0, ntt_a} + {48'd0, q_modulus} - {32'd0, ntt_b}, q_modulus);
        intt_out_lo = intt_sum;
        intt_out_hi = mont_mul(ntt_w, intt_diff, q_modulus);
    end

    // -- Centered Binomial Distribution (CBD) Sampling --
    reg [31:0] cbd_out [0:3];
    integer cbd_i;

    always @(*) begin
        for (cbd_i = 0; cbd_i < 4; cbd_i = cbd_i + 1) begin
            cbd_out[cbd_i] = 32'd0;
        end

        if (rs2_data[7:0] == 8'd2) begin
            for (cbd_i = 0; cbd_i < 4; cbd_i = cbd_i + 1) begin
                cbd_out[cbd_i] =
                    (rs1_data[cbd_i*16]     + rs1_data[cbd_i*16+1]) -
                    (rs1_data[cbd_i*16+2]   + rs1_data[cbd_i*16+3]);
            end
        end else if (rs2_data[7:0] == 8'd3) begin
            for (cbd_i = 0; cbd_i < 2; cbd_i = cbd_i + 1) begin
                cbd_out[cbd_i] =
                    (rs1_data[cbd_i*24]   + rs1_data[cbd_i*24+1] + rs1_data[cbd_i*24+2]) -
                    (rs1_data[cbd_i*24+3] + rs1_data[cbd_i*24+4] + rs1_data[cbd_i*24+5]);
            end
        end
    end

    // -- Rejection Sampling --
    reg [15:0] rej_candidates [0:3];
    reg [63:0] rej_result;

    always @(*) begin
        rej_candidates[0] = rs1_data[15:0];
        rej_candidates[1] = rs1_data[31:16];
        rej_candidates[2] = rs1_data[47:32];
        rej_candidates[3] = rs1_data[63:48];

        rej_result = {
            (rej_candidates[3] < q_modulus) ? rej_candidates[3] : 16'hFFFF,
            (rej_candidates[2] < q_modulus) ? rej_candidates[2] : 16'hFFFF,
            (rej_candidates[1] < q_modulus) ? rej_candidates[1] : 16'hFFFF,
            (rej_candidates[0] < q_modulus) ? rej_candidates[0] : 16'hFFFF
        };
    end

    // -- Coefficient Compression/Decompression --
    reg [31:0] comp_d;
    reg [31:0] comp_result;
    reg [31:0] decomp_result;

    always @(*) begin
        comp_d = rs2_data[31:0];

        comp_result = (((rs1_data[31:0] << comp_d) + {16'd0, q_modulus} / 2) /
                       {16'd0, q_modulus}) & ((32'd1 << comp_d) - 1);

        decomp_result = ({16'd0, q_modulus} * rs1_data[31:0] + (32'd1 << (comp_d - 1))) >> comp_d;
    end

    // -- Result multiplexer --
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            rd_data <= 64'd0;
            rd_wen  <= 1'b0;
            busy    <= 1'b0;
        end else if (valid_in) begin
            rd_wen <= 1'b1;
            busy   <= 1'b0;

            case (funct7)
                PQC_NTT_BF:   rd_data <= {ntt_out_hi, ntt_out_lo};
                PQC_INTT_BF:  rd_data <= {intt_out_hi, intt_out_lo};
                PQC_MOD_RED:  rd_data <= {32'd0, barrett_reduce(rs1_data, q_modulus)};
                PQC_MOD_MUL:  rd_data <= {32'd0, mont_mul(rs1_data[31:0], rs2_data[31:0], q_modulus)};
                PQC_MOD_ADD:  rd_data <= {32'd0, barrett_reduce(
                                          {32'd0, rs1_data[31:0]} + {32'd0, rs2_data[31:0]}, q_modulus)};
                PQC_CBD_SAMP: rd_data <= {cbd_out[3], cbd_out[2], cbd_out[1][15:0], cbd_out[0][15:0]};
                PQC_REJ_SAMP: rd_data <= rej_result;
                PQC_POLY_MAC: rd_data <= {32'd0, barrett_reduce(
                                          {32'd0, mont_mul(rs1_data[31:0], rs2_data[31:0], q_modulus)} +
                                          {32'd0, rs1_data[63:32]}, q_modulus)};
                PQC_COMPRESS: rd_data <= {32'd0, comp_result};
                PQC_DECOMP:   rd_data <= {32'd0, decomp_result};
                default:      begin rd_data <= 64'd0; rd_wen <= 1'b0; end
            endcase
        end else begin
            rd_wen <= 1'b0;
        end
    end

endmodule
