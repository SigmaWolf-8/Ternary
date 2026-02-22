# Ternary Mathematics — Formal Specification

**Salvi Framework · libternary**
**Version 0.1.0**

---

## 1. GF(3) Field Arithmetic — Foundation

Everything in this library builds on the Galois field of order 3, denoted GF(3) or F₃.

### 1.1 Elements and Representation

GF(3) = {0, 1, 2} under modular arithmetic, or equivalently {−1, 0, +1} in balanced representation.

The isomorphism between standard and balanced forms:

| Standard | Balanced | Semantic       |
|----------|----------|----------------|
| 0        | 0        | False / Null   |
| 1        | +1       | True / Positive|
| 2        | −1       | Unknown / Negative |

### 1.2 Field Axioms (Verification Targets)

A field must satisfy the following properties. These are the exact properties our test suite verifies exhaustively over all elements of GF(3):

**Addition (mod 3):**

| +   | 0 | 1 | 2 |
|-----|---|---|---|
| **0** | 0 | 1 | 2 |
| **1** | 1 | 2 | 0 |
| **2** | 2 | 0 | 1 |

- **A1 Closure:** ∀ a,b ∈ GF(3): a + b ∈ GF(3)
- **A2 Associativity:** ∀ a,b,c: (a + b) + c = a + (b + c)
- **A3 Identity:** ∃ 0: ∀ a: a + 0 = a
- **A4 Inverses:** ∀ a: ∃ (−a): a + (−a) = 0
- **A5 Commutativity:** ∀ a,b: a + b = b + a

**Multiplication (mod 3):**

| ×   | 0 | 1 | 2 |
|-----|---|---|---|
| **0** | 0 | 0 | 0 |
| **1** | 0 | 1 | 2 |
| **2** | 0 | 2 | 1 |

- **M1 Closure:** ∀ a,b ∈ GF(3): a × b ∈ GF(3)
- **M2 Associativity:** ∀ a,b,c: (a × b) × c = a × (b × c)
- **M3 Identity:** ∃ 1: ∀ a: a × 1 = a
- **M4 Inverses:** ∀ a ≠ 0: ∃ a⁻¹: a × a⁻¹ = 1
- **M5 Commutativity:** ∀ a,b: a × b = b × a

**Distributivity:**
- **D1:** ∀ a,b,c: a × (b + c) = (a × b) + (a × c)

Since GF(3) has only 3 elements, all 27 triples (a,b,c) can be checked exhaustively. This is not sampling — it is complete proof by enumeration.

### 1.3 Extended Operations for VM Opcodes

Beyond basic field operations, the 176-opcode ISA v2.1 includes operations that must preserve GF(3) invariants:

- **Ternary NOT (TNOT):** a → (3 − a) mod 3, equivalently: 0→0, 1→2, 2→1
- **Ternary MIN:** min(a, b) under natural ordering 0 < 1 < 2
- **Ternary MAX:** max(a, b) under natural ordering
- **Consensus:** If a = b then a, else 0
- **Any:** If a ≠ 0 then a, else b

Each opcode must satisfy: for all inputs in GF(3)ⁿ, the output is in GF(3).

---

## 2. Clifford Algebra Cl(3,0) over GF(3)

### 2.1 Motivation

A ternary processor manipulates trits. Groups of three trits form a *trit-triple* — a vector in GF(3)³. To describe transformations on trit-triples (rotations, reflections, compositions of gates), we need an algebraic structure richer than plain vector operations.

Clifford algebra provides exactly this: a framework where vectors can be multiplied, and the products encode geometric transformations. Over GF(3), this gives us a native ternary transformation algebra.

### 2.2 Construction

**Generators:** Three basis vectors e₁, e₂, e₃ satisfying:
- eᵢ · eᵢ = 1 (positive-definite signature, hence "Cl(3,0)")
- eᵢ · eⱼ = −eⱼ · eᵢ for i ≠ j (anticommutativity)

All scalar arithmetic is performed in GF(3), so −1 ≡ 2.

**Basis Elements (8-dimensional):**

| Grade | Elements        | Count | Interpretation              |
|-------|-----------------|-------|-----------------------------|
| 0     | 1               | 1     | Scalars                     |
| 1     | e₁, e₂, e₃     | 3     | Vectors (single trits)      |
| 2     | e₁₂, e₁₃, e₂₃  | 3     | Bivectors (trit-pair planes) |
| 3     | e₁₂₃            | 1     | Pseudoscalar (volume)       |

A general multivector:
```
M = a + b₁e₁ + b₂e₂ + b₃e₃ + c₁e₁₂ + c₂e₁₃ + c₃e₂₃ + d·e₁₂₃
```
where all coefficients a, bᵢ, cᵢ, d ∈ GF(3).

Total multivector space: 3⁸ = 6561 elements.

### 2.3 The Geometric Product

The geometric product of two basis elements follows from the defining relations. The full multiplication table for basis elements (result mod 3):

```
       1    e₁   e₂   e₃   e₁₂  e₁₃  e₂₃  e₁₂₃
1      1    e₁   e₂   e₃   e₁₂  e₁₃  e₂₃  e₁₂₃
e₁     e₁   1    e₁₂  e₁₃  e₂   e₃   e₁₂₃ e₂₃
e₂     e₂  −e₁₂  1    e₂₃ −e₁   e₁₂₃ e₃  −e₁₃
e₃     e₃  −e₁₃ −e₂₃  1   −e₁₂₃−e₁  −e₂   e₁₂
e₁₂    e₁₂  e₂  −e₁   e₁₂₃−1    e₂₃ −e₁₃ −e₃
e₁₃    e₁₃  e₃  −e₁₂₃−e₁  −e₂₃ −1    e₁₂  e₂
e₂₃    e₂₃  e₁₂₃ e₃  −e₂   e₁₃ −e₁₂ −1   −e₁
e₁₂₃   e₁₂₃ e₂₃ −e₁₃  e₁₂ −e₃   e₂  −e₁  −1
```

(Remember: −1 ≡ 2 in GF(3), so −e₁₂ means coefficient 2 times e₁₂.)

### 2.4 Rotors and Ternary Gate Composition

**Even subalgebra:** Elements of grade 0 and grade 2 form a 4-dimensional subalgebra:
```
R = α + β₁e₁₂ + β₂e₁₃ + β₃e₂₃    where α, βᵢ ∈ GF(3)
```

This gives 3⁴ = 81 possible even elements. The invertible ones (where R has a geometric product inverse) form the **rotor group** — the ternary analogue of rotation operators.

**Gate composition via rotors:** If gates G₁ and G₂ are represented as rotors R₁ and R₂, then:
```
G₁ ∘ G₂ = R₁ · R₂  (single geometric product)
```

This means: composing N sequential ternary gates reduces to a single rotor multiplication — O(1) application cost regardless of circuit depth, after a one-time O(N) composition.

**Practical application:** For the PlenumNET VM, a sequence of ternary operations on a trit-triple can be pre-compiled into a single rotor. This is the concrete version of "temporal compression" — not 4.7x, but a real and measurable reduction from O(N) sequential gate applications to O(1) rotor application.

### 2.5 The Reverse and Norm

For a multivector M, the **reverse** M̃ is obtained by reversing the order of basis vector products:
- Grade 0, 1: unchanged
- Grade 2: sign flip (multiply by −1 ≡ 2)
- Grade 3: sign flip

The **norm** of a rotor: N(R) = R · R̃ ∈ GF(3). A rotor is invertible iff N(R) ≠ 0.

---

## 3. Radix Economy — Quantifying Ternary Efficiency

### 3.1 Definition

The radix economy measures the total "cost" of representing a number N in base b:
```
E(b, N) = b · ⌈log_b(N)⌉
```

This counts: (number of possible digit values) × (number of digit positions needed).

### 3.2 Asymptotic Efficiency

For large N, the cost per unit of information is proportional to:
```
e(b) = b / ln(b)
```

Evaluating:

| Base | b / ln(b) | Relative to base 3 |
|------|-----------|---------------------|
| 2    | 2.885     | 1.057× (5.7% worse) |
| 3    | 2.731     | 1.000× (optimal integer) |
| 4    | 2.885     | 1.057× (same as binary) |
| e    | 2.718     | 0.995× (theoretical optimum) |
| 10   | 4.343     | 1.590× (59% worse) |

**Key result: Base 3 is the most efficient integer radix.** It is 5.7% more efficient than binary for large number representation.

### 3.3 Balanced Ternary Bonus

Beyond raw radix economy, balanced ternary {−1, 0, +1} provides additional efficiencies:

1. **Signed representation is free.** No two's complement overhead. The MST (most significant trit) naturally encodes sign.
2. **Rounding is truncation.** Truncating a balanced ternary number rounds to nearest, not toward zero.
3. **Negation is complement.** Negate by swapping +1 ↔ −1. No carry propagation (O(n) instead of O(n) with potential O(n) carry for two's complement).

### 3.4 Concrete Benchmarking Methodology

For PlenumNET benchmarks, we measure:

1. **Representation density:** For integers in range [0, N], compare:
   - Binary: ⌈log₂(N)⌉ bits, each storing 1 bit
   - Ternary: ⌈log₃(N)⌉ trits, each storing log₂(3) ≈ 1.585 bits
   - Efficiency ratio: (trits × 1.585) / bits

2. **Operation count:** For arithmetic on n-digit numbers:
   - Addition: Binary needs n full-adders; balanced ternary needs n half-adders (no borrow propagation in common cases)
   - Multiplication: Binary n² AND+ADD; ternary n² MUL₃+ADD₃ where MUL₃ is simpler (−1,0,+1 multiply is sign-copy or zero)

3. **VM instruction density:** For the 176-opcode ISA v2.1:
   - Binary encoding: ⌈log₂(176)⌉ = 8 bits per opcode
   - Ternary encoding: ⌈log₃(176)⌉ = 5 trits per opcode
   - Information per symbol: 5 × 1.585 = 7.925 bits vs 8 bits → ternary encodes all 176 opcodes in 5 trits (97% efficiency)

---

## 4. Ternary Torus Network Topology

### 4.1 Definition

A **k-ary n-cube** is a network where:
- n dimensions, each containing k nodes arranged in a ring (cycle)
- Each node connects to its two neighbors along each dimension
- Total nodes: kⁿ
- Node degree: 2n (constant, independent of network size)

For k = 3, we get the **ternary n-cube** (or 3-ary n-cube).

### 4.2 Properties of the 3-ary n-cube

| Property | Formula | 3D Example (27 nodes) |
|----------|---------|----------------------|
| Node count | 3ⁿ | 27 |
| Degree | 2n | 6 |
| Diameter | n · ⌊3/2⌋ = n | 3 |
| Bisection bandwidth | 2 · 3ⁿ⁻¹ | 18 |
| Total links | n · 3ⁿ | 81 |
| Average distance | n · (2/3) | 2 |

### 4.3 Why Ternary Torus for PlenumNET

**Low diameter:** In a ring of 3 nodes, the maximum distance is 1 hop (every node is adjacent to every other node in its ring). So the diameter of a 3-ary n-cube is exactly n. Compare to a binary hypercube which also has diameter n but with 2ⁿ nodes — the ternary torus packs 3ⁿ/2ⁿ = 1.5ⁿ times more nodes at the same diameter.

**Natural trit addressing:** Each node's address is an n-trit word. Routing from address A to address B is dimension-order routing: for each dimension i, move from Aᵢ to Bᵢ along ring i. Since each ring has only 3 nodes, the routing decision per dimension is trivial: stay (0 hops), go clockwise (1 hop), or go counterclockwise (1 hop). This maps directly to balanced ternary: the routing vector is B − A (mod 3), and each component is in {0, +1, −1}.

**Fault tolerance:** Each node has 2n neighbors. The vertex connectivity is 2n (by Whitney's theorem for vertex-transitive graphs), meaning 2n − 1 node failures can be tolerated while maintaining connectivity.

### 4.4 Application to Torsion Network Layer

For PlenumNET's Torsion Network:
- **Node identity:** Each node addressed by an n-trit balanced ternary word
- **Routing:** Subtraction in GF(3)ⁿ gives the routing vector directly
- **Message forwarding:** At each hop, decrement the corresponding trit of the routing vector. When all trits are 0, the message has arrived.
- **Load balancing:** The torus is vertex-transitive (all nodes are structurally identical), so no node is a natural bottleneck.

### 4.5 Comparison to Binary Topologies

| Metric (N ≈ 1000 nodes) | Binary Hypercube (2¹⁰=1024) | Ternary Torus (3⁶=729) | Ternary Torus (3⁷=2187) |
|--------------------------|------------------------------|-------------------------|--------------------------|
| Degree | 10 | 12 | 14 |
| Diameter | 10 | 6 | 7 |
| Avg distance | 5.0 | 4.0 | 4.67 |
| Bisection BW | 512 | 486 | 1458 |

The ternary torus achieves lower diameter and average distance at comparable node counts, at the cost of slightly higher degree. For latency-sensitive applications (trading, timing), this is favorable.

---

## 5. Connecting the Pieces

These four mathematical foundations combine in the PlenumNET stack:

1. **GF(3) field operations** are the atomic computational primitive — verified correct by exhaustive enumeration.
2. **Cl(3,0) over GF(3)** provides the algebraic framework for composing and optimizing sequences of ternary operations into single rotor applications.
3. **Radix economy** quantifies the information-theoretic advantage of the ternary representation, providing concrete efficiency claims for documentation and outreach.
4. **Ternary torus topology** gives the network layer a mathematically natural addressing and routing scheme that leverages balanced ternary arithmetic directly.

None of these require metaphysics. All are provable, testable, and benchmarkable.