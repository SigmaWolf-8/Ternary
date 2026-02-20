# =============================================================================
# Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
# XPlenum Security Fuzzer — Build
# =============================================================================

CXX      = g++
CXXFLAGS = -std=c++17 -O2 -Wall -Wextra
TARGET   = xplenum_fuzz

.PHONY: all clean fuzz fuzz-quick

all: $(TARGET)

$(TARGET): xplenum_fuzz_harness.cpp ../spike/xplenum_spike_extension.h
	$(CXX) $(CXXFLAGS) -o $@ xplenum_fuzz_harness.cpp

fuzz: $(TARGET)
	./$(TARGET) 10000000

fuzz-quick: $(TARGET)
	./$(TARGET) 1000000

clean:
	rm -f $(TARGET)
