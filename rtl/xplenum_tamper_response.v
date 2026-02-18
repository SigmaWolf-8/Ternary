// ===================================================================
// XPlenum Tamper Response Module (Task 8D.2)
//
// Monitors internal health signals and triggers lockdown when
// anomalous conditions are detected. On lockdown:
//   1. All sensitive CSRs are zeroised
//   2. Security instruction execution is disabled
//   3. Domain/capability tables are cleared
//   4. DRBG state is zeroised
//   5. Lockdown status is latched until hardware reset
// ===================================================================
`timescale 1ns/1ps

module xplenum_tamper_response (
    input             clk,
    input             rst_n,

    // -- Health monitoring inputs --
    input             drbg_health_fail,
    input             domain_integrity_fail,
    input             cap_integrity_fail,
    input             csr_parity_fail,
    input             pipeline_anomaly,
    input             redundancy_mismatch,

    // -- Anomaly detection thresholds --
    input  [7:0]      anomaly_threshold,
    input             force_lockdown,

    // -- Lockdown outputs --
    output reg        lockdown,
    output reg        zeroise_csrs,
    output reg        zeroise_tables,
    output reg        zeroise_drbg,
    output reg        disable_security,

    // -- Status --
    output reg [7:0]  tamper_cause,
    output reg [31:0] tamper_cycle
);

    // -- Tamper cause codes --
    localparam CAUSE_NONE           = 8'h00;
    localparam CAUSE_DRBG           = 8'h01;
    localparam CAUSE_DOMAIN         = 8'h02;
    localparam CAUSE_CAP            = 8'h04;
    localparam CAUSE_CSR            = 8'h08;
    localparam CAUSE_PIPELINE       = 8'h10;
    localparam CAUSE_REDUNDANCY     = 8'h20;
    localparam CAUSE_FORCED         = 8'h40;
    localparam CAUSE_THRESHOLD      = 8'h80;

    // -- Anomaly counter --
    reg [7:0] anomaly_count;
    wire      any_anomaly = drbg_health_fail | domain_integrity_fail |
                            cap_integrity_fail | csr_parity_fail |
                            pipeline_anomaly | redundancy_mismatch;

    // -- Cycle counter --
    reg [31:0] global_cycle;
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n)
            global_cycle <= 32'd0;
        else
            global_cycle <= global_cycle + 1;
    end

    // -- FSM --
    localparam S_MONITORING = 2'd0;
    localparam S_LOCKDOWN   = 2'd1;
    localparam S_ZEROISE    = 2'd2;
    localparam S_LOCKED     = 2'd3;

    reg [1:0] state;

    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            state            <= S_MONITORING;
            lockdown         <= 1'b0;
            zeroise_csrs     <= 1'b0;
            zeroise_tables   <= 1'b0;
            zeroise_drbg     <= 1'b0;
            disable_security <= 1'b0;
            tamper_cause     <= CAUSE_NONE;
            tamper_cycle     <= 32'd0;
            anomaly_count    <= 8'd0;
        end else begin
            zeroise_csrs   <= 1'b0;
            zeroise_tables <= 1'b0;
            zeroise_drbg   <= 1'b0;

            case (state)
                S_MONITORING: begin
                    if (any_anomaly)
                        anomaly_count <= anomaly_count + 1;
                    else if (anomaly_count > 0)
                        anomaly_count <= anomaly_count - 1;

                    if (force_lockdown) begin
                        tamper_cause <= CAUSE_FORCED;
                        state <= S_LOCKDOWN;
                    end else if (redundancy_mismatch) begin
                        tamper_cause <= CAUSE_REDUNDANCY;
                        state <= S_LOCKDOWN;
                    end else if (drbg_health_fail) begin
                        tamper_cause <= CAUSE_DRBG;
                        state <= S_LOCKDOWN;
                    end
                    else if (anomaly_count >= anomaly_threshold) begin
                        tamper_cause <= CAUSE_THRESHOLD |
                            {1'b0, pipeline_anomaly, csr_parity_fail,
                             cap_integrity_fail, domain_integrity_fail,
                             drbg_health_fail, 2'b00};
                        state <= S_LOCKDOWN;
                    end
                end

                S_LOCKDOWN: begin
                    lockdown         <= 1'b1;
                    disable_security <= 1'b1;
                    tamper_cycle     <= global_cycle;
                    state            <= S_ZEROISE;
                end

                S_ZEROISE: begin
                    zeroise_csrs   <= 1'b1;
                    zeroise_tables <= 1'b1;
                    zeroise_drbg   <= 1'b1;
                    state          <= S_LOCKED;
                end

                S_LOCKED: begin
                    lockdown         <= 1'b1;
                    disable_security <= 1'b1;
                end
            endcase
        end
    end

endmodule
