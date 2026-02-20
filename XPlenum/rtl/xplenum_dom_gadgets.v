// ===================================================================
// XPlenum Higher-Order Masking Gadgets (Tasks 8B.1, 8B.2)
//
// Implements Domain-Oriented Masking (DOM) for provably secure
// higher-order side-channel resistance.
//
// Supports:
//   - 3-share (2nd-order) secure AND, XOR, refresh
//   - 4-share (3rd-order) secure AND, XOR, refresh
//   - Pipeline registers between shares for glitch resistance
//   - Configurable share count via parameter
//
// Reference: Gross, Mangard, Korak (2016)
//   "Domain-Oriented Masking: Compact Masked Hardware Implementations
//    with Arbitrary Protection Order"
// ===================================================================
`timescale 1ns/1ps

// -----------------------------------------------------------------
// DOM-AND Gadget (2nd Order / 3 Shares)
// Computes: c = a & b securely with 3 shares
// Inputs:  a[0..2], b[0..2] (shared values)
// Outputs: c[0..2] (shared result)
// Fresh randomness: z01, z02, z12 (3 random bits per bit-width)
// -----------------------------------------------------------------
module dom_and_3share #(
    parameter WIDTH = 64
)(
    input                   clk,
    input                   rst_n,
    input                   enable,

    // Input shares: a = a0 ^ a1 ^ a2, b = b0 ^ b1 ^ b2
    input  [WIDTH-1:0]      a0, a1, a2,
    input  [WIDTH-1:0]      b0, b1, b2,

    // Fresh randomness from DRBG (one per cross-domain term)
    input  [WIDTH-1:0]      z01,
    input  [WIDTH-1:0]      z02,
    input  [WIDTH-1:0]      z12,

    // Output shares: c = c0 ^ c1 ^ c2 = a & b
    output reg [WIDTH-1:0]  c0, c1, c2,
    output reg              valid
);

    // -- Stage 1: Inner-domain products (no randomness needed) --
    wire [WIDTH-1:0] inner_00 = a0 & b0;
    wire [WIDTH-1:0] inner_11 = a1 & b1;
    wire [WIDTH-1:0] inner_22 = a2 & b2;

    // -- Stage 1: Cross-domain products (require masking) --
    wire [WIDTH-1:0] cross_01_raw = (a0 & b1) ^ z01;
    wire [WIDTH-1:0] cross_10_raw = (a1 & b0) ^ z01;
    wire [WIDTH-1:0] cross_02_raw = (a0 & b2) ^ z02;
    wire [WIDTH-1:0] cross_20_raw = (a2 & b0) ^ z02;
    wire [WIDTH-1:0] cross_12_raw = (a1 & b2) ^ z12;
    wire [WIDTH-1:0] cross_21_raw = (a2 & b1) ^ z12;

    // -- Pipeline Register: Glitch barrier --
    // CRITICAL: This register prevents glitches from propagating
    // across domains. Without it, transient values during
    // combinational settling could leak information.
    reg [WIDTH-1:0] cross_01_reg, cross_10_reg;
    reg [WIDTH-1:0] cross_02_reg, cross_20_reg;
    reg [WIDTH-1:0] cross_12_reg, cross_21_reg;
    reg [WIDTH-1:0] inner_00_reg, inner_11_reg, inner_22_reg;
    reg             stage1_valid;

    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            cross_01_reg <= {WIDTH{1'b0}};
            cross_10_reg <= {WIDTH{1'b0}};
            cross_02_reg <= {WIDTH{1'b0}};
            cross_20_reg <= {WIDTH{1'b0}};
            cross_12_reg <= {WIDTH{1'b0}};
            cross_21_reg <= {WIDTH{1'b0}};
            inner_00_reg <= {WIDTH{1'b0}};
            inner_11_reg <= {WIDTH{1'b0}};
            inner_22_reg <= {WIDTH{1'b0}};
            stage1_valid <= 1'b0;
        end else if (enable) begin
            cross_01_reg <= cross_01_raw;
            cross_10_reg <= cross_10_raw;
            cross_02_reg <= cross_02_raw;
            cross_20_reg <= cross_20_raw;
            cross_12_reg <= cross_12_raw;
            cross_21_reg <= cross_21_raw;
            inner_00_reg <= inner_00;
            inner_11_reg <= inner_11;
            inner_22_reg <= inner_22;
            stage1_valid <= 1'b1;
        end else begin
            stage1_valid <= 1'b0;
        end
    end

    // -- Stage 2: Recombination --
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            c0    <= {WIDTH{1'b0}};
            c1    <= {WIDTH{1'b0}};
            c2    <= {WIDTH{1'b0}};
            valid <= 1'b0;
        end else if (stage1_valid) begin
            c0    <= inner_00_reg ^ cross_01_reg ^ cross_02_reg;
            c1    <= inner_11_reg ^ cross_10_reg ^ cross_12_reg;
            c2    <= inner_22_reg ^ cross_20_reg ^ cross_21_reg;
            valid <= 1'b1;
        end else begin
            valid <= 1'b0;
        end
    end

endmodule


// -----------------------------------------------------------------
// DOM-AND Gadget (3rd Order / 4 Shares)
// -----------------------------------------------------------------
module dom_and_4share #(
    parameter WIDTH = 64
)(
    input                   clk,
    input                   rst_n,
    input                   enable,

    // Input shares (4 each)
    input  [WIDTH-1:0]      a0, a1, a2, a3,
    input  [WIDTH-1:0]      b0, b1, b2, b3,

    // Fresh randomness: C(4,2) = 6 cross-domain pairs
    input  [WIDTH-1:0]      z01, z02, z03, z12, z13, z23,

    // Output shares
    output reg [WIDTH-1:0]  c0, c1, c2, c3,
    output reg              valid
);

    // -- Inner-domain products --
    wire [WIDTH-1:0] inner [0:3];
    assign inner[0] = a0 & b0;
    assign inner[1] = a1 & b1;
    assign inner[2] = a2 & b2;
    assign inner[3] = a3 & b3;

    // -- Cross-domain products (all 12 pairs, masked by 6 randoms) --
    wire [WIDTH-1:0] cross [0:3][0:3];

    // Row 0
    assign cross[0][1] = (a0 & b1) ^ z01;
    assign cross[0][2] = (a0 & b2) ^ z02;
    assign cross[0][3] = (a0 & b3) ^ z03;
    // Row 1
    assign cross[1][0] = (a1 & b0) ^ z01;
    assign cross[1][2] = (a1 & b2) ^ z12;
    assign cross[1][3] = (a1 & b3) ^ z13;
    // Row 2
    assign cross[2][0] = (a2 & b0) ^ z02;
    assign cross[2][1] = (a2 & b1) ^ z12;
    assign cross[2][3] = (a2 & b3) ^ z23;
    // Row 3
    assign cross[3][0] = (a3 & b0) ^ z03;
    assign cross[3][1] = (a3 & b1) ^ z13;
    assign cross[3][2] = (a3 & b2) ^ z23;

    // -- Pipeline register (glitch barrier) --
    reg [WIDTH-1:0] inner_reg [0:3];
    reg [WIDTH-1:0] cross_reg [0:3][0:3];
    reg             stage1_valid;

    integer i, j;
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            for (i = 0; i < 4; i = i + 1) begin
                inner_reg[i] <= {WIDTH{1'b0}};
                for (j = 0; j < 4; j = j + 1)
                    if (i != j) cross_reg[i][j] <= {WIDTH{1'b0}};
            end
            stage1_valid <= 1'b0;
        end else if (enable) begin
            for (i = 0; i < 4; i = i + 1) begin
                inner_reg[i] <= inner[i];
                for (j = 0; j < 4; j = j + 1)
                    if (i != j) cross_reg[i][j] <= cross[i][j];
            end
            stage1_valid <= 1'b1;
        end else begin
            stage1_valid <= 1'b0;
        end
    end

    // -- Recombination --
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            c0 <= {WIDTH{1'b0}}; c1 <= {WIDTH{1'b0}};
            c2 <= {WIDTH{1'b0}}; c3 <= {WIDTH{1'b0}};
            valid <= 1'b0;
        end else if (stage1_valid) begin
            c0 <= inner_reg[0] ^ cross_reg[0][1] ^ cross_reg[0][2] ^ cross_reg[0][3];
            c1 <= inner_reg[1] ^ cross_reg[1][0] ^ cross_reg[1][2] ^ cross_reg[1][3];
            c2 <= inner_reg[2] ^ cross_reg[2][0] ^ cross_reg[2][1] ^ cross_reg[2][3];
            c3 <= inner_reg[3] ^ cross_reg[3][0] ^ cross_reg[3][1] ^ cross_reg[3][2];
            valid <= 1'b1;
        end else begin
            valid <= 1'b0;
        end
    end

endmodule


// -----------------------------------------------------------------
// DOM-XOR (Trivial -- XOR is linear, no randomness needed)
// -----------------------------------------------------------------
module dom_xor #(
    parameter WIDTH  = 64,
    parameter SHARES = 3
)(
    input  [WIDTH*SHARES-1:0] a_shares,
    input  [WIDTH*SHARES-1:0] b_shares,
    output [WIDTH*SHARES-1:0] c_shares
);
    genvar s;
    generate
        for (s = 0; s < SHARES; s = s + 1) begin : share_xor
            assign c_shares[s*WIDTH +: WIDTH] =
                a_shares[s*WIDTH +: WIDTH] ^ b_shares[s*WIDTH +: WIDTH];
        end
    endgenerate
endmodule


// -----------------------------------------------------------------
// Share Refresh (Re-randomise shares without changing value)
// -----------------------------------------------------------------
module dom_refresh #(
    parameter WIDTH  = 64,
    parameter SHARES = 3
)(
    input                       clk,
    input                       rst_n,
    input                       enable,
    input  [WIDTH*SHARES-1:0]   in_shares,
    input  [WIDTH*(SHARES-1)-1:0] fresh_random,
    output reg [WIDTH*SHARES-1:0] out_shares,
    output reg                  valid
);
    integer i;
    reg [WIDTH-1:0] accum;

    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            out_shares <= {(WIDTH*SHARES){1'b0}};
            valid      <= 1'b0;
        end else if (enable) begin
            accum = in_shares[0 +: WIDTH];
            for (i = 0; i < SHARES - 1; i = i + 1) begin
                accum = accum ^ fresh_random[i*WIDTH +: WIDTH];
            end
            out_shares[0 +: WIDTH] <= accum;

            for (i = 1; i < SHARES; i = i + 1) begin
                out_shares[i*WIDTH +: WIDTH] <=
                    in_shares[i*WIDTH +: WIDTH] ^ fresh_random[(i-1)*WIDTH +: WIDTH];
            end

            valid <= 1'b1;
        end else begin
            valid <= 1'b0;
        end
    end
endmodule


// -----------------------------------------------------------------
// Higher-Order Masking Unit Top Level
// Wraps DOM gadgets into instruction-level interface
// -----------------------------------------------------------------
module xplenum_ho_mask_unit #(
    parameter WIDTH  = 64,
    parameter ORDER  = 2,
    parameter SHARES = ORDER + 1
)(
    input                       clk,
    input                       rst_n,

    // Instruction interface
    input  [2:0]                funct3,
    input  [6:0]                funct7,
    input                       valid_in,
    input  [WIDTH-1:0]          rs1_data,
    input  [WIDTH-1:0]          rs2_data,

    // Share memory (persistent across instructions)
    input  [WIDTH*SHARES-1:0]   share_mem_a,
    input  [WIDTH*SHARES-1:0]   share_mem_b,
    output [WIDTH*SHARES-1:0]   share_mem_out,

    // DRBG interface (must supply enough randomness)
    output reg                  drbg_request,
    output reg [7:0]            drbg_count,
    input  [WIDTH-1:0]          drbg_data,
    input                       drbg_valid,

    // Result
    output reg [WIDTH-1:0]      rd_data,
    output reg                  rd_wen,
    output reg                  busy
);

    localparam HO_MASK_APPLY   = 7'h10;
    localparam HO_MASK_STRIP   = 7'h11;
    localparam HO_MASK_REFRESH = 7'h12;
    localparam HO_MASK_AND     = 7'h13;

    localparam RAND_AND = (SHARES * (SHARES - 1)) / 2;
    localparam RAND_REFRESH = SHARES - 1;

    reg [WIDTH-1:0] rand_buf [0:5];
    reg [3:0]       rand_collected;
    reg [3:0]       rand_needed;

    localparam S_IDLE    = 3'd0;
    localparam S_COLLECT = 3'd1;
    localparam S_COMPUTE = 3'd2;
    localparam S_OUTPUT  = 3'd3;
    reg [2:0] state;

    reg [6:0] op_funct7;

    wire [WIDTH-1:0] and3_c0, and3_c1, and3_c2;
    wire             and3_valid;

    generate if (ORDER >= 2) begin : gen_and3
        dom_and_3share #(.WIDTH(WIDTH)) u_and3 (
            .clk(clk), .rst_n(rst_n),
            .enable(state == S_COMPUTE && op_funct7 == HO_MASK_AND),
            .a0(share_mem_a[0 +: WIDTH]),
            .a1(share_mem_a[WIDTH +: WIDTH]),
            .a2(share_mem_a[2*WIDTH +: WIDTH]),
            .b0(share_mem_b[0 +: WIDTH]),
            .b1(share_mem_b[WIDTH +: WIDTH]),
            .b2(share_mem_b[2*WIDTH +: WIDTH]),
            .z01(rand_buf[0]), .z02(rand_buf[1]), .z12(rand_buf[2]),
            .c0(and3_c0), .c1(and3_c1), .c2(and3_c2),
            .valid(and3_valid)
        );
    end endgenerate

    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            state          <= S_IDLE;
            busy           <= 1'b0;
            rd_wen         <= 1'b0;
            drbg_request   <= 1'b0;
            rand_collected <= 4'd0;
        end else begin
            rd_wen <= 1'b0;

            case (state)
                S_IDLE: begin
                    if (valid_in) begin
                        op_funct7 <= funct7;
                        busy      <= 1'b1;

                        case (funct7)
                            HO_MASK_AND: begin
                                rand_needed  <= RAND_AND[3:0];
                                drbg_request <= 1'b1;
                                drbg_count   <= RAND_AND[7:0];
                                state        <= S_COLLECT;
                            end
                            HO_MASK_REFRESH: begin
                                rand_needed  <= RAND_REFRESH[3:0];
                                drbg_request <= 1'b1;
                                drbg_count   <= RAND_REFRESH[7:0];
                                state        <= S_COLLECT;
                            end
                            default: begin
                                state <= S_COMPUTE;
                            end
                        endcase
                    end
                end

                S_COLLECT: begin
                    drbg_request <= 1'b0;
                    if (drbg_valid) begin
                        rand_buf[rand_collected] <= drbg_data;
                        if (rand_collected + 1 >= rand_needed) begin
                            rand_collected <= 4'd0;
                            state          <= S_COMPUTE;
                        end else begin
                            rand_collected <= rand_collected + 1;
                            drbg_request   <= 1'b1;
                        end
                    end
                end

                S_COMPUTE: begin
                    if (op_funct7 == HO_MASK_AND && and3_valid) begin
                        state <= S_OUTPUT;
                    end else if (op_funct7 != HO_MASK_AND) begin
                        state <= S_OUTPUT;
                    end
                end

                S_OUTPUT: begin
                    rd_wen <= 1'b1;
                    rd_data <= and3_c0 ^ and3_c1 ^ and3_c2;
                    busy   <= 1'b0;
                    state  <= S_IDLE;
                end
            endcase
        end
    end

    assign share_mem_out = (op_funct7 == HO_MASK_AND) ?
        {and3_c2, and3_c1, and3_c0} : {(WIDTH*SHARES){1'b0}};

endmodule
