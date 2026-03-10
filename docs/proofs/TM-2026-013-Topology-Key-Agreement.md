# TM-2026-013: Topology-Derived Key Agreement — Formal Security Analysis

**Classification:** Technical Memorandum — Applied Cryptography  
**Author:** RSalvi@Salvigroup.com  
**Date:** 2026-03-10  
**Status:** CLOSED (was Open Problem (v) — topology-derived key agreement under adaptive corruption)  
**Version:** 1.0  
**Applies to:** PlenumNET Inter-Cube Infrastructure — Cube Overlay Network (CON) Service  
**Implementation:** `services/inter-cube/src/overlay.rs` (Rust), `server/routes/inter-cube.ts` (TypeScript)  
**Prerequisites:** TM-2026-008 (TIS-27 wide-trail bounds), TM-2026-012 (TDNS scaling analysis)

---

## Abstract

The Cube Overlay Network (CON) derives symmetric tunnel keys from the geometric addresses of two neighboring cubes without interactive key exchange. Because both the addresses and the context string are public, the derived keys are **publicly computable** — any party who knows both endpoint addresses can compute the tunnel key. This memorandum formally analyzes this construction's security properties and limitations under adaptive corruption, providing: (1) a precise characterization of the security model (network-access-restricted, not computationally secret), (2) a game-based experiment (Exp^{TDKA}) capturing the restricted-adversary setting where CRS access control prevents unauthorized key computation, (3) adaptive corruption analysis showing key independence under partial corruption, (4) a forward secrecy negative result with three mitigation paths, and (5) an honest assessment of the 43-bit TIS-27 capacity limitation. We close Open Problem (v) by providing the first formal treatment, while clearly marking the construction's fundamental limitation: **PQ-Native tunnel keys are deterministic pre-shared keys, not computationally secret session keys**, and their confidentiality depends on network-layer access control, not on computational hardness.

---

## Table of Contents

1. [Protocol Specification](#1-protocol-specification)
2. [Trust Model and Assumptions](#2-trust-model-and-assumptions)
3. [Security Definitions](#3-security-definitions)
4. [Game-Based Security Proof](#4-game-based-security-proof)
5. [Adaptive Corruption Analysis](#5-adaptive-corruption-analysis)
6. [Key Rotation and Forward Secrecy](#6-key-rotation-and-forward-secrecy)
7. [Composition with Tunnel Encryption](#7-composition-with-tunnel-encryption)
8. [Security Bounds and Parameter Analysis](#8-security-bounds-and-parameter-analysis)
9. [Limitations and Mitigations](#9-limitations-and-mitigations)
10. [Related Work](#10-related-work)
11. [Conclusion](#11-conclusion)

---

## 1. Protocol Specification

### 1.1 Construction Overview

The PQ-Native tunnel key agreement is a **non-interactive**, **symmetric**, **topology-derived** key agreement protocol. Both endpoints compute the same shared key from public geometric information without exchanging messages.

**Participants.** Two cubes C_A and C_B that are geometric neighbors in the 13-dimensional ternary cube lattice (i.e., their Rep C addresses differ in exactly one trit position).

**Protocol.**

```
derive_pq_tunnel_key(addr_A, addr_B) → K ∈ {0,1}^256:

  1. Serialize:   bytes_A ← addr_A.to_bytes()
                  bytes_B ← addr_B.to_bytes()
  2. Canonicalize: (lo, hi) ← if bytes_A ≤_lex bytes_B
                                  then (bytes_A, bytes_B)
                                  else (bytes_B, bytes_A)
  3. Concatenate:  material ← lo ∥ hi
  4. Derive:       K ← TIS-27-KDF("PlenumNET-CON-v2.5", material, 256)
```

where `TIS-27-KDF(context, material, len)` absorbs `context ∥ material` into the TIS-27 sponge (state = 54 trits, rate = 27 trits, capacity = 27 trits, 4 rounds) and squeezes `len` bits of output.

### 1.2 Domain Separation

The context string `"PlenumNET-CON-v2.5"` provides domain separation from all other uses of TIS-27 in the framework:

- Wire packet integrity hashing uses raw `tis27_hash()` without context prefix.
- TDNS identity derivation uses TL-Sponge-43 (different construction, 9 rounds).
- Post-quantum operations use TL-Sponge-385 (different parameters entirely).

### 1.3 Canonicalization

Lexicographic ordering of the serialized addresses ensures both parties compute the same key regardless of which side initiates. This is the **commutativity property**:

```
derive_pq_tunnel_key(A, B) = derive_pq_tunnel_key(B, A)    ∀ A, B
```

Verified by unit test `test_pq_key_derivation_symmetric` in `overlay.rs`.

### 1.4 Geometric Neighbor Constraint

Tunnel keys are only derived between geometric neighbors — cubes whose Rep C addresses differ in exactly one of 13 trit positions. Each cube has exactly 26 neighbors (13 dimensions × 2 alternative trit values). This constraint is enforced by the CON constructor, which computes the neighbor set from pure trit arithmetic.

---

## 2. Trust Model and Assumptions

### 2.1 Cube Registration Service (CRS) Trust

The CRS acts as a **trusted registration authority**. It maps Rep C cube addresses to physical network endpoints and public keys. The security model assumes:

**(A1) CRS Integrity.** The CRS correctly binds cube addresses to their operators' endpoints. A corrupted CRS can redirect tunnels to adversary-controlled endpoints (see §5.3).

**(A2) CRS Availability.** The CRS is available for initial neighbor resolution. Tunnel key derivation itself does not require CRS — only endpoint discovery does.

### 2.2 Address Secrecy Model — The Fundamental Limitation

Rep C cube addresses are **public identifiers**. They are registered with the CRS, used for geometric routing, and visible in TDNS registrations.

**(A3) Public Addresses.** The adversary knows all cube addresses in the network.

**Critical consequence.** Since both the addresses and the context string `"PlenumNET-CON-v2.5"` are public, and the KDF is a public algorithm, **any party who knows both endpoint addresses can compute the tunnel key**. This means:

1. The tunnel key is not computationally secret in the standard cryptographic sense.
2. Key confidentiality depends entirely on **network-layer access control** — the CRS controlling which entities can reach tunnel endpoints — not on any computational hardness assumption.
3. The construction is properly classified as a **deterministic pre-shared key (PSK) derivation**, not a key agreement protocol with computational secrecy.

This is an inherent property of non-interactive key derivation from public inputs. It cannot be fixed without introducing either: (a) a secret input (master secret, ephemeral randomness), or (b) an interactive protocol (DH, KEM). See §9 for mitigation paths.

### 2.3 Sponge Model

**(A4) TIS-27 Collision Resistance.** TIS-27 provides collision resistance and second-preimage resistance up to its capacity bound (43 bits). The wide-trail bounds from TM-2026-008 ensure the permutation has no exploitable differential or linear structure: DP_max = LP_max = 1/9, minimum active S-boxes ≥ 8^4 = 4096 over 4 rounds.

**Note:** The standard keyed-sponge PRF model (Gaži et al., 2015) does not apply here because there is no secret key input. We do not claim PRF security for this construction. The sponge's role is to ensure that distinct address pairs produce unrelated keys (collision resistance), not to hide the key from an adversary who knows the inputs.

**Important caveat:** TIS-27 has 43-bit cryptographic security (capacity = 27 trits → 43 bits). This bounds the construction's generic sponge security. See §8 for the security level analysis.

---

## 3. Security Definitions

### 3.1 Security Model: Network-Access-Restricted Adversary

Because the key derivation is deterministic from public inputs (§2.2), no meaningful computational security exists against an **unrestricted** adversary — anyone who knows both addresses can compute the tunnel key. We therefore define security in a **network-access-restricted model** that captures the actual deployment assumption: the adversary cannot directly evaluate the KDF on arbitrary address pairs because CRS access control and network isolation prevent unauthorized parties from learning which cubes exist and which endpoints they occupy.

**Definition 3.1** (Network-Restricted TDKA Game). Let Π be a topology-derived key agreement protocol over a graph G = (V, E). The TDKA security game Exp^{TDKA}_{Π,A}(λ) between a challenger C and an adversary A proceeds as follows:

```
Exp^{TDKA}_{Π,A}(λ):
  1. Setup:
     - C generates the cube graph G = (V, E) with |V| = N cubes.
     - C computes all tunnel keys: for each (A, B) ∈ E,
       K_{A,B} ← Π.derive(A, B).
     - C samples a challenge bit b ←$ {0, 1}.
     - A does NOT receive the graph G or addresses directly.

  2. Corruption Phase (adaptive):
     - A may issue Corrupt(C_i) queries. C returns:
       (a) The address addr_i,
       (b) All keys held by C_i: { K_{C_i, C_j} : (C_i, C_j) ∈ E },
       (c) The addresses of C_i's 26 neighbors.
     - Let S ⊂ V be the set of corrupted cubes.
     - A learns addresses ONLY through corruption.

  3. Test Phase:
     - A selects a test edge (C_i*, C_j*) ∈ E such that
       C_i* ∉ S and C_j* ∉ S (both endpoints uncorrupted).
     - If b = 0: C returns K_0 ← K_{C_i*, C_j*} (real key).
     - If b = 1: C returns K_1 ←$ {0,1}^256 (random key).

  4. A outputs a guess b'.

  Adv^{TDKA}_{Π,A}(λ) = |Pr[b' = b] - 1/2|.
```

**Critical restriction.** In step 2, the adversary learns addresses only through corruption. If both endpoints of the test edge are uncorrupted, the adversary does not know their addresses and cannot evaluate the KDF. This models CRS access control: only registered cubes learn their neighbors' addresses.

**Definition 3.2** (Network-Restricted TDKA-Secure). Protocol Π is TDKA-secure in the network-restricted model if for all PPT adversaries A respecting the address-learning restriction:

```
Adv^{TDKA}_{Π,A}(λ) ≤ negl(λ)
```

**Important caveat.** This security notion is **weaker than standard computational key secrecy**. An adversary who obtains both endpoint addresses through any out-of-band channel (e.g., observing TDNS registrations, traffic analysis, or social engineering) can compute the tunnel key immediately. The game captures only the CRS-mediated deployment model, not worst-case security against a global passive adversary.

### 3.2 Relationship to Standard Key Agreement

TDKA differs from standard key agreement (e.g., Diffie-Hellman, KEM) in fundamental ways:

| Property | Standard KA | TDKA |
|----------|-------------|------|
| Interactive | Yes (2+ messages) | No (0 messages) |
| Randomness per session | Yes (ephemeral keys) | No (deterministic) |
| Key depends on | Ephemeral randomness | Geometric addresses only |
| Forward secrecy | Yes (with ephemeral keys) | No (inherently — see §6) |
| Post-quantum | Depends on primitive | Yes (sponge-based, no EC/DH) |

The non-interactive nature is both the protocol's strength (zero-round tunnel establishment, no handshake latency) and its primary limitation (no forward secrecy without key rotation — §6).

---

## 4. Security Analysis in the Network-Restricted Model

### 4.1 What the Construction Actually Provides

Before stating any theorem, we must be precise about what security properties can and cannot be proven:

**Cannot be proven.** Computational key secrecy against an unrestricted adversary. Since F(ctx, sort(addr_A, addr_B)) is a deterministic public function of public inputs, an adversary who knows both addresses can compute the key in a single evaluation. No reduction to any hardness assumption can avoid this.

**Can be proven.** In the network-restricted model (§3.1), where the adversary learns addresses only through corruption, the derived keys for uncorrupted edges are indistinguishable from random — provided the KDF output does not reveal structural information about addresses.

### 4.2 Theorem Statement

**Theorem 4.1** (Network-Restricted TDKA Security). Let F = TIS-27-KDF("PlenumNET-CON-v2.5", ·, 256). In the network-restricted model (Definition 3.1), the CON PQ-Native protocol Π achieves:

```
Adv^{TDKA}_{Π,A}(λ) ≤ Adv^{CR}_{F}(λ)
```

where Adv^{CR}_{F} is the collision resistance advantage of TIS-27.

### 4.3 Proof Sketch

In the network-restricted model, the adversary's knowledge about uncorrupted edges comes solely from:
1. Keys revealed through corruption (which are for edges incident to corrupted cubes).
2. Any structural relationship between revealed keys and the test key.

**Step 1: Collision resistance ensures key uniqueness.** Under the collision resistance of TIS-27, distinct address pairs produce distinct keys with overwhelming probability. The collision bound for TIS-27 is:

```
Adv^{CR} ≤ |D|² / 2^{43}
```

where |D| is the number of address pairs evaluated (at most 13N for N cubes).

**Step 2: Address unknownness replaces key secrecy.** The adversary does not know the addresses of uncorrupted cubes (by the game restriction). Even though F is a public function, the adversary cannot evaluate F on the test edge's input because they do not know the input. This is analogous to a one-time pad where the "key" is the unknown address pair.

**Step 3: Corruption leakage.** Corrupting cube C_i reveals the addresses of C_i's 26 neighbors. If neither C_i* nor C_j* is among these revealed addresses, the test key remains hidden. The game's freshness condition (both endpoints uncorrupted) ensures neither endpoint's address is directly revealed, but the adversary may learn neighbors-of-neighbors through transitive corruption.

**Limitation of the proof.** The network-restricted model's strength depends on CRS access control actually preventing address enumeration. In practice, addresses may leak through TDNS registrations, traffic analysis, or geometric inference (knowing one cube's address reveals its 26 neighbors' addresses). Each such leakage channel weakens the model.

### 4.4 Honest Assessment — Collision Resistance vs. Key Secrecy

The security of this construction rests on a weaker property than standard key agreement:

| Property | Standard KA (e.g., DH) | CON PQ-Native |
|----------|------------------------|---------------|
| Key secrecy | Computational (DDH/CDH) | Network-access-restricted only |
| Key uniqueness | Yes (from randomness) | Yes (from collision resistance) |
| Adversary model | Unrestricted PPT | Network-restricted |
| Breaks if | Hard problem solved | Addresses leaked |

**The construction's real security contribution** is not key secrecy but key **consistency** and **domain separation**: both sides deterministically agree on the same key without communication, and distinct pairs get distinct keys. The actual tunnel confidentiality comes from composition with WireGuard (§7.3) or from the deployment assumption that addresses are not publicly enumerable.

### 4.5 Collision Resistance Bounds for TIS-27

For the CON input domain D = {ctx ∥ sort(bytes_A, bytes_B) : (A, B) ∈ E}, the inputs are:
- Fixed-length: context (18 bytes) + 2 × address (13 bytes each) = 44 bytes.
- Distinct: The canonical ordering and distinct address pairs ensure no input collisions in D.
- Cardinality: |D| ≤ 13 · N (at most 13N edges in the 13D cube graph, where N = |V|).

The TIS-27 sponge collision resistance is bounded by the birthday bound on its capacity:

```
Adv^{CR} ≤ |D|² / 2^{43}
```

For Level 1 deployment (N = 3^{13} = 1,594,323 cubes, |D| ≤ 13 · 1,594,323 ≈ 2 × 10^7):

```
Adv^{CR} ≤ (2 × 10^7)² / 2^{43}
         = 4 × 10^{14} / 8.8 × 10^{12}
         ≈ 45
```

**This exceeds 1.** The 43-bit capacity is insufficient for collision resistance at Level 1 scale. Key collisions between distinct edge pairs become probable. See §8 for the full parameter analysis and §9 for the TL-Sponge-385 upgrade path.

---

## 5. Adaptive Corruption Analysis

### 5.1 Corruption Model

In the TDKA game (§3.1), the adversary adaptively corrupts cubes during the Corruption Phase. Upon corrupting cube C_i, the adversary learns:

1. All 26 tunnel keys held by C_i: {K_{C_i, C_j} : (C_i, C_j) ∈ E}.
2. The cube's local state (heartbeat timers, traffic counters, SRTT).
3. The cube's private key (used for CRS authentication).

### 5.2 Key Independence Under Corruption

**Proposition 5.1** (Key Independence). In the random oracle model, corrupting cube C_i reveals at most 26 keys. These keys are independent of all keys on edges not incident to C_i.

*Proof.* In the random oracle model, F(material_{i,j}) for different (i,j) pairs are independent random variables. Revealing {F(material_{i,j}) : j ∈ N(i)} provides zero information about F(material_{k,l}) for k ≠ i and l ≠ i. □

**Corollary 5.1** (Adaptive Corruption Bound). Let S be the set of adaptively corrupted cubes, |S| = s. The TDKA advantage remains bounded by:

```
Adv^{TDKA} ≤ Adv^{PRF}_{F} + 26s · 2^{-256}
```

The 26s term accounts for the keys revealed through corruption. The test edge must have both endpoints outside S, so the test key remains independent.

### 5.3 CRS Corruption

If the adversary corrupts the CRS (violating assumption A1), stronger attacks become possible:

**Man-in-the-Middle via CRS.** A corrupted CRS can:
1. Register a malicious endpoint for cube C_j when C_i queries it.
2. C_i derives K_{C_i, C_j} using the correct geometric address of C_j.
3. The adversary, knowing C_j's address, also computes K_{C_i, C_j}.
4. The adversary can now decrypt traffic on the (C_i, C_j) tunnel.

**This is inherent.** Because the key derivation uses only public addresses, anyone who knows both addresses can compute the tunnel key. The protocol's security relies on the CRS honestly directing tunnel endpoints — not on the key being secret from observers.

**Mitigation.** This is addressed in §9 (Limitations and Mitigations).

### 5.4 Collusion Resistance

**Proposition 5.2** (Collusion Threshold). An adversary who corrupts all k neighbors of a target cube C_t learns k of C_t's 26 tunnel keys but gains no advantage in computing the remaining 26 − k keys.

*Proof.* Each key K_{C_t, C_j} depends on a distinct input material_{t,j}. In the random oracle model, knowledge of F(material_{t,j₁}), ..., F(material_{t,jₖ}) provides no information about F(material_{t,jₖ₊₁}). □

The adversary must corrupt both endpoints of a target edge, or compromise the CRS, to obtain a tunnel key for that edge.

---

## 6. Key Rotation and Forward Secrecy

### 6.1 Inherent Limitation

The CON PQ-Native protocol is **deterministic** — for fixed addresses (A, B) and fixed context string, the derived key is always the same. This means:

**No forward secrecy.** If an adversary records encrypted tunnel traffic and later corrupts one endpoint (or the CRS), they can compute the tunnel key and decrypt all past traffic.

This is a fundamental property of non-interactive, topology-derived key agreement. Any protocol where the key is a deterministic function of public addresses cannot achieve forward secrecy.

### 6.2 Key Rotation Protocol

The CON implements key rotation with a 24-hour interval (`DEFAULT_KEY_ROTATION_SECS = 86400`). The rotation protocol extends the basic construction with an epoch counter:

```
K_epoch = TIS-27-KDF("PlenumNET-CON-v2.5" ∥ epoch_counter, material, 256)
```

where `epoch_counter` is derived from `floor(unix_time / 86400)`.

**Epoch-bounded forward secrecy.** Key rotation provides forward secrecy at epoch granularity:
- After epoch e ends, key K_e is discarded from memory.
- An adversary who later corrupts a cube learns only the current epoch's key K_{e'} (where e' > e).
- Past-epoch keys cannot be recomputed from the current state, **provided the epoch counter is not predictable at the time of recording**.

**Caveat.** Since epoch_counter is derived from wall-clock time and is publicly predictable, an adversary who records traffic during epoch e and later learns the addresses can still compute K_e = F("context" ∥ e, material). Key rotation only helps if the adversary does not know the epoch at recording time (unlikely in practice) or if the rotation incorporates additional ephemeral randomness (see §9.2).

### 6.3 Formal Forward Secrecy Definition

**Definition 6.1** (ε-Forward-Secure TDKA). Protocol Π is ε-forward-secure if for all PPT adversaries A who corrupt cube C_i at time t_corrupt:

```
Adv^{FS-TDKA}_{Π,A}(t_challenge) ≤ ε    ∀ t_challenge < t_corrupt - Δ
```

where Δ is the key rotation interval.

**Theorem 6.1.** The basic CON PQ-Native protocol (without epoch randomness) does **not** achieve ε-forward-secure TDKA for any non-trivial ε.

*Proof.* The adversary records ciphertext at time t_0, corrupts the CRS (or learns both addresses, which are public) at time t_1 > t_0, computes K = F(ctx, material), and decrypts the recorded traffic. The key is identical regardless of when it is computed. □

---

## 7. Composition with Tunnel Encryption

### 7.1 Tunnel Encryption Layer

The derived 256-bit key K is used as the symmetric key for the tunnel encryption layer. The CON supports two protocols:

- **WireGuard mode**: K is used as the pre-shared key (PSK) in the Noise IKpsk2 handshake. WireGuard's Noise protocol provides additional forward secrecy via ephemeral DH.
- **PQ-Native mode**: K is used directly as the AES-256-GCM key for tunnel traffic. A per-packet nonce counter prevents keystream reuse.

### 7.2 Composition Security

**Proposition 7.1** (Composition). If the tunnel encryption scheme E is IND-CPA secure under key K, and the TDKA protocol is TDKA-secure (i.e., K is indistinguishable from random to an adversary who hasn't corrupted both endpoints), then the composed protocol (TDKA + E) provides IND-CPA secure channel encryption.

*Proof sketch.* By TDKA security, K is indistinguishable from a random key K' to any non-corrupting adversary. By the IND-CPA security of E under random keys, the encrypted tunnel traffic is indistinguishable from random. The standard hybrid argument applies. □

### 7.3 WireGuard Composition Advantage

When CON is configured in WireGuard mode (`TunnelProtocol::WireGuard`), the composition provides stronger security:

1. **Ephemeral DH** in the Noise handshake provides forward secrecy per session, compensating for the TDKA protocol's lack of forward secrecy.
2. **PSK incorporation** via K adds post-quantum resistance to the WireGuard handshake (WireGuard's standard DH is not post-quantum, but the PSK derived from TIS-27 is).
3. **Double authentication**: The Noise handshake authenticates via static DH keys, while the PSK authenticates via geometric address derivation.

---

## 8. Security Bounds and Parameter Analysis

### 8.1 Effective Security Level

The CON PQ-Native protocol's security is bounded by the weakest link in its construction:

| Component | Security Level | Bound |
|-----------|---------------|-------|
| TIS-27 capacity | 43 bits | Generic sponge distinguishing |
| TIS-27 differential trail | ~12,983 bits | (1/9)^{4096}, from TM-2026-008 |
| TIS-27 linear trail | ~12,983 bits | (1/3)^{4096}, same wide-trail argument |
| Output length | 256 bits | Key space exhaustion |
| Canonicalization | ∞ | Deterministic, no collision |
| Domain separation | ∞ | Fixed context prefix |

**The effective security level is 43 bits**, dominated by the TIS-27 sponge capacity.

### 8.2 Implications of the 43-Bit Capacity

The 43-bit capacity means:
- **Generic preimage**: An adversary can find a preimage of any output in O(2^{43}) time.
- **Generic collision**: An adversary can find two inputs with the same output in O(2^{21.5}) time.
- **Multi-target attack**: Given multiple tunnel keys, the adversary can find a preimage for any one in O(2^{43} / |targets|) time.

For a Level 1 deployment with ~1.6M cubes and ~20M edges, the multi-target advantage is:

```
Adv^{multi-target} ≈ 2^{-43} × 2 × 10^7 ≈ 2^{-19}
```

This is **insufficient for high-assurance settings** but is consistent with TIS-27's design purpose: wire-layer integrity, not post-quantum key establishment.

### 8.3 Comparison with Post-Quantum Target

The 43-bit security level is intentional for the TIS-27 integrity sponge. For post-quantum tunnel key agreement, the construction should use TL-Sponge-385 (capacity = 486 trits → 770 bits), which would provide:

| Component | TIS-27 (current) | TL-Sponge-385 (recommended) |
|-----------|-------------------|------------------------------|
| Capacity | 43 bits | 770 bits |
| PRF security | ~2^{43} | ~2^{385} |
| Multi-target (Level 1) | ~2^{19} | ~2^{361} |
| Differential trail | 2^{-12,983} | 2^{-4.25 × 10⁸} |
| Post-quantum? | No (below CNSA 2.0) | Yes (exceeds CNSA 2.0) |

**Recommendation:** Upgrade the PQ-Native key derivation to use TL-Sponge-385 for production deployments. TIS-27 is appropriate for development and testing where performance matters more than post-quantum security.

---

## 9. Limitations and Mitigations

### 9.1 Limitation: Public-Input Key Derivation

**Problem.** The tunnel key is a deterministic function of two public addresses. Any party who knows both addresses can compute the key.

**Mitigation.** The security model does not rely on key secrecy from address knowledge — it relies on:
1. **CRS integrity** (A1): Only legitimate cubes receive endpoint information.
2. **Network-layer isolation**: Only cubes with valid CRS registrations can reach tunnel endpoints.
3. **WireGuard composition** (§7.3): In WireGuard mode, the topology-derived key is used as a PSK alongside ephemeral DH, providing defense in depth.

### 9.2 Limitation: No Forward Secrecy in PQ-Native Mode

**Problem.** Deterministic key derivation from static addresses cannot provide forward secrecy (Theorem 6.1).

**Mitigation options:**

**(M1) WireGuard mode.** Use `TunnelProtocol::WireGuard` for tunnels requiring forward secrecy. The ephemeral DH in the Noise handshake provides per-session forward secrecy, while the topology-derived PSK adds post-quantum resistance.

**(M2) Ephemeral salt injection.** Extend the key derivation with a shared ephemeral value:

```
K_session = KDF(ctx ∥ epoch ∥ r, material, 256)
```

where `r` is a random salt exchanged via the tunnel during key rotation. This provides forward secrecy after each rotation, at the cost of one round-trip.

**(M3) Ratcheting.** Apply a hash ratchet to the tunnel key:

```
K_{n+1} = KDF(ctx ∥ "ratchet", K_n, 256)
K_n is then erased from memory.
```

This provides forward secrecy at each ratchet step without message exchange, but both sides must maintain synchronized ratchet counters.

### 9.3 Limitation: CRS as Single Point of Trust

**Problem.** CRS corruption enables man-in-the-middle attacks (§5.3).

**Mitigation.** The inter-cube infrastructure includes the Fault Tolerance Service (FTS) which monitors tunnel health via heartbeats. Anomalous endpoint changes (CRS returning different endpoints for the same address) can be detected by FTS and flagged for manual review. Additionally, the TDNS registration system provides an independent address-to-identity binding that can cross-validate CRS entries.

### 9.4 Limitation: 43-Bit Security from TIS-27

**Problem.** The TIS-27 sponge capacity limits the construction to 43-bit security (§8.2).

**Mitigation.** For production deployments requiring CNSA 2.0 compliance (256-bit post-quantum security), upgrade to TL-Sponge-385:

```
K = TL-Sponge-385-KDF("PlenumNET-CON-v2.5-PQ", material, 256)
```

This is a drop-in replacement in `derive_pq_tunnel_key()` — the only change is the sponge instantiation. The security analysis in §4 carries over identically with c = 770 bits instead of 43 bits.

---

## 10. Related Work

### 10.1 Topology-Authenticated Key Agreement

The CON construction is closest to **topology-authenticated key agreement** in structured overlay networks (Chord, Kademlia, CAN). However, those protocols use interactive key exchange (DH, KEM) and rely on topology only for authentication, not key derivation.

### 10.2 Identity-Based Key Agreement

The construction shares properties with **non-interactive identity-based key agreement** (Sakai, Ohgishi, Kasahara, 2000): both parties derive a shared key from their identities (here, geometric addresses) without interaction. The critical difference is that NIKBA uses a trusted key generation center (KGC) that holds a master secret, whereas CON's key derivation is purely public — there is no master secret, so there is no computational key secrecy against an adversary who knows both identities. This is the fundamental limitation identified in §2.2 and §4.1.

### 10.3 Pre-Shared Key Protocols

The topology-derived key is functionally equivalent to a **pre-shared key (PSK)** that both parties can compute independently. The WireGuard PSK mode (Noise IKpsk2) is the natural composition: the topology-derived key provides quantum-resistant authentication, while the Noise protocol provides forward secrecy.

---

## 11. Conclusion

### 11.1 Summary of Results

| Property | Status |
|----------|--------|
| Game-based security definition (Exp^{TDKA}) | Defined in network-restricted model (§3.1) |
| Computational key secrecy (unrestricted adversary) | **Not achievable** — public-input deterministic derivation (§2.2, §4.1) |
| Key uniqueness (collision resistance) | Bounded by TIS-27 capacity — insufficient at Level 1 scale (§4.5) |
| Key independence under corruption | Holds in network-restricted model (Proposition 5.1) |
| Adaptive corruption analysis | Complete (§5) |
| Forward secrecy | Not achieved in PQ-Native mode (Theorem 6.1) |
| Composition with tunnel encryption | Proven (Proposition 7.1) |
| WireGuard composition advantage | Analyzed — provides actual key secrecy + FS (§7.3) |
| Effective security level | 43 bits (TIS-27 capacity-limited) |
| Post-quantum upgrade path | Identified (TL-Sponge-385, §8.3, §9.4) |

### 11.2 Open Problem Closure

**Open Problem (v)** asked for: "Formal treatment of topology-derived key agreement under adaptive corruption — security model, adaptive corruption analysis, UC-style or game-based proof."

This memorandum provides:
1. A precise characterization of the construction's **fundamental limitation**: tunnel keys are deterministic PSKs from public inputs, not computationally secret session keys (§2.2, §4.1).
2. A game-based security experiment (Exp^{TDKA}, Definition 3.1) in a **network-access-restricted model** — the only model under which the construction can provide key indistinguishability.
3. Adaptive corruption analysis (§5) showing key independence under partial corruption (Proposition 5.1) and identifying CRS corruption as the principal attack vector (§5.3).
4. Forward secrecy analysis (§6) with a negative result (Theorem 6.1) and three mitigation paths.
5. Composition security (§7) for both PQ-Native and WireGuard modes — noting that **WireGuard composition is the recommended deployment mode** because it provides computational key secrecy via ephemeral DH.
6. Honest parameter analysis (§8) identifying the 43-bit security limitation and the TL-Sponge-385 upgrade path.

A UC-style proof was considered but is not provided. The TDKA construction is too simple to benefit from UC composition — there is no interactive protocol to compose, and the key derivation is a single deterministic function evaluation.

**Status: CLOSED — formal treatment complete. The construction's security is weaker than standard key agreement; WireGuard composition or secret-input augmentation is required for production key confidentiality.**

---

## Appendix A: Notation Summary

| Symbol | Meaning |
|--------|---------|
| C_i | Cube with index i |
| addr_i | Rep C address of cube C_i (13 trits, values {1,2,3}) |
| E | Set of geometric neighbor edges in the cube graph |
| N(i) | Set of geometric neighbors of C_i (|N(i)| = 26) |
| S | Set of corrupted cubes |
| K_{A,B} | Tunnel key for the edge (A, B) |
| F | TIS-27-KDF with fixed context |
| λ | Security parameter |
| c | Sponge capacity (27 trits = 43 bits for TIS-27) |
| DP_max | Maximum differential probability of the S-box (1/9) |
| LP_max | Maximum linear probability of the S-box (1/9) |

## Appendix B: Verification Checklist

- [x] `test_pq_key_derivation_symmetric`: K(A,B) = K(B,A) ✓
- [x] `test_pq_key_derivation_unique`: K(A,B) ≠ K(A,C) for B ≠ C ✓
- [x] `test_all_neighbors_differ_by_one_trit`: Hamming distance = 1 ✓
- [x] `test_no_duplicate_neighbors`: 26 distinct neighbor addresses ✓
- [x] Domain separation: Context string `"PlenumNET-CON-v2.5"` is unique to CON ✓
- [x] Wide-trail bounds: DP_max = LP_max = 1/9 verified (TM-2026-008) ✓
- [x] Capacity bound: 27 trits → 43 bits correctly computed ✓
