# =============================================================================
# Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
# Applied Physics Division
#
# XPLENUM — Xilinx Design Constraints (Pin Mapping)
# Phase 6, Task 6.5: FPGA Pin Assignment
#
# Target: Digilent Nexys A7 (Artix-7 XC7A200T-1SBG484C)
# Alternative: Digilent Genesys 2 (Kintex-7 XC7K325T-2FFG900C)
# =============================================================================

# ---------------------------------------------------------------------------
# System Clock (100 MHz oscillator on Nexys A7)
# ---------------------------------------------------------------------------
set_property -dict { PACKAGE_PIN E3 IOSTANDARD LVCMOS33 } [get_ports clk]
create_clock -add -name sys_clk_pin -period 10.000 -waveform {0 5} [get_ports clk]

# ---------------------------------------------------------------------------
# Reset (active low — active low push button BTN_C on Nexys A7)
# ---------------------------------------------------------------------------
set_property -dict { PACKAGE_PIN N17 IOSTANDARD LVCMOS33 } [get_ports rst_n]

# ---------------------------------------------------------------------------
# Status LEDs
# ---------------------------------------------------------------------------

# LED[0] — DRBG ready
set_property -dict { PACKAGE_PIN H17 IOSTANDARD LVCMOS33 } [get_ports drbg_ready_o]

# LED[1] — DRBG health error
set_property -dict { PACKAGE_PIN K15 IOSTANDARD LVCMOS33 } [get_ports drbg_health_err_o]

# LED[2] — Exception active
set_property -dict { PACKAGE_PIN J13 IOSTANDARD LVCMOS33 } [get_ports xp_exception]

# ---------------------------------------------------------------------------
# PMOD Header JA — External Entropy Source Interface
# (Connect to discrete TRNG module via PMOD)
# ---------------------------------------------------------------------------

# Note: Full 256-bit entropy requires high-speed parallel interface.
# For PMOD prototyping, entropy is loaded serially via SPI from TRNG module.
# The entropy_i[255:0] bus is managed internally by an SPI-to-parallel bridge.

# JA[0] — Entropy SPI SCLK
set_property -dict { PACKAGE_PIN C17 IOSTANDARD LVCMOS33 } [get_ports entropy_spi_sclk]

# JA[1] — Entropy SPI MISO (data from TRNG)
set_property -dict { PACKAGE_PIN D18 IOSTANDARD LVCMOS33 } [get_ports entropy_spi_miso]

# JA[2] — Entropy SPI CS_N
set_property -dict { PACKAGE_PIN E18 IOSTANDARD LVCMOS33 } [get_ports entropy_spi_cs_n]

# JA[3] — Entropy valid strobe
set_property -dict { PACKAGE_PIN G17 IOSTANDARD LVCMOS33 } [get_ports entropy_valid_i]

# JA[4] — Reseed request
set_property -dict { PACKAGE_PIN D17 IOSTANDARD LVCMOS33 } [get_ports reseed_req_i]

# ---------------------------------------------------------------------------
# PMOD Header JB — Debug / Trace Interface
# ---------------------------------------------------------------------------

# JB[0] — Debug instruction valid
set_property -dict { PACKAGE_PIN D14 IOSTANDARD LVCMOS33 } [get_ports debug_instr_valid]

# JB[1] — Debug exception out
set_property -dict { PACKAGE_PIN F16 IOSTANDARD LVCMOS33 } [get_ports debug_exc_out]

# JB[2] — Debug rd write enable
set_property -dict { PACKAGE_PIN G16 IOSTANDARD LVCMOS33 } [get_ports debug_rd_wen]

# JB[3] — Debug trigger (logic analyzer sync)
set_property -dict { PACKAGE_PIN H14 IOSTANDARD LVCMOS33 } [get_ports debug_trigger]

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------

set_property CFGBVS VCCO [current_design]
set_property CONFIG_VOLTAGE 3.3 [current_design]

# Bitstream settings
set_property BITSTREAM.GENERAL.COMPRESS TRUE [current_design]
set_property BITSTREAM.CONFIG.SPI_BUSWIDTH 4 [current_design]

# ---------------------------------------------------------------------------
# Placement constraints (optional — for timing closure on critical paths)
# ---------------------------------------------------------------------------

# Constrain AES core to single clock region for shorter routing
# create_pblock pblock_aes
# add_cells_to_pblock [get_pblocks pblock_aes] [get_cells -hierarchical u_mask/u_ctr_drbg/u_aes/*]
# resize_pblock [get_pblocks pblock_aes] -add {CLOCKREGION_X0Y2:CLOCKREGION_X0Y2}
