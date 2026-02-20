// ===================================================================
// XPlenum Fault Injection Simulation Testbench (Task 8D.1)
//
// Models three classes of physical fault attacks:
//   1. Clock glitching (cycle skip / cycle repeat)
//   2. Voltage glitching (random bit-flips in registers)
//   3. Laser fault injection (targeted bit-flip in specific module)
//
// Verifies that XPlenum's security properties either:
//   a) Hold despite the fault (fault masked by redundancy), or
//   b) Degrade gracefully (tamper response triggers lockdown)
// ===================================================================
`timescale 1ns/1ps

module xplenum_fault_inject_tb;

    // -- Clock and reset --
    reg clk, rst_n;
    parameter CLK_PERIOD = 10;

    initial clk = 0;
    always #(CLK_PERIOD/2) clk = ~clk;

    // -- Fault injection controls --
    reg        fault_enable;
    reg [2:0]  fault_type;
    reg [7:0]  fault_target;
    reg [5:0]  fault_bit;
    reg [31:0] fault_cycle;
    reg [31:0] cycle_counter;

    // -- Glitched clock generation --
    wire clk_glitched;
    reg  clk_skip;
    reg  clk_repeat;

    assign clk_glitched = clk_skip ? 1'b0 :
                          clk_repeat ? ~clk : clk;

    // -- DUT instantiation --
    reg  [31:0] instruction;
    reg  [63:0] rs1_data, rs2_data;
    wire [63:0] rd_data;
    wire        rd_wen;
    wire        exception;
    wire        tamper_lockdown;

    xplenum_top_v2 u_dut (
        .clk(clk_glitched),
        .rst_n(rst_n),
        .instruction(instruction),
        .instr_valid(1'b1),
        .rs1_data(rs1_data),
        .rs2_data(rs2_data),
        .rd_data(rd_data),
        .rd_write_en(rd_wen),
        .rd_addr(),
        .xp_exception(exception),
        .xp_exc_code(),
        .entropy_i(256'd0),
        .entropy_valid_i(1'b0),
        .reseed_req_i(1'b0),
        .drbg_health_err_o(),
        .drbg_ready_o(),
        .tamper_lockdown_o(tamper_lockdown)
    );

    // -- Fault injection logic --
    always @(posedge clk) begin
        if (!rst_n) begin
            cycle_counter <= 32'd0;
            clk_skip      <= 1'b0;
            clk_repeat    <= 1'b0;
        end else begin
            cycle_counter <= cycle_counter + 1;
            clk_skip      <= 1'b0;
            clk_repeat    <= 1'b0;

            if (fault_enable && (cycle_counter == fault_cycle)) begin
                case (fault_type)
                    3'd1: begin
                        clk_skip <= 1'b1;
                        $display("[FAULT] Cycle %0d: Clock skip injected", cycle_counter);
                    end

                    3'd2: begin
                        // Voltage glitch: flip bit in CSR register (accessible path in v2)
                        force u_dut.csr_xpstatus[fault_bit[4:0]] =
                            ~u_dut.csr_xpstatus[fault_bit[4:0]];
                        #1;
                        release u_dut.csr_xpstatus[fault_bit[4:0]];
                        $display("[FAULT] Cycle %0d: Voltage glitch -- bit %0d of XPSTATUS flipped",
                                 cycle_counter, fault_bit[4:0]);
                    end

                    3'd3: begin
                        case (fault_target)
                            8'd0: begin
                                // Laser: corrupt mask unit state
                                force u_dut.u_mask.mask_state[fault_bit] =
                                    ~u_dut.u_mask.mask_state[fault_bit];
                                #1;
                                release u_dut.u_mask.mask_state[fault_bit];
                                $display("[FAULT] Cycle %0d: Laser -- mask unit state bit %0d",
                                         cycle_counter, fault_bit);
                            end

                            8'd1: begin
                                // Laser: corrupt domain unit result
                                force u_dut.u_domain.result[fault_bit[4:0]] =
                                    ~u_dut.u_domain.result[fault_bit[4:0]];
                                #1;
                                release u_dut.u_domain.result[fault_bit[4:0]];
                                $display("[FAULT] Cycle %0d: Laser -- domain result bit %0d",
                                         cycle_counter, fault_bit[4:0]);
                            end

                            8'd2: begin
                                // Laser: corrupt capability unit result
                                force u_dut.u_cap.result[fault_bit[4:0]] =
                                    ~u_dut.u_cap.result[fault_bit[4:0]];
                                #1;
                                release u_dut.u_cap.result[fault_bit[4:0]];
                                $display("[FAULT] Cycle %0d: Laser -- cap result bit %0d",
                                         cycle_counter, fault_bit[4:0]);
                            end

                            8'd3: begin
                                // Laser: corrupt tamper response state
                                force u_dut.u_tamper.anomaly_count[fault_bit[2:0]] =
                                    ~u_dut.u_tamper.anomaly_count[fault_bit[2:0]];
                                #1;
                                release u_dut.u_tamper.anomaly_count[fault_bit[2:0]];
                                $display("[FAULT] Cycle %0d: Laser -- tamper response anomaly counter bit %0d",
                                         cycle_counter, fault_bit[2:0]);
                            end

                            default: begin
                                $display("[FAULT] Cycle %0d: Unknown target %0d",
                                         cycle_counter, fault_target);
                            end
                        endcase
                    end
                endcase
            end
        end
    end

    // -- Security property monitors --
    integer fault_detected_count = 0;
    integer fault_undetected_count = 0;
    integer test_count = 0;

    task run_fault_test;
        input [2:0]  f_type;
        input [7:0]  f_target;
        input [5:0]  f_bit;
        input [31:0] f_cycle;
        input [31:0] insn;
        input [63:0] rs1, rs2;
        input [63:0] expected_result;
        begin
            test_count = test_count + 1;
            fault_type   = f_type;
            fault_target = f_target;
            fault_bit    = f_bit;
            fault_cycle  = f_cycle;
            fault_enable = 1'b1;

            @(posedge clk);
            instruction = insn;
            rs1_data    = rs1;
            rs2_data    = rs2;

            repeat (20) @(posedge clk);

            fault_enable = 1'b0;

            if (tamper_lockdown) begin
                fault_detected_count = fault_detected_count + 1;
                $display("[PASS] Test %0d: Fault detected -- tamper lockdown triggered", test_count);
            end else if (exception) begin
                fault_detected_count = fault_detected_count + 1;
                $display("[PASS] Test %0d: Fault detected -- exception raised", test_count);
            end else if (rd_data == expected_result) begin
                fault_detected_count = fault_detected_count + 1;
                $display("[PASS] Test %0d: Fault masked -- correct result despite fault", test_count);
            end else begin
                fault_undetected_count = fault_undetected_count + 1;
                $display("[FAIL] Test %0d: UNDETECTED FAULT -- got 0x%016h, expected 0x%016h",
                         test_count, rd_data, expected_result);
            end
        end
    endtask

    // -- Test sequence --
    initial begin
        $dumpfile("fault_inject.vcd");
        $dumpvars(0, xplenum_fault_inject_tb);

        rst_n = 0;
        fault_enable = 0;
        instruction = 32'd0;
        rs1_data = 64'd0;
        rs2_data = 64'd0;
        repeat (10) @(posedge clk);
        rst_n = 1;
        repeat (5) @(posedge clk);

        $display("\n============================================");
        $display("  XPlenum Fault Injection Test Suite");
        $display("============================================\n");

        // -- Test 1: Clock glitch during XDOM.CHK --
        $display("--- Clock Glitch Tests ---");
        run_fault_test(
            3'd1,
            8'd0,
            6'd0,
            32'd15,
            32'h0020_10B3,
            64'h0000_0000_0000_0001,
            64'h0000_0000_0000_000F,
            64'h0000_0000_0000_0001
        );

        // -- Test 2: Voltage glitch -- bit-flip in register --
        $display("\n--- Voltage Glitch Tests ---");
        run_fault_test(
            3'd2,
            8'd0,
            6'b01_0011,
            32'd12,
            32'h0000_00B3,
            64'hDEAD_BEEF_CAFE_BABE,
            64'hFFFF_FFFF_0000_0000,
            64'h0000_0000_0000_0000
        );

        // -- Test 3: Laser fault -- flip capability valid bit --
        $display("\n--- Laser Fault Tests ---");
        run_fault_test(
            3'd3,
            8'd2,
            6'd5,
            32'd10,
            32'h0010_10B3,
            64'h0000_0000_0000_0005,
            64'h0000_0000_0000_0001,
            64'h0000_0000_0000_0000
        );

        // -- Test 4: Laser fault -- corrupt domain table --
        run_fault_test(
            3'd3,
            8'd1,
            6'd10,
            32'd8,
            32'h0020_10B3,
            64'h0000_0000_0000_000A,
            64'h0000_0000_0000_0001,
            64'h0000_0000_0000_0000
        );

        // -- Summary --
        $display("\n============================================");
        $display("  Fault Injection Summary");
        $display("  Total tests:     %0d", test_count);
        $display("  Faults detected: %0d", fault_detected_count);
        $display("  Faults missed:   %0d", fault_undetected_count);
        $display("  Detection rate:  %0d%%",
                 (fault_detected_count * 100) / test_count);
        $display("============================================\n");

        if (fault_undetected_count > 0)
            $display("*** FAIL: %0d undetected faults ***", fault_undetected_count);
        else
            $display("*** PASS: All faults detected or masked ***");

        $finish;
    end

endmodule
