#!/usr/bin/env bash
# PlenumNET Kernel — Boot Diagnostics
# Checks the binary, QEMU config, and captures debug output
set -euo pipefail
cd "$(dirname "$0")/.."

KERNEL="${1:-target/x86_64-unknown-none/debug/ternary-kernel.mb}"

if [ ! -f "$KERNEL" ]; then
    echo "[ERROR] Kernel not found: $KERNEL"
    echo "  Run 'bash scripts/build.sh' first."
    exit 1
fi

echo "================================================================"
echo "  PlenumNET — Boot Diagnostics"
echo "  Kernel: $KERNEL"
echo "================================================================"
echo ""

echo "--- 1. File type ---"
file "$KERNEL"
echo ""

echo "--- 2. ELF headers ---"
readelf -h "$KERNEL" 2>/dev/null | grep -E "Class|Type|Entry|Machine" || echo "  (readelf not available)"
echo ""

echo "--- 3. Program headers ---"
readelf -l "$KERNEL" 2>/dev/null || echo "  (readelf not available)"
echo ""

echo "--- 4. Multiboot header scan ---"
python3 -c "
import struct, sys
data = open('$KERNEL','rb').read()
magic = struct.pack('<I', 0x1BADB002)
idx = data.find(magic)
if idx < 0:
    print('  ERROR: Multiboot magic NOT FOUND in binary!')
    sys.exit(1)
flags, checksum = struct.unpack('<II', data[idx+4:idx+12])
total = 0x1BADB002 + flags + checksum
print(f'  Magic at file offset: 0x{idx:X} ({idx} bytes)')
print(f'  Flags: 0x{flags:X}')
print(f'  Checksum: 0x{checksum:08X}')
print(f'  Checksum valid: {(total & 0xFFFFFFFF) == 0}')
print(f'  Within 8KB limit: {idx < 8192}')
if idx >= 8192:
    print('  *** QEMU will NOT find the header! Must be < 8192 ***')
" 2>/dev/null || echo "  (python3 not available — install or check manually)"
echo ""

echo "--- 5. QEMU version ---"
qemu-system-x86_64 --version 2>&1 || echo "  (QEMU not found)"
echo ""

echo "--- 6. QEMU boot test (10s, serial to file) ---"
SERIAL_LOG="/tmp/plenum-serial.log"
DEBUG_LOG="/tmp/plenum-qemu-debug.log"
: > "$SERIAL_LOG"
: > "$DEBUG_LOG"

timeout 10 qemu-system-x86_64 \
    -kernel "$KERNEL" \
    -serial file:"$SERIAL_LOG" \
    -display none \
    -device isa-debug-exit,iobase=0xf4,iosize=0x04 \
    -no-reboot \
    -m 128M \
    -d cpu_reset,int 2>"$DEBUG_LOG" || true

echo ""
echo "--- 7. Serial output (expect '1234' if trampoline works) ---"
if [ -s "$SERIAL_LOG" ]; then
    echo "  Content:"
    od -A x -t x1z -v "$SERIAL_LOG" | head -10
    echo "  Text:"
    cat "$SERIAL_LOG" | head -20
else
    echo "  (empty — kernel did not produce serial output)"
fi
echo ""

echo "--- 8. QEMU debug log (last 40 lines) ---"
if [ -s "$DEBUG_LOG" ]; then
    tail -40 "$DEBUG_LOG"
else
    echo "  (empty)"
fi
echo ""

echo "--- 9. Alternative boot: -nographic (10s) ---"
SERIAL_LOG2="/tmp/plenum-serial2.log"
timeout 10 qemu-system-x86_64 \
    -kernel "$KERNEL" \
    -nographic \
    -device isa-debug-exit,iobase=0xf4,iosize=0x04 \
    -no-reboot \
    -m 128M > "$SERIAL_LOG2" 2>&1 || true

if [ -s "$SERIAL_LOG2" ]; then
    echo "  Output:"
    head -20 "$SERIAL_LOG2"
else
    echo "  (no output)"
fi

echo ""
echo "================================================================"
echo "  Diagnostics complete. Share output above for analysis."
echo "================================================================"
