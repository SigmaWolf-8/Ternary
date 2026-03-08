// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// xplenum_crt_unit.v — CRT Fast Path Pipeline
//
// Hardware implementation of the 364 = 13 × 28 progressive refinement.
// Exploits CRT decomposition Z₃₆₄ ≅ Z₁₃ × Z₂₈ to deliver partial
// routing information at earlier pipeline stages than full mod-364.
//
// PIPELINE STAGES (clock-by-clock):
//   Stage 0 (wire):     mod_4 — quarter-day phase (2 bits, 0 cycles)
//   Stage 1 (1 clock):  mod_7 — via octal digit sum + LUT
//   Stage 2 (1 clock):  mod_28 — CRT of mod_4 × mod_7; mod_13 partial
//   Stage 3 (1 clock):  mod_13 — via weighted nibble sum + LUT
//   Stage 4 (1 clock):  full circle_position — CRT of mod_28 × mod_13
//
// LATENCY:
//   First routing bit (quarter): 0 cycles (combinational)
//   Coarse sector (mod-28):      2 cycles
//   Full position (mod-364):     4 cycles
//
// At 200 MHz FPGA: coarse = 10 ns, full = 20 ns → 10 ns head start
// At 1 GHz ASIC:   coarse = 2 ns, full = 4 ns  → 2 ns head start
//
// INTEGRATION:
//   Sits on the XPlenum custom-1 (0x2B) opcode bus alongside xplenum_pqc_unit.
//   Input: rs1 (32-bit register). Output: rd (32-bit packed result).

module xplenum_crt_unit (
    input  wire        clk,
    input  wire        rst_n,          // active-low reset

    // Pipeline input
    input  wire        in_valid,       // input handshake
    input  wire [31:0] rs1_data,       // 32-bit input (timestamp or position)
    input  wire        in_mode,        // 0: full 32-bit reduction, 1: pre-reduced 9-bit

    // Pipeline output — available progressively
    output reg  [31:0] rd_data,        // packed result (see encoding below)
    output reg         out_valid,      // full result ready
    output wire        coarse_valid,   // mod-28 (day component) ready
    output wire        quarter_valid,  // mod-4 (quarter phase) ready

    // Individual outputs for direct routing
    output wire [1:0]  quarter_phase,  // Stage 0: rs1[1:0], combinational
    output reg  [2:0]  mod7_result,    // Stage 1: clock source index (0-6)
    output reg  [4:0]  mod28_result,   // Stage 2: day component (0-27)
    output reg  [3:0]  mod13_result,   // Stage 3: moon component (0-12)
    output reg  [8:0]  circle_pos      // Stage 4: full position (0-363)
);

    // ═══════════════════════════════════════════════════════════
    // CONSTANTS (from INVARIANT 4 and CRT precomputation)
    // ═══════════════════════════════════════════════════════════

    // CRT reconstruction: mod-28 from (mod-4, mod-7)
    // 4⁻¹ mod 7 = 2, 7⁻¹ mod 4 = 3
    // result = (21 × r4 + 8 × r7) mod 28
    localparam [4:0] CRT28_COEFF_R4 = 5'd21;
    localparam [3:0] CRT28_COEFF_R7 = 4'd8;

    // CRT reconstruction: mod-364 from (mod-13, mod-28)
    // 28⁻¹ mod 13 = 7, 13⁻¹ mod 28 = 13
    // result = (196 × mod13 + 169 × mod28) mod 364
    localparam [7:0] CRT364_COEFF_FINE = 8'd196;  // 28 × 7
    localparam [7:0] CRT364_COEFF_FAST = 8'd169;  // 13 × 13

    // ═══════════════════════════════════════════════════════════
    // STAGE 0: mod-4 (COMBINATIONAL — zero latency)
    // ═══════════════════════════════════════════════════════════

    // mod-4 is literally just the two least significant bits.
    // This is a wire, not a register. Available IMMEDIATELY.
    assign quarter_phase = rs1_data[1:0];
    assign quarter_valid = in_valid;  // valid the instant input arrives

    // ═══════════════════════════════════════════════════════════
    // STAGE 1: mod-7 via octal digit sum (1 clock cycle)
    // ═══════════════════════════════════════════════════════════
    //
    // Mathematical basis: 8 ≡ 1 (mod 7), therefore:
    //   n = Σ d_i × 8^i ≡ Σ d_i (mod 7)
    //
    // Split 32-bit input into 11 octal digits (3-bit groups),
    // sum them, then reduce the 7-bit sum via small LUT.

    wire [2:0] octal_digits [0:10];
    wire [6:0] octal_sum;

    assign octal_digits[0]  = rs1_data[2:0];
    assign octal_digits[1]  = rs1_data[5:3];
    assign octal_digits[2]  = rs1_data[8:6];
    assign octal_digits[3]  = rs1_data[11:9];
    assign octal_digits[4]  = rs1_data[14:12];
    assign octal_digits[5]  = rs1_data[17:15];
    assign octal_digits[6]  = rs1_data[20:18];
    assign octal_digits[7]  = rs1_data[23:21];
    assign octal_digits[8]  = rs1_data[26:24];
    assign octal_digits[9]  = rs1_data[29:27];
    assign octal_digits[10] = {1'b0, rs1_data[31:30]};  // 2 MSBs, zero-padded

    // Adder tree: sum of 11 values, each 0-7. Max sum = 77. Fits in 7 bits.
    assign octal_sum = octal_digits[0] + octal_digits[1] + octal_digits[2]
                     + octal_digits[3] + octal_digits[4] + octal_digits[5]
                     + octal_digits[6] + octal_digits[7] + octal_digits[8]
                     + octal_digits[9] + octal_digits[10];

    // mod-7 LUT for the reduced sum (0-77 → 0-6)
    // Synthesizer will implement this as a small ROM or logic cone
    reg [2:0] mod7_lut_out;
    always @(*) begin
        // For values 0-77, compute mod 7
        // The synthesizer optimizes this into minimal logic
        case (octal_sum % 7)  // synth hint: constant folding at elaboration
            3'd0: mod7_lut_out = 3'd0;
            3'd1: mod7_lut_out = 3'd1;
            3'd2: mod7_lut_out = 3'd2;
            3'd3: mod7_lut_out = 3'd3;
            3'd4: mod7_lut_out = 3'd4;
            3'd5: mod7_lut_out = 3'd5;
            3'd6: mod7_lut_out = 3'd6;
            default: mod7_lut_out = 3'd0;
        endcase
    end

    // Actually, let's use a proper modular reduction for synthesis.
    // For a 7-bit value (0-77), mod 7 via iterative subtraction is fine
    // in hardware — the synthesizer will flatten this.
    wire [2:0] mod7_comb;
    assign mod7_comb = (octal_sum < 7)  ? octal_sum[2:0] :
                       (octal_sum < 14) ? (octal_sum - 7) :
                       (octal_sum < 21) ? (octal_sum - 14) :
                       (octal_sum < 28) ? (octal_sum - 21) :
                       (octal_sum < 35) ? (octal_sum - 28) :
                       (octal_sum < 42) ? (octal_sum - 35) :
                       (octal_sum < 49) ? (octal_sum - 42) :
                       (octal_sum < 56) ? (octal_sum - 49) :
                       (octal_sum < 63) ? (octal_sum - 56) :
                       (octal_sum < 70) ? (octal_sum - 63) :
                       (octal_sum < 77) ? (octal_sum - 70) :
                                          (octal_sum - 77);

    // Pipeline register: Stage 1
    reg        s1_valid;
    reg [1:0]  s1_quarter;  // carry forward from Stage 0

    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            s1_valid    <= 1'b0;
            mod7_result <= 3'd0;
            s1_quarter  <= 2'd0;
        end else begin
            s1_valid    <= in_valid;
            mod7_result <= mod7_comb;
            s1_quarter  <= quarter_phase;
        end
    end

    // ═══════════════════════════════════════════════════════════
    // STAGE 2: mod-28 via CRT of mod-4 and mod-7 (1 clock cycle)
    // ═══════════════════════════════════════════════════════════
    //
    // mod-28 = (21 × r4 + 8 × r7) mod 28
    // Max value: 21*3 + 8*6 = 63+48 = 111. Need mod-28 of a 7-bit value.

    wire [6:0] crt28_sum;
    assign crt28_sum = CRT28_COEFF_R4 * {5'd0, s1_quarter}
                     + CRT28_COEFF_R7 * {4'd0, mod7_result};

    wire [4:0] mod28_comb;
    assign mod28_comb = (crt28_sum < 28)  ? crt28_sum[4:0] :
                        (crt28_sum < 56)  ? (crt28_sum - 28) :
                        (crt28_sum < 84)  ? (crt28_sum - 56) :
                                            (crt28_sum - 84);

    // Pipeline register: Stage 2
    reg        s2_valid;
    reg [2:0]  s2_mod7;    // carry forward

    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            s2_valid     <= 1'b0;
            mod28_result <= 5'd0;
            s2_mod7      <= 3'd0;
        end else begin
            s2_valid     <= s1_valid;
            mod28_result <= mod28_comb;
            s2_mod7      <= mod7_result;
        end
    end

    // Coarse decision is valid at Stage 2
    assign coarse_valid = s2_valid;

    // ═══════════════════════════════════════════════════════════
    // STAGE 2-3: mod-13 via weighted nibble sum (parallel path)
    // ═══════════════════════════════════════════════════════════
    //
    // Mathematical basis: 16^k mod 13 cycles as {1, 3, 9, 1, 3, 9, ...}
    //   n = Σ d_i × 16^i ≡ Σ (d_i × w_i) (mod 13)
    //   where w = {1, 3, 9, 1, 3, 9, 1, 3} for 8 nibbles
    //
    // Starts at Stage 1, completes at Stage 3 (2-cycle path).

    wire [3:0] nibbles [0:7];
    assign nibbles[0] = rs1_data[3:0];
    assign nibbles[1] = rs1_data[7:4];
    assign nibbles[2] = rs1_data[11:8];
    assign nibbles[3] = rs1_data[15:12];
    assign nibbles[4] = rs1_data[19:16];
    assign nibbles[5] = rs1_data[23:20];
    assign nibbles[6] = rs1_data[27:24];
    assign nibbles[7] = rs1_data[31:28];

    // Weighted sum: weights are {1, 3, 9, 1, 3, 9, 1, 3}
    // Max weighted sum: 15*(1+3+9+1+3+9+1+3) = 15*30 = 450. Fits in 9 bits.
    wire [8:0] weighted_sum;
    assign weighted_sum = (nibbles[0] * 4'd1)  + (nibbles[1] * 4'd3)
                        + (nibbles[2] * 4'd9)  + (nibbles[3] * 4'd1)
                        + (nibbles[4] * 4'd3)  + (nibbles[5] * 4'd9)
                        + (nibbles[6] * 4'd1)  + (nibbles[7] * 4'd3);

    // Stage 2 register for mod-13 partial result (weighted sum)
    reg [8:0] s2_weighted_sum;
    reg       s2_mod13_partial_valid;

    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            s2_weighted_sum        <= 9'd0;
            s2_mod13_partial_valid <= 1'b0;
        end else begin
            s2_weighted_sum        <= weighted_sum;
            s2_mod13_partial_valid <= s1_valid;
        end
    end

    // Stage 3: reduce weighted_sum mod 13 via cascaded subtraction
    // Input range: 0-450. Max iterations: 450/13 = 34. Too many for cascade.
    // Instead: two-stage reduction.
    // First reduce: weighted_sum mod 13 using a 512-entry LUT (9-bit → 4-bit).
    // In FPGA this maps to a single BRAM or distributed ROM.

    // For synthesis, use behavioral mod — the tool optimizes this
    wire [3:0] mod13_comb;
    assign mod13_comb = s2_weighted_sum % 13;  // Synthesizer handles this

    // Pipeline register: Stage 3
    reg        s3_valid;
    reg [4:0]  s3_mod28;   // carry forward

    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            s3_valid     <= 1'b0;
            mod13_result <= 4'd0;
            s3_mod28     <= 5'd0;
        end else begin
            s3_valid     <= s2_valid;
            mod13_result <= mod13_comb;
            s3_mod28     <= mod28_result;
        end
    end

    // ═══════════════════════════════════════════════════════════
    // STAGE 4: CRT reconstruction — full circle position
    // ═══════════════════════════════════════════════════════════
    //
    // circle_pos = (196 × mod13 + 169 × mod28) mod 364
    // Max value: 196*12 + 169*27 = 2352 + 4563 = 6915. Fits in 13 bits.

    wire [12:0] crt364_sum;
    assign crt364_sum = CRT364_COEFF_FINE * {9'd0, mod13_result}
                      + CRT364_COEFF_FAST * {8'd0, s3_mod28};

    // mod-364 of a 13-bit value (0-6915): max 6915/364 = 18 iterations.
    // Use behavioral mod — synthesizer generates a divider or ROM.
    wire [8:0] circle_pos_comb;
    assign circle_pos_comb = crt364_sum % 364;

    // Pipeline register: Stage 4 (final)
    // ALL components must be captured here so rd_data reads synchronized values.
    // rd_data is registered in the SAME always block as out_valid so they're
    // guaranteed aligned — no non-blocking assignment timing hazard.
    reg [1:0]  s4_quarter;
    reg [2:0]  s4_mod7;
    reg [4:0]  s4_mod28;
    reg [3:0]  s4_mod13;

    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            out_valid  <= 1'b0;
            circle_pos <= 9'd0;
            s4_quarter <= 2'd0;
            s4_mod7    <= 3'd0;
            s4_mod28   <= 5'd0;
            s4_mod13   <= 4'd0;
            rd_data    <= 32'd0;
        end else begin
            out_valid  <= s3_valid;
            circle_pos <= circle_pos_comb;
            s4_mod13   <= mod13_result;
            s4_mod28   <= s3_mod28;
            s4_mod7    <= s2_mod7;
            s4_quarter <= quarter_phase;

            // Pack rd_data in the same always block — synchronized with out_valid
            if (s3_valid) begin
                // Full result will be valid next cycle (when out_valid rises)
                rd_data <= {7'd0,                       // [31:25] reserved
                            1'b1,                        // [24]    fine_valid
                            1'b1,                        // [23]    coarse_valid
                            circle_pos_comb,             // [22:14] full position
                            mod13_result,                // [13:10] moon
                            s3_mod28,                    // [9:5]   day
                            s2_mod7,                     // [4:2]   clock source
                            quarter_phase};              // [1:0]   quarter
            end else if (s2_valid) begin
                // Coarse result — will be valid when coarse_valid rises
                rd_data <= {7'd0,                       // [31:25] reserved
                            1'b0,                        // [24]    fine_valid = 0
                            1'b1,                        // [23]    coarse_valid = 1
                            9'd0,                        // [22:14] not yet
                            4'd0,                        // [13:10] not yet
                            mod28_result,                // [9:5]   day
                            mod7_result,                 // [4:2]   clock source
                            quarter_phase};              // [1:0]   quarter
            end
        end
    end

endmodule
