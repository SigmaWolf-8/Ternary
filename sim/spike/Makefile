# =============================================================================
# Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
# XPlenum Spike ISS Extension — Build
# =============================================================================

CXX      = g++
CXXFLAGS = -std=c++17 -O2 -Wall -Wextra
TARGET   = xplenum_spike_test

.PHONY: all clean test

all: $(TARGET)

$(TARGET): xplenum_spike_test.cpp xplenum_spike_extension.h
	$(CXX) $(CXXFLAGS) -o $@ xplenum_spike_test.cpp

test: $(TARGET)
	./$(TARGET)

clean:
	rm -f $(TARGET)
