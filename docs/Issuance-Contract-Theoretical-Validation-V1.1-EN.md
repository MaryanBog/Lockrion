# Lockrion Issuance Contract — Theoretical Validation Document v1.1

Status: Draft  
Standard: Lockrion Issuance Contract v1  
Scope: Mathematical model conformity and behavioral equivalence  

---

## 1. Purpose

This document validates that the on-chain implementation
of Lockrion Issuance Contract v1.1
is mathematically equivalent to the formal model
defined in Specification v1.1.

The objective is to verify:

- Accumulator correctness
- Reward proportionality correctness
- Temporal discreteness correctness
- Settlement boundary correctness
- Algebraic reproducibility

This document does not validate code structure.
It validates behavioral equivalence to the mathematical model.

---

## 2. Formal Mathematical Model

### 2.1 Time Model

Let:

- \( t \) = block timestamp
- \( t_0 \) = start_ts
- \( t_m \) = maturity_ts
- \( \Delta = 86400 \) seconds (accounting period)

Define:

\[
d(t) =
\begin{cases}
0 & \text{if } t < t_0 \\
\left\lfloor \dfrac{t - t_0}{\Delta} \right\rfloor & \text{if } t \ge t_0
\end{cases}
\]

Define bounded day index:

\[
d^*(t) = \min(d(t), D_f)
\]

Where:

\[
D_f = \frac{t_m - t_0}{\Delta}
\]

This defines discrete time steps.

---

### 2.2 Global Weight Model

Let:

- \( L(d) \) = total_locked at day index \( d \)

Total accumulated global weight:

\[
W = \sum_{d=0}^{D_f-1} L(d)
\]

On-chain implementation compresses this as:

\[
W \leftarrow W + L \cdot (d^* - d_{\text{last}})
\]

This is algebraically equivalent to summation over discrete days.

---

### 2.3 Per-User Weight Model

For user \( i \), define:

- \( l_i(d) \) = user locked amount at day \( d \)

User weight:

\[
W_i = \sum_{d=0}^{D_f-1} l_i(d)
\]

On-chain accumulator update:

\[
W_i \leftarrow W_i + l_i \cdot (d^* - d_{i,\text{last}})
\]

This is algebraically equivalent to discrete-time integration.

---

### 2.4 Reward Distribution Model

Let:

- \( R \) = reserve_total
- \( W \) = total_weight_accum
- \( W_i \) = user_weight_accum

Reward formula:

\[
r_i = \left\lfloor \frac{R \cdot W_i}{W} \right\rfloor
\]

Subject to:

\[
\sum_i r_i \le R
\]

Floor division guarantees bounded distribution.

---

### 2.5 Zero Participation Model

If:

\[
W = 0
\]

Then:

\[
\forall i, r_i = 0
\]

And issuer reclaim is permitted.

This matches on-chain reclaim logic.

---

## 3. Accumulator Equivalence Proof Sketch

### 3.1 Discrete Summation Equivalence

Given that:

- Weight increments only when day index increases
- Locked amount remains constant within a day

The accumulator update:

\[
W += L \cdot \Delta d
\]

is equivalent to:

\[
\sum_{d} L(d)
\]

because no intra-day fractional accumulation exists.

Thus compression does not alter total weight.

---

### 3.2 Same-Day Neutrality Proof

If multiple deposits occur within same day:

\[
d^*(t_1) = d^*(t_2)
\]

Therefore:

\[
\Delta d = 0
\]

Thus:

\[
W \text{ unchanged}
\]

No micro-timestamp advantage exists.

---

### 3.3 Maturity Boundary Proof

Because:

\[
d^*(t) = \min(d(t), D_f)
\]

Then for all \( t \ge t_m \):

\[
d^*(t) = D_f
\]

Thus:

\[
W \text{ is constant for } t \ge t_m
\]

No accumulation beyond maturity.

---

## 4. Reward Bound Proof

Given:

\[
r_i = \left\lfloor \frac{R \cdot W_i}{W} \right\rfloor
\]

Then:

\[
\sum_i r_i \le \sum_i \frac{R \cdot W_i}{W}
= \frac{R}{W} \sum_i W_i
= R
\]

Thus:

\[
\sum_i r_i \le R
\]

No over-distribution possible.

---

## 5. Determinism Proof Sketch

All model variables depend exclusively on:

- Immutable parameters
- Discrete day index
- Locked amounts
- Integer arithmetic

There is:

- No floating point
- No randomness
- No hidden state

Therefore:

Given identical state and timestamp,
all outputs are deterministic.

---

## 6. Invariant Conformity Mapping

Theoretical invariants:

- Monotonic W
- Non-negative W_i
- Bounded distribution
- Discrete time accumulation
- Irreversibility of settlement

Each invariant maps directly to:

- Accumulator update rules
- Checked arithmetic
- Escrow-bound transfers
- Settlement flags

The implementation preserves all model invariants.

---

## 7. Theoretical Validation Conclusion

The Issuance Contract v1.1 implementation is:

- Algebraically equivalent to the discrete-time summation model.
- Bounded in reward distribution.
- Deterministic in time progression.
- Resistant to intra-day exploitation.
- Closed under settlement boundaries.

The mathematical model and on-chain implementation are behaviorally equivalent.

Theoretical validation: PASS (subject to arithmetic domain bounds).
