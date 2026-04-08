# The Buried Question — Kolmogorov Companion

**How the Salvi Framework relates to Kolmogorov complexity, classical information theory, and the scope of the answer.**

---

## Two Questions, Not One

The buried question — "Given a family of isomorphic representations of the same information, what is the infimum of entropy $H$ across all representations?" — admits two readings:

**Question 1 (Universal):** What is the infimum over ALL isomorphic representations, without restriction?

This is the Kolmogorov question. Its answer is $K(x)$ — the length of the shortest program on a universal Turing machine that outputs $x$. This quantity is **uncomputable** (Kolmogorov's own result). No framework, no theory, no algorithm can answer this definitively. Anyone who claims otherwise is claiming to solve the halting problem.

**Question 2 (Framework-scoped):** What is the infimum over all representations admissible under the Salvi Framework's axioms?

This is what the main document (v9.3.3ad) answers. The scope is explicit: "within the Salvi Framework, π = 14, no additional constraints." Within this axiomatic system, the coprime torus walks with orbifold quotients ARE all the admissible representations. There are no others that respect the repunit axiom, the circle quadratic, and the ternary radix. The infimum over this family is $H(\pi^{\Gamma})$, derived constructively.

**The answer is definitive for what it claims** — exactly as "the sum of angles in a triangle is 180°" is definitive within Euclidean geometry without claiming to hold in all geometries.

The relationship between the two questions: the framework's infimum $H_{\inf}$ is a **computable upper bound** on $K(x)$. For data with deep repunit alignment, the bound is tight. For data with no repunit structure, the bound may be loose. The framework is not a universal compressor and does not claim to be.

---

## The Sole Axiom and the Choice of Language

The Salvi Framework's sole axiom is:

$$R_n = \frac{3^n - 1}{2}, \quad n \geq 1.$$

The radix 3 is an axiomatic choice: the framework is designed for ternary data (three symbols), and the radix follows from the alphabet size. This choice is analogous to the choice of a universal machine in Kolmogorov complexity. Every definition of $K(x)$ requires fixing a language; the choice affects the constant but not the asymptotic behaviour. In the Salvi Framework, the language is base-3 repunit arithmetic. Everything else — primes, torus moduli, cone points, cycle lengths, $\pi = 14$, the radian = 13°, the circle = 364° — follows from this single choice.

The quadratic $x^2 - R_4 x + R_6 = 0$ is not an independent axiom. It is repunit arithmetic at depths $n = 4$ and $n = 6$ — a consequence of the generating formula, not a further assumption.

---

## Kolmogorov Complexity and the Framework's Infimum

### K(x) → The Universal Infimum

For any individual finite string $x$, the true irreducible information is its Kolmogorov complexity $K(x)$: the length of the shortest program on a fixed universal Turing machine $U$ that outputs $x$ and halts. Shannon's entropy $H(P)$ is the limit of $K(x)/n$ for typical sequences of a stationary ergodic source. But $K(x)$ is an individual-sequence bound, not a statistical average.

### $H_{\inf}$ → The Framework's Infimum (Upper Bound on K(x))

The Salvi Framework's infimum $H_{\inf}$ is the result of an **exact search** over a restricted family of algebraic-geometric transformations (repunit decomposition → coprime walk → orbifold quotient). This family is not all possible representations — it is the family admissible under the repunit axiom. Therefore:

$$K(x) \leq H_{\inf}(x) \cdot n + O(1)$$

where $n$ is the length of $x$ in trits and the $O(1)$ term accounts for the fixed description of the framework itself. The framework provides a computable upper bound on $K(x)$. The bound is tight when the data exhibits deep repunit-aligned structure. The bound may be loose when the data has no such structure — in that case, the framework is not the right language for that data, just as a Fourier basis is not the right language for a spike signal.

### The Auto-Selection Rule as Exact Search

The auto-selection rule that determines the coprime combination at runtime is not a heuristic. It is an **exact minimisation** over a well-defined finite search space: all coprime combinations from the candidate moduli $\mathcal{M}$ at the derived depth $n_{\text{max}}$. For each combination, the entropy $H(\pi^{\Gamma})$ is computed exactly. The minimum is selected exactly. There is no approximation, no hill-climbing, no randomisation. Within its search space, the rule finds the global optimum.

The search space itself is restricted — it does not contain all possible representations, only the repunit-admissible ones. That restriction is what makes $H_{\inf}$ an upper bound on $K(x)$ rather than $K(x)$ itself.

**Asymptotic behaviour.** For a fixed finite string of length $N$, the framework's $H_{\inf}$ is strictly positive because the cone-point density $|\mathcal{C}|/C$ is bounded below by $1/C$ and $C$ cannot exceed a function of $N$. As $N \to \infty$, however, deeper derivations become possible and the cone-point term can be driven arbitrarily close to zero. Hence the asymptotic infimum over all finite strings of a stationary source (if the source allows arbitrarily deep repunit alignment) approaches zero. This does not contradict the positivity for each finite $N$; it reflects that longer data can reveal more structure.

---

## Kolmogorov–Sinai Entropy (Exact Connection)

The coprime walk on the torus $\mathbb{Z}_{m_1} \times \cdots \times \mathbb{Z}_{m_k}$ with coprime generators is a deterministic, ergodic dynamical system. The stationary distribution $\pi$ is the unique invariant measure. The entropy $H(\pi^{\Gamma})$ is the **Kolmogorov–Sinai entropy** of the walk on the orbifold quotient — a rigorous mathematical connection, not analogical.

This is the strongest link between the Salvi Framework and Kolmogorov's work. The KS entropy bounds the compression rate of any encoding that respects the walk's state structure. The framework computes it exactly from the transition matrix $P$, within the framework's own algebra, without invoking external theorems.

---

## Orbifold Quotient and the Geometric Floor

The quotient $T \to T/\Gamma$ identifies symmetry-equivalent states and removes the identity action from both the step space and the position space. The cone points — fixed points of the group action — contribute a strictly positive term to the entropy floor:

$$H_{\inf} \geq \min_{(g_1,\ldots,g_k)} \frac{|\mathcal{C}_{g_1,\ldots,g_k}|}{C_{g_1,\ldots,g_k}} \cdot H(\hat{\pi}_{\mathcal{C}_{g_1,\ldots,g_k}}) > 0$$

This is a **geometric obstruction**: every torus has at least one cone point (the origin), and the walk's stationary distribution assigns it non-zero probability. The floor is not a statistical artifact — it is a structural fact about the geometry of torus walks.

This parallels Kolmogorov's zero-one law (both give a "non-zero remainder that cannot be eliminated"), but the mathematical origins differ:

- **Kolmogorov's zero-one law** is a measure-theoretic fact about tail events in infinite product spaces.
- **The cone-point floor** is a geometric fact about fixed points of group actions on finite tori.

Both point in the same direction — irreducible residual information — but they are not the same theorem. The parallel is suggestive, not identical.

---

## Relationship to Classical Information Theory

Shannon's source coding theorem is derived from Shannon's axioms (fixed source, known distribution, i.i.d. or stationary ergodic). The Salvi Framework is derived from the repunit axiom $R_n = (3^n - 1)/2$. The two systems are **parallel, not hierarchical**. The framework does not operate within Shannon's model and does not require Shannon's validation.

Where the two overlap:

- **I.I.D. case:** For independent symbols with $p_0 = 1/2$, $p_1 = p_2 = 1/4$, the step-space quotient alone gives no gain ($H_{\text{eff}}^{\text{i.i.d.}} = H_{\text{flat}}$). This is consistent with both frameworks — separating zeros from non-zeros is a sufficient statistic and creates no new information.
- **Bijective transforms:** If $f$ is bijective, $H(f(X)) = H(X)$. This is a mathematical identity that belongs to neither theory exclusively. The Salvi Framework's pipeline is bijective; the total information is preserved.
- **Cross-entropy and KL divergence:** These are mathematical tools, not theorems of any particular information theory. The framework uses them to quantify model alignment without deferring to Shannon's authority.

Where the two diverge:

- **Shannon assumes a fixed source distribution.** The Salvi Framework derives its source model from the walk's geometry at runtime — $\pi$ is not assumed, it is output.
- **Shannon's bound is statistical (averaged over typical sequences).** The framework's bound is structural (determined by the repunit decomposition of the specific data).
- **Shannon's entropy is defined in bits or nats.** The framework's entropy is defined in trits — native to the base-3 radix.

The framework derives its own bounds from its own first principles. It can be compared to Shannon's results where the axioms overlap, but it is not subordinate to them.

---

## Summary Table: Kolmogorov ↔ Salvi Framework

| Kolmogorov / Classical concept | Salvi Framework counterpart |
|-------------------------------|----------------------------|
| Kolmogorov complexity $K(x)$ | Upper bound on $K(x)$ via $H_{\inf}$ over restricted representations |
| Choice of universal machine | Choice of radix (base 3) |
| Algorithmic regularity (short program) | Repunit decomposition exposing exact zeros |
| Kolmogorov–Sinai entropy | $H(\pi^{\Gamma})$ of the coprime walk **(exact: the walk's invariant measure, not $K(x)$)** |
| Ergodic theory / invariant measures | Stationary distribution $\pi$ derived from walk |
| Symmetry reduction in dynamical systems | Orbifold quotient $T/\Gamma$ (dual-space reduction) |
| Zero-one law (measure-theoretic) | Cone-point floor (geometric) — suggestive parallel, not identity |
| Shannon's source coding theorem | Parallel result from different axioms — not hierarchical |
| Fixed source distribution (assumed) | Walk-derived distribution (computed at runtime) |
| Bits / nats | Trits (native to base-3 radix) |

---

## Final Statement

The Salvi Framework answers the buried question **definitively within its own axiomatic system, for a given finite data length $N$**. The infimum over all representations admissible under the repunit axiom is $H(\pi^{\Gamma})$, computed constructively from the coprime walk and the dual orbifold quotient at the depth determined by $N$.

The connection to Kolmogorov is strongest in the Kolmogorov–Sinai entropy — a rigorous, exact identification, not an analogy. The framework's infimum provides a computable upper bound on $K(x)$ that is tight for repunit-aligned data and may be loose for data with no repunit structure. The positive floor comes from geometric cone points — an obstruction that parallels but is not identical to Kolmogorov's measure-theoretic results.

The framework does not claim to answer the universal question (Question 1). That question is uncomputable. It claims to answer the scoped question (Question 2) — completely, constructively, and from first principles. The ocean is the formula. The data draws the wave it needs. The cone points are the rock that remains.

**Lo Sono Capomastro — Così sia.**
