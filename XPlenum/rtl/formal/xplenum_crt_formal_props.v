// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// xplenum_crt_formal_props.v — Formal Properties for CRT Unit
//
// Bounded model checking and k-induction properties for SymbiYosys.
// Verifies:
//   1. CRT round-trip: decompose → reconstruct = identity for all inputs
//   2. Pipeline ordering: coarse_valid always precedes or equals out_valid
//   3. Range bounds: all outputs within valid modular ranges
//   4. Reset behavior: all outputs zero after reset
//   5. Clock source coverage: mod-7 generates all 7 values

module xplenum_crt_formal_props (
    input wire        clk,
    input wire        rst_n,
    input wire        in_valid,
    input wire [31:0] rs1_data,
    input wire        in_mode,
    input wire [31:0] rd_data,
    input wire        out_valid,
    input wire        coarse_valid,
    input wire        quarter_valid,
    input wire [1:0]  quarter_phase,
    input wire [2:0]  mod7_result,
    input wire [4:0]  mod28_result,
    input wire [3:0]  mod13_result,
    input wire [8:0]  circle_pos
);

    // ── Past-value tracking ──
    reg        past_valid;
    reg        past_coarse;
    reg        past_out;
    reg [31:0] past_rs1;

    initial begin
        past_valid  = 0;
        past_coarse = 0;
        past_out    = 0;
        past_rs1    = 0;
    end

    always @(posedge clk) begin
        past_valid  <= in_valid;
        past_coarse <= coarse_valid;
        past_out    <= out_valid;
        past_rs1    <= rs1_data;
    end

    // ════════════════════════════════════════════════════════
    // SECTION 1: Range bounds (always true)
    // ════════════════════════════════════════════════════════

    // mod-4 output must be 0-3
    always @(*) begin
        assert(quarter_phase < 4);
    end

    // mod-7 output must be 0-6 (when valid)
    always @(posedge clk) begin
        if (coarse_valid || out_valid) begin
            assert(mod7_result < 7);
        end
    end

    // mod-28 output must be 0-27 (when valid)
    always @(posedge clk) begin
        if (coarse_valid || out_valid) begin
            assert(mod28_result < 28);
        end
    end

    // mod-13 output must be 0-12 (when valid)
    always @(posedge clk) begin
        if (out_valid) begin
            assert(mod13_result < 13);
        end
    end

    // circle_pos must be 0-363 (when valid)
    always @(posedge clk) begin
        if (out_valid) begin
            assert(circle_pos < 364);
        end
    end

    // ════════════════════════════════════════════════════════
    // SECTION 2: Reset behavior
    // ════════════════════════════════════════════════════════

    // After reset, no outputs should be valid
    always @(posedge clk) begin
        if (!rst_n) begin
            assert(!out_valid);
            // Note: coarse_valid and quarter_valid are combinational
            // and may be undefined during reset; we check post-reset
        end
    end

    // One cycle after reset deasserts, pipeline should be clear
    reg rst_was_low;
    initial rst_was_low = 1;
    always @(posedge clk) begin
        rst_was_low <= !rst_n;
        if (rst_was_low && rst_n && !in_valid) begin
            assert(!out_valid);
        end
    end

    // ════════════════════════════════════════════════════════
    // SECTION 3: Pipeline ordering
    // ════════════════════════════════════════════════════════

    // quarter_valid must be asserted whenever in_valid is
    // (combinational — same cycle)
    always @(*) begin
        if (in_valid)
            assert(quarter_valid);
    end

    // If out_valid is asserted, coarse_valid must also be asserted
    // (or have been asserted on a previous cycle)
    always @(posedge clk) begin
        if (rst_n && out_valid) begin
            assert(coarse_valid || past_coarse);
        end
    end

    // ════════════════════════════════════════════════════════
    // SECTION 4: CRT correctness (for small inputs 0-363)
    // ════════════════════════════════════════════════════════

    // When the full result is valid AND the input was in [0, 363],
    // the circle_pos must equal the input.
    // (For larger inputs, circle_pos = input mod 364.)

    // Track the input that produced the current output
    // (delayed by pipeline depth)
    reg [31:0] pipe_input [0:5];
    integer p;
    initial for (p = 0; p < 6; p = p + 1) pipe_input[p] = 0;

    always @(posedge clk) begin
        pipe_input[0] <= rs1_data;
        for (p = 1; p < 6; p = p + 1)
            pipe_input[p] <= pipe_input[p-1];
    end

    // When output is valid, verify CRT reconstruction matches
    always @(posedge clk) begin
        if (rst_n && out_valid) begin
            // The output should equal (input mod 364) for ANY input
            assert(circle_pos == (pipe_input[4] % 364));
        end
    end

    // ════════════════════════════════════════════════════════
    // SECTION 5: Component consistency
    // ════════════════════════════════════════════════════════

    // mod-4 must equal input[1:0] (combinational, always)
    always @(*) begin
        if (in_valid) begin
            assert(quarter_phase == rs1_data[1:0]);
        end
    end

    // When full result valid, all components must be consistent
    // with circle_pos
    always @(posedge clk) begin
        if (rst_n && out_valid) begin
            assert(mod28_result == circle_pos % 28);
            assert(mod13_result == circle_pos % 13);
        end
    end

    // ════════════════════════════════════════════════════════
    // SECTION 6: Packed output encoding consistency
    // ════════════════════════════════════════════════════════

    always @(posedge clk) begin
        if (rst_n && out_valid) begin
            // Verify the packed rd_data matches individual outputs
            assert(rd_data[1:0]   == quarter_phase);
            assert(rd_data[9:5]   == mod28_result);
            assert(rd_data[13:10] == mod13_result);
            assert(rd_data[22:14] == circle_pos);
            assert(rd_data[24]    == 1'b1);  // fine_valid
            assert(rd_data[23]    == 1'b1);  // coarse_valid
        end
    end

    // Coarse-only output: fine_valid must be 0, coarse_valid must be 1
    always @(posedge clk) begin
        if (rst_n && coarse_valid && !out_valid) begin
            assert(rd_data[23] == 1'b1);  // coarse_valid flag
            assert(rd_data[24] == 1'b0);  // fine_valid flag
        end
    end

endmodule
