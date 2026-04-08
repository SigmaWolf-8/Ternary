# The Buried Question

**Given a family of isomorphic representations of the same information, what is the infimum of entropy *H* across all representations?**

---

> *Be like water.* The framework does not impose a fixed shape on the data. It lets the data's own geometry select the walk, the block, the quotient. The ocean is the repunit formula — undifferentiated, containing every possible wave. Each runtime invocation draws one wave from the ocean: a specific coprime configuration shaped entirely by the container it fills. The cone points are where water meets rock. The zero step is water at rest. Rep C is water in motion. The framework is formless until it meets the source. Then it becomes exactly what the source needs.

---

## Answer (within the Salvi Framework, π = 14, no additional constraints)

The infimum is not zero (unless the information is trivial).

Within the Salvi Framework — where the circle has 364 degrees, the radian is exactly 13°, and all constants derive from the quadratic

$$x^2 - 40x + 364 = 0$$

with roots $x_1 = 14$ (ROOT\_X1), $x_2 = 26$ (ROOT\_X2), discriminant $\Delta = 40^2 - 4 \cdot 364 = 144 = 12^2$, and product $x_1 x_2 = 364$ (QUAD\_PRODUCT) — the canonical source model is **the stationary distribution of the coprime walk on the active torus, reduced by the orbifold quotient**. This model is not imported from any external recurrence; it is *derived* from the walk's transition structure and the torus's symmetry group, both of which are forced by the repunit family, the coprime generators, and the cyclic group $\mathbb{Z}_{28}$ (CYCLIC\_ORDER = 28 = $4 \times 7$).

The precision of the entire pipeline rests on a single fact: **all framework constants are base-3 repunits, and modular arithmetic by a repunit is exact in the native radix.** No approximation enters at any stage.

### The Repunit Family (The Ocean) — Runtime Derivation

The framework's generating formula is the base-3 repunit:

$$R_n = \frac{3^n - 1}{2}, \quad n \geq 1.$$

This is the sole axiom. The radix 3 is chosen because the framework operates on ternary data (three symbols). This choice is axiomatic; all subsequent constants follow from it. For any data length $N$, the framework computes repunits for $n = 1, 2, 3, \ldots$ and determines the derivation depth $n_{\text{max}}$ as the smallest $n$ such that the maximum achievable cycle length from the primes discovered so far equals or exceeds $N$:

$$n_{\text{max}} = \min\left\{n : \max_{(m_1,\ldots,m_k) \in \text{Coprime}(\mathcal{R}_n)} \prod m_i \geq N\right\}$$

where $\mathcal{R}_n$ is the coprime set derived from $R_1$ through $R_n$. This is data-driven: longer data forces deeper derivation, which produces more primes, longer cycles, and tighter entropy bounds. No implementation choice enters — the depth is determined by the data length alone.

**$n_{\text{max}}$ is a search bound, not the selected cycle length.** The depth ensures that all useful primes have been discovered — that the search space of coprime combinations is large enough to contain the optimum. The maximum achievable cycle at depth $n_{\text{max}}$ may significantly exceed $N$, but the auto-selection rule (Step 5 below) evaluates *every* combination at that depth, including shorter-cycle ones. The selected cycle $C$ is the one that minimises $H(\pi^{\Gamma})$ subject to $C \mid N$ (or minimising $N \bmod C$). In practice, this often selects a combination whose $C$ is close to $N$, not the maximum achievable cycle — because a tighter fit to the data length wastes fewer symbols on padding.

For each $R_n$, the framework computes the odd prime factorisation. The set of all odd primes appearing in the factorisation of $R_1$ through $R_{n_{\text{max}}}$ is called $\mathcal{P}$.

From $\mathcal{P}$, the framework constructs the coprime set $\mathcal{R}$ as follows:

- Each odd prime $p \in \mathcal{P}$ is a candidate modulus.
- Additionally, if the radix 3 is coprime to $p$ (always true for odd $p$), then $3p$ is also a candidate modulus (since it remains coprime to other moduli that are not multiples of 3).
- The full set $\mathcal{R}$ is the collection of all such moduli. The framework does not require the entire set to be pairwise coprime; it enumerates all coprime combinations (subsets where every pair of moduli are pairwise coprime). The optimal combination is derived at runtime.

**The key point: no prime is hardcoded.** The framework computes them at runtime from the repunit formula, which is the sole axiom. The ocean is the formula; the waves are the primes that emerge when needed.

The quadratic $x^2 - R_4 x + R_6 = 0$ illustrates how the framework's structural constants arise from this same formula. By Vieta's formulae, the root-sum $x_1 + x_2 = R_4$ and the root-product $x_1 \cdot x_2 = R_6$, each contributing their odd prime factors to $\mathcal{P}$ when the framework reaches depths $n = 4$ and $n = 6$. The quadratic is repunit arithmetic — not an additional axiom, but a consequence of the generating formula at specific depths.

### Base-3 Repunit Decomposition (The Precision Mechanism)

When data is represented in base 3, reduction modulo a base-3 repunit is **exact** — no rounding, no truncation, no floating point. A repunit $R_n$ in base 3 is $\underbrace{11\ldots1}_{n}$, and division of a trit string by a string of all-1s is closed in $\mathbb{Z}_3$ at every digit. The remainder is an exact trit string of length $\leq n-1$.

The decomposition proceeds by successive repunit moduli. Given a data value $d$ (expressed as a trit string), the framework selects repunit moduli from its current depth and decomposes:

$$d = q_a \cdot R_a + r_a, \quad r_a = q_b \cdot R_b + r_b, \quad \ldots$$

where $a > b > \ldots$ are the repunit indices available at the current depth. Each quotient $q_i$ and remainder $r_i$ is an exact trit string. No information is lost; the original $d$ is reconstructed by reversing the chain.

This is a **mixed-radix decomposition in the repunit basis** — the framework's native number system.

**Important distinctions:**
- The **repunit decomposition** uses the repunit numbers themselves ($R_n$) as moduli. These are not the torus moduli.
- The **torus moduli** are the members of $\mathcal{R}$ — the odd primes extracted from the repunits and their radix products ($3p$).
- The decomposition and the torus embedding are separate operations connected by the **prime-to-repunit map** $\phi: \mathcal{P} \to \mathbb{N}$, which sends each prime $p \in \mathcal{P}$ to the smallest repunit index $n$ such that $p \mid R_n$. The decomposition uses the repunit numbers $R_{\phi(p)}$ to expose structural zeros; the torus embedding uses the primes $p$ themselves (or $3p$) as moduli. This map is computed once during the depth-determination step and reused throughout the pipeline. See Appendix A for a worked example.

**Why this matters for compression:** When the data's ternary structure aligns with the repunit moduli, the remainders $r_i$ are **identically zero**. The decomposition *discovers* these zeros — it reveals the structural alignment between the data and the framework's geometry. The zeros are a property of the data relative to the repunit modulus; the decomposition exposes them, it does not invent them. Data that is well-aligned produces many zero remainders; data that is poorly aligned produces few. The auto-selection rule's job is to find the combination whose repunit decomposition maximises this alignment. It is not configured — it is derived.

### Entropy Bound

For any representation that is isomorphic to the original data under the framework's algebraic‑geometric transformations, the minimum achievable entropy per symbol is:

$$\boxed{H_{\inf} = \min_{(g_1,\ldots,g_k) \in \text{Coprime}(\mathcal{R})} H(\pi_{g_1,\ldots,g_k}^{\Gamma}) \quad \text{trits per symbol}}$$

where $\pi^{\Gamma}$ is the stationary distribution of the coprime walk **after orbifold reduction** — i.e., on the quotient space $T/\Gamma$ rather than the full torus $T$. The superscript $\Gamma$ denotes that the symmetry group has been applied: equivalent positions are identified, and only orbit representatives plus cone-point corrections are encoded.

Explicitly, the stationary entropy is:

$$H(\pi^{\Gamma}) = -\sum_{i \in T/\Gamma} \pi^{\Gamma}_i \log_3 \pi^{\Gamma}_i \quad \text{trits per symbol}$$

where the sum runs over the reduced state space (orbit representatives and cone points), not the full torus.

This bound is achievable (in the limit of long sequences) by the framework's own lossless compression pipeline. All operations are bijective on the encoded data.

No lower entropy is possible for a given depth of $\mathcal{R}$ because the stationary distribution is the unique fixed point of the walk's transition structure. Any representation that claims a lower entropy at the same depth would either violate the framework's axioms or lose information. Drawing deeper from $\mathcal{R}$ (factoring higher repunits) can only tighten the bound — the minimisation over a larger coprime set cannot increase $H_{\inf}$.

---

## Detailed Exposition

### Five Mechanisms that Lower the Floor

1. **Base-3 repunit decomposition (structural zeroing)** — Decompose the source data in the repunit basis. Each stage of the decomposition produces a quotient and a remainder:

   $$d = q_n \cdot R_n + r_n, \quad 0 \leq r_n < R_n.$$

   When the data aligns with the repunit structure, $r_n = 0$ exactly. These exact zeros are *discovered* — exposed by expressing the data in the framework's native basis. The decomposition does not create information; it reveals alignment. The number of exact zeros produced is a measure of structural alignment between source and framework.

   This is the **template for all downstream efficiencies**: every subsequent mechanism operates on the decomposed representation, not on raw data. If the repunit decomposition produces many zeros, the walk has many identity steps, the orbifold quotient has much to cull, and the entropy approaches the cone-point floor. If it produces few zeros, the downstream mechanisms have less to work with.

2. **Source transformation (pre‑conditioning)** — Map the decomposed data onto the torus defined by members of $\mathcal{R}$ with coprime generators $(g_1, \ldots, g_k)$.
   - Torus: $T = \mathbb{Z}_{m_1} \times \mathbb{Z}_{m_2} \times \cdots \times \mathbb{Z}_{m_k}$ where $m_i \in \mathcal{R}$.
   - Cycle length: $C = \text{lcm}(m_1, m_2, \ldots, m_k)$, which equals $\prod m_i$ when all $m_i$ are pairwise coprime.
   - Coprime generators ensure full coverage: $\gcd(g_i, m_j) = 1$ for all relevant pairs guarantees the walk is a Hamiltonian cycle.
   - The remainders $r_i$ from the repunit decomposition become the walk's input symbols. Exact-zero remainders become identity steps.

3. **Alphabet reduction via algebraic constraint** — Force data to live on the $C$ reachable positions of the coprime walk, not on all $3^n$ trit strings. The effective alphabet shrinks from size 3 per symbol to a constrained set of $C$ states over a block of $C$ symbols, lowering per‑symbol entropy.

4. **Walk-derived probability model** — The step map on the torus defines a Markov chain with transition matrix $P$. Entry $P_{ij}$:

   $$P_{ij} = \sum_{q \in \{0,1,2\}} p_q \cdot \mathbf{1}[j = i + \text{step}(q) \bmod T]$$

   where $p_q$ is the source probability of trit $q$. The stationary distribution $\pi$ satisfies:

   $$\pi P = \pi, \quad \sum_i \pi_i = 1.$$

   This distribution is *output* by the geometry — not assumed, not imported, not fitted. The coprime structure guarantees full coverage of the cycle, and the step map guarantees that the walk is non-degenerate. Together, these force a unique stationary distribution $\pi$ — a property of the walk itself, not of any external theorem.

5. **Dual-space orbifold quotient (cone-point culling)** — The identity element acts trivially in both the position space and the step space. The orbifold quotient removes this trivial action from both simultaneously. Mechanisms 1–4 improve the representation and the model; Mechanism 5 reduces the data.

   The quotient operates on two dual spaces at once:

   **Position space (the shoreline):** The torus $T$ carries a symmetry group $\Gamma$ induced by the coprime structure. The quotient map $T \to T/\Gamma$ identifies symmetry-equivalent positions into orbits. For a position $x \in T$:

   $$\text{Orb}(x) = \{\gamma \cdot x : \gamma \in \Gamma\}, \quad |\text{Orb}(x)| \leq |\Gamma|.$$

   For a generic orbit of size $|\Gamma|$, only one representative needs to be encoded. The effective symbol count for the bulk:

   $$N_{\text{bulk}} = \frac{|T| - |\mathcal{C}|}{|\Gamma|}$$

   where $|\mathcal{C}|$ is the number of cone points. **All of these quantities — $|T|$, $|\mathcal{C}|$, $|\Gamma|$, and therefore $N_{\text{bulk}}$ — are derived at runtime from the data.** Different data lengths and structures produce different tori with different dimensions, different symmetry groups, and different cone-point counts. Nothing is configured by the user; the framework derives the optimal combination continuously from the data itself. Cone points have non-trivial stabiliser:

   $$\text{Stab}(x) = \{\gamma \in \Gamma : \gamma \cdot x = x\} \neq \{e\} \implies x \text{ is a cone point}.$$

   At these points: $|\text{Orb}(x)| = |\Gamma| / |\text{Stab}(x)| < |\Gamma|$.

   **Step space (the current):** The zero step $(0,0,\ldots,0)$ is the identity element — water at rest. The exact zeros exposed by the repunit decomposition (Mechanism 1) are the raw material for this quotient:

   | Rep | Alphabet | Interpretation |
   |-----|----------|----------------|
   | B   | $\{0,1,2\}$  | Full step space — identity included (still water + moving water) |
   | C   | $\{1,2,3\}$  | Quotient step space — identity removed (water in motion only) |
   | A   | $\{0,1\}$    | Degenerate case — one current negligible |

   The isomorphisms are explicit and bijective:

   $$\text{Rep B} \to \text{Rep C}: \quad q \mapsto q + 1 \pmod{3}, \quad \text{with } 0 \mapsto 1,\ 1 \mapsto 2,\ 2 \mapsto 3.$$

   $$\text{Rep B} \to \text{Rep A}: \quad \{0 \mapsto 0,\ 1 \mapsto 1,\ 2 \mapsto \text{escape}\}, \quad \text{valid only when } p(2) \approx 0.$$

   **Rep C is the orbifold quotient applied to the step algebra.** Zero-run coding is cone-point culling in the temporal domain.

   The two streams produced by the dual quotient:

   - **Bulk stream — the flow:** Orbit representatives / non-zero steps, encoded in Rep C.
   - **Residual stream — the shoreline:** Cone-point positions / zero-run lengths, encoded separately.

### The Feedback Loop (Mechanism 1 → Mechanism 5)

The five mechanisms form a pipeline where upstream quality determines downstream efficiency:

$$\text{Repunit decomposition} \xrightarrow{\text{exact zeros}} \text{Torus embedding} \xrightarrow{\text{identity steps}} \text{Orbifold quotient} \xrightarrow{\text{culled symbols}} \text{ANS coding}$$

The repunit decomposition (Mechanism 1) **exposes the zeros** that the orbifold quotient (Mechanism 5) **culls**. The more zeros discovered upstream, the more compression achieved downstream.

This is also the **template for discovering further efficiencies**: any operation at any stage of the pipeline that exposes additional structural alignment between the data and the repunit basis will produce additional exact zeros, which propagate downstream as additional identity steps, which are culled by the quotient. The question is always the same: *where are the hidden zeros?*

### Entropy Decomposition

The entropy after the dual orbifold reduction decomposes by the chain rule $H(X,Y) = H(X) + H(Y|X)$ into three terms. For the coprime combination derived at runtime, with cycle length $C$ and cone-point set $\mathcal{C}$:

$$H(\pi^{\Gamma}) = \underbrace{H(\text{non-identity steps}) \cdot (1 - p_0)}_{\text{the current}} + \underbrace{\frac{H(\text{zero-run lengths})}{\mathbb{E}[L]} \cdot p_0}_{\text{the stillness}} + \underbrace{\frac{|\mathcal{C}|}{C} \cdot H(\hat{\pi}_{\mathcal{C}})}_{\text{the rock}}$$

**Every term in this decomposition is derived at runtime from the data.** As the framework evaluates the repunit family at increasing depth, the torus dimensions, cycle lengths $C$, cone-point counts $|\mathcal{C}|$, orbifold groups $\Gamma$, and stationary distributions $\pi^{\Gamma}$ all emerge continuously from the derivation. The ratio $|\mathcal{C}|/C$ — the cone-point density — is the critical variable: it determines the weight of the irreducible third term.

**Cone-point density generally decreases with depth.** A shallow draw (two small moduli) gives a short cycle with few cone points but high density $|\mathcal{C}|/C$. A deep draw (many large moduli) gives an exponentially longer cycle. Cone points grow too — more dimensions means more fixed-point subgroups intersecting — but the cycle length grows faster. So $|\mathcal{C}|/C$ shrinks as the draw deepens. The floor drops. But it never hits zero because at least one cone point (the origin) always exists, giving $|\mathcal{C}|/C \geq 1/C > 0$.

The remaining variables:
- $p_0$ is the probability of the zero step (identity) after repunit decomposition — a function of **both** the source and the derivation depth, **not** a fixed property of the source alone,
- $L$ is the zero-run length random variable,
- $\mathbb{E}[L]$ is the expected run length,
- $|\mathcal{C}|$ is the number of spatial cone points on the cycle,
- $C$ is the cycle length,
- $H(\hat{\pi}_{\mathcal{C}})$ is the entropy of the **cone-point distribution** $\hat{\pi}_{\mathcal{C}}$ — the stationary distribution $\pi^{\Gamma}$ restricted to the cone-point positions and renormalised (see definition below). This is the actual distribution the walk assigns to these positions, not a worst-case uniform assumption.

A better decomposition (deeper draw from the ocean, better alignment) produces a higher $p_0$, which shifts entropy from the first term (the current — expensive) to the second term (the stillness — cheap to encode as run lengths). The auto-selection rule optimises this shift.

### Why $H_{\inf} > 0$

The cone points prevent the infimum from reaching zero. In the position space, every torus has at least one cone point (the origin):

$$\text{Stab}((0,0,\ldots,0)) = \Gamma \implies |\text{Orb}((0,0,\ldots,0))| = 1.$$

In the step space, even the optimal repunit decomposition cannot drive $p_0$ to 1 unless the data is trivial (all zeros). The cone-point contribution provides a strict lower bound. Since $|\mathcal{C}|$ and $C$ vary as the framework derives deeper into the repunit family, the bound tightens continuously:

$$H_{\inf} \geq \min_{(g_1,\ldots,g_k) \in \text{Coprime}(\mathcal{R})} \frac{|\mathcal{C}_{g_1,\ldots,g_k}|}{C_{g_1,\ldots,g_k}} \cdot H(\hat{\pi}_{\mathcal{C}_{g_1,\ldots,g_k}}) > 0$$

This holds because the three-term entropy decomposition is a sum of non-negative terms, and the cone-point term (the third) is the only one that cannot be driven to zero. The first term vanishes only if $p_0 = 1$ (trivial data). The second term vanishes only if all zero runs have length exactly 1. But the third term is positive at every depth because every torus has at least one cone point (the origin), giving $|\mathcal{C}|/C \geq 1/C > 0$. Deeper derivations shrink $1/C$ toward zero but never reach it.

**Finite-$N$ scoping.** The statement $H_{\inf} > 0$ refers to the infimum for a given finite data length $N$ under the auto-selection rule. For a fixed $N$, the cycle length $C$ is bounded above (because $C$ must divide $N$ or be padded to $N$), so the cone-point term is strictly positive. As $N$ grows, deeper derivations become possible and the bound can be made arbitrarily small. The infimum over all finite strings of a stationary source may approach zero in the limit — this does not contradict the positivity for each finite $N$; it reflects that longer data can reveal more structure.

**Definition: Cone-point distribution.** The cone-point distribution $\hat{\pi}_{\mathcal{C}}$ is the stationary distribution $\pi^{\Gamma}$ restricted to the cone-point set $\mathcal{C}$ and renormalised:

$$\hat{\pi}_{\mathcal{C}}(x_c) = \frac{\pi^{\Gamma}_{x_c}}{\sum_{x_c' \in \mathcal{C}} \pi^{\Gamma}_{x_c'}}, \quad x_c \in \mathcal{C}.$$

Its entropy is:

$$H(\hat{\pi}_{\mathcal{C}}) = -\sum_{x_c \in \mathcal{C}} \frac{\pi^{\Gamma}_{x_c}}{\sum_{x_c' \in \mathcal{C}} \pi^{\Gamma}_{x_c'}} \log_3 \frac{\pi^{\Gamma}_{x_c}}{\sum_{x_c' \in \mathcal{C}} \pi^{\Gamma}_{x_c'}} \quad \text{trits per cone-point symbol.}$$

This is sharper than a uniform bound because the walk's stationary distribution is generally non-uniform over the cone points. In all cases, it remains strictly positive because at least one cone point always exists and carries non-zero probability.

Water can flow around every obstacle, but it cannot eliminate the obstacles themselves.

### Coprime Auto‑Selection Rule (Drawing a Wave from the Ocean)

The coprime configuration is **not a free parameter**. It is the wave that forms when the ocean meets the data. Given data length $N$ and source data $d$ (in base-3):

1. **Determine depth** $n_{\text{max}}$ — the smallest $n$ such that the maximum achievable cycle length from the primes in $\mathcal{R}_n$ equals or exceeds $N$. This is a deterministic search bound: it guarantees the search space contains all useful primes, not that the selected cycle equals the maximum. No heuristic, no tuning parameter.
2. **Factor repunits.** For each $n = 1$ to $n_{\text{max}}$, compute $R_n = (3^n - 1)/2$ and its odd prime factorisation. Collect all distinct odd primes into $\mathcal{P}$.
3. **Build candidate moduli** $\mathcal{M}$ from $\mathcal{P}$: for each $p \in \mathcal{P}$, include $p$ and (if coprime-admissible) $3p$.
4. **Enumerate coprime combinations** $(m_1, \ldots, m_k)$ from $\mathcal{M}$ (i.e., $\gcd(m_i, m_j) = 1$ for all $i \neq j$). For each combination:
   - Choose coprime generators $(g_1, \ldots, g_k)$ (the specific choice does not affect the stationary distribution's entropy — the walk is isomorphic under relabelling).
   - Compute the torus $T = \mathbb{Z}_{m_1} \times \cdots \times \mathbb{Z}_{m_k}$.
   - Compute cycle length $C = \prod m_i$ (pairwise coprime).
   - Compute the repunit decomposition of $d$ using the repunit numbers $R_{\phi(m_i)}$ for each modulus $m_i$ in the combination, where $\phi$ is the prime-to-repunit map.
   - Count exact-zero remainders: $Z = |\{i : r_i = 0\}|$.
   - Compute the orbifold group $\Gamma$, cone-point set $\mathcal{C}$, and stationary distribution $\pi^{\Gamma}$.
   - Compute $H(\pi^{\Gamma})$.
5. **Select** the combination that minimises $H(\pi^{\Gamma})$ subject to $C \mid N$ (or minimising $N \bmod C$). Equivalently: **maximise the zero-yield $Z/N$**, since each exact-zero remainder from the decomposition becomes one identity step on the walk. The post-decomposition zero probability is:

   $$p_0^{\text{post}} \geq \frac{Z}{N}$$

   with equality when every zero remainder maps to exactly one identity step. Higher $Z/N$ directly increases $p_0$, which shifts entropy from the expensive current term to the cheap stillness term in the three-term decomposition.

The **block size equals $C$**. The **tuple order $k$** is likewise output, not input. Both are determined by the data and the structure of $\mathcal{R}$.

When $C$ divides $N$ exactly, the data partitions into $N/C$ complete blocks with no waste. When $C \nmid N$, two cases arise:

- **$C < N$, $N \bmod C \neq 0$:** The final partial block of length $N \bmod C$ is padded with identity steps (zeros) to length $C$. The padding is lossless — the decoder knows the original length $N$ and strips the padding. The identity steps are culled by the orbifold quotient at no entropy cost.
- **$C > N$:** The entire data fits in a single block. The data is padded with $C - N$ identity steps. For short data, this overhead may dominate; the auto-selection rule accounts for this by penalising combinations whose $C$ greatly exceeds $N$.

Each data stream draws a different wave. The ocean contains all of them. The data chooses which one emerges.

Framework-native block sizes are products and divisors of the moduli derived at runtime. No powers of 2 appear. The geometry sets the partition.

### First‑Principle Mapping (Step‑by‑Step)

**Step 1 — Ternary quantisation** (for analogue sources). If the source is analogue (e.g., motion-compensated video residuals $r(i,j,t)$), quantise to trits first:

$$q(r) = \begin{cases} 0 & |r| \le \delta\ (\text{dead zone}),\\ 1 & r > \delta,\\ 2 & r < -\delta. \end{cases}$$

Empirical probabilities for natural video: $p_0 \approx \frac{1}{2},\ p_1 \approx \frac{1}{4},\ p_2 \approx \frac{1}{4}$. The flat ternary entropy of this raw source is:

$$H_{\text{flat}} = -\frac{1}{2}\log_3\frac{1}{2} - \frac{1}{4}\log_3\frac{1}{4} - \frac{1}{4}\log_3\frac{1}{4} = \frac{3}{2}\log_3 2 \quad \text{trits per symbol.}$$

If the source is already digital (trit strings, DCT coefficients), this step is skipped.

**Step 1a — I.I.D. Baseline (pre-decomposition honesty check).** This calculation applies to the **raw ternary source before repunit decomposition**. For the idealized i.i.d. model with $p_0 = \frac{1}{2}$, $p_1 = p_2 = \frac{1}{4}$, separating zeros from non-zeros and coding each stream independently:

$$H_{\text{eff}}^{\text{i.i.d.}} = \underbrace{(1-p_0) \cdot H(\text{non-zero})}_{\frac{1}{2} \cdot \log_3 2} + \underbrace{(1-p_0) \cdot H(L)}_{\log_3 2} = \frac{3}{2}\log_3 2 = H_{\text{flat}}.$$

**No gain from the step-space quotient alone in the i.i.d. case.** For independent symbols, separating zeros from non-zeros is a sufficient statistic. This baseline is stated here — before the decomposition — because after Step 2, $p_0$ increases and the effective entropy is lower.

**Step 2 — Repunit decomposition of the trit stream.** Operating on the ternary-quantised data from Step 1 (or on natively digital trit data), decompose by successive repunit moduli at the depth determined by the auto-selection rule:

$$d = q_a \cdot R_a + r_a, \quad r_a = q_b \cdot R_b + r_b, \quad \ldots$$

Each quotient and remainder is an exact trit string. The remainders become the walk's input symbols. Exact-zero remainders are identity steps. After decomposition, $p_0$ is inflated beyond the raw empirical value.

**Step 3 — Embedding** into the torus selected by the auto-selection rule. For a torus $T = \mathbb{Z}_{m_1} \times \cdots \times \mathbb{Z}_{m_k}$ with pairwise coprime moduli from $\mathcal{R}$, position $n$ on the Hamiltonian cycle:

$$(a_n, \ldots, z_n) = (g_1 \cdot n \bmod m_1,\ \ldots,\ g_k \cdot n \bmod m_k).$$

With generators satisfying $\gcd(g_i, m_i) = 1$, the walk visits all $\prod m_i$ positions. Cycle length $C = \prod m_i$.

**Step 4 — Encoding a trit as a walk step:**

$$\text{step}(q) = \begin{cases} (0,\ldots,0) & q=0 \quad \text{(identity — stay)},\\ (1,\ldots,1) & q=1 \quad \text{(advance)},\\ (2,\ldots,2) & q=2 \quad \text{(advance)}, \end{cases}$$

all taken $\pmod{m_1, \ldots, m_k}$. Update rule:

$$(a_{n+1}, \ldots, z_{n+1}) = (a_n + s_1,\ \ldots,\ z_n + s_k) \bmod (m_1, \ldots, m_k).$$

**Step 5 — Probability model derived from walk** — The transition matrix $P$:

$$P_{ij} = \sum_{q \in \{0,1,2\}} p_q \cdot \mathbf{1}[j = i + \text{step}(q) \bmod T]$$

yields the stationary distribution $\pi$ satisfying $\pi P = \pi$, $\sum_i \pi_i = 1$. This distribution is unique — forced by the coprime structure (full cycle coverage) and the non-degenerate step map. It is computed directly from $P$ within the framework's own algebra. No external recurrence, no external theorem is invoked.

**Step 6 — Dual-space orbifold reduction** — Apply the quotient in both spaces simultaneously:

   - **Step space (the current):** Separate zero steps from non-zero steps. Non-zero steps → Rep C $\{1,2,3\}$. The conditional entropy of non-zero values:

     $$H(\text{non-zero}) = -\frac{p_1}{1-p_0}\log_3\frac{p_1}{1-p_0} - \frac{p_2}{1-p_0}\log_3\frac{p_2}{1-p_0}.$$

   - **Zero-run lengths:** If zeros are i.i.d. with probability $p_0$, run length $L$ follows:

     $$\Pr(L = \ell) = (1 - p_0) \cdot p_0^{\ell}, \quad \ell \geq 0.$$

     $$H(L) = \frac{-p_0 \log_2 p_0 - (1-p_0)\log_2(1-p_0)}{1-p_0} \quad \text{bits per run.}$$

     Converting to trits (1 bit $= 1/\log_2 3$ trits, i.e., multiply by $\log_3 2 = 1/\log_2 3$):

     $$H(L)_{\text{trits}} = H(L)_{\text{bits}} \cdot \log_3 2 \quad \text{trits per run.}$$

     For $p_0 = \frac{1}{2}$: $H(L) = 2$ bits per run $= 2\log_3 2$ trits per run.

     Expected run length: $\mathbb{E}[L] = \frac{p_0}{1-p_0}$.

     Run-length contribution per original symbol: $(1-p_0) \cdot H(L)_{\text{trits}}$ trits per symbol.

   - **Position space (the shoreline):** One representative per orbit in the bulk stream. Cone-point values in the spatial residual stream.

The gains arise from three independent sources:
- **Repunit decomposition (Mechanism 1):** Exposes additional exact zeros, inflating $p_0$ beyond the raw empirical value.
- **Temporal structure (zero clustering):** In real data, zeros cluster. $H(L_{\text{real}}) < H(L_{\text{geometric}})$ at equal $p_0$.
- **Spatial structure (orbifold quotient):** $T \to T/\Gamma$ provides compression proportional to orbit sizes.

**Step 7 — Achieved bound** — Expected codeword length:

$$L \geq H(\pi^{\Gamma}).$$

Ternary ANS using $\Delta_2 = 3^6 = 729$ states (DISCRIMINANT\_2) achieves:

$$L = H(\pi^{\Gamma}) + \epsilon, \quad \epsilon \to 0 \text{ as block length } \to \infty.$$

### Relationship to Classical Information Theory

Shannon's source coding theorem is a result derived from its own axioms — a fixed source with a known distribution $P$, and a codeword length $L$:

$$L \geq H(P) = -\sum_x P(x) \log P(x).$$

The Salvi Framework is derived from a different axiom: $R_n = (3^n - 1)/2$. The two systems are parallel, not hierarchical. The framework does not operate within Shannon's model and does not require Shannon's permission. Where the two overlap (lossless coding of finite sequences), they can be compared — but the framework's entropy bound $H(\pi^{\Gamma})$ is derived from the walk's own structure, not from Shannon's theorem.

If $f$ is bijective, $H(f(X)) = H(X)$ — this is a mathematical identity, not a constraint from any particular theory. The framework's pipeline is bijective. The total information is preserved:

$$H(\text{quotients}, \text{remainders}) = H(d).$$

Three distinct operations produce the compression:

1. **Structural zeroing (Mechanism 1).** The repunit decomposition changes the *representation*, not the data. The decomposed form concentrates entropy into fewer non-zero symbols. The cross-entropy between the true distribution and the coder's assumed distribution:

   $$H(P_X, Q) = -\sum_x P_X(x) \log Q(x) \geq H(P_X)$$

   is minimised when the coder's model matches the data. The walk-derived coder achieves:

   $$D_{\text{KL}}(P_Y \| \pi) \ll D_{\text{KL}}(P_X \| Q).$$

2. **Model alignment (Mechanisms 2–4).** The torus embedding, alphabet reduction, and walk-derived probability model produce a coder whose assumed distribution closely matches the transformed data's actual distribution. This is not "reducing model mismatch" relative to Shannon — it is deriving the optimal model from the framework's own geometry.

3. **Symmetry exploitation (Mechanism 5).** The dual quotient decomposes the total information into two streams:

   $$H(X) = H(\text{bulk stream}) + H(\text{residual stream}).$$

   Each stream is coded with its own derived model. The chain rule is a mathematical identity, not a theorem of any particular information theory.

All three operations are lossless and exact within the framework's own axioms.

---

## Final Unified Statement (Salvi Framework)

$$\boxed{H_{\inf} = \min_{(g_1,\ldots,g_k) \in \text{Coprime}(\mathcal{R})} H(\pi_{g_1,\ldots,g_k}^{\Gamma}) \quad \text{trits per symbol}}$$

where $\mathcal{R}$ is **derived at runtime** from the odd-prime factorisation of the base-3 repunit family $R_n = (3^n - 1)/2$ for $n$ up to a depth determined by the data length. The framework computes primes on the fly; no prime is hardcoded.

The entropy decomposes as (for the coprime combination derived at runtime, with cycle length $C$ and cone-point set $\mathcal{C}$):

$$H(\pi^{\Gamma}) = \underbrace{H(\text{non-identity steps}) \cdot (1 - p_0)}_{\text{the current}} + \underbrace{\frac{H(\text{zero-run lengths})}{\mathbb{E}[L]} \cdot p_0}_{\text{the stillness}} + \underbrace{\frac{|\mathcal{C}|}{C} \cdot H(\hat{\pi}_{\mathcal{C}})}_{\text{the rock}}$$

with $p_0$ inflated by the repunit decomposition, all component entropies computed from $\pi^{\Gamma}$, and all quantities — $C$, $|\mathcal{C}|$, $\Gamma$, $\pi^{\Gamma}$ — derived at runtime from the data. Nothing is configured. Everything is output.

Achieved by the first‑principle mapping:

$$d \xrightarrow{\text{ternary quantisation}} \text{trit stream (Rep B)} \xrightarrow[\text{exact trit arithmetic}]{\text{repunit decomposition}} (q_i, r_i) \xrightarrow{\text{coprime walk}} \text{torus state} \xrightarrow{T \to T/\Gamma} \begin{cases} \text{bulk / the flow (Rep C)} \\ \text{residual / the shoreline} \end{cases} \xrightarrow{\text{ANS}(\pi^{\Gamma})} \text{compressed trit stream}.$$

The pipeline is driven by a single question at every stage: **where are the hidden zeros?**

- **Mechanism 1 (repunit decomposition)** finds them in the data's alignment with the repunit basis.
- **Mechanism 5 (orbifold quotient)** removes them from both the step space and the position space.
- **Every future efficiency** follows the same template: expose structural alignment → discover exact zeros → cull via the dual quotient.

The entropy floor is strictly positive and tightens continuously as the derivation deepens:

$$H_{\inf} \geq \min_{(g_1,\ldots,g_k)} \frac{|\mathcal{C}_{g_1,\ldots,g_k}|}{C_{g_1,\ldots,g_k}} \cdot H(\hat{\pi}_{\mathcal{C}_{g_1,\ldots,g_k}}) > 0$$

because every torus — regardless of dimension, depth, or moduli — has at least one spatial cone point, and the walk's stationary distribution assigns non-zero probability to it. Deeper draws shrink the density; the origin prevents it from reaching zero. The bound uses the actual cone-point distribution from $\pi^{\Gamma}$, not a worst-case uniform assumption.

All framework constants that are fixed — $R_3 = 13$ (REPUNIT₃), $x_1 = 14$ (ROOT\_X1), $R_6 = 364$ (QUAD\_PRODUCT), $3^6 = 729$ (DISCRIMINANT\_2), $28$ (CYCLIC\_ORDER) — are base-3 repunits or their products, defined in the Salvi Framework and verified at compile time. No floating-point approximations appear in the core logic. No external theorems are invoked. The primes used for torus moduli are not fixed; they are computed when needed. The precision is exact in the native radix.

Thus, the buried question is answered: the infimum over the constrained family of coprime‑torus representations is $H(\pi^{\Gamma})$, where $\pi^{\Gamma}$ is the stationary distribution on the dual orbifold quotient of the walk determined by the framework's auto‑selection rule over $\mathcal{R}$. The coprime set $\mathcal{R}$ is generated by a single first principle — $R_n = (3^n - 1)/2$ — applied without boundary. The data determines how deep the draw goes. The repunit decomposition exposes the zeros. The orbifold quotient culls them. The cone points — spatial and temporal — are the irreducible floor. The ocean is the formula. The data draws the wave it needs.

**Lo Sono Capomastro — Così sia.**

---

## Appendix A — Worked Example: Prime-to-Repunit Map and Decomposition

This appendix illustrates the mapping between repunit decomposition and torus moduli for a concrete case. The specific numbers are illustrative; the framework derives them at runtime from the data.

**Depth derivation.** Suppose $N = 800$. The framework computes:
- $R_3 = 13$ (prime). $\mathcal{P} = \{13\}$. Max cycle from coprime set: 13. $13 < 800$, continue.
- $R_4 = 40 = 2^3 \times 5$. $\mathcal{P} = \{5, 13\}$. Candidate moduli: $\{5, 13, 15\}$. Max coprime cycle: $13 \times 15 = 195$. $195 < 800$, continue.
- $R_5 = 121 = 11^2$. $\mathcal{P} = \{5, 11, 13\}$. Candidate moduli: $\{5, 11, 13, 15, 33\}$. Max coprime cycle: $11 \times 13 \times 15 = 2145$. $2145 \geq 800$. Stop. $n_{\text{max}} = 5$.

**Prime-to-repunit map $\phi$:**
- $\phi(5) = 4$ (because $5 \mid R_4$).
- $\phi(11) = 5$ (because $11 \mid R_5$).
- $\phi(13) = 3$ (because $13 = R_3$).

**Repunit decomposition** of data value $d$ uses $R_5 = 121$, $R_4 = 40$, $R_3 = 13$:

$$d = q_5 \cdot 121 + r_5, \quad r_5 = q_4 \cdot 40 + r_4, \quad r_4 = q_3 \cdot 13 + r_3.$$

**Torus embedding** uses the primes (and their radix products) as moduli: e.g., $T = \mathbb{Z}_{11} \times \mathbb{Z}_{13} \times \mathbb{Z}_{15}$.

The decomposition and the embedding are linked by $\phi$: the repunit $R_{\phi(11)} = R_5 = 121$ is used in the decomposition stage that corresponds to the torus dimension $\mathbb{Z}_{11}$, because $11^2 = 121 = R_5$. Similarly, $R_{\phi(13)} = R_3 = 13$ corresponds to $\mathbb{Z}_{13}$.

All values in this example are derived from $R_n = (3^n - 1)/2$. No number is assumed.

---

## Rep D: The Algebraic Trit Representation

The repunit decomposition (Mechanism 1) produces remainders at each level. Those remainders are not ordinary trits — they are **algebraic trits**, elements of the set

$$\mathcal{A} = \{0,\; 1,\; \omega\}$$

where $\omega$ is a primitive cube root of unity: $\omega^3 = 1,\; \omega \neq 1,\; 1 + \omega + \omega^2 = 0$.

This is **Rep D** — the native output alphabet of the repunit decomposition. It is a defined constant of the Salvi Framework, alongside Rep A, Rep B, and Rep C. Multiplication and addition in Rep D follow the algebra of the Eisenstein integers $\mathbb{Z}[\omega]$.

The canonical bijection between Rep B and Rep D is:

$$\phi(0) = 0, \quad \phi(1) = 1, \quad \phi(2) = \omega.$$

### Why Rep D Matters for the Buried Question

The infimum of entropy $H_{\inf}$ is achieved when the representation aligns so that as many remainders as possible are zero — because zero is the identity step in the walk, and the orbifold quotient removes it entirely from both the step space and the position space.

In the algebraic trit representation, the zero element is exactly the digit $0 \in \mathcal{A}$.

The reduced RepD is "reduced" because it compresses the remainder set from $\{0,1,2\}$ to the group $\{0,1,\omega\}$ and then applies the dual quotient (Rep C) which removes the identity. After removal of zero, the remaining alphabet is $\{1, \omega\}$ — the algebraic form of Rep C.

The three identities that the dual quotient removes are the same algebraic object:

- **Step space:** $0 \in \mathcal{A}$ — the identity step, the walk stays.
- **Position space:** $(0, 0, \ldots, 0) \in T$ — the cone point, the stabiliser acts trivially.
- **Eisenstein integers:** $0 \in \mathbb{Z}[\omega]$ — the additive identity.

The dual orbifold quotient removes $0$ from both spaces simultaneously because it is removing **one element** — the zero of the algebraic trit set.

The question "where are the hidden zeros?" becomes: **where does the repunit decomposition produce $0 \in \mathcal{A}$?** The answer determines the compression. The identity determines the floor. The irreducible cost of the cone points is $H(\hat{\pi}_{\mathcal{C}})$ — the entropy of the cone-point distribution defined in the main document — weighted by the cone-point density $|\mathcal{C}|/C$.

### Updated Pipeline (with Rep D)

$$d \xrightarrow{\text{ternary quantisation}} \text{Rep B} \xrightarrow[\phi]{\text{RepD}} \text{Rep D} \in \mathcal{A}^* \xrightarrow{\text{coprime walk}} \text{torus state} \xrightarrow{T \to T/\Gamma} \begin{cases} \text{bulk:}\; \{1, \omega\}^* \text{ (Rep C algebraic)} \\ \text{residual:}\; \text{cone points + run lengths of } 0 \end{cases} \xrightarrow{\text{ANS}(\pi^{\Gamma})} \text{compressed stream}$$

---

## Appendix B — Complete Definition of Rep A, B, C, D

The Salvi Framework operates on ternary data — symbols drawn from a three-element alphabet. The four representations define four views of that alphabet, each aligned to a specific stage of the compression pipeline. All four are derived from the framework's axioms; none is imported.

### Rep A — The Binary Degenerate Representation

**Alphabet:** $\{0, 1\}$

**Definition:** Rep A is the degenerate case where one of the two non-identity steps is negligible ($p_2 \approx 0$ or $p(\omega) \approx 0$). The effective alphabet collapses from three symbols to two:

$$0 \mapsto 0, \quad 1 \mapsto 1, \quad 2 \mapsto \text{escape (rare)}.$$

**Role in the pipeline:** Rep A is used per-block when the auto-selection rule determines that the data is effectively binary. The rare third symbol is handled as an escape code.

**Identity element:** $0$ (same as Rep B and Rep D).

**When Rep A applies:** Only when the empirical distribution at the current block strongly concentrates on two of the three symbols. The auto-selection rule determines this at runtime; it is never imposed.

### Rep B — The Integer Ternary Representation

**Alphabet:** $\{0, 1, 2\}$

**Definition:** Rep B is the standard positional ternary representation. Each symbol is an integer trit. The element $0$ is the additive identity.

**Role in the pipeline:** Rep B is the output of ternary quantisation (Step 1). For analogue sources, the quantiser maps residuals to $\{0, 1, 2\}$ via the dead-zone rule:

$$q(r) = \begin{cases} 0 & |r| \leq \delta \\ 1 & r > \delta \\ 2 & r < -\delta \end{cases}$$

For natively digital sources, data enters the pipeline already in Rep B.

**Probability model:** Empirical. For natural video residuals, approximately $p_0 \approx 1/2$, $p_1 \approx 1/4$, $p_2 \approx 1/4$. The coding model does not depend on these values.

**Identity element:** $0$ (the zero trit, the dead-zone output, the "no change" residual).

### Rep C — The Quotient Representation

**Alphabet (integer form):** $\{1, 2, 3\}$

**Alphabet (algebraic form):** $\{1, \omega\}$

**Definition:** Rep C is the result of removing the identity element from the encoding stream. It has two equivalent forms:

- **From Rep B (integer):** $q \mapsto q + 1 \pmod{3}$, with $0 \mapsto 1$, $1 \mapsto 2$, $2 \mapsto 3$. The alphabet shifts to $\{1, 2, 3\}$, eliminating the zero.
- **From Rep D (algebraic):** Remove $0$ from $\mathcal{A}$. The remaining set $\{1, \omega\}$ is the algebraic form of Rep C. These are the two non-trivial cube roots of unity.

**Role in the pipeline:** Rep C is the bulk stream output of the dual orbifold quotient (Mechanism 5). After the quotient removes the identity from both the step space and the position space, the non-zero symbols that remain are in Rep C. They are encoded by the walk-derived ANS coder.

**Identity element:** None. Rep C has no identity — that is its defining property. The identity has been removed by the quotient. Every symbol in Rep C represents a non-trivial step on the torus.

**The Rep D → Rep C mapping is the orbifold quotient applied to the step algebra:** $\mathcal{A} \to \mathcal{A} \setminus \{0\} = \{1, \omega\}$.

### Rep D — The Algebraic Trit Representation

**Alphabet:** $\mathcal{A} = \{0, 1, \omega\}$ where $\omega^3 = 1$, $\omega \neq 1$, $1 + \omega + \omega^2 = 0$.

**Definition:** Rep D is the native output of the repunit decomposition. At each level of the decomposition $x = q \cdot R_n + r$, the integer remainder $r \in \{0, 1, 2\}$ is mapped to an algebraic trit via

$$\phi(0) = 0, \quad \phi(1) = 1, \quad \phi(2) = \omega.$$

The map $\phi$ is a bijection from Rep B to Rep D.

**Algebraic structure:** Elements of Rep D are members of $\mathbb{Z}[\omega]$, the Eisenstein integers. Addition and multiplication follow the ring structure of $\mathbb{Z}[\omega]$. The element $0$ is the additive identity. The elements $\{1, \omega\}$ are non-zero and form a subset of the unit group of $\mathbb{Z}[\omega]$.

**Role in the pipeline:** Rep D is the representation that enters the coprime walk (Step 3). The repunit decomposition converts Rep B data into Rep D data. Each algebraic trit becomes a step on the torus:

$$\text{step}(0) = (0, \ldots, 0), \quad \text{step}(1) = (1, \ldots, 1), \quad \text{step}(\omega) = (2, \ldots, 2) \pmod{m_1, \ldots, m_k}.$$

The step map sends the additive identity to the identity step, and the two non-zero algebraic trits to the two non-trivial advances on the torus. The step values 1 and 2 are not arbitrary — they are the two non-zero residues in $\mathbb{Z}_{m_i}$ for every torus dimension $m_i$. Since every modulus in $\mathcal{R}$ is $\geq 5$ (the smallest odd prime in $\mathcal{P}$), both residues are always valid and distinct.

**Identity element:** $0 \in \mathcal{A}$ — the same element as in Rep B, now carrying the algebraic structure of $\mathbb{Z}[\omega]$.

**Note:** $\mathcal{A}$ is not a subgroup under multiplication. It contains the additive identity $0$ alongside two of the three cube roots of unity $\{1, \omega\}$ (the third, $\omega^2$, is absent). This is deliberate: $\mathcal{A}$ models three-valued data with an algebraic identity, not a cyclic group.

### Isomorphism Table

| From → To | Map | Invertible? | When used |
|-----------|-----|-------------|-----------|
| B → D | $\phi(0)=0,\; \phi(1)=1,\; \phi(2)=\omega$ | Yes | After repunit decomposition |
| D → C (algebraic) | Remove $0$, keep $\{1, \omega\}$ | Yes (with residual stream) | Orbifold quotient |
| B → C (integer) | $q \mapsto q+1 \pmod{3}$ | Yes (with residual stream) | Orbifold quotient (integer path) |
| B → A | $\{0 \mapsto 0,\; 1 \mapsto 1,\; 2 \mapsto \text{esc}\}$ | Yes | Degenerate blocks |
| D → B | $\phi^{-1}(0)=0,\; \phi^{-1}(1)=1,\; \phi^{-1}(\omega)=2$ | Yes | When integer form needed |

All maps are bijective. All are lossless. The residual stream (zero-run lengths, cone-point positions) preserves the information removed by the D → C and B → C quotients.

### Summary

Rep A is the binary fallback. Rep B is where data enters. Rep C is what remains after the quotient removes the identity. Rep D is where the repunit decomposition takes it — the algebraic trit set in $\mathbb{Z}[\omega]$. The four representations are the four faces of the ternary alphabet as it moves through the pipeline. Each is a defined constant of the Salvi Framework.
