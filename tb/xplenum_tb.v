// =============================================================================
// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
// Patent(s) Pending.
//
// XPLENUM — RISC-V Ternary Security Extension
// Comprehensive Testbench (xplenum_tb.v)
// Stage 8 — 22 directed test cases across all functional groups
// =============================================================================

`include "xplenum_pkg.vh"

`timescale 1ns / 1ps

module xplenum_tb;

    // -----------------------------------------------------------------------
    // Clock and reset
    // -----------------------------------------------------------------------
    reg        clk;
    reg        rst_n;

    // DUT signals
    reg  [31:0] instruction;
    reg         instr_valid;
    reg  [31:0] rs1_data;
    reg  [31:0] rs2_data;

    wire [31:0] rd_data;
    wire        rd_write_en;
    wire [4:0]  rd_addr;
    wire        xp_exception;
    wire [3:0]  xp_exc_code;

    // -----------------------------------------------------------------------
    // DUT instantiation
    // -----------------------------------------------------------------------
    xplenum_top dut (
        .clk          (clk),
        .rst_n        (rst_n),
        .instruction  (instruction),
        .instr_valid  (instr_valid),
        .rs1_data     (rs1_data),
        .rs2_data     (rs2_data),
        .rd_data      (rd_data),
        .rd_write_en  (rd_write_en),
        .rd_addr      (rd_addr),
        .xp_exception (xp_exception),
        .xp_exc_code  (xp_exc_code)
    );

    // -----------------------------------------------------------------------
    // Clock generation: 10ns period (100 MHz)
    // -----------------------------------------------------------------------
    initial clk = 0;
    always #5 clk = ~clk;

    // -----------------------------------------------------------------------
    // Test counters
    // -----------------------------------------------------------------------
    integer test_num;
    integer pass_count;
    integer fail_count;

    // -----------------------------------------------------------------------
    // Instruction encoding helper
    // funct7[6:0] | rs2[4:0] | rs1[4:0] | funct3[2:0] | rd[4:0] | opcode[6:0]
    // -----------------------------------------------------------------------
    function [31:0] encode_xp;
        input [6:0] f7;
        input [4:0] r2;
        input [4:0] r1;
        input [2:0] f3;
        input [4:0] d;
        begin
            encode_xp = {f7, r2, r1, f3, d, `XP_OPCODE};
        end
    endfunction

    // CSR encoding: funct7[6]=WR, rs2[4:0]=CSR_OFFSET, rs1=src, rd=dest
    function [31:0] encode_csr_wr;
        input [3:0] csr_offset;
        input [4:0] r1;
        input [4:0] d;
        begin
            encode_csr_wr = {7'b1000000, {1'b0, csr_offset}, r1, `F3_TCSR, d, `XP_OPCODE};
        end
    endfunction

    function [31:0] encode_csr_rd;
        input [3:0] csr_offset;
        input [4:0] d;
        begin
            encode_csr_rd = {7'b0000000, {1'b0, csr_offset}, 5'b0, `F3_TCSR, d, `XP_OPCODE};
        end
    endfunction

    // -----------------------------------------------------------------------
    // Task: execute instruction and wait for result
    // -----------------------------------------------------------------------
    task exec;
        input [31:0] instr;
        input [31:0] src1;
        input [31:0] src2;
        begin
            @(posedge clk);
            instruction = instr;
            rs1_data    = src1;
            rs2_data    = src2;
            instr_valid = 1'b1;
            @(posedge clk);
            instr_valid = 1'b0;
            @(posedge clk); // Subunit processes
            @(posedge clk); // Top output stage registers result
        end
    endtask

    // -----------------------------------------------------------------------
    // Task: check result
    // -----------------------------------------------------------------------
    task check;
        input [31:0] expected;
        input [159:0] name;  // 20-char test name
        begin
            test_num = test_num + 1;
            if (rd_data === expected && rd_write_en === 1'b1) begin
                pass_count = pass_count + 1;
                $display("TEST %02d [PASS] %0s | rd=0x%08X", test_num, name, rd_data);
            end else begin
                fail_count = fail_count + 1;
                $display("TEST %02d [FAIL] %0s | got=0x%08X expected=0x%08X wr_en=%b",
                    test_num, name, rd_data, expected, rd_write_en);
            end
        end
    endtask

    task check_exc;
        input        expect_exc;
        input [3:0]  expect_code;
        input [159:0] name;
        begin
            test_num = test_num + 1;
            if (saved_exc === expect_exc && saved_exc_code === expect_code) begin
                pass_count = pass_count + 1;
                $display("TEST %02d [PASS] %0s | exc=%b code=%01X", test_num, name, saved_exc, saved_exc_code);
            end else begin
                fail_count = fail_count + 1;
                $display("TEST %02d [FAIL] %0s | exc=%b/%b code=%01X/%01X",
                    test_num, name, saved_exc, expect_exc, saved_exc_code, expect_code);
            end
        end
    endtask

    // Exception capture registers — sample during subunit output cycle
    reg        saved_exc;
    reg [3:0]  saved_exc_code;

    task exec_exc;
        input [31:0] instr;
        input [31:0] src1;
        input [31:0] src2;
        begin
            @(posedge clk);
            instruction = instr;
            rs1_data    = src1;
            rs2_data    = src2;
            instr_valid = 1'b1;
            @(posedge clk);
            instr_valid = 1'b0;
            @(posedge clk); // Subunit processes — exception valid here
            saved_exc      = xp_exception;
            saved_exc_code = xp_exc_code;
            @(posedge clk); // Top output stage
        end
    endtask

    // -----------------------------------------------------------------------
    // Main test sequence — 22 tests
    // -----------------------------------------------------------------------
    initial begin
        $dumpfile("sim/xplenum_sim.vcd");
        $dumpvars(0, xplenum_tb);

        test_num   = 0;
        pass_count = 0;
        fail_count = 0;

        // Reset
        rst_n       = 0;
        instruction = 32'h0;
        instr_valid = 0;
        rs1_data    = 32'h0;
        rs2_data    = 32'h0;

        repeat(4) @(posedge clk);
        rst_n = 1;
        repeat(2) @(posedge clk);

        $display("");
        $display("=============================================================");
        $display("  XPLENUM RISC-V Ternary Security Extension — Testbench");
        $display("  22 Directed Test Cases");
        $display("=============================================================");
        $display("");

        // =================================================================
        // TEST 1: CSR version register read (hardwired 0x01_00_00)
        // =================================================================
        exec(encode_csr_rd(4'hB, 5'd1), 32'h0, 32'h0);
        check(`XP_VERSION, "CSR_VERSION_READ    ");

        // =================================================================
        // TEST 2: CSR status write + readback — enable all subsystems
        // =================================================================
        exec(encode_csr_wr(4'h0, 5'd1, 5'd2), 32'h0000_000F, 32'h0);
        exec(encode_csr_rd(4'h0, 5'd3), 32'h0, 32'h0);
        check(32'h0000_000F, "CSR_STATUS_WR_RD    ");

        // =================================================================
        // TEST 3: TMASK — apply mask (trit addition)
        // data=0x55555555 (all +1), mask=0x55555555 (all +1)
        // Expected: all (+1+1)mod3 = -1 = 0xAAAAAAAA
        // =================================================================
        exec(encode_xp(`F7_TMASK, 5'd0, 5'd1, `F3_TMASK, 5'd3),
             32'h55555555, 32'h55555555);
        check(32'hAAAAAAAA, "TMASK_ADD           ");

        // =================================================================
        // TEST 4: TUNMASK — remove mask (trit subtraction)
        // data=0xAAAAAAAA (all -1), mask=0x55555555 (all +1)
        // Expected: (-1 - +1)mod3 = -2mod3 = +1 = 0x55555555
        // Note: balanced ternary: -1 - 1 = -2 → mod3 wrap → 1
        // Actually: trit_sub(-1,+1) = trit_add(-1,-1) = (-1+-1) = +1 (mod3 wrap)
        // =================================================================
        exec(encode_xp(`F7_TUNMASK, 5'd0, 5'd1, `F3_TMASK, 5'd3),
             32'hAAAAAAAA, 32'h55555555);
        check(32'h55555555, "TUNMASK_SUB         ");

        // =================================================================
        // TEST 5: TMASK identity — mask with zero
        // =================================================================
        exec(encode_xp(`F7_TMASK, 5'd0, 5'd1, `F3_TMASK, 5'd3),
             32'h55555555, 32'h00000000);
        check(32'h55555555, "TMASK_ZERO_IDENTITY ");

        // =================================================================
        // TEST 6: TMASKR — random mask generation
        // Result should differ from input (with high probability)
        // =================================================================
        exec(encode_xp(`F7_TMASKR, 5'd0, 5'd1, `F3_TMASK, 5'd3),
             32'h55555555, 32'h0);
        // Just verify write-back happens
        test_num = test_num + 1;
        if (rd_write_en === 1'b1) begin
            pass_count = pass_count + 1;
            $display("TEST %02d [PASS] TMASKR_RANDOM_GEN    | rd=0x%08X", test_num, rd_data);
        end else begin
            fail_count = fail_count + 1;
            $display("TEST %02d [FAIL] TMASKR_RANDOM_GEN    | wr_en=0", test_num);
        end

        // =================================================================
        // TEST 7: TMASK disabled — expect exception
        // =================================================================
        // Disable masking by writing 0 to status
        exec(encode_csr_wr(4'h0, 5'd1, 5'd2), 32'h0000_000E, 32'h0); // mask_en=0
        exec_exc(encode_xp(`F7_TMASK, 5'd0, 5'd1, `F3_TMASK, 5'd3),
             32'h55555555, 32'h55555555);
        check_exc(1'b1, `XP_EXC_MASK_FAULT, "TMASK_DISABLED_EXC  ");

        // Re-enable all
        exec(encode_csr_wr(4'h0, 5'd1, 5'd2), 32'h0000_000F, 32'h0);

        // =================================================================
        // TEST 8: TDOMSET — set domain tag
        // =================================================================
        // Set current domain ID first
        exec(encode_csr_wr(4'h1, 5'd1, 5'd2), 32'h0000_0001, 32'h0);
        // TDOMSET idx=0, tag={owner=01, perms=FF, xfer=07, state=ACTIVE}
        exec(encode_xp(`F7_TDOMSET, 5'd0, 5'd1, `F3_TDOM, 5'd3),
             32'h00000000, 32'h01FF0701);
        check(32'h00000000, "TDOMSET_ENTRY0      "); // returns previous (was 0)

        // =================================================================
        // TEST 9: TDOMCHK — check domain permissions
        // Check read permission (bit 0)
        // =================================================================
        exec(encode_xp(`F7_TDOMCHK, 5'd0, 5'd1, `F3_TDOM, 5'd3),
             32'h00000000, 32'h00000001);
        check(32'h00000001, "TDOMCHK_READ_OK     ");

        // =================================================================
        // TEST 10: TDOMCHK — check cross-domain permission (bit 3)
        // =================================================================
        exec(encode_xp(`F7_TDOMCHK, 5'd0, 5'd1, `F3_TDOM, 5'd3),
             32'h00000000, 32'h00000008);
        check(32'h00000001, "TDOMCHK_CROSS_OK    ");

        // =================================================================
        // TEST 11: TDOMCLR — clear domain entry
        // =================================================================
        exec(encode_xp(`F7_TDOMCLR, 5'd0, 5'd1, `F3_TDOM, 5'd3),
             32'h00000000, 32'h0);
        check(32'h01FF0701, "TDOMCLR_RETURNS_OLD ");

        // =================================================================
        // TEST 12: TDOMSET non-owner — expect domain violation
        // =================================================================
        // Set entry 5 owned by domain 99
        exec(encode_csr_wr(4'h1, 5'd1, 5'd2), 32'h00000063, 32'h0); // domid=99
        exec(encode_xp(`F7_TDOMSET, 5'd0, 5'd1, `F3_TDOM, 5'd3),
             32'h00000005, 32'h63FF0701);
        // Now switch to domain 1 and try to modify
        exec(encode_csr_wr(4'h1, 5'd1, 5'd2), 32'h00000001, 32'h0);
        exec_exc(encode_xp(`F7_TDOMSET, 5'd0, 5'd1, `F3_TDOM, 5'd3),
             32'h00000005, 32'h01FF0701);
        check_exc(1'b1, `XP_EXC_DOM_VIOLATION, "TDOM_NONOWNER_EXC   ");

        // =================================================================
        // TEST 13: TCAPST — store capability
        // =================================================================
        exec(encode_xp(`F7_TCAPST, 5'd0, 5'd1, `F3_TCAP, 5'd3),
             32'h00000000, 32'hDEAD_BEEF);
        check(32'h00000000, "TCAPST_ENTRY0       "); // returns previous lower half

        // =================================================================
        // TEST 14: TCAPLD — load capability (lower half)
        // =================================================================
        exec(encode_xp(`F7_TCAPLD, 5'd0, 5'd1, `F3_TCAP, 5'd3),
             32'h00000000, 32'h00000000);
        check(32'hDEAD_BEEF, "TCAPLD_LOWER_HALF   ");

        // =================================================================
        // TEST 15: TCAPREV — revoke capability
        // =================================================================
        exec(encode_xp(`F7_TCAPREV, 5'd0, 5'd1, `F3_TCAP, 5'd3),
             32'h00000000, 32'h0);
        check(32'h00000000, "TCAPREV_ENTRY0      "); // was not revoked before

        // =================================================================
        // TEST 16: TCAPLD after revoke — expect revocation exception
        // =================================================================
        exec_exc(encode_xp(`F7_TCAPLD, 5'd0, 5'd1, `F3_TCAP, 5'd3),
             32'h00000000, 32'h00000000);
        check_exc(1'b1, `XP_EXC_CAP_REVOKED, "TCAPLD_REVOKED_EXC  ");

        // =================================================================
        // TEST 17: TCAPCHK — out-of-range index
        // =================================================================
        exec(encode_xp(`F7_TCAPCHK, 5'd0, 5'd1, `F3_TCAP, 5'd3),
             32'h00000FFF, 32'h0);
        // idx = 0xFFF & 0x3F = 63, which is valid (63 < 64)
        // But check perms on empty entry → result should be 0
        check(32'h00000000, "TCAPCHK_EMPTY_ENT   ");

        // =================================================================
        // TEST 18: TTRIT — binary to balanced ternary
        // Input: 5 (decimal)
        // 5 = 1*3^0 + 2*3^1 → BT: +1,-1,+1 → 01_10_01 + zeros
        // Actually: 5/3 = 1 rem 2 → trit=-1, carry up: (5+1)/3=2
        // 2/3 = 0 rem 2 → trit=-1, carry up: (2+1)/3=1
        // 1/3 = 0 rem 1 → trit=+1, carry up: 0 → done
        // So 5 = +1,-1,-1 (MSB to LSB) → packed: [5:4]=01,[3:2]=10,[1:0]=10
        // In 32-bit: 0x0000_002A... let me calculate:
        // pos0: 5%3=2 → trit=-1 (10), remaining=(5+1)/3=2
        // pos1: 2%3=2 → trit=-1 (10), remaining=(2+1)/3=1
        // pos2: 1%3=1 → trit=+1 (01), remaining=0
        // Result: [5:4]=01, [3:2]=10, [1:0]=10 = 0b011010 = 0x0000_001A
        // =================================================================
        exec(encode_xp(`F7_TTRIT, 5'd0, 5'd1, `F3_TENC, 5'd3),
             32'h00000005, 32'h0);
        check(32'h0000001A, "TTRIT_BIN5_TO_BT    ");

        // =================================================================
        // TEST 19: TDETRIT — balanced ternary back to binary
        // Input: result from TEST 18 = 0x1A
        // =================================================================
        exec(encode_xp(`F7_TDETRIT, 5'd0, 5'd1, `F3_TENC, 5'd3),
             32'h0000001A, 32'h0);
        check(32'h00000005, "TDETRIT_BT_TO_BIN5  ");

        // =================================================================
        // TEST 20: TROTL — ternary rotate left by 2 trits
        // =================================================================
        exec(encode_xp(`F7_TROTL, 5'd0, 5'd1, `F3_TROT, 5'd3),
             32'h0000_005A, 32'h00000002);
        // 0x5A = 0101_1010 → rotl by 4 bits → 0xA50 (within 32 bits)
        check(32'h0000_05A0, "TROTL_BY2_TRITS     ");

        // =================================================================
        // TEST 21: TROTR — ternary rotate right by 1 trit
        // =================================================================
        exec(encode_xp(`F7_TROTR, 5'd0, 5'd1, `F3_TROT, 5'd3),
             32'h0000_005A, 32'h00000001);
        // 0x5A = ...0101_1010 → rotr by 2 bits → 10...0001_0110 = 0x80000016
        check(32'h8000_0016, "TROTR_BY1_TRIT      ");

        // =================================================================
        // TEST 22: TTBOX — T-box substitution
        // Input: 6 bits of trit data at [5:0]
        // tbox[13] maps input 0x00_00_00 (trits 0,0,0 = idx 13)
        // → output 6'b10_10_01 = -1,-1,+1
        // =================================================================
        exec(encode_xp(`F7_TTBOX, 5'd0, 5'd1, `F3_TROT, 5'd3),
             32'h00000000, 32'h0);
        // tbox[13] for input (0,0,0) → trit3_to_idx(000000) = (0+1)*9+(0+1)*3+(0+1) = 13
        // tbox[13] = 6'b10_10_01 = 0x29
        // Output: bits[5:0]=0x29, bits[31:6]=result of other groups (all idx 13 too)
        // Full result: each 6-bit group from 0 input → idx 13 → 0x29
        // [5:0]=0x29, [11:6]=0x29, [17:12]=0x29, [23:18]=0x29, [29:24]=0x29, [31:30]=00
        // = 0x0A4A_4A69... let me compute bit by bit
        // Bit [5:0]  = 10_10_01 = 0x29
        // Bit [11:6] = 10_10_01 shifted left 6 → 0xA40
        // Bit [17:12] = shifted left 12 → ...
        // Actually: 0b_00_101001_101001_101001_101001_101001 =
        // Let's compute: 5 groups of 101001 = 5 groups of 6 bits = 30 bits + 2 pad
        // 10_1001_1010_0110_1001_1010_0110_1001 pad 00 at top
        // = 0x29A69A69 with top 2 bits = 00 → 0x29A69A69
        check(32'h29A69A69, "TTBOX_ZERO_INPUT    ");

        // =================================================================
        // Summary
        // =================================================================
        $display("");
        $display("=============================================================");
        $display("  RESULTS:  %02d / %02d PASS,  %02d FAIL", pass_count, test_num, fail_count);
        $display("=============================================================");
        $display("");

        if (fail_count == 0)
            $display("  >>> ALL TESTS PASSED <<<");
        else
            $display("  >>> SOME TESTS FAILED — SEE ABOVE <<<");

        $display("");
        $finish;
    end

    // -----------------------------------------------------------------------
    // Watchdog timer
    // -----------------------------------------------------------------------
    initial begin
        #50000;
        $display("WATCHDOG: Simulation timed out at 50us");
        $finish;
    end

endmodule
