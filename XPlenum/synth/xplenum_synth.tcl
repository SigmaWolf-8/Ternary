# =============================================================================
# Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
# Applied Physics Division
#
# XPLENUM — Vivado Synthesis Script
# Phase 6, Task 6.5: Automated FPGA Synthesis and Reporting
#
# Usage: vivado -mode batch -source xplenum_synth.tcl
# =============================================================================

# ---------------------------------------------------------------------------
# Project configuration
# ---------------------------------------------------------------------------
set PROJECT_NAME    "xplenum_fpga"
set PART            "xc7a200tsbg484-1"
set TOP_MODULE      "xplenum_top"
set RTL_DIR         "../rtl"
set CONSTRAINT_DIR  "."
set OUTPUT_DIR      "output"

# ---------------------------------------------------------------------------
# Create project
# ---------------------------------------------------------------------------
file mkdir $OUTPUT_DIR
create_project $PROJECT_NAME $OUTPUT_DIR/$PROJECT_NAME -part $PART -force

# ---------------------------------------------------------------------------
# Add RTL sources
# ---------------------------------------------------------------------------
add_files -norecurse [list \
    $RTL_DIR/xplenum_pkg.vh \
    $RTL_DIR/xplenum_top.v \
    $RTL_DIR/xplenum_mask_unit.v \
    $RTL_DIR/xplenum_domain_unit.v \
    $RTL_DIR/xplenum_cap_unit.v \
    $RTL_DIR/xplenum_trit_unit.v \
    $RTL_DIR/xplenum_signal_unit.v \
    $RTL_DIR/xplenum_aes256_core.v \
    $RTL_DIR/xplenum_ctr_drbg.v \
]

# Add integration sources (optional — for full CVA6+XPlenum synthesis)
# add_files -norecurse [list \
#     $RTL_DIR/integration/xplenum_cva6_wrapper.v \
#     $RTL_DIR/integration/xplenum_stall_controller.v \
#     $RTL_DIR/integration/xplenum_cva6_top.v \
# ]

set_property top $TOP_MODULE [current_fileset]

# Add constraint files
add_files -fileset constrs_1 -norecurse [list \
    $CONSTRAINT_DIR/xplenum_fpga.sdc \
    $CONSTRAINT_DIR/xplenum_pinmap.xdc \
]

# ---------------------------------------------------------------------------
# Include paths
# ---------------------------------------------------------------------------
set_property verilog_define {} [current_fileset]
set_property include_dirs $RTL_DIR [current_fileset]

# ---------------------------------------------------------------------------
# Synthesis settings
# ---------------------------------------------------------------------------
set_property strategy Flow_PerfOptimized_high [get_runs synth_1]

set_property -name {STEPS.SYNTH_DESIGN.ARGS.MORE OPTIONS} \
    -value {-directive PerformanceOptimized} \
    -objects [get_runs synth_1]

# Enable retiming for better timing closure
set_property STEPS.SYNTH_DESIGN.ARGS.RETIMING true [get_runs synth_1]

# Flatten hierarchy for AES core (better optimization)
set_property STEPS.SYNTH_DESIGN.ARGS.FLATTEN_HIERARCHY rebuilt [get_runs synth_1]

# ---------------------------------------------------------------------------
# Run synthesis
# ---------------------------------------------------------------------------
puts "INFO: Starting synthesis..."
launch_runs synth_1
wait_on_run synth_1

# Check for errors
if {[get_property STATUS [get_runs synth_1]] ne "synth_design Complete!"} {
    puts "ERROR: Synthesis failed!"
    exit 1
}

puts "INFO: Synthesis complete."

# ---------------------------------------------------------------------------
# Generate reports
# ---------------------------------------------------------------------------
open_run synth_1

# Utilization report
report_utilization -file $OUTPUT_DIR/xplenum_utilization.rpt
puts "INFO: Utilization report generated."

# Timing summary
report_timing_summary -file $OUTPUT_DIR/xplenum_timing_summary.rpt
puts "INFO: Timing summary generated."

# Timing report (worst paths)
report_timing -max_paths 50 -file $OUTPUT_DIR/xplenum_timing_paths.rpt
puts "INFO: Detailed timing report generated."

# Power estimate
report_power -file $OUTPUT_DIR/xplenum_power.rpt
puts "INFO: Power estimate generated."

# DRC check
report_drc -file $OUTPUT_DIR/xplenum_drc.rpt
puts "INFO: DRC report generated."

# Clock network report
report_clock_networks -file $OUTPUT_DIR/xplenum_clock_networks.rpt
puts "INFO: Clock network report generated."

# ---------------------------------------------------------------------------
# Implementation (optional — for full place-and-route)
# ---------------------------------------------------------------------------
# Uncomment to run full implementation:
#
# launch_runs impl_1
# wait_on_run impl_1
# open_run impl_1
#
# report_utilization -file $OUTPUT_DIR/xplenum_impl_utilization.rpt
# report_timing_summary -file $OUTPUT_DIR/xplenum_impl_timing.rpt
# report_power -file $OUTPUT_DIR/xplenum_impl_power.rpt
#
# # Generate bitstream
# launch_runs impl_1 -to_step write_bitstream
# wait_on_run impl_1

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
puts ""
puts "========================================"
puts "XPlenum FPGA Synthesis Summary"
puts "========================================"
puts "Part:          $PART"
puts "Top Module:    $TOP_MODULE"
puts "Reports:       $OUTPUT_DIR/"
puts "========================================"
puts ""

close_project
exit 0
