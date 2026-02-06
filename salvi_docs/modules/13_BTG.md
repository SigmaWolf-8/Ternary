# Module Guide: Binary-Ternary Gateway (BTG)

**Module:** `salvi_btg`  
**Status:** Complete (P3-010 to P3-013)  
**Tests:** ~65 tests

---

## Overview

The Binary-Ternary Gateway (BTG) enables seamless interoperability between traditional binary systems and the Salvi ternary computing stack. It provides bidirectional conversion for integers, floating-point, and arbitrary byte streams.

### Key Features

- **Bidirectional Conversion** - Binary <-> Ternary for all data types
- **Three Representations** - Type A (balanced), Type B (unsigned), Type C (packed)
- **Universal Adapter** - Format-transparent DataBuffer
- **Multi-Step Pipelines** - Chained conversion operations
- **Zero-Copy Where Possible** - Efficient memory handling

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Binary World                              │
│         (i64, f64, bytes, binary protocols)                 │
└──────────────────────────┬──────────────────────────────────┘
                           │
                    ┌──────v──────┐
                    │    BTG      │
                    │  Gateway    │
                    └──────┬──────┘
                           │
        ┌──────────────────┼──────────────────┐
        │                  │                  │
        v                  v                  v
┌───────────────┐  ┌───────────────┐  ┌───────────────┐
│   Type A      │  │   Type B      │  │   Type C      │
│  (Balanced)   │  │  (Unsigned)   │  │   (Packed)    │
│  -1, 0, +1    │  │   0, 1, 2     │  │  Byte-aligned │
└───────────────┘  └───────────────┘  └───────────────┘
        │                  │                  │
        └──────────────────┼──────────────────┘
                           │
                    ┌──────v──────┐
                    │  Universal  │
                    │   Adapter   │
                    └──────┬──────┘
                           │
┌──────────────────────────v──────────────────────────────────┐
│                    Ternary World                             │
│      (Tryte, TernaryAddress, TVM, Torsion Network)          │
└─────────────────────────────────────────────────────────────┘
```

---

## Ternary Representations

### Type A: Balanced Ternary

Standard balanced ternary with values {-1, 0, +1}:

| Trit | Symbol | Value |
|------|--------|-------|
| N | - | -1 |
| Z | 0 | 0 |
| P | + | +1 |

**Use case:** Arithmetic operations, signed values, most internal computation.

```rust
use salvi_btg::{TypeA, Trit};

// Balanced ternary: 42 = P*27 + P*9 + N*3 + P*1 + Z
let value = TypeA::from_i64(42);
assert_eq!(value.trits(), &[Trit::P, Trit::P, Trit::N, Trit::P, Trit::Z]);
```

### Type B: Unsigned Ternary

Unsigned ternary with values {0, 1, 2}:

| Trit | Value |
|------|-------|
| 0 | 0 |
| 1 | 1 |
| 2 | 2 |

**Use case:** Indexing, positive-only values, GF(3) elements.

```rust
use salvi_btg::TypeB;

// Unsigned: 42 = 1*27 + 1*9 + 2*3 + 0*1 = 1120 (base 3)
let value = TypeB::from_u64(42);
assert_eq!(value.digits(), &[1, 1, 2, 0]);
```

### Type C: Packed Binary

Byte-aligned packing for efficient binary interop:

| Encoding | Bits per Trit | Efficiency |
|----------|---------------|------------|
| 2-bit | 2 | 75% (3 states in 4) |
| 5-of-8 | 1.6 | 94% (243 states in 256) |

**Use case:** Network transmission, file storage, binary protocol bridging.

```rust
use salvi_btg::TypeC;

// Pack trytes into bytes (5-of-8 encoding)
let trytes = vec![Tryte::from_i16(100), Tryte::from_i16(200)];
let packed = TypeC::pack(&trytes);

// Unpack back
let unpacked = TypeC::unpack(&packed);
assert_eq!(trytes, unpacked);
```

---

## Basic Usage

### Creating a Gateway

```rust
use salvi_btg::Gateway;

// Create gateway with default settings
let gateway = Gateway::new();

// Or with custom configuration
let gateway = Gateway::builder()
    .default_representation(Representation::TypeA)
    .float_precision(FloatPrecision::Double)
    .byte_encoding(ByteEncoding::FiveOfEight)
    .build();
```

### Integer Conversion

```rust
// Binary to ternary
let binary_value: i64 = 1000;
let ternary = gateway.i64_to_ternary(binary_value);

// Ternary to binary
let back: i64 = gateway.ternary_to_i64(&ternary);
assert_eq!(back, binary_value);

// Unsigned
let unsigned: u64 = 12345;
let ternary = gateway.u64_to_ternary(unsigned);
let back: u64 = gateway.ternary_to_u64(&ternary);
```

### Floating-Point Conversion

```rust
// f64 to ternary (preserves full precision)
let float_val: f64 = 3.14159265358979;
let ternary = gateway.f64_to_ternary(float_val);

// Back to binary
let back: f64 = gateway.ternary_to_f64(&ternary);
assert!((back - float_val).abs() < 1e-15);

// f32 also supported
let float32: f32 = 2.71828;
let ternary = gateway.f32_to_ternary(float32);
```

### Byte Stream Conversion

```rust
// Arbitrary bytes to ternary
let data = b"Hello, Ternary World!";
let ternary = gateway.bytes_to_ternary(data);

// Back to bytes
let back = gateway.ternary_to_bytes(&ternary);
assert_eq!(data.as_slice(), back.as_slice());

// Streaming conversion for large data
let file = File::open("large_file.bin")?;
let reader = BufReader::new(file);
let mut ternary_writer = gateway.streaming_encoder(output);

for chunk in reader.chunks(4096) {
    ternary_writer.write_chunk(&chunk?)?;
}
ternary_writer.finish()?;
```

---

## Universal Adapter

The Universal Adapter provides format-transparent data handling:

### DataBuffer

```rust
use salvi_btg::adapter::{DataBuffer, Format};

// Create buffer (auto-detects or specify format)
let buffer = DataBuffer::from_bytes(data);
let buffer = DataBuffer::from_trytes(trytes);
let buffer = DataBuffer::new(data, Format::Binary);
let buffer = DataBuffer::new(trytes, Format::TernaryA);

// Access in any format
let as_bytes: &[u8] = buffer.as_bytes();
let as_trytes: &[Tryte] = buffer.as_trytes();
let as_i64: i64 = buffer.as_i64()?;

// Format conversion is lazy/cached
let buffer = DataBuffer::from_bytes(big_data);
let trytes = buffer.as_trytes();  // Converts on first access
let trytes2 = buffer.as_trytes(); // Returns cached result
```

### Format Detection

```rust
use salvi_btg::adapter::FormatDetector;

// Automatically detect data format
let unknown_data = receive_from_network();
let format = FormatDetector::detect(&unknown_data);

match format {
    Format::Binary => println!("Binary data"),
    Format::TernaryA => println!("Balanced ternary"),
    Format::TernaryB => println!("Unsigned ternary"),
    Format::TernaryC => println!("Packed ternary"),
    Format::Unknown => println!("Cannot determine"),
}
```

---

## Multi-Step Pipelines

Chain multiple conversion operations:

```rust
use salvi_btg::pipeline::{Pipeline, Step};

// Build a conversion pipeline
let pipeline = Pipeline::new()
    .add(Step::BytesToTernary)
    .add(Step::Compress)
    .add(Step::Encrypt { key: &encryption_key })
    .add(Step::ToTypeC)
    .build();

// Apply pipeline
let input = b"sensitive data";
let output = pipeline.apply(input)?;

// Reverse pipeline
let reverse = pipeline.reverse();
let decrypted = reverse.apply(&output)?;
assert_eq!(input.as_slice(), decrypted.as_slice());
```

### Common Pipelines

```rust
use salvi_btg::pipeline::CommonPipelines;

// Binary file -> Ternary storage
let storage_pipeline = CommonPipelines::binary_to_storage();

// Network transmission pipeline
let network_pipeline = CommonPipelines::network_transport();

// Secure transmission
let secure_pipeline = CommonPipelines::secure_transport(&key);
```

---

## Protocol Bridging

### TCP/IP to TTP Bridge

```rust
use salvi_btg::bridge::{TcpToTtpBridge, BridgeConfig};

let config = BridgeConfig {
    listen_addr: "0.0.0.0:8080".parse()?,  // TCP side
    ternary_addr: TernaryAddress::from_str("ternary://node:3333")?,
    buffer_size: 64 * 1024,
    conversion: ConversionMode::Streaming,
};

let bridge = TcpToTtpBridge::new(config)?;

// Bridge automatically converts:
// - Incoming TCP -> Ternary -> Forward to TTP
// - Incoming TTP -> Binary -> Forward to TCP
bridge.run().await?;
```

### HTTP to T3P Bridge

```rust
use salvi_btg::bridge::HttpToT3pBridge;

// Bridge HTTP requests to T3P protocol
let bridge = HttpToT3pBridge::new()
    .http_listen("0.0.0.0:80")
    .t3p_upstream("ternary://api.example:3333")
    .build()?;

// HTTP GET /resource -> T3P GET request
// T3P response -> HTTP response
bridge.serve().await?;
```

---

## Performance Optimization

### Batch Conversion

```rust
// Convert many values at once
let binary_values: Vec<i64> = vec![1, 2, 3, 4, 5, /* ... */];

// Slow: Individual conversions
let ternary: Vec<_> = binary_values.iter()
    .map(|v| gateway.i64_to_ternary(*v))
    .collect();

// Fast: Batch conversion
let ternary = gateway.batch_i64_to_ternary(&binary_values);
```

### SIMD Acceleration

```rust
use salvi_btg::simd::SimdGateway;

// Use SIMD-accelerated conversions (when available)
let gateway = SimdGateway::new();

// Converts 8 values in parallel on AVX2
let values: [i64; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
let ternary = gateway.simd_i64x8_to_ternary(&values);
```

### Memory Mapping

```rust
use salvi_btg::mmap::MappedConverter;

// Memory-map large files for conversion
let converter = MappedConverter::open("huge_file.bin")?;

// Convert without loading entire file into memory
let ternary_file = converter.convert_to("huge_file.tern")?;
```

---

## Error Handling

```rust
use salvi_btg::Error;

match gateway.bytes_to_ternary(data) {
    Ok(ternary) => { /* success */ },
    Err(Error::InvalidInput(msg)) => {
        eprintln!("Bad input: {}", msg);
    },
    Err(Error::Overflow) => {
        eprintln!("Value too large for ternary representation");
    },
    Err(Error::InvalidEncoding) => {
        eprintln!("Data is not valid packed ternary");
    },
    Err(Error::PrecisionLoss) => {
        eprintln!("Conversion would lose precision");
    },
}
```

---

## Use Cases

### 1. Legacy System Integration

```rust
// Existing binary database -> Ternary processing -> Binary output
let record = database.fetch_record(id)?;
let ternary_record = gateway.bytes_to_ternary(&record);

// Process in ternary (faster, more efficient)
let processed = ternary_processor.process(&ternary_record)?;

// Convert back for legacy system
let binary_result = gateway.ternary_to_bytes(&processed);
database.store_result(&binary_result)?;
```

### 2. Network Protocol Translation

```rust
// Accept connections from binary clients
let tcp_listener = TcpListener::bind("0.0.0.0:8080")?;

for stream in tcp_listener.incoming() {
    let mut stream = stream?;
    
    // Read binary request
    let mut request = Vec::new();
    stream.read_to_end(&mut request)?;
    
    // Convert and forward to ternary network
    let ternary_request = gateway.bytes_to_ternary(&request);
    let ternary_response = ternary_network.send(&ternary_request).await?;
    
    // Convert response back to binary
    let response = gateway.ternary_to_bytes(&ternary_response);
    stream.write_all(&response)?;
}
```

### 3. File Format Conversion

```rust
// Convert binary file to ternary format
let input = File::open("data.bin")?;
let output = File::create("data.tern")?;

let converter = StreamingConverter::new(&gateway);
converter.convert(input, output)?;

// Convert back
let input = File::open("data.tern")?;
let output = File::create("data_restored.bin")?;
converter.reverse(input, output)?;
```

---

## Best Practices

### 1. Choose the Right Representation

```rust
// Arithmetic: Type A (balanced)
let value = TypeA::from_i64(-42);  // Handles negatives naturally

// Indexing: Type B (unsigned)
let index = TypeB::from_u64(100);  // Positive only, simpler

// Network/Storage: Type C (packed)
let packed = TypeC::pack(&trytes);  // Byte-aligned, efficient
```

### 2. Use Streaming for Large Data

```rust
// Don't load entire file
let ternary = gateway.bytes_to_ternary(&huge_file);  // Bad: OOM risk

// Stream instead
let converter = StreamingConverter::new(&gateway);
converter.convert(input_file, output_file)?;  // Good: constant memory
```

### 3. Cache Repeated Conversions

```rust
// Enable conversion cache for repeated values
let gateway = Gateway::builder()
    .with_cache(CacheSize::Medium)  // ~10K entries
    .build();
```

---

## Related Modules

- [Kernel Memory](./01_KERNEL_MEMORY.md) - Memory layout for conversions
- [TVM](./11_TVM.md) - Execute ternary programs on converted data
- [Torsion Network](./09_TORSION_NETWORK.md) - Network transport of converted data
- [Cryptography](./05_CRYPTOGRAPHY.md) - Encrypt data during conversion

---

*Part of the Salvi Framework Documentation. Cosi sia.*
