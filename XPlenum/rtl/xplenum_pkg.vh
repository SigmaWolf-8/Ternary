// =============================================================================
// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
// Patent(s) Pending.
//
// XPLENUM — RISC-V Ternary Security Extension
// Package Defines (xplenum_pkg.vh)
// Version 1.0.0 — Draft Specification, February 2026
// =============================================================================

`ifndef XPLENUM_PKG_VH
`define XPLENUM_PKG_VH

// ---------------------------------------------------------------------------
// Opcode Allocation — RISC-V custom-0 space
// ---------------------------------------------------------------------------
`define XP_OPCODE        7'b0001011   // custom-0 = 0x0B

// ---------------------------------------------------------------------------
// Functional Group Encoding (funct3)
// ---------------------------------------------------------------------------
`define F3_TMASK         3'b000       // Ternary Masking Operations
`define F3_TDOM          3'b001       // Domain Isolation Operations
`define F3_TCAP          3'b010       // Capability Operations
`define F3_TROT          3'b011       // Ternary Rotation / Crypto
`define F3_TENC          3'b100       // Trit Encoding / Decoding
`define F3_TSIG          3'b101       // Signal Processing
`define F3_RSVD          3'b110       // Reserved — future expansion
`define F3_TCSR          3'b111       // CSR Access

// ---------------------------------------------------------------------------
// funct7 — Ternary Masking (funct3 = 000)
// ---------------------------------------------------------------------------
`define F7_TMASK         7'b0000000   // TMASK  — apply mask
`define F7_TUNMASK       7'b0000001   // TUNMASK — remove mask
`define F7_TMASKR        7'b0000010   // TMASKR — generate random mask
`define F7_TMASKRF       7'b0000011   // TMASKRF — refresh mask

// ---------------------------------------------------------------------------
// funct7 — Domain Isolation (funct3 = 001)
// ---------------------------------------------------------------------------
`define F7_TDOMSET       7'b0000000   // TDOMSET — set domain tag
`define F7_TDOMCHK       7'b0000001   // TDOMCHK — check domain permission
`define F7_TDOMCLR       7'b0000010   // TDOMCLR — clear domain tag
`define F7_TDOMXFR       7'b0000011   // TDOMXFR — transfer domain ownership

// ---------------------------------------------------------------------------
// funct7 — Capability Operations (funct3 = 010)
// ---------------------------------------------------------------------------
`define F7_TCAPLD        7'b0000000   // TCAPLD  — load capability
`define F7_TCAPCHK       7'b0000001   // TCAPCHK — check capability
`define F7_TCAPST        7'b0000010   // TCAPST  — store capability
`define F7_TCAPREV       7'b0000011   // TCAPREV — revoke capability

// ---------------------------------------------------------------------------
// funct7 — Ternary Cryptographic Primitives (funct3 = 011)
// ---------------------------------------------------------------------------
`define F7_TROTL         7'b0000000   // TROTL — ternary rotate left
`define F7_TROTR         7'b0000001   // TROTR — ternary rotate right
`define F7_TTBOX         7'b0000010   // TTBOX — ternary substitution box
`define F7_TPERM         7'b0000011   // TPERM — ternary permutation

// ---------------------------------------------------------------------------
// funct7 — Trit Encoding / Decoding (funct3 = 100)
// ---------------------------------------------------------------------------
`define F7_TTRIT         7'b0000000   // TTRIT   — binary to ternary encode
`define F7_TDETRIT       7'b0000001   // TDETRIT — ternary to binary decode

// ---------------------------------------------------------------------------
// funct7 — Signal Processing (funct3 = 101)
// ---------------------------------------------------------------------------
`define F7_TSIGFLT       7'b0000000   // TSIGFLT — signal filter
`define F7_TSIGCMP       7'b0000001   // TSIGCMP — signal compare
`define F7_TSIGACC       7'b0000010   // TSIGACC — signal accumulate

// ---------------------------------------------------------------------------
// CSR Addresses (machine-level custom RW: 0x7C0–0x7CB)
// ---------------------------------------------------------------------------
`define CSR_XPSTATUS     12'h7C0      // Global status (MASK_EN, DOM_EN, CAP_EN, SIG_EN)
`define CSR_XPDOMID      12'h7C1      // Current domain ID (8-bit)
`define CSR_XPCAPBASE    12'h7C2      // Capability table base address
`define CSR_XPCAPBOUND   12'h7C3      // Capability table bound
`define CSR_XPMASK_SEED  12'h7C4      // TRNG seed register
`define CSR_XPMASK_STATE 12'h7C5      // Current mask state (RO)
`define CSR_XPTRIT_MODE  12'h7C6      // Trit encoding mode
`define CSR_XPSIG_CFG    12'h7C7      // Signal processing configuration
`define CSR_XPEXC_CAUSE  12'h7C8      // Exception cause (RO)
`define CSR_XPEXC_ADDR   12'h7C9      // Exception address (RO)
`define CSR_XPPERF_CNT   12'h7CA      // Performance counter
`define CSR_XPVERSION    12'h7CB      // Version register (RO): 0x01_00_00

// ---------------------------------------------------------------------------
// XPSTATUS bit positions
// ---------------------------------------------------------------------------
`define XPSTATUS_MASK_EN 0            // Bit [0]: Masking subsystem enable
`define XPSTATUS_DOM_EN  1            // Bit [1]: Domain isolation enable
`define XPSTATUS_CAP_EN  2            // Bit [2]: Capability subsystem enable
`define XPSTATUS_SIG_EN  3            // Bit [3]: Signal processing enable

// ---------------------------------------------------------------------------
// Exception Cause Codes
// ---------------------------------------------------------------------------
`define XP_EXC_NONE          4'h0     // No exception
`define XP_EXC_DOM_VIOLATION 4'h1     // Domain permission check failed
`define XP_EXC_CAP_INVALID   4'h2     // Capability index out of range
`define XP_EXC_CAP_REVOKED   4'h3     // Access to revoked capability
`define XP_EXC_CAP_BOUNDS    4'h4     // Capability bounds check failed
`define XP_EXC_MASK_FAULT    4'h5     // Masking op with subsystem disabled
`define XP_EXC_TRIT_OVERFLOW 4'h6     // Invalid trit encoding (11)
`define XP_EXC_PRIV_FAULT    4'h7     // Insufficient privilege

// ---------------------------------------------------------------------------
// Trit Encoding (2-bit pairs)
// ---------------------------------------------------------------------------
`define TRIT_ZERO        2'b00        // Trit value  0
`define TRIT_POS         2'b01        // Trit value +1
`define TRIT_NEG         2'b10        // Trit value -1
`define TRIT_INVALID     2'b11        // Reserved / invalid

// ---------------------------------------------------------------------------
// Hardware Table Sizes
// ---------------------------------------------------------------------------
`define DOM_TABLE_SIZE   256           // Domain tag table: 256 entries
`define CAP_TABLE_SIZE   64            // Capability table: 64 entries
`define CAP_DESC_WIDTH   64            // Capability descriptor: 64 bits
`define TBOX_SIZE        27            // T-box: 27-entry (3^3) lookup

// ---------------------------------------------------------------------------
// Capability Descriptor Field Positions (64-bit)
// ---------------------------------------------------------------------------
`define CAP_TAG_HI       63           // [63:56] Validity tag
`define CAP_TAG_LO       56
`define CAP_PERM_HI      55           // [55:48] Permission bitmap
`define CAP_PERM_LO      48
`define CAP_BASE_HI      47           // [47:32] Base address
`define CAP_BASE_LO      32
`define CAP_BOUND_HI     31           // [31:16] Bound
`define CAP_BOUND_LO     16
`define CAP_OTYPE_HI     15           // [15:8]  Object type
`define CAP_OTYPE_LO     8
`define CAP_SEAL_HI      7            // [7:0]   Seal state
`define CAP_SEAL_LO      0

// ---------------------------------------------------------------------------
// Seal States
// ---------------------------------------------------------------------------
`define SEAL_OPEN        8'h00        // Unsealed — modifiable
`define SEAL_SEALED      8'h01        // Sealed — immutable permissions
`define SEAL_FROZEN      8'h02        // Frozen — fully immutable

// ---------------------------------------------------------------------------
// Domain Tag Field Positions (32-bit)
// ---------------------------------------------------------------------------
`define DOM_OWNER_HI     31           // [31:24] Owner domain ID
`define DOM_OWNER_LO     24
`define DOM_PERM_HI      23           // [23:16] Permission bitmap
`define DOM_PERM_LO      16
`define DOM_XFER_HI      15           // [15:8]  Transfer authorization
`define DOM_XFER_LO      8
`define DOM_STATE_HI     7            // [7:0]   Lifecycle state
`define DOM_STATE_LO     0

// ---------------------------------------------------------------------------
// Domain Lifecycle States
// ---------------------------------------------------------------------------
`define DOM_INVALID      8'h00        // Invalid / unallocated
`define DOM_ACTIVE       8'h01        // Active
`define DOM_LOCKED       8'h02        // Locked — no modifications
`define DOM_TRANSFER     8'h03        // Transfer in progress

// ---------------------------------------------------------------------------
// Domain Permission Bits
// ---------------------------------------------------------------------------
`define DOM_PERM_READ    0            // Bit [0]: Read
`define DOM_PERM_WRITE   1            // Bit [1]: Write
`define DOM_PERM_EXEC    2            // Bit [2]: Execute
`define DOM_PERM_CROSS   3            // Bit [3]: Cross-domain access

// ---------------------------------------------------------------------------
// Phase 8: Higher-Order Masking (funct3 = 000, extended funct7)
// ---------------------------------------------------------------------------
`define F7_HO_MASK_APPLY   7'b0010000  // 0x10 — HO mask apply (share split)
`define F7_HO_MASK_STRIP   7'b0010001  // 0x11 — HO mask strip (recombine)
`define F7_HO_MASK_REFRESH 7'b0010010  // 0x12 — HO mask refresh
`define F7_HO_MASK_AND     7'b0010011  // 0x13 — HO secure AND (DOM gadget)

// ---------------------------------------------------------------------------
// Phase 8: PQC Acceleration (Custom-1 opcode 0x2B, funct3 = 100)
// ---------------------------------------------------------------------------
`define XP_OPCODE_PQC    7'b0101011   // custom-1 = 0x2B
`define F3_PQC           3'b100       // PQC functional group

`define F7_PQC_NTT_BF    7'b0100000  // 0x20 — NTT butterfly (forward)
`define F7_PQC_INTT_BF   7'b0100001  // 0x21 — Inverse NTT butterfly
`define F7_PQC_MOD_RED   7'b0100010  // 0x22 — Modular reduction (Barrett)
`define F7_PQC_MOD_MUL   7'b0100011  // 0x23 — Modular multiplication (Montgomery)
`define F7_PQC_MOD_ADD   7'b0100100  // 0x24 — Modular addition
`define F7_PQC_CBD_SAMP  7'b0100101  // 0x25 — CBD sampling
`define F7_PQC_REJ_SAMP  7'b0100110  // 0x26 — Rejection sampling
`define F7_PQC_POLY_MAC  7'b0100111  // 0x27 — Polynomial MAC
`define F7_PQC_COMPRESS  7'b0101000  // 0x28 — Coefficient compression
`define F7_PQC_DECOMP    7'b0101001  // 0x29 — Coefficient decompression

// ---------------------------------------------------------------------------
// Phase 8: Extended CSR Addresses
// ---------------------------------------------------------------------------
`define CSR_PQC_CONFIG   12'h7CC      // PQC parameter set configuration

// ---------------------------------------------------------------------------
// Phase 8: Extended Exception Cause Codes
// ---------------------------------------------------------------------------
`define XP_EXC_DRBG_HEALTH  4'h8     // DRBG health check failure
`define XP_EXC_TAMPER       4'h9     // Tamper detection lockdown
`define XP_EXC_PQC_FAULT    4'hA     // PQC unit fault

// ---------------------------------------------------------------------------
// Phase 8: XPSTATUS Extended Bit Positions
// ---------------------------------------------------------------------------
`define XPSTATUS_HO_EN   4            // Bit [4]: Higher-order masking enable
`define XPSTATUS_PQC_EN  5            // Bit [5]: PQC unit enable
`define XPSTATUS_TAMPER   6            // Bit [6]: Tamper response enable

// ---------------------------------------------------------------------------
// Version
// ---------------------------------------------------------------------------
`define XP_VERSION       32'h02_00_00 // v2.0.0 (Phase 8)

// ---------------------------------------------------------------------------
// LFSR Polynomial (32-bit maximal-length)
// x^32 + x^22 + x^2 + x + 1
// ---------------------------------------------------------------------------
`define LFSR_POLY        32'h00400007

`endif // XPLENUM_PKG_VH
