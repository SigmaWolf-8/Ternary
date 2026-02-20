// =============================================================================
// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
// Patent(s) Pending.
//
// XPLENUM — CTR_DRBG (NIST SP 800-90A Section 10.2.1)
// Deterministic Random Bit Generator using AES-256 in counter mode
// Replaces LFSR in xplenum_mask_unit.v for FIPS 140-3 compliance
//
// SP 800-90A Functions Implemented:
//   - CTR_DRBG_Instantiate (seed_valid_i)
//   - CTR_DRBG_Generate    (generate_i)
//   - CTR_DRBG_Reseed      (reseed_i)
//   - CTR_DRBG_Update      (internal)
//
// SP 800-90B Health Tests:
//   - Repetition Count Test (identical consecutive outputs)
//   - Adaptive Proportion Test (statistical bias in window)
// =============================================================================

`include "xplenum_pkg.vh"

module xplenum_ctr_drbg (
    input  wire         clk,
    input  wire         rst_n,

    input  wire [255:0] seed_i,
    input  wire         seed_valid_i,
    input  wire         reseed_i,

    input  wire         generate_i,

    output reg  [31:0]  drbg_data_o,
    output reg          drbg_valid_o,

    output wire         health_error_o,
    output wire         ready_o
);

    // -----------------------------------------------------------------------
    // CTR_DRBG Internal State (SP 800-90A Section 10.2.1)
    // -----------------------------------------------------------------------
    reg [255:0] drbg_key;
    reg [127:0] drbg_v;
    reg [47:0]  reseed_counter;
    reg         instantiated;

    localparam [47:0] RESEED_INTERVAL = 48'h1_0000_0000;

    // -----------------------------------------------------------------------
    // AES-256 Core Instance
    // -----------------------------------------------------------------------
    reg  [127:0] aes_plaintext;
    reg  [255:0] aes_key;
    reg          aes_start;
    wire [127:0] aes_ciphertext;
    wire         aes_done;
    wire         aes_busy;

    xplenum_aes256_core u_aes (
        .clk          (clk),
        .rst_n        (rst_n),
        .plaintext_i  (aes_plaintext),
        .key_i        (aes_key),
        .start_i      (aes_start),
        .ciphertext_o (aes_ciphertext),
        .done_o       (aes_done),
        .busy_o       (aes_busy)
    );

    // -----------------------------------------------------------------------
    // SP 800-90B Health Tests
    // -----------------------------------------------------------------------
    reg [127:0] prev_output;
    reg         prev_valid;
    reg         rep_count_fail;

    reg [7:0]   prop_count;
    reg [7:0]   prop_window_pos;
    reg [127:0] prop_reference;
    reg         prop_test_fail;

    localparam [7:0] PROP_WINDOW   = 8'd64;
    localparam [7:0] PROP_CUTOFF   = 8'd9;
    localparam [7:0] REP_CUTOFF    = 8'd5;

    reg [7:0] rep_count;

    assign health_error_o = rep_count_fail | prop_test_fail;

    // -----------------------------------------------------------------------
    // CTR_DRBG FSM
    // -----------------------------------------------------------------------
    localparam S_IDLE         = 4'd0;
    localparam S_INSTANTIATE  = 4'd1;
    localparam S_UPDATE_ENC1  = 4'd2;
    localparam S_UPDATE_WAIT1 = 4'd3;
    localparam S_UPDATE_ENC2  = 4'd4;
    localparam S_UPDATE_WAIT2 = 4'd5;
    localparam S_UPDATE_FINAL = 4'd6;
    localparam S_GENERATE_INC = 4'd7;
    localparam S_GENERATE_ENC = 4'd8;
    localparam S_GENERATE_WAIT= 4'd9;
    localparam S_GENERATE_OUT = 4'd10;
    localparam S_GEN_UPDATE1  = 4'd11;
    localparam S_GEN_UPDATE2  = 4'd12;
    localparam S_GEN_UPDATE3  = 4'd13;
    localparam S_GEN_UPDATE4  = 4'd14;

    reg [3:0]   fsm_state;
    reg [127:0] update_block1;
    reg [127:0] update_block2;
    reg [255:0] update_seed;
    reg [127:0] gen_output;
    reg [1:0]   output_word_idx;

    assign ready_o = (fsm_state == S_IDLE) && instantiated && !health_error_o;

    // -----------------------------------------------------------------------
    // Increment V (big-endian 128-bit counter)
    // -----------------------------------------------------------------------
    function [127:0] inc_v;
        input [127:0] v;
        begin
            inc_v = v + 128'd1;
        end
    endfunction

    // -----------------------------------------------------------------------
    // Main FSM
    // -----------------------------------------------------------------------
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            fsm_state      <= S_IDLE;
            drbg_key       <= 256'h0;
            drbg_v         <= 128'h0;
            reseed_counter <= 48'h0;
            instantiated   <= 1'b0;
            drbg_data_o    <= 32'h0;
            drbg_valid_o   <= 1'b0;
            aes_start      <= 1'b0;
            aes_plaintext  <= 128'h0;
            aes_key        <= 256'h0;
            update_block1  <= 128'h0;
            update_block2  <= 128'h0;
            update_seed    <= 256'h0;
            gen_output     <= 128'h0;
            output_word_idx <= 2'h0;

            prev_output    <= 128'h0;
            prev_valid     <= 1'b0;
            rep_count_fail <= 1'b0;
            rep_count      <= 8'h0;
            prop_count     <= 8'h0;
            prop_window_pos <= 8'h0;
            prop_reference <= 128'h0;
            prop_test_fail <= 1'b0;
        end else begin
            aes_start    <= 1'b0;
            drbg_valid_o <= 1'b0;

            case (fsm_state)
                // ---------------------------------------------------------
                // IDLE: Accept instantiate, reseed, or generate requests
                // ---------------------------------------------------------
                S_IDLE: begin
                    if (seed_valid_i) begin
                        update_seed <= seed_i;
                        drbg_key    <= 256'h0;
                        drbg_v      <= 128'h0;
                        fsm_state   <= S_UPDATE_ENC1;
                    end else if (reseed_i && instantiated) begin
                        update_seed <= seed_i;
                        fsm_state   <= S_UPDATE_ENC1;
                    end else if (generate_i && instantiated && !health_error_o) begin
                        if (reseed_counter >= RESEED_INTERVAL) begin
                            drbg_valid_o <= 1'b0;
                        end else begin
                            fsm_state <= S_GENERATE_INC;
                        end
                    end
                end

                // ---------------------------------------------------------
                // CTR_DRBG_Update: Encrypt two blocks to produce new Key||V
                // Block 1: V+1
                // ---------------------------------------------------------
                S_UPDATE_ENC1: begin
                    drbg_v        <= inc_v(drbg_v);
                    aes_plaintext <= inc_v(drbg_v);
                    aes_key       <= drbg_key;
                    aes_start     <= 1'b1;
                    fsm_state     <= S_UPDATE_WAIT1;
                end

                S_UPDATE_WAIT1: begin
                    if (aes_done) begin
                        update_block1 <= aes_ciphertext;
                        fsm_state     <= S_UPDATE_ENC2;
                    end
                end

                // Block 2: V+2
                S_UPDATE_ENC2: begin
                    drbg_v        <= inc_v(drbg_v);
                    aes_plaintext <= inc_v(drbg_v);
                    aes_start     <= 1'b1;
                    fsm_state     <= S_UPDATE_WAIT2;
                end

                S_UPDATE_WAIT2: begin
                    if (aes_done) begin
                        update_block2 <= aes_ciphertext;
                        fsm_state     <= S_UPDATE_FINAL;
                    end
                end

                // XOR with provided_data (seed material)
                S_UPDATE_FINAL: begin
                    drbg_key       <= {update_block1, update_block2} ^ update_seed;
                    drbg_v         <= update_block2 ^ update_seed[127:0];
                    reseed_counter <= (seed_valid_i || reseed_i) ? 48'd1 : reseed_counter;
                    instantiated   <= 1'b1;
                    fsm_state      <= S_IDLE;

                    prev_valid     <= 1'b0;
                    rep_count      <= 8'h0;
                    prop_window_pos <= 8'h0;
                    prop_count     <= 8'h0;
                end

                // ---------------------------------------------------------
                // CTR_DRBG_Generate: Produce one 128-bit output block
                // ---------------------------------------------------------
                S_GENERATE_INC: begin
                    drbg_v        <= inc_v(drbg_v);
                    aes_plaintext <= inc_v(drbg_v);
                    aes_key       <= drbg_key;
                    aes_start     <= 1'b1;
                    fsm_state     <= S_GENERATE_WAIT;
                end

                S_GENERATE_WAIT: begin
                    if (aes_done) begin
                        gen_output     <= aes_ciphertext;
                        output_word_idx <= 2'h0;
                        fsm_state      <= S_GENERATE_OUT;

                        // Health tests on raw 128-bit output
                        if (prev_valid && (aes_ciphertext == prev_output)) begin
                            rep_count <= rep_count + 1;
                            if (rep_count + 1 >= REP_CUTOFF)
                                rep_count_fail <= 1'b1;
                        end else begin
                            rep_count <= 8'd1;
                        end
                        prev_output <= aes_ciphertext;
                        prev_valid  <= 1'b1;

                        if (prop_window_pos == 8'd0) begin
                            prop_reference  <= aes_ciphertext;
                            prop_count      <= 8'd1;
                            prop_window_pos <= 8'd1;
                        end else begin
                            if (aes_ciphertext == prop_reference)
                                prop_count <= prop_count + 1;
                            prop_window_pos <= prop_window_pos + 1;
                            if (prop_window_pos + 1 >= PROP_WINDOW) begin
                                if (prop_count >= PROP_CUTOFF)
                                    prop_test_fail <= 1'b1;
                                prop_window_pos <= 8'd0;
                                prop_count      <= 8'd0;
                            end
                        end
                    end
                end

                // Output 32-bit words from the 128-bit generated block
                S_GENERATE_OUT: begin
                    case (output_word_idx)
                        2'h0: drbg_data_o <= gen_output[127:96];
                        2'h1: drbg_data_o <= gen_output[ 95:64];
                        2'h2: drbg_data_o <= gen_output[ 63:32];
                        2'h3: drbg_data_o <= gen_output[ 31: 0];
                    endcase
                    drbg_valid_o    <= 1'b1;
                    output_word_idx <= output_word_idx + 1;

                    if (output_word_idx == 2'h0) begin
                        reseed_counter <= reseed_counter + 1;
                        fsm_state      <= S_GEN_UPDATE1;
                    end
                end

                // Post-generate Update (SP 800-90A mandates state update after generate)
                S_GEN_UPDATE1: begin
                    update_seed   <= 256'h0;
                    drbg_v        <= inc_v(drbg_v);
                    aes_plaintext <= inc_v(drbg_v);
                    aes_key       <= drbg_key;
                    aes_start     <= 1'b1;
                    fsm_state     <= S_GEN_UPDATE2;
                end

                S_GEN_UPDATE2: begin
                    if (aes_done) begin
                        update_block1 <= aes_ciphertext;
                        drbg_v        <= inc_v(drbg_v);
                        aes_plaintext <= inc_v(drbg_v);
                        aes_start     <= 1'b1;
                        fsm_state     <= S_GEN_UPDATE3;
                    end
                end

                S_GEN_UPDATE3: begin
                    if (aes_done) begin
                        update_block2 <= aes_ciphertext;
                        fsm_state     <= S_GEN_UPDATE4;
                    end
                end

                S_GEN_UPDATE4: begin
                    drbg_key  <= {update_block1, update_block2};
                    drbg_v    <= update_block2;
                    fsm_state <= S_IDLE;
                end
            endcase
        end
    end

endmodule
