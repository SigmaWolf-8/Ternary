# Module Guide: Network Protocols

**Module:** `salvi_network::protocols`  
**Status:** Complete (P2-017 to P2-020)  
**Tests:** ~85 tests

---

## Overview

The Network Protocols module implements the complete ternary networking stack for the UPQTTI architecture. It provides TCP-like reliable transport (TTP), application-layer protocol (T3P), and domain name resolution (TDNS).

### Key Features

- **TTP (Ternary Transport Protocol)** — 11-state reliable transport
- **T3P (Ternary Third-gen Protocol)** — Application protocol with Witness method
- **TDNS (Ternary DNS)** — Name resolution with ternary record types
- **Ternary Checksums** — GF(3)-based error detection

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Application Layer                         │
│               (User applications, services)                  │
├─────────────────────────────────────────────────────────────┤
│                    T3P Protocol Layer                        │
│   ┌────────────────────────────────────────────────────┐   │
│   │ Request/Response │ Witness Method │ Session Mgmt   │   │
│   └────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│                    TTP Transport Layer                       │
│   ┌────────────────────────────────────────────────────┐   │
│   │ 11-State Machine │ Ternary Checksum │ Flow Control │   │
│   └────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│                    TDNS Resolution                           │
│   ┌────────────────────────────────────────────────────┐   │
│   │ Ternary Records │ Hierarchical Cache │ Resolvers   │   │
│   └────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│                    Torsion Network Layer                     │
│              (13D Torus Topology, Geodesic Routing)         │
└─────────────────────────────────────────────────────────────┘
```

---

## TTP (Ternary Transport Protocol)

### Overview

TTP provides reliable, ordered delivery of data streams over the torsion network. It's analogous to TCP but designed for ternary computing with an 11-state connection machine.

### 11-State Machine

```
                        ┌─────────────┐
                        │   CLOSED    │
                        └──────┬──────┘
                   passive open│    │active open
                               │    │send SYN
                               ▼    ▼
            ┌─────────────┐        ┌─────────────┐
            │   LISTEN    │        │  SYN_SENT   │
            └──────┬──────┘        └──────┬──────┘
         recv SYN  │                      │ recv SYN+ACK
         send SYN+ACK                     │ send ACK
                   ▼                      ▼
            ┌─────────────┐        ┌─────────────┐
            │  SYN_RCVD   │        │             │
            └──────┬──────┘        │             │
            recv ACK               │             │
                   │               │             │
                   ▼               ▼             │
            ┌──────────────────────┐             │
            │     ESTABLISHED      │◄────────────┘
            └──────────┬───────────┘
                       │
      ┌────────────────┼────────────────┐
      │ close          │          close │
      │ send FIN       │          recv FIN
      ▼                │                ▼
┌───────────┐          │         ┌────────────┐
│ FIN_WAIT_1│          │         │ CLOSE_WAIT │
└─────┬─────┘          │         └──────┬─────┘
      │recv FIN+ACK    │                │close
      │                │                │send FIN
      ▼                │                ▼
┌───────────┐          │         ┌────────────┐
│ FIN_WAIT_2│          │         │  LAST_ACK  │
└─────┬─────┘          │         └──────┬─────┘
      │recv FIN        │                │recv ACK
      │send ACK        │                │
      ▼                │                ▼
┌───────────┐          │         ┌────────────┐
│ TIME_WAIT │──────────┴────────▶│   CLOSED   │
└───────────┘   timeout          └────────────┘

Additional states:
- CLOSING: Both sides close simultaneously
- TERNARY_SYNC: Ternary phase synchronization
- TERNARY_WAIT: Waiting for phase alignment
```

### TTP Socket API

```rust
use salvi_network::ttp::{TtpSocket, TtpAddress, TtpListener};

// Client: Connect to server
let mut socket = TtpSocket::new()?;
socket.connect(TtpAddress::from_str("ternary://server:3333")?)?;

// Send data
socket.send(b"Hello, Ternary World!")?;

// Receive data
let mut buffer = vec![0u8; 1024];
let received = socket.recv(&mut buffer)?;
println!("Received: {}", String::from_utf8_lossy(&buffer[..received]));

// Close connection
socket.close()?;
```

### TTP Server

```rust
// Server: Listen for connections
let listener = TtpListener::bind(TtpAddress::from_str("ternary://0.0.0.0:3333")?)?;

println!("Listening on port 3333...");

for stream in listener.incoming() {
    let mut stream = stream?;
    
    // Handle connection
    let mut buffer = vec![0u8; 1024];
    let n = stream.recv(&mut buffer)?;
    
    // Echo back
    stream.send(&buffer[..n])?;
}
```

### Ternary Checksum

TTP uses GF(3)-based checksums for error detection:

```rust
use salvi_network::ttp::checksum::TernaryChecksum;

impl TernaryChecksum {
    pub fn compute(data: &[Tryte]) -> Tryte {
        let mut sum = Tryte::zero();
        
        for (i, tryte) in data.iter().enumerate() {
            // GF(3) weighted sum
            let weight = GF3::new((i % 3) as u8);
            let weighted = tryte.gf3_mul(weight);
            sum = sum.gf3_add(weighted);
        }
        
        // Return complement for zero-sum property
        sum.gf3_negate()
    }
    
    pub fn verify(data: &[Tryte], checksum: Tryte) -> bool {
        let computed = Self::compute(data);
        computed.gf3_add(checksum) == Tryte::zero()
    }
}
```

### Flow Control

```rust
use salvi_network::ttp::flow::{FlowControl, WindowSize};

// Configure flow control
let config = FlowControlConfig {
    initial_window: WindowSize::trytes(65536),  // 64KB
    max_window: WindowSize::trytes(1048576),    // 1MB
    congestion_algorithm: CongestionAlgorithm::TernaryCubic,
};

socket.set_flow_control(config)?;

// Monitor flow control
let stats = socket.flow_stats();
println!("Window: {} trytes", stats.current_window);
println!("In-flight: {} trytes", stats.bytes_in_flight);
println!("RTT: {:?}", stats.smoothed_rtt);
```

---

## T3P (Ternary Third-gen Protocol)

### Overview

T3P is the application-layer protocol for ternary networks, similar to HTTP but with native support for the Witness method (for blockchain operations).

### Request/Response

```rust
use salvi_network::t3p::{T3pClient, T3pRequest, T3pMethod};

let client = T3pClient::new();

// Simple GET request
let response = client.get("t3p://api.example.tern/data")?;
println!("Status: {}", response.status());
println!("Body: {:?}", response.body());

// POST request with body
let response = client.post("t3p://api.example.tern/submit")
    .header("Content-Type", "application/ternary")
    .body(ternary_data)
    .send()?;
```

### T3P Methods

| Method | Description |
|--------|-------------|
| GET | Retrieve resource |
| POST | Submit data |
| PUT | Update resource |
| DELETE | Remove resource |
| WITNESS | Blockchain witness operation |
| SYNC | Ternary state synchronization |
| PHASE | Phase alignment request |

### Witness Method

The Witness method enables blockchain-like attestation:

```rust
use salvi_network::t3p::{WitnessRequest, WitnessResponse};

// Create witness request
let witness_req = WitnessRequest::new()
    .transaction_hash(&tx_hash)
    .timestamp(Timestamp::now())
    .proof_of_stake(&stake_proof)
    .build()?;

// Submit witness
let response = client.witness("t3p://chain.example.tern/witness", witness_req)?;

match response.status() {
    WitnessStatus::Accepted => println!("Witness accepted"),
    WitnessStatus::Pending => println!("Witness pending confirmation"),
    WitnessStatus::Rejected(reason) => println!("Rejected: {}", reason),
}
```

### T3P Server

```rust
use salvi_network::t3p::{T3pServer, T3pRequest, T3pResponse};

let server = T3pServer::bind("t3p://0.0.0.0:3380")?;

server.route("/api/data", |req: T3pRequest| {
    match req.method() {
        T3pMethod::GET => {
            let data = fetch_data(&req.path())?;
            Ok(T3pResponse::ok().body(data))
        },
        T3pMethod::POST => {
            store_data(&req.body())?;
            Ok(T3pResponse::created())
        },
        _ => Ok(T3pResponse::method_not_allowed()),
    }
});

server.route("/api/witness", |req: T3pRequest| {
    if req.method() != T3pMethod::WITNESS {
        return Ok(T3pResponse::method_not_allowed());
    }
    
    let witness = process_witness(&req)?;
    Ok(T3pResponse::ok().body(witness))
});

server.serve()?;
```

### Session Management

```rust
use salvi_network::t3p::session::{T3pSession, SessionConfig};

// Create persistent session
let session = T3pSession::new(SessionConfig {
    timeout: Duration::from_secs(300),
    keep_alive: true,
    compression: true,
})?;

// Multiple requests on same session
let resp1 = session.get("t3p://api.example.tern/resource1")?;
let resp2 = session.get("t3p://api.example.tern/resource2")?;

// Session maintains connection pool
println!("Active connections: {}", session.connection_count());
```

---

## TDNS (Ternary DNS)

### Overview

TDNS provides name resolution for the ternary network with native support for ternary record types and torsion topology addresses.

### Basic Resolution

```rust
use salvi_network::tdns::{Resolver, RecordType};

let resolver = Resolver::system()?;

// Resolve hostname to torsion address
let addresses = resolver.lookup("example.tern", RecordType::TERN)?;
for addr in addresses {
    println!("Address: {:?}", addr);
}

// Reverse lookup
let names = resolver.reverse_lookup(&torsion_addr)?;
for name in names {
    println!("Name: {}", name);
}
```

### Ternary Record Types

| Type | Description |
|------|-------------|
| TERN | Torsion network address (13D coordinates) |
| TERN6 | Ternary IPv6-compatible address |
| TKEY | Ternary public key |
| TWIT | Witness node reference |
| TTOR | Torsion routing hint |
| TXT | Text record |
| MX | Mail exchange |
| SRV | Service location |

### Record Format

```rust
use salvi_network::tdns::record::{TernRecord, TorsionAddress};

// TERN record contains 13D torus coordinates
pub struct TernRecord {
    pub name: DomainName,
    pub ttl: Duration,
    pub coordinates: [Trit; 13],  // Position in 13D torus
    pub port: u16,
    pub priority: u8,
}

// Create TERN record
let record = TernRecord {
    name: "api.example.tern".parse()?,
    ttl: Duration::from_secs(3600),
    coordinates: [Z, P, N, Z, P, Z, N, P, Z, P, N, Z, P],
    port: 3380,
    priority: 10,
};
```

### TDNS Cache

```rust
use salvi_network::tdns::cache::{DnsCache, CacheConfig};

// Configure cache
let config = CacheConfig {
    max_entries: 10_000,
    negative_ttl: Duration::from_secs(60),
    min_ttl: Duration::from_secs(30),
};

let cache = DnsCache::new(config);

// Lookup with caching
let result = cache.lookup_or_query("example.tern", RecordType::TERN)?;

// Cache statistics
let stats = cache.stats();
println!("Hit rate: {:.1}%", stats.hit_rate * 100.0);
println!("Entries: {}", stats.entry_count);
```

### Running TDNS Server

```rust
use salvi_network::tdns::server::{TdnsServer, Zone, ZoneRecord};

// Create zone
let mut zone = Zone::new("example.tern");

zone.add_record(ZoneRecord::tern(
    "api",
    Duration::from_secs(3600),
    torsion_address,
    3380,
)?);

zone.add_record(ZoneRecord::txt(
    "info",
    Duration::from_secs(86400),
    "Example ternary domain",
)?);

// Start server
let server = TdnsServer::new()
    .add_zone(zone)
    .bind("ternary://0.0.0.0:53")?;

server.serve()?;
```

---

## Protocol Security

### TTP Security

```rust
use salvi_network::security::{TlsTernary, TlsConfig};

// TLS-like encryption for TTP
let config = TlsConfig {
    certificate: load_cert("server.crt")?,
    private_key: load_key("server.key")?,
    cipher_suites: vec![
        CipherSuite::TERNARY_GF3_256,
        CipherSuite::TERNARY_LAMPORT_OTS,
    ],
};

let secure_listener = TtpListener::bind_tls(addr, config)?;
```

### T3P Authentication

```rust
use salvi_network::t3p::auth::{T3pAuth, AuthMethod};

// Client authentication
let auth = T3pAuth::bearer("ternary_token_here");
let response = client.get("t3p://api.example.tern/secure")
    .auth(auth)
    .send()?;

// Server-side verification
server.route("/secure", |req| {
    let token = req.auth().bearer_token()?;
    if !verify_token(token) {
        return Ok(T3pResponse::unauthorized());
    }
    // Handle authenticated request
});
```

### TDNS Security (TDNSSEC)

```rust
use salvi_network::tdns::security::{TdnsSec, DnskeyRecord};

// Verify DNSSEC signatures
let resolver = Resolver::with_dnssec()?;

let result = resolver.lookup_secure("example.tern", RecordType::TERN)?;
if result.is_authenticated() {
    println!("Response authenticated via TDNSSEC");
}
```

---

## Best Practices

### 1. Use Connection Pooling

```rust
use salvi_network::t3p::pool::{ConnectionPool, PoolConfig};

// Create connection pool
let pool = ConnectionPool::new(PoolConfig {
    max_connections: 100,
    idle_timeout: Duration::from_secs(60),
});

// Connections are reused
let conn = pool.get("t3p://api.example.tern")?;
```

### 2. Handle Network Errors

```rust
use salvi_network::error::{NetworkError, RetryPolicy};

let policy = RetryPolicy::exponential(3, Duration::from_millis(100));

let result = policy.retry(|| {
    client.get("t3p://api.example.tern/data")
})?;
```

### 3. Use Appropriate Timeouts

```rust
// Set connection timeout
socket.set_connect_timeout(Duration::from_secs(5))?;

// Set read/write timeout
socket.set_read_timeout(Duration::from_secs(30))?;
socket.set_write_timeout(Duration::from_secs(30))?;
```

---

## Performance

| Operation | Latency | Throughput |
|-----------|---------|------------|
| TTP connect | ~5 ms | N/A |
| TTP send (1KB) | ~100 µs | ~1 GB/s |
| T3P request | ~10 ms | ~10K req/s |
| TDNS lookup (cached) | ~100 µs | N/A |
| TDNS lookup (remote) | ~20 ms | N/A |

---

## Related Modules

- [Torsion Network](./09_TORSION_NETWORK.md) — Underlying topology
- [Cryptography](./05_CRYPTOGRAPHY.md) — Security primitives
- [High-Precision Timing](./12_TIMING_PROTOCOL.md) — Time synchronization

---

*Part of the Salvi Framework Documentation. Così sia.* 🔱
