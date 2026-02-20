# ============================================================================
# XPLENUM RISC-V Ternary Security Extension — Build & Simulation
# Capomastro Holdings Ltd. / Applied Physics Division
# Patent(s) Pending
#
# Usage:
#   make -f Makefile.xplenum          — Compile & run 22-test simulation
#   make -f Makefile.xplenum wave     — Open VCD waveform (requires GTKWave)
#   make -f Makefile.xplenum clean    — Remove build artifacts
# ============================================================================

IVERILOG  = iverilog
VVP       = vvp
GTKWAVE   = gtkwave
IVFLAGS   = -g2012 -I rtl

RTL_SRC   = rtl/xplenum_mask_unit.v \
            rtl/xplenum_domain_unit.v \
            rtl/xplenum_cap_unit.v \
            rtl/xplenum_trit_unit.v \
            rtl/xplenum_top.v

TB_SRC    = tb/xplenum_tb.v
SIM_DIR   = sim
SIM_BIN   = $(SIM_DIR)/xplenum_sim
VCD_FILE  = $(SIM_DIR)/xplenum_sim.vcd

.PHONY: all run wave clean

all: run

$(SIM_BIN): $(RTL_SRC) $(TB_SRC) rtl/xplenum_pkg.vh
	@mkdir -p $(SIM_DIR)
	$(IVERILOG) $(IVFLAGS) -o $@ $(RTL_SRC) $(TB_SRC)

run: $(SIM_BIN)
	$(VVP) $(SIM_BIN)

wave: run
	$(GTKWAVE) $(VCD_FILE) &

clean:
	rm -rf $(SIM_DIR)
