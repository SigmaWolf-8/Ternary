# =============================================================================
# Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
# Applied Physics Division
#
# PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
# Patent(s) Pending.
#
# XPLENUM — Synopsys Design Constraints (SDC)
# Phase 6, Task 6.5: FPGA Synthesis Preparation
#
# Target: Xilinx Artix-7 (xc7a200t) or Kintex-7 (xc7k325t)
# Tool:   Vivado 2024.2+
# =============================================================================

# ---------------------------------------------------------------------------
# Clock definitions
# ---------------------------------------------------------------------------

# Primary system clock — 100 MHz (10 ns period)
create_clock -name sys_clk -period 10.000 [get_ports clk]

# Clock uncertainty for FPGA routing
set_clock_uncertainty -setup 0.200 [get_clocks sys_clk]
set_clock_uncertainty -hold  0.050 [get_clocks sys_clk]

# Source clock latency (PCB trace + PLL)
set_clock_latency -source -max 1.500 [get_clocks sys_clk]
set_clock_latency -source -min 0.500 [get_clocks sys_clk]

# ---------------------------------------------------------------------------
# Input delay constraints
# ---------------------------------------------------------------------------

# Instruction and register data from CVA6 pipeline
# Setup: data valid 2 ns before clock edge
# Hold:  data holds 0.5 ns after clock edge
set_input_delay -clock sys_clk -max 2.000 [get_ports {instruction[*]}]
set_input_delay -clock sys_clk -min 0.500 [get_ports {instruction[*]}]
set_input_delay -clock sys_clk -max 2.000 [get_ports {instr_valid}]
set_input_delay -clock sys_clk -min 0.500 [get_ports {instr_valid}]
set_input_delay -clock sys_clk -max 2.000 [get_ports {rs1_data[*]}]
set_input_delay -clock sys_clk -min 0.500 [get_ports {rs1_data[*]}]
set_input_delay -clock sys_clk -max 2.000 [get_ports {rs2_data[*]}]
set_input_delay -clock sys_clk -min 0.500 [get_ports {rs2_data[*]}]

# External entropy source
set_input_delay -clock sys_clk -max 3.000 [get_ports {entropy_i[*]}]
set_input_delay -clock sys_clk -min 0.500 [get_ports {entropy_i[*]}]
set_input_delay -clock sys_clk -max 3.000 [get_ports {entropy_valid_i}]
set_input_delay -clock sys_clk -min 0.500 [get_ports {entropy_valid_i}]
set_input_delay -clock sys_clk -max 3.000 [get_ports {reseed_req_i}]
set_input_delay -clock sys_clk -min 0.500 [get_ports {reseed_req_i}]

# Reset (active low, asynchronous — treat as multi-cycle)
set_input_delay -clock sys_clk -max 5.000 [get_ports rst_n]
set_input_delay -clock sys_clk -min 0.000 [get_ports rst_n]

# ---------------------------------------------------------------------------
# Output delay constraints
# ---------------------------------------------------------------------------

# Result path back to CVA6 writeback stage
set_output_delay -clock sys_clk -max 2.000 [get_ports {rd_data[*]}]
set_output_delay -clock sys_clk -min 0.500 [get_ports {rd_data[*]}]
set_output_delay -clock sys_clk -max 2.000 [get_ports {rd_write_en}]
set_output_delay -clock sys_clk -min 0.500 [get_ports {rd_write_en}]
set_output_delay -clock sys_clk -max 2.000 [get_ports {rd_addr[*]}]
set_output_delay -clock sys_clk -min 0.500 [get_ports {rd_addr[*]}]

# Exception path
set_output_delay -clock sys_clk -max 1.500 [get_ports {xp_exception}]
set_output_delay -clock sys_clk -min 0.500 [get_ports {xp_exception}]
set_output_delay -clock sys_clk -max 1.500 [get_ports {xp_exc_code[*]}]
set_output_delay -clock sys_clk -min 0.500 [get_ports {xp_exc_code[*]}]

# DRBG status outputs (low-speed monitoring)
set_output_delay -clock sys_clk -max 3.000 [get_ports {drbg_health_err_o}]
set_output_delay -clock sys_clk -min 0.000 [get_ports {drbg_health_err_o}]
set_output_delay -clock sys_clk -max 3.000 [get_ports {drbg_ready_o}]
set_output_delay -clock sys_clk -min 0.000 [get_ports {drbg_ready_o}]

# ---------------------------------------------------------------------------
# False paths
# ---------------------------------------------------------------------------

# Reset is asynchronous — do not time reset-to-register paths
set_false_path -from [get_ports rst_n]

# DRBG health error is an asynchronous alert, not on critical timing path
set_false_path -from [get_cells -hierarchical *drbg_health_error*]

# Cross-clock domain: entropy input may come from separate TRNG clock domain
set_false_path -from [get_ports {entropy_i[*]}] -to [get_clocks sys_clk]

# ---------------------------------------------------------------------------
# Multi-cycle paths
# ---------------------------------------------------------------------------

# AES-256 core has 14-round pipeline — result available after 14 cycles
set_multicycle_path 14 -setup -from [get_cells -hierarchical u_mask/u_ctr_drbg/u_aes/*] \
                               -to   [get_cells -hierarchical u_mask/u_ctr_drbg/generate_buffer*]
set_multicycle_path 13 -hold  -from [get_cells -hierarchical u_mask/u_ctr_drbg/u_aes/*] \
                               -to   [get_cells -hierarchical u_mask/u_ctr_drbg/generate_buffer*]

# DRBG state machine transitions — multi-cycle by design
set_multicycle_path 2 -setup -from [get_cells -hierarchical u_mask/u_ctr_drbg/state_*] \
                              -to   [get_cells -hierarchical u_mask/u_ctr_drbg/state_*]

# ---------------------------------------------------------------------------
# Design rule constraints
# ---------------------------------------------------------------------------

# Maximum fanout for synthesis optimization
set_max_fanout 32 [current_design]

# Maximum transition time
set_max_transition 1.500 [current_design]

# ---------------------------------------------------------------------------
# Area constraints (for integrated design with CVA6)
# ---------------------------------------------------------------------------

# XPlenum target: < 8% of total CVA6 area
# Estimated: ~30K gates (AES core) + ~5K gates (rest) = ~35K gates
# CVA6 @ Artix-7: ~40K LUTs → XPlenum target: < 3,200 LUTs

# ---------------------------------------------------------------------------
# Power constraints
# ---------------------------------------------------------------------------

# Operating conditions
set_operating_conditions -process max -voltage 1.0 -temperature 85

# Switching activity estimation
set_switching_activity -default_static_probability 0.1
set_switching_activity -default_toggle_rate 0.1

# ---------------------------------------------------------------------------
# Design-for-test constraints
# ---------------------------------------------------------------------------

# Scan chain configuration (for production testing)
# set_scan_configuration -clock_mixing no_mix
# set_scan_configuration -style multiplexed_flip_flop
