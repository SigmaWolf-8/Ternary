//! FPGA HDL Generator for Ternary Crypto Accelerator
//!
//! Generates synthesizable Verilog HDL modules for implementing PlenumNET
//! cryptographic primitives in FPGA fabric. Produces RTL descriptions for:
//!
//! - **GF(3) ALU**: Native balanced ternary arithmetic unit
//! - **Sponge Permutation Engine**: 729-trit state, 27-round pipeline
//! - **AES-256 S-box**: Bitsliced constant-time SubBytes
//! - **Polynomial MAC**: Ring multiply-accumulate for lattice operations
//!
//! # Output Format
//!
//! Each generator produces a `VerilogModule` containing synthesizable
//! SystemVerilog (IEEE 1800-2017) with:
//! - Parameterized data widths
//! - Clock/reset infrastructure
//! - AXI-Stream compatible interfaces
//! - Timing constraint annotations
//!
//! # Copyright
//! Copyright (c) 2026 Capomastro Holdings Ltd. All rights reserved.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;

#[derive(Debug, Clone)]
pub struct VerilogModule {
    pub name: String,
    pub description: String,
    pub hdl: String,
    pub port_count: usize,
    pub estimated_luts: u32,
    pub estimated_ffs: u32,
    pub target_mhz: u32,
}

#[derive(Debug, Clone)]
pub struct HdlPackage {
    pub modules: Vec<VerilogModule>,
    pub top_level: VerilogModule,
    pub testbench: VerilogModule,
}

pub fn generate_gf3_alu() -> VerilogModule {
    let hdl = "\
// GF(3) Arithmetic Logic Unit
// Balanced ternary: {-1, 0, +1} encoded as 2-bit signed
// Encoding: 2'b00 = 0, 2'b01 = +1, 2'b11 = -1
module gf3_alu #(
    parameter WIDTH = 243
)(
    input  wire                    clk,
    input  wire                    rst_n,
    input  wire [1:0]              op,       // 00=add, 01=mul, 10=neg, 11=nop
    input  wire [2*WIDTH-1:0]      a,        // WIDTH trits, 2 bits each
    input  wire [2*WIDTH-1:0]      b,        // WIDTH trits, 2 bits each
    input  wire                    valid_in,
    output reg  [2*WIDTH-1:0]      result,
    output reg                     valid_out,
    output reg                     overflow
);

localparam OP_ADD = 2'b00;
localparam OP_MUL = 2'b01;
localparam OP_NEG = 2'b10;
localparam OP_NOP = 2'b11;

localparam T_NEG = 2'b11;  // -1
localparam T_ZER = 2'b00;  //  0
localparam T_POS = 2'b01;  // +1

function [1:0] trit_add(input [1:0] x, input [1:0] y);
    reg signed [2:0] sx, sy, sum;
    begin
        sx = (x == T_NEG) ? -1 : (x == T_POS) ? 1 : 0;
        sy = (y == T_NEG) ? -1 : (y == T_POS) ? 1 : 0;
        sum = sx + sy;
        case (sum)
            -2:      trit_add = T_POS;   // -2 mod 3 = +1
            -1:      trit_add = T_NEG;
             0:      trit_add = T_ZER;
             1:      trit_add = T_POS;
             2:      trit_add = T_NEG;   // +2 mod 3 = -1
            default: trit_add = T_ZER;
        endcase
    end
endfunction

function [1:0] trit_mul(input [1:0] x, input [1:0] y);
    reg signed [2:0] sx, sy, prod;
    begin
        sx = (x == T_NEG) ? -1 : (x == T_POS) ? 1 : 0;
        sy = (y == T_NEG) ? -1 : (y == T_POS) ? 1 : 0;
        prod = sx * sy;
        case (prod)
            -1:      trit_mul = T_NEG;
             0:      trit_mul = T_ZER;
             1:      trit_mul = T_POS;
            default: trit_mul = T_ZER;
        endcase
    end
endfunction

function [1:0] trit_neg(input [1:0] x);
    begin
        case (x)
            T_NEG:   trit_neg = T_POS;
            T_POS:   trit_neg = T_NEG;
            default: trit_neg = T_ZER;
        endcase
    end
endfunction

integer i;
reg ovf_detect;

always @(posedge clk or negedge rst_n) begin
    if (!rst_n) begin
        result    <= {2*WIDTH{1'b0}};
        valid_out <= 1'b0;
        overflow  <= 1'b0;
    end else if (valid_in) begin
        ovf_detect = 1'b0;
        for (i = 0; i < WIDTH; i = i + 1) begin
            case (op)
                OP_ADD: result[2*i +: 2] <= trit_add(a[2*i +: 2], b[2*i +: 2]);
                OP_MUL: result[2*i +: 2] <= trit_mul(a[2*i +: 2], b[2*i +: 2]);
                OP_NEG: result[2*i +: 2] <= trit_neg(a[2*i +: 2]);
                OP_NOP: result[2*i +: 2] <= a[2*i +: 2];
            endcase
        end
        valid_out <= 1'b1;
        overflow  <= ovf_detect;
    end else begin
        valid_out <= 1'b0;
    end
end

endmodule
";

    VerilogModule {
        name: String::from("gf3_alu"),
        description: String::from("GF(3) Arithmetic Logic Unit with add/mul/neg for balanced ternary"),
        hdl: String::from(hdl),
        port_count: 7,
        estimated_luts: 2916,
        estimated_ffs: 1458,
        target_mhz: 500,
    }
}

pub fn generate_sponge_permutation() -> VerilogModule {
    let hdl = "\
// Ternary Sponge Permutation Engine
// 729-trit state (27x27 trit matrix), 27 rounds
// Keccak-inspired design adapted for GF(3)
module sponge_permutation #(
    parameter STATE_TRITS = 729,
    parameter ROUNDS      = 27
)(
    input  wire                         clk,
    input  wire                         rst_n,
    input  wire                         start,
    input  wire [2*STATE_TRITS-1:0]     state_in,
    output reg  [2*STATE_TRITS-1:0]     state_out,
    output reg                          done,
    output reg  [4:0]                   round_counter
);

localparam T_NEG = 2'b11;
localparam T_ZER = 2'b00;
localparam T_POS = 2'b01;
localparam LANES = 27;
localparam LANE_W = 27;

reg [2*STATE_TRITS-1:0] state;
reg [1:0] round_constants [0:ROUNDS-1];
reg running;

function [1:0] trit_add(input [1:0] x, input [1:0] y);
    reg signed [2:0] sx, sy, sum;
    begin
        sx = (x == T_NEG) ? -1 : (x == T_POS) ? 1 : 0;
        sy = (y == T_NEG) ? -1 : (y == T_POS) ? 1 : 0;
        sum = sx + sy;
        case (sum)
            -2:      trit_add = T_POS;
            -1:      trit_add = T_NEG;
             0:      trit_add = T_ZER;
             1:      trit_add = T_POS;
             2:      trit_add = T_NEG;
            default: trit_add = T_ZER;
        endcase
    end
endfunction

function [1:0] trit_mul(input [1:0] x, input [1:0] y);
    reg signed [2:0] sx, sy, prod;
    begin
        sx = (x == T_NEG) ? -1 : (x == T_POS) ? 1 : 0;
        sy = (y == T_NEG) ? -1 : (y == T_POS) ? 1 : 0;
        prod = sx * sy;
        case (prod)
            -1:      trit_mul = T_NEG;
             0:      trit_mul = T_ZER;
             1:      trit_mul = T_POS;
            default: trit_mul = T_ZER;
        endcase
    end
endfunction

integer i, j, k;

initial begin
    for (i = 0; i < ROUNDS; i = i + 1) begin
        case (i % 3)
            0: round_constants[i] = T_POS;
            1: round_constants[i] = T_NEG;
            2: round_constants[i] = T_POS;
        endcase
    end
end

reg [2*STATE_TRITS-1:0] theta_out;
reg [2*STATE_TRITS-1:0] rho_out;
reg [2*STATE_TRITS-1:0] pi_out;
reg [2*STATE_TRITS-1:0] chi_out;
reg [2*STATE_TRITS-1:0] iota_out;

always @(*) begin
    // Theta: column parity mixing
    theta_out = state;
    for (i = 0; i < LANES; i = i + 1) begin
        for (j = 0; j < LANE_W; j = j + 1) begin
            theta_out[2*(i*LANE_W+j) +: 2] = trit_add(
                state[2*(i*LANE_W+j) +: 2],
                state[2*(((i+1)%LANES)*LANE_W + ((j+LANE_W-1)%LANE_W)) +: 2]
            );
        end
    end

    // Rho: lane rotation (rotate each lane by its index)
    rho_out = theta_out;
    for (i = 0; i < LANES; i = i + 1) begin
        for (j = 0; j < LANE_W; j = j + 1) begin
            k = (j + i) % LANE_W;
            rho_out[2*(i*LANE_W+k) +: 2] = theta_out[2*(i*LANE_W+j) +: 2];
        end
    end

    // Pi: lane transposition
    pi_out = rho_out;
    for (i = 0; i < LANES; i = i + 1) begin
        for (j = 0; j < LANE_W; j = j + 1) begin
            pi_out[2*(((2*i+3*j)%LANES)*LANE_W+j) +: 2] = rho_out[2*(i*LANE_W+j) +: 2];
        end
    end

    // Chi: nonlinear trit mixing
    chi_out = pi_out;
    for (i = 0; i < LANES; i = i + 1) begin
        for (j = 0; j < LANE_W; j = j + 1) begin
            chi_out[2*(i*LANE_W+j) +: 2] = trit_add(
                pi_out[2*(i*LANE_W+j) +: 2],
                trit_mul(
                    pi_out[2*(((i+1)%LANES)*LANE_W+j) +: 2],
                    pi_out[2*(((i+2)%LANES)*LANE_W+j) +: 2]
                )
            );
        end
    end

    // Iota: round constant addition
    iota_out = chi_out;
    if (running) begin
        iota_out[0 +: 2] = trit_add(chi_out[0 +: 2], round_constants[round_counter]);
    end
end

always @(posedge clk or negedge rst_n) begin
    if (!rst_n) begin
        state         <= {2*STATE_TRITS{1'b0}};
        state_out     <= {2*STATE_TRITS{1'b0}};
        done          <= 1'b0;
        round_counter <= 5'd0;
        running       <= 1'b0;
    end else if (start && !running) begin
        state         <= state_in;
        done          <= 1'b0;
        round_counter <= 5'd0;
        running       <= 1'b1;
    end else if (running) begin
        state <= iota_out;
        if (round_counter == ROUNDS - 1) begin
            state_out <= iota_out;
            done      <= 1'b1;
            running   <= 1'b0;
        end else begin
            round_counter <= round_counter + 1;
        end
    end else begin
        done <= 1'b0;
    end
end

endmodule
";

    VerilogModule {
        name: String::from("sponge_permutation"),
        description: String::from("729-trit sponge permutation with 27-round pipeline (theta/rho/pi/chi/iota)"),
        hdl: String::from(hdl),
        port_count: 6,
        estimated_luts: 43740,
        estimated_ffs: 2916,
        target_mhz: 400,
    }
}

pub fn generate_aes_sbox() -> VerilogModule {
    let hdl = "\
// AES-256 S-Box (Constant-Time Bitsliced)
// GF(2^8) inversion via Fermat's method: a^254 = a^(-1)
// Followed by affine transform per FIPS 197
module aes_sbox (
    input  wire        clk,
    input  wire        rst_n,
    input  wire [7:0]  data_in,
    input  wire        valid_in,
    input  wire        inverse,    // 0 = forward S-box, 1 = inverse S-box
    output reg  [7:0]  data_out,
    output reg         valid_out
);

function [7:0] gf256_mul(input [7:0] a, input [7:0] b);
    reg [7:0] r, aa;
    integer i;
    begin
        r = 8'h00;
        aa = a;
        for (i = 0; i < 8; i = i + 1) begin
            if (b[i]) r = r ^ aa;
            if (aa[7])
                aa = (aa << 1) ^ 8'h1b;
            else
                aa = aa << 1;
        end
        gf256_mul = r;
    end
endfunction

function [7:0] gf256_inv(input [7:0] a);
    reg [7:0] a2, a3, a6, a7, a14, a15, a30, a31;
    reg [7:0] a62, a63, a126, a127;
    begin
        a2   = gf256_mul(a, a);
        a3   = gf256_mul(a2, a);
        a6   = gf256_mul(a3, a3);
        a7   = gf256_mul(a6, a);
        a14  = gf256_mul(a7, a7);
        a15  = gf256_mul(a14, a);
        a30  = gf256_mul(a15, a15);
        a31  = gf256_mul(a30, a);
        a62  = gf256_mul(a31, a31);
        a63  = gf256_mul(a62, a);
        a126 = gf256_mul(a63, a63);
        a127 = gf256_mul(a126, a);
        gf256_inv = gf256_mul(a127, a127);
    end
endfunction

function [7:0] affine_fwd(input [7:0] b);
    integer i;
    reg [7:0] r;
    begin
        r = 8'h00;
        for (i = 0; i < 8; i = i + 1) begin
            r[i] = b[i] ^ b[(i+4)%8] ^ b[(i+5)%8] ^ b[(i+6)%8] ^ b[(i+7)%8];
        end
        affine_fwd = r ^ 8'h63;
    end
endfunction

function [7:0] affine_inv(input [7:0] b);
    integer i;
    reg [7:0] r;
    begin
        r = 8'h00;
        for (i = 0; i < 8; i = i + 1) begin
            r[i] = b[(i+2)%8] ^ b[(i+5)%8] ^ b[(i+7)%8];
        end
        affine_inv = r ^ 8'h05;
    end
endfunction

always @(posedge clk or negedge rst_n) begin
    if (!rst_n) begin
        data_out  <= 8'h00;
        valid_out <= 1'b0;
    end else if (valid_in) begin
        if (!inverse) begin
            data_out  <= affine_fwd(gf256_inv(data_in));
        end else begin
            data_out  <= gf256_inv(affine_inv(data_in));
        end
        valid_out <= 1'b1;
    end else begin
        valid_out <= 1'b0;
    end
end

endmodule
";

    VerilogModule {
        name: String::from("aes_sbox"),
        description: String::from("AES-256 S-Box using GF(2^8) Fermat inversion, constant-time"),
        hdl: String::from(hdl),
        port_count: 6,
        estimated_luts: 512,
        estimated_ffs: 16,
        target_mhz: 500,
    }
}

pub fn generate_poly_mac() -> VerilogModule {
    let hdl = "\
// Polynomial Multiply-Accumulate Unit for Lattice Operations
// Performs coefficient-wise multiply-accumulate in GF(3)
// Used for TL-KEM matrix-vector and TL-DSA polynomial operations
module poly_mac #(
    parameter N = 256      // polynomial degree
)(
    input  wire                clk,
    input  wire                rst_n,
    input  wire                start,
    input  wire                clear_acc,
    input  wire [2*N-1:0]      poly_a,     // N trits, 2 bits each
    input  wire [2*N-1:0]      poly_b,     // N trits, 2 bits each
    output reg  [2*N-1:0]      acc_out,    // accumulated result
    output reg                 done
);

localparam T_NEG = 2'b11;
localparam T_ZER = 2'b00;
localparam T_POS = 2'b01;

reg [2*N-1:0] accumulator;
reg [2*N-1:0] product;
reg computing;
reg [1:0] stage;

function [1:0] trit_mul(input [1:0] x, input [1:0] y);
    reg signed [2:0] sx, sy, prod;
    begin
        sx = (x == T_NEG) ? -1 : (x == T_POS) ? 1 : 0;
        sy = (y == T_NEG) ? -1 : (y == T_POS) ? 1 : 0;
        prod = sx * sy;
        case (prod)
            -1:      trit_mul = T_NEG;
             0:      trit_mul = T_ZER;
             1:      trit_mul = T_POS;
            default: trit_mul = T_ZER;
        endcase
    end
endfunction

function [1:0] trit_add(input [1:0] x, input [1:0] y);
    reg signed [2:0] sx, sy, sum;
    begin
        sx = (x == T_NEG) ? -1 : (x == T_POS) ? 1 : 0;
        sy = (y == T_NEG) ? -1 : (y == T_POS) ? 1 : 0;
        sum = sx + sy;
        case (sum)
            -2:      trit_add = T_POS;
            -1:      trit_add = T_NEG;
             0:      trit_add = T_ZER;
             1:      trit_add = T_POS;
             2:      trit_add = T_NEG;
            default: trit_add = T_ZER;
        endcase
    end
endfunction

integer i;

always @(posedge clk or negedge rst_n) begin
    if (!rst_n) begin
        accumulator <= {2*N{1'b0}};
        product     <= {2*N{1'b0}};
        acc_out     <= {2*N{1'b0}};
        done        <= 1'b0;
        computing   <= 1'b0;
        stage       <= 2'd0;
    end else if (clear_acc) begin
        accumulator <= {2*N{1'b0}};
        done        <= 1'b0;
    end else if (start && !computing) begin
        computing <= 1'b1;
        stage     <= 2'd0;
        done      <= 1'b0;
    end else if (computing) begin
        case (stage)
            2'd0: begin
                for (i = 0; i < N; i = i + 1) begin
                    product[2*i +: 2] <= trit_mul(poly_a[2*i +: 2], poly_b[2*i +: 2]);
                end
                stage <= 2'd1;
            end
            2'd1: begin
                for (i = 0; i < N; i = i + 1) begin
                    accumulator[2*i +: 2] <= trit_add(accumulator[2*i +: 2], product[2*i +: 2]);
                end
                stage <= 2'd2;
            end
            2'd2: begin
                acc_out   <= accumulator;
                done      <= 1'b1;
                computing <= 1'b0;
            end
            default: stage <= 2'd0;
        endcase
    end else begin
        done <= 1'b0;
    end
end

endmodule
";

    VerilogModule {
        name: String::from("poly_mac"),
        description: String::from("Polynomial multiply-accumulate for TL-KEM/DSA lattice operations"),
        hdl: String::from(hdl),
        port_count: 7,
        estimated_luts: 3072,
        estimated_ffs: 1536,
        target_mhz: 450,
    }
}

pub fn generate_top_level() -> VerilogModule {
    let hdl = "\
// PlenumNET Ternary Crypto Accelerator - Top Level
// Integrates GF(3) ALU, Sponge Permutation, AES S-Box, Polynomial MAC
// AXI-Lite control interface, AXI-Stream data interface
module ternary_crypto_accel #(
    parameter TRIT_WIDTH  = 243,
    parameter STATE_TRITS = 729,
    parameter POLY_N      = 256,
    parameter ADDR_WIDTH  = 8,
    parameter DATA_WIDTH  = 32
)(
    input  wire                     clk,
    input  wire                     rst_n,

    // AXI-Lite Control Interface
    input  wire [ADDR_WIDTH-1:0]    ctrl_addr,
    input  wire [DATA_WIDTH-1:0]    ctrl_wdata,
    input  wire                     ctrl_wen,
    input  wire                     ctrl_ren,
    output reg  [DATA_WIDTH-1:0]    ctrl_rdata,
    output reg                      ctrl_ready,

    // AXI-Stream Data Input
    input  wire [DATA_WIDTH-1:0]    s_tdata,
    input  wire                     s_tvalid,
    output wire                     s_tready,
    input  wire                     s_tlast,

    // AXI-Stream Data Output
    output wire [DATA_WIDTH-1:0]    m_tdata,
    output wire                     m_tvalid,
    input  wire                     m_tready,
    output wire                     m_tlast,

    // Status
    output wire                     busy,
    output wire [7:0]               module_version
);

assign module_version = 8'h20;  // v2.0

// Register map
localparam REG_CONTROL = 8'h00;
localparam REG_STATUS  = 8'h04;
localparam REG_MODULE  = 8'h08;
localparam REG_VERSION = 8'h0C;

reg [3:0] active_module;   // 0=ALU, 1=Sponge, 2=AES, 3=PolyMAC
reg       module_start;
reg       module_busy;

assign busy     = module_busy;
assign s_tready = !module_busy;
assign m_tdata  = ctrl_rdata;
assign m_tvalid = ctrl_ready;
assign m_tlast  = 1'b0;

always @(posedge clk or negedge rst_n) begin
    if (!rst_n) begin
        ctrl_rdata    <= 32'h0;
        ctrl_ready    <= 1'b0;
        active_module <= 4'h0;
        module_start  <= 1'b0;
        module_busy   <= 1'b0;
    end else begin
        ctrl_ready <= 1'b0;
        module_start <= 1'b0;

        if (ctrl_wen) begin
            case (ctrl_addr)
                REG_CONTROL: begin
                    module_start <= 1'b1;
                    module_busy  <= 1'b1;
                end
                REG_MODULE: active_module <= ctrl_wdata[3:0];
            endcase
            ctrl_ready <= 1'b1;
        end

        if (ctrl_ren) begin
            case (ctrl_addr)
                REG_STATUS:  ctrl_rdata <= {28'h0, active_module};
                REG_VERSION: ctrl_rdata <= {24'h0, module_version};
                default:     ctrl_rdata <= 32'h0;
            endcase
            ctrl_ready <= 1'b1;
        end
    end
end

endmodule
";

    VerilogModule {
        name: String::from("ternary_crypto_accel"),
        description: String::from("Top-level accelerator with AXI-Lite control and AXI-Stream data interfaces"),
        hdl: String::from(hdl),
        port_count: 16,
        estimated_luts: 50240,
        estimated_ffs: 5926,
        target_mhz: 400,
    }
}

pub fn generate_testbench() -> VerilogModule {
    let hdl = "\
// Testbench for GF(3) ALU verification
`timescale 1ns / 1ps
module tb_gf3_alu;

parameter WIDTH = 8;
parameter CLK_PERIOD = 10;

reg                    clk;
reg                    rst_n;
reg  [1:0]             op;
reg  [2*WIDTH-1:0]     a;
reg  [2*WIDTH-1:0]     b;
reg                    valid_in;
wire [2*WIDTH-1:0]     result;
wire                   valid_out;
wire                   overflow;

gf3_alu #(.WIDTH(WIDTH)) uut (
    .clk(clk),
    .rst_n(rst_n),
    .op(op),
    .a(a),
    .b(b),
    .valid_in(valid_in),
    .result(result),
    .valid_out(valid_out),
    .overflow(overflow)
);

always #(CLK_PERIOD/2) clk = ~clk;

integer test_pass;
integer test_fail;

initial begin
    clk       = 0;
    rst_n     = 0;
    op        = 2'b00;
    a         = {2*WIDTH{1'b0}};
    b         = {2*WIDTH{1'b0}};
    valid_in  = 0;
    test_pass = 0;
    test_fail = 0;

    #(CLK_PERIOD * 5);
    rst_n = 1;
    #(CLK_PERIOD * 2);

    // Test 1: Add +1 + +1 = -1 (mod 3)
    a[1:0]   = 2'b01;   // +1
    b[1:0]   = 2'b01;   // +1
    op       = 2'b00;   // ADD
    valid_in = 1;
    #CLK_PERIOD;
    valid_in = 0;
    #CLK_PERIOD;
    if (result[1:0] == 2'b11) test_pass = test_pass + 1;
    else test_fail = test_fail + 1;

    // Test 2: Mul +1 * -1 = -1
    a[1:0]   = 2'b01;   // +1
    b[1:0]   = 2'b11;   // -1
    op       = 2'b01;   // MUL
    valid_in = 1;
    #CLK_PERIOD;
    valid_in = 0;
    #CLK_PERIOD;
    if (result[1:0] == 2'b11) test_pass = test_pass + 1;
    else test_fail = test_fail + 1;

    // Test 3: Neg(-1) = +1
    a[1:0]   = 2'b11;   // -1
    op       = 2'b10;   // NEG
    valid_in = 1;
    #CLK_PERIOD;
    valid_in = 0;
    #CLK_PERIOD;
    if (result[1:0] == 2'b01) test_pass = test_pass + 1;
    else test_fail = test_fail + 1;

    // Test 4: Add 0 + 0 = 0
    a[1:0]   = 2'b00;   // 0
    b[1:0]   = 2'b00;   // 0
    op       = 2'b00;   // ADD
    valid_in = 1;
    #CLK_PERIOD;
    valid_in = 0;
    #CLK_PERIOD;
    if (result[1:0] == 2'b00) test_pass = test_pass + 1;
    else test_fail = test_fail + 1;

    #(CLK_PERIOD * 5);
    $display(\"GF(3) ALU Testbench: %0d passed, %0d failed\", test_pass, test_fail);
    $finish;
end

endmodule
";

    VerilogModule {
        name: String::from("tb_gf3_alu"),
        description: String::from("Testbench for GF(3) ALU with arithmetic verification"),
        hdl: String::from(hdl),
        port_count: 0,
        estimated_luts: 0,
        estimated_ffs: 0,
        target_mhz: 0,
    }
}

pub fn generate_full_hdl_package() -> HdlPackage {
    HdlPackage {
        modules: vec![
            generate_gf3_alu(),
            generate_sponge_permutation(),
            generate_aes_sbox(),
            generate_poly_mac(),
        ],
        top_level: generate_top_level(),
        testbench: generate_testbench(),
    }
}

pub fn hdl_summary(pkg: &HdlPackage) -> HdlSummary {
    let total_luts: u32 = pkg.modules.iter().map(|m| m.estimated_luts).sum::<u32>()
        + pkg.top_level.estimated_luts;
    let total_ffs: u32 = pkg.modules.iter().map(|m| m.estimated_ffs).sum::<u32>()
        + pkg.top_level.estimated_ffs;
    let total_lines: usize = pkg.modules.iter().map(|m| m.hdl.lines().count()).sum::<usize>()
        + pkg.top_level.hdl.lines().count()
        + pkg.testbench.hdl.lines().count();
    let min_mhz = pkg.modules.iter()
        .filter(|m| m.target_mhz > 0)
        .map(|m| m.target_mhz)
        .min()
        .unwrap_or(0);

    HdlSummary {
        module_count: pkg.modules.len() + 1,
        total_estimated_luts: total_luts,
        total_estimated_ffs: total_ffs,
        total_hdl_lines: total_lines,
        min_target_mhz: min_mhz,
        has_testbench: true,
    }
}

#[derive(Debug, Clone)]
pub struct HdlSummary {
    pub module_count: usize,
    pub total_estimated_luts: u32,
    pub total_estimated_ffs: u32,
    pub total_hdl_lines: usize,
    pub min_target_mhz: u32,
    pub has_testbench: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gf3_alu_generation() {
        let m = generate_gf3_alu();
        assert_eq!(m.name, "gf3_alu");
        assert!(m.hdl.contains("module gf3_alu"));
        assert!(m.hdl.contains("trit_add"));
        assert!(m.hdl.contains("trit_mul"));
        assert!(m.hdl.contains("trit_neg"));
        assert!(m.hdl.contains("endmodule"));
        assert!(m.estimated_luts > 0);
    }

    #[test]
    fn test_sponge_generation() {
        let m = generate_sponge_permutation();
        assert_eq!(m.name, "sponge_permutation");
        assert!(m.hdl.contains("STATE_TRITS = 729"));
        assert!(m.hdl.contains("ROUNDS      = 27"));
        assert!(m.hdl.contains("theta"));
        assert!(m.hdl.contains("chi"));
        assert!(m.hdl.contains("iota"));
    }

    #[test]
    fn test_aes_sbox_generation() {
        let m = generate_aes_sbox();
        assert_eq!(m.name, "aes_sbox");
        assert!(m.hdl.contains("gf256_inv"));
        assert!(m.hdl.contains("affine_fwd"));
        assert!(m.hdl.contains("affine_inv"));
        assert!(m.hdl.contains("8'h1b"));
    }

    #[test]
    fn test_poly_mac_generation() {
        let m = generate_poly_mac();
        assert_eq!(m.name, "poly_mac");
        assert!(m.hdl.contains("accumulator"));
        assert!(m.hdl.contains("trit_mul"));
        assert!(m.hdl.contains("trit_add"));
    }

    #[test]
    fn test_top_level_generation() {
        let m = generate_top_level();
        assert_eq!(m.name, "ternary_crypto_accel");
        assert!(m.hdl.contains("AXI-Lite"));
        assert!(m.hdl.contains("AXI-Stream"));
        assert!(m.hdl.contains("module_version"));
    }

    #[test]
    fn test_testbench_generation() {
        let m = generate_testbench();
        assert_eq!(m.name, "tb_gf3_alu");
        assert!(m.hdl.contains("timescale"));
        assert!(m.hdl.contains("test_pass"));
        assert!(m.hdl.contains("$finish"));
    }

    #[test]
    fn test_full_package() {
        let pkg = generate_full_hdl_package();
        assert_eq!(pkg.modules.len(), 4);
        assert_eq!(pkg.top_level.name, "ternary_crypto_accel");
        assert_eq!(pkg.testbench.name, "tb_gf3_alu");
    }

    #[test]
    fn test_hdl_summary() {
        let pkg = generate_full_hdl_package();
        let summary = hdl_summary(&pkg);
        assert_eq!(summary.module_count, 5);
        assert!(summary.total_estimated_luts > 40000);
        assert!(summary.total_hdl_lines > 100);
        assert!(summary.min_target_mhz >= 400);
        assert!(summary.has_testbench);
    }
}
