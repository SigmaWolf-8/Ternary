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
│              Torsion Network Layer (13D Torus)              │
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
                               ▼    ▼
            ┌─────────────┐        ┌─────────────┐
            │   LISTEN    │        │  SYN_SENT   │
            └──────┬──────┘        └──────┬──────┘
                   │                      │
                   ▼                      ▼
            ┌─────────────┐        ┌──────────────────────┐
            │  SYN_RCVD   │───────▶│     ESTABLISHED      │
            └─────────────┘        └──────────┬───────────┘
                                              │
                    ┌─────────────────────────┼──────────────┐
                    │                         │              │
                    ▼                         │              ▼
              ┌───────────┐                   │       ┌────────────┐
              │ FIN_WAIT_1│                   │       │ CLOSE_WAIT │
              └─────┬─────┘                   │       └──────┬─────┘
                    ▼                         │              ▼
              ┌───────────┐                   │       ┌────────────┐
              │ FIN_WAIT_2│                   │       │  LAST_ACK  │
              └─────┬─────┘                   │       └──────┬─────┘
                    ▼                         │              ▼
              ┌───────────┐                   │       ┌────────────┐
              │ TIME_WAIT │───────────────────┴──────▶│   CLOSED   │
              └───────────┘                           └────────────┘

Additional: CLOSING, TERNARY_SYNC, TERNARY_WAIT
```

### TTP Socket API

```rust
use salvi_network::ttp::{TtpSocket, TtpAddress, TtpListener};

let mut socket = TtpSocket::new()?;
socket.connect(TtpAddress::from_str("ternary://server:3333")?)?;
socket.send(b"Hello, Ternary World!")?;

let mut buffer = vec![0u8; 1024];
let received = socket.recv(&mut buffer)?;
socket.close()?;
```

### TTP Server

```rust
let listener = TtpListener::bind(TtpAddress::from_str("ternary://0.0.0.0:3333")?)?;

for stream in listener.incoming() {
    let mut stream = stream?;
    let mut buffer = vec![0u8; 1024];
    let n = stream.recv(&mut buffer)?;
    stream.send(&buffer[..n])?;
}
```

### Ternary Checksum

```rust
use salvi_network::ttp::checksum::TernaryChecksum;

impl TernaryChecksum {
    pub fn compute(data: &[Tryte]) -> Tryte {
        let mut sum = Tryte::zero();
        for (i, tryte) in data.iter().enumerate() {
            let weight = GF3::new((i % 3) as u8);
            let weighted = tryte.gf3_mul(weight);
            sum = sum.gf3_add(weighted);
        }
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

let config = FlowControlConfig {
    initial_window: WindowSize::trytes(65536),
    max_window: WindowSize::trytes(1048576),
    congestion_algorithm: CongestionAlgorithm::TernaryCubic,
};

socket.set_flow_control(config)?;
```

---

## T3P (Ternary Third-gen Protocol)

### Request/Response

```rust
use salvi_network::t3p::{T3pClient, T3pRequest, T3pMethod};

let client = T3pClient::new();

let response = client.get("t3p://api.example.tern/data")?;
println!("Status: {}", response.status());

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

```rust
use salvi_network::t3p::{WitnessRequest, WitnessResponse};

let witness_req = WitnessRequest::new()
    .transaction_hash(&tx_hash)
    .timestamp(femtosecond_now())
    .attestation_data(&data)
    .build()?;

let response = client.witness("t3p://witness.tern/attest", witness_req)?;

match response.status() {
    WitnessStatus::Confirmed => println!("Witnessed at: {}", response.timestamp()),
    WitnessStatus::Pending => println!("Queued for witnessing"),
    WitnessStatus::Rejected(reason) => println!("Rejected: {}", reason),
}
```

### T3P Server

```rust
use salvi_network::t3p::server::{T3pServer, Handler};

let server = T3pServer::bind("ternary://0.0.0.0:3380")?;

server.route("/api/data", |req| {
    match req.method() {
        T3pMethod::GET => handle_get(req),
        T3pMethod::POST => handle_post(req),
        T3pMethod::WITNESS => handle_witness(req),
        _ => T3pResponse::method_not_allowed(),
    }
});

server.run()?;
```

---

## TDNS (Ternary DNS)

### Name Resolution

```rust
use salvi_network::tdns::{TdnsResolver, TdnsRecord};

let resolver = TdnsResolver::new()?;

let records = resolver.lookup("example.tern", RecordType::TADDR)?;
for record in records {
    println!("Address: {:?}", record.address());
}
```

### Ternary Record Types

| Type | Description |
|------|-------------|
| TADDR | Ternary network address |
| TNAME | Ternary canonical name |
| TMUX | Ternary mail exchange |
| TSRV | Ternary service record |
| TTXT | Ternary text record |
| TKEY | Ternary DNSSEC key |
| TPTR | Ternary pointer record |

### TDNS Server

```rust
use salvi_network::tdns::server::{TdnsServer, Zone};

let mut server = TdnsServer::new()?;

let zone = Zone::new("example.tern")
    .add_record(TdnsRecord::taddr("@", node_address))
    .add_record(TdnsRecord::taddr("www", web_address))
    .add_record(TdnsRecord::tmux("@", mail_address, 10))
    .add_record(TdnsRecord::ttxt("@", "v=spf1 +ternary:example.tern ~all"));

server.add_zone(zone)?;
server.run()?;
```

---

## Best Practices

### 1. Use TTP for Reliable Communication
TTP provides ordering and reliability guarantees similar to TCP.

### 2. Leverage T3P Witness for Attestation
The Witness method provides blockchain-backed proof of data existence.

### 3. Cache TDNS Results
Use the built-in hierarchical cache to reduce resolution latency.

### 4. Handle Connection States Properly
Always handle all 11 TTP states to prevent resource leaks.

---

*Così sia.*
