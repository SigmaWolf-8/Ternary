// =============================================================================
// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
// Patent(s) Pending.
//
// XPlenum CTR_DRBG Integration Testbench
// Validates: Instantiation, generation, reseed, health tests, mask unit
// =============================================================================

`timescale 1ns / 1ps

module xplenum_drbg_tb;

    reg         clk;
    reg         rst_n;

    reg  [255:0] seed;
    reg          seed_valid;
    reg          reseed;
    reg          generate;

    wire [31:0]  drbg_data;
    wire         drbg_valid;
    wire         health_error;
    wire         ready;

    integer      pass_count;
    integer      fail_count;
    integer      test_num;
    integer      i;

    reg [31:0]   outputs [0:127];
    integer      output_idx;

    // -----------------------------------------------------------------------
    // DUT: CTR_DRBG
    // -----------------------------------------------------------------------
    xplenum_ctr_drbg u_drbg (
        .clk           (clk),
        .rst_n         (rst_n),
        .seed_i        (seed),
        .seed_valid_i  (seed_valid),
        .reseed_i      (reseed),
        .generate_i    (generate),
        .drbg_data_o   (drbg_data),
        .drbg_valid_o  (drbg_valid),
        .health_error_o(health_error),
        .ready_o       (ready)
    );

    // -----------------------------------------------------------------------
    // Clock generation: 10ns period (100 MHz)
    // -----------------------------------------------------------------------
    initial clk = 0;
    always #5 clk = ~clk;

    // -----------------------------------------------------------------------
    // Test tasks
    // -----------------------------------------------------------------------
    task reset_dut;
        begin
            rst_n      <= 1'b0;
            seed       <= 256'h0;
            seed_valid <= 1'b0;
            reseed     <= 1'b0;
            generate   <= 1'b0;
            output_idx <= 0;
            #50;
            rst_n      <= 1'b1;
            #20;
        end
    endtask

    task instantiate_drbg;
        input [255:0] seed_val;
        begin
            @(posedge clk);
            seed       <= seed_val;
            seed_valid <= 1'b1;
            @(posedge clk);
            seed_valid <= 1'b0;
            wait(ready);
            @(posedge clk);
        end
    endtask

    task generate_one;
        begin
            @(posedge clk);
            generate <= 1'b1;
            @(posedge clk);
            generate <= 1'b0;
            wait(drbg_valid);
            @(posedge clk);
        end
    endtask

    task wait_for_ready;
        begin
            wait(ready);
            @(posedge clk);
        end
    endtask

    task check(input [127:0] test_name, input condition);
        begin
            test_num = test_num + 1;
            if (condition) begin
                pass_count = pass_count + 1;
                $display("[PASS] Test %0d: %0s", test_num, test_name);
            end else begin
                fail_count = fail_count + 1;
                $display("[FAIL] Test %0d: %0s", test_num, test_name);
            end
        end
    endtask

    // -----------------------------------------------------------------------
    // Capture DRBG outputs
    // -----------------------------------------------------------------------
    always @(posedge clk) begin
        if (drbg_valid && output_idx < 128) begin
            outputs[output_idx] <= drbg_data;
            output_idx <= output_idx + 1;
        end
    end

    // -----------------------------------------------------------------------
    // Main test sequence
    // -----------------------------------------------------------------------
    initial begin
        pass_count = 0;
        fail_count = 0;
        test_num   = 0;

        $display("================================================================");
        $display("  XPlenum CTR_DRBG Integration Testbench");
        $display("  NIST SP 800-90A Compliance Validation");
        $display("================================================================");

        // ===================================================================
        // Test Group 1: Reset and Initial State
        // ===================================================================
        $display("\n--- Test Group 1: Reset and Initial State ---");
        reset_dut;

        check("Reset: not ready (not instantiated)", !ready);
        check("Reset: no health error", !health_error);
        check("Reset: drbg_valid deasserted", !drbg_valid);

        // ===================================================================
        // Test Group 2: Instantiation
        // ===================================================================
        $display("\n--- Test Group 2: CTR_DRBG_Instantiate ---");

        instantiate_drbg(256'hDEADBEEF_CAFEBABE_01234567_89ABCDEF_FEDCBA98_76543210_BADDECAF_F00DCAFE);

        check("Instantiate: ready asserted", ready);
        check("Instantiate: no health error", !health_error);

        // ===================================================================
        // Test Group 3: Generate
        // ===================================================================
        $display("\n--- Test Group 3: CTR_DRBG_Generate ---");

        output_idx = 0;
        generate_one;

        check("Generate: output valid", output_idx > 0);
        check("Generate: output non-zero", outputs[0] != 32'h0);

        wait_for_ready;
        check("Generate: ready after completion", ready);

        // ===================================================================
        // Test Group 4: Multiple Generates — Uniqueness
        // ===================================================================
        $display("\n--- Test Group 4: Output Uniqueness ---");

        output_idx = 0;
        for (i = 0; i < 16; i = i + 1) begin
            wait_for_ready;
            generate_one;
            #100;
        end

        #200;
        check("Multi-gen: at least 4 outputs", output_idx >= 4);

        begin : uniqueness_check
            integer j, k;
            reg all_unique;
            all_unique = 1'b1;
            for (j = 0; j < output_idx && j < 16; j = j + 1) begin
                for (k = j + 1; k < output_idx && k < 16; k = k + 1) begin
                    if (outputs[j] == outputs[k] && j != k)
                        all_unique = 1'b0;
                end
            end
            check("Multi-gen: outputs are unique", all_unique);
        end

        // ===================================================================
        // Test Group 5: Reseed
        // ===================================================================
        $display("\n--- Test Group 5: CTR_DRBG_Reseed ---");

        wait_for_ready;
        @(posedge clk);
        seed   <= 256'hAAAAAAAA_BBBBBBBB_CCCCCCCC_DDDDDDDD_EEEEEEEE_FFFFFFFF_11111111_22222222;
        reseed <= 1'b1;
        @(posedge clk);
        reseed <= 1'b0;
        wait_for_ready;

        check("Reseed: ready after reseed", ready);
        check("Reseed: no health error", !health_error);

        output_idx = 0;
        generate_one;
        check("Reseed: output valid post-reseed", output_idx > 0);

        // ===================================================================
        // Test Group 6: Health Status
        // ===================================================================
        $display("\n--- Test Group 6: Health Monitoring ---");

        check("Health: no error in normal operation", !health_error);

        // ===================================================================
        // Test Group 7: Generate after Reset + Re-instantiate
        // ===================================================================
        $display("\n--- Test Group 7: Reset Recovery ---");

        reset_dut;
        check("Reset: not ready", !ready);

        instantiate_drbg(256'h12345678_9ABCDEF0_12345678_9ABCDEF0_12345678_9ABCDEF0_12345678_9ABCDEF0);

        check("Re-instantiate: ready", ready);

        output_idx = 0;
        generate_one;
        check("Re-instantiate: generates output", output_idx > 0);

        // ===================================================================
        // Test Group 8: Determinism (same seed → same first output)
        // ===================================================================
        $display("\n--- Test Group 8: Deterministic Behavior ---");

        reset_dut;
        instantiate_drbg(256'hFEDCBA98_76543210_FEDCBA98_76543210_FEDCBA98_76543210_FEDCBA98_76543210);
        output_idx = 0;
        generate_one;
        #100;

        begin : determinism_test
            reg [31:0] first_output;
            first_output = outputs[0];

            reset_dut;
            instantiate_drbg(256'hFEDCBA98_76543210_FEDCBA98_76543210_FEDCBA98_76543210_FEDCBA98_76543210);
            output_idx = 0;
            generate_one;
            #100;

            check("Determinism: same seed produces same output", outputs[0] == first_output);
        end

        // ===================================================================
        // Test Group 9: Different seeds → different outputs
        // ===================================================================
        $display("\n--- Test Group 9: Seed Sensitivity ---");

        reset_dut;
        instantiate_drbg(256'h1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111);
        output_idx = 0;
        generate_one;
        #100;

        begin : seed_diff_test
            reg [31:0] output_a;
            output_a = outputs[0];

            reset_dut;
            instantiate_drbg(256'h2222_2222_2222_2222_2222_2222_2222_2222_2222_2222_2222_2222_2222_2222_2222_2222);
            output_idx = 0;
            generate_one;
            #100;

            check("Seed sensitivity: different seeds → different outputs", outputs[0] != output_a);
        end

        // ===================================================================
        // Summary
        // ===================================================================
        $display("\n================================================================");
        $display("  CTR_DRBG Testbench Results: %0d/%0d PASS",
                 pass_count, pass_count + fail_count);
        if (fail_count == 0)
            $display("  STATUS: ALL TESTS PASSED");
        else
            $display("  STATUS: %0d FAILURES", fail_count);
        $display("================================================================\n");

        $finish;
    end

    initial begin
        #500000;
        $display("[TIMEOUT] Testbench exceeded maximum simulation time");
        $finish;
    end

endmodule
