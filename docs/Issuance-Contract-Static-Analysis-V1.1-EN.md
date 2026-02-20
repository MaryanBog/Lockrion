# Lockrion Issuance Contract — Static Analysis Document v1.1

Status: Draft  
Standard: Lockrion Issuance Contract v1  
Applies To: Implementation v1.1 (Rust, Anchor-compatible)  
Scope: Undefined Behavior, Memory Safety, Determinism, Arithmetic Strictness  

---

# 1. Purpose

This document defines the static analysis framework for Lockrion Issuance Contract v1.1.

The objective is to verify, through compile-time and static inspection, that the implementation:

- contains no Undefined Behavior (UB),
- is memory-safe under Rust and Solana constraints,
- is deterministic under identical inputs and state,
- uses strict integer arithmetic with checked operations,
- contains no floating-point operations,
- does not rely on architecture-dependent behavior,
- preserves critical invariants by construction.

This document defines static checks only.
It does not replace runtime testing, integration testing, or on-chain validation.

---

# 2. Undefined Behavior (UB) Prevention

This section defines structural constraints that eliminate Undefined Behavior (UB)
at compile-time and prevent unsafe execution paths.

---

## 2.1 Rust Safety Constraints

The implementation MUST:

- Contain no `unsafe` blocks.
- Avoid raw pointer dereferencing.
- Avoid `std::mem::transmute`.
- Avoid manual memory layout manipulation.
- Avoid unchecked array indexing.
- Avoid `unwrap()` in critical execution paths.
- Avoid `expect()` in state-mutating logic.

All logic MUST be implemented using safe Rust constructs only.

If `unsafe` is ever introduced in future versions,
its necessity MUST be formally justified and isolated.

---

## 2.2 Integer Arithmetic Safety

The implementation MUST NOT use:

- wrapping_add
- wrapping_sub
- wrapping_mul
- saturating_add
- saturating_sub
- saturating_mul
- unchecked arithmetic
- implicit overflow behavior

All arithmetic MUST use:

- checked_add
- checked_sub
- checked_mul
- checked_div

Overflow or underflow MUST return a defined error
and abort execution atomically.

---

## 2.3 Explicit Type Casting Rules

Conversions between signed and unsigned integers MUST:

- Validate non-negativity before casting.
- Avoid implicit `as` casting without bounds verification.
- Prevent negative timestamps from propagating into unsigned arithmetic.

All conversions MUST be explicit and guarded.

---

## 2.4 Layout Stability

All account structs MUST:

- Use fixed field ordering.
- Avoid dynamic resizing.
- Avoid conditional struct layouts.
- Avoid feature-gated layout changes.

Binary layout MUST remain stable across builds.

---

## 2.5 Deterministic Compilation Constraints

The build configuration MUST:

- Disable floating-point optimizations.
- Avoid architecture-specific intrinsics.
- Avoid target-dependent integer width assumptions.
- Produce identical behavior across supported validator environments.

No compilation flag may alter arithmetic semantics.

---

# 3. Memory Safety and Account Layout Integrity

This section defines static guarantees that the program is memory-safe and that
on-chain account layouts remain stable and correctly validated.

---

## 3.1 No Dynamic Memory Growth in Core Logic

Instruction handlers MUST NOT:

- allocate large dynamic buffers,
- build unbounded vectors from on-chain inputs,
- perform unbounded string operations,
- rely on heap growth for correctness.

Any allocations required by framework plumbing MUST remain bounded and non-adversarial.

---

## 3.2 Fixed Account Size Requirements

The implementation MUST enforce that:

- Issuance State account size is fixed at deployment and never changes.
- Per-User State account size is fixed and never changes.
- Escrow token accounts follow SPL Token standard layout.

Account structures MUST NOT:

- add, remove, or reorder fields in v1,
- introduce optional fields that change layout,
- depend on conditional compilation for layout.

---

## 3.3 Anchor Discriminator and Padding Rules

If Anchor is used, the implementation MUST account for:

- 8-byte discriminator,
- padding and alignment requirements,
- stable serialization order.

Any account size constant MUST include all discriminator and padding bytes.

---

## 3.4 Deterministic PDA Derivation Safety

All PDAs MUST be derived deterministically using the declared seed model.

Instruction handlers MUST:

- recompute PDA inside the instruction,
- verify provided PDA matches derived PDA,
- verify escrow authorities are the issuance PDA.

Any mismatch MUST abort execution with an explicit error.

---

## 3.5 Account Ownership and Program ID Validation

Static validation requirements:

- All token accounts MUST be owned by the SPL Token Program.
- token_program account MUST match canonical SPL Token Program ID.
- Issuance State and User State accounts MUST be owned by the issuance program.

Any ownership mismatch MUST be treated as invalid input and MUST fail.

---

## 3.6 No Implicit Trust of Provided Accounts

All instruction handlers MUST validate:

- mint consistency for all token accounts,
- authority consistency for all escrow accounts,
- that accounts correspond to the expected issuance instance.

No handler may assume account correctness based on UI behavior.

---

# 4. Determinism and IEEE-754 Strictness

This section formally verifies that the implementation is strictly deterministic
and contains no floating-point or architecture-dependent arithmetic behavior.

---

## 4.1 Absolute Prohibition of Floating-Point Arithmetic

The implementation MUST NOT use:

- f32
- f64
- floating-point literals
- floating-point division
- floating-point multiplication
- floating-point rounding
- floating-point formatting in core logic

All economic and accounting logic MUST use fixed-width unsigned integers.

Floating-point instructions MUST NOT appear in compiled program bytecode.

---

## 4.2 IEEE-754 Strictness Requirement

Although Rust supports IEEE-754 compliant floating-point arithmetic,
the issuance contract MUST avoid floating-point entirely.

Therefore:

- No reliance on IEEE-754 rounding modes.
- No dependence on hardware FPU behavior.
- No architecture-specific floating-point optimization.

Arithmetic behavior MUST be invariant across validator hardware.

---

## 4.3 Integer Determinism

All arithmetic MUST use:

- u128 for value and weight arithmetic,
- u64 for day indices,
- i64 for timestamps.

Division MUST use deterministic floor semantics.

The following MUST be guaranteed:

- Identical inputs produce identical outputs.
- No rounding variability exists.
- No hidden precision loss occurs.

---

## 4.4 Deterministic Time Model

Time calculations MUST use:

Clock::get()?.unix_timestamp

The implementation MUST NOT use:

- slot numbers,
- block height,
- local system time,
- off-chain clocks,
- randomness seeds.

Day index calculation MUST depend exclusively on:

- start_ts
- maturity_ts
- accounting_period
- block_timestamp

---

## 4.5 Transaction Ordering Neutrality

Within a single accounting day:

- raw_day_index MUST remain constant.
- current_day_index MUST remain constant.
- days_elapsed MUST equal zero.
- No additional weight accumulation occurs.

Therefore:

- Transaction ordering inside a single day MUST NOT affect reward share.
- Micro-deposit strategies MUST NOT influence proportional distribution.

Deterministic neutrality MUST be verifiable through static inspection of accumulator logic.

---

## 4.6 Architecture Independence

The implementation MUST:

- Avoid architecture-specific integer widths.
- Avoid platform-dependent casting behavior.
- Avoid endianness-sensitive manual serialization.
- Avoid unsafe memory reinterpretation.

Behavior MUST remain identical across:

- Different validator machines,
- Different OS environments,
- Different CPU architectures supported by Solana.

---

## 4.7 Compiler Optimization Safety

The build configuration MUST ensure:

- No overflow optimizations bypass checked arithmetic.
- No dead-code elimination alters invariant checks.
- No conditional compilation changes arithmetic logic.

Compilation flags MUST NOT change economic semantics.

---

## 4.8 Deterministic Conclusion

Under static inspection, the issuance contract v1.1:

- Contains no floating-point arithmetic.
- Uses strictly checked integer operations.
- Relies only on deterministic on-chain inputs.
- Is independent of hardware-level arithmetic variance.

Determinism is structural, not assumed.

---

# 5. Invariant Preservation Under Static Inspection

This section verifies that all critical invariants defined in Specification v1.1
are structurally enforced by the implementation and cannot be violated through
any valid instruction path.

Static analysis confirms that invariants are:

- encoded in state structure,
- protected by validation checks,
- bounded by arithmetic constraints,
- enforced before and after state mutation.

---

## 5.1 total_locked Consistency

Invariant:

total_locked == sum(user.locked_amount)

Static Enforcement:

- deposit() increases both user.locked_amount and total_locked in the same instruction.
- withdraw_deposit() decreases total_locked before transfer and sets user.locked_amount = 0.
- No other instruction mutates these fields.
- All arithmetic uses checked_sub and checked_add.
- Failure atomicity ensures no partial update persists.

Static Conclusion:

No execution path permits divergence between total_locked and aggregate user balances.

---

## 5.2 Monotonic total_weight_accum

Invariant:

total_weight_accum is monotonically non-decreasing.

Static Enforcement:

- total_weight_accum only increases inside global accumulator update.
- No instruction decreases total_weight_accum.
- Accumulation is bounded by final_day_index.
- Overflow is prevented via checked_mul and checked_add.

Static Conclusion:

Monotonicity is structurally enforced.

---

## 5.3 last_day_index Bound

Invariant:

last_day_index ≤ final_day_index

Static Enforcement:

- current_day_index = min(raw_day_index, final_day_index)
- last_day_index is set only to current_day_index.
- final_day_index is immutable.

Static Conclusion:

last_day_index can never exceed final_day_index.

---

## 5.4 No Accumulation Beyond Maturity

Invariant:

Weight accumulation MUST NOT occur beyond final_day_index.

Static Enforcement:

- All accumulator logic uses bounded current_day_index.
- All state-changing instructions invoke accumulator update.
- raw_day_index is never used directly for accumulation.

Static Conclusion:

Post-maturity accumulation is structurally impossible.

---

## 5.5 Bounded Reward Distribution

Invariant:

Sum of claimed rewards ≤ reserve_total

Static Enforcement:

- reward calculation uses floor division.
- Reward Escrow initial balance == reserve_total.
- All reward transfers originate from escrow.
- No minting mechanism exists.
- sweep() transfers only remaining balance.

Static Conclusion:

Over-distribution is mathematically impossible.

---

## 5.6 Single-Execution Flags

Invariant:

- reward_claimed irreversible
- sweep_executed irreversible
- reclaim_executed irreversible

Static Enforcement:

- Flags are checked before execution.
- Flags are set before outbound CPI transfer.
- No instruction resets flags.

Static Conclusion:

Settlement operations are irreversible.

---

## 5.7 Escrow Isolation

Invariant:

Deposit and reward escrows are fully isolated.

Static Enforcement:

- Separate token accounts.
- Separate mint validation.
- PDA authority enforced.
- No shared account references.

Static Conclusion:

Funds cannot be commingled through defined instructions.

---

## 5.8 Division Safety

Invariant:

Division by zero MUST never occur.

Static Enforcement:

- claim_reward() checks total_weight_accum > 0.
- checked_div used for reward computation.

Static Conclusion:

Division-by-zero is structurally prevented.

---

## 5.9 Deterministic Accumulator State

Invariant:

Weight accumulation depends only on:

- locked_amount
- days_elapsed

Static Enforcement:

- No alternative weight logic.
- No multipliers.
- No nonlinear coefficients.
- No hidden offsets.

Static Conclusion:

Reward share is algebraically reproducible.

---

## 5.10 Structural Invariant Conclusion

Static inspection confirms:

- All Specification invariants have direct enforcement paths.
- No invariant relies on UI or off-chain behavior.
- No invariant depends on runtime assumptions beyond Solana correctness.
- No instruction path bypasses invariant validation.

Invariant preservation is encoded in program structure.

---

# 6. Static Analysis Tooling and Verification Methodology

This section defines the concrete tooling, compiler configuration,
and static verification procedures required to certify
Lockrion Issuance Contract v1.1 as structurally safe and deterministic.

Static analysis MUST be reproducible and documented.

---

## 6.1 Compiler Configuration Requirements

The build process MUST:

- Use a fixed Rust toolchain version.
- Use a fixed Anchor version (if Anchor is used).
- Lock dependency versions via Cargo.lock.
- Disable debug-only arithmetic behavior differences.
- Avoid target-dependent feature flags.

Build reproducibility MUST be verifiable.

---

## 6.2 Clippy Static Lint Enforcement

The implementation MUST pass:

cargo clippy --all-targets -- -D warnings

The following lints MUST NOT be allowed:

- arithmetic_side_effects
- unchecked_cast
- float_arithmetic
- unwrap_used
- expect_used
- panic
- manual_assert
- use_self

All warnings MUST be resolved explicitly.

---

## 6.3 Unsafe Code Audit

The codebase MUST be inspected for:

- `unsafe` blocks
- `extern "C"` usage
- raw pointer usage
- manual memory operations

Requirement:

grep -R "unsafe" src/

Result MUST be empty.

---

## 6.4 Floating-Point Audit

The codebase MUST be inspected for:

- f32
- f64
- floating-point literals
- float formatting in core logic

Verification command example:

grep -R "f32\|f64" src/

Result MUST be empty.

Compiled artifact MUST NOT contain floating-point instructions.

---

## 6.5 Integer Overflow Audit

All arithmetic operations MUST be reviewed to ensure:

- checked_add is used instead of +
- checked_mul instead of *
- checked_sub instead of -
- checked_div instead of /

Manual code review MUST confirm absence of unchecked arithmetic.

---

## 6.6 PDA Validation Audit

Static verification MUST confirm that:

- PDA seeds are recomputed in every instruction.
- PDA key equality is checked against provided accounts.
- Escrow authorities are validated against derived PDA.
- No CPI transfer uses unchecked authority.

Seed derivation MUST be consistent across all instructions.

---

## 6.7 Account Validation Audit

Each instruction MUST validate:

- Issuance State account ownership.
- User State account derivation.
- Token program ID.
- Mint consistency.
- Escrow authority.

No instruction may assume account correctness.

---

## 6.8 Deterministic Build Verification

Two independent builds MUST produce:

- Identical BPF bytecode hashes.
- Identical program behavior under identical inputs.

Build artifacts MUST be hash-verified.

---

## 6.9 Static Analysis Report Output

The static analysis phase MUST produce:

- Toolchain version
- Anchor version
- Cargo.lock hash
- Clippy output
- Unsafe scan result
- Float scan result
- Arithmetic audit confirmation
- PDA validation confirmation
- Account validation confirmation

The report MUST state:

Status: PASS / FAIL

No partial pass is permitted.

---

## 6.10 Static Certification Statement

If all static checks pass:

Lockrion Issuance Contract v1.1 is certified as:

- Free of Undefined Behavior
- Memory-safe under Rust constraints
- Deterministic
- Integer-arithmetic strict
- Architecture-independent
- Structurally invariant-preserving

Static certification does not replace runtime testing,
but confirms structural safety guarantees.

---

# 7. Static Analysis Conclusion and Certification Boundary

This section defines the formal boundary of static certification for
Lockrion Issuance Contract v1.1.

Static analysis verifies structural correctness.
It does not replace dynamic testing or economic validation.

---

## 7.1 Verified Properties

Upon successful completion of Sections 1–6, the following properties are certified:

- No Undefined Behavior (UB).
- No unsafe memory operations.
- No floating-point arithmetic.
- Strict checked integer arithmetic.
- Deterministic time handling.
- Deterministic accumulator logic.
- Explicit invariant enforcement.
- Stable account layout.
- Deterministic PDA derivation.
- No hidden execution paths.
- No architecture-dependent logic.

These guarantees are compile-time and structural.

---

## 7.2 Non-Verified Properties

Static analysis does NOT verify:

- Economic attractiveness of issuance parameters.
- Market behavior of lock_mint.
- Network-level liveness or congestion behavior.
- UI correctness.
- Wallet integration behavior.
- Off-chain governance correctness.

Static analysis verifies structural correctness only.

---

## 7.3 Boundary of Trust

The structural guarantees depend on:

- Rust compiler correctness.
- Solana runtime correctness.
- SPL Token Program correctness.

Static analysis assumes:

- No malicious validator modifications.
- Correct BPF execution semantics.

No additional trust assumptions are introduced.

---

## 7.4 Deterministic Certification Scope

Certification applies strictly to:

Lockrion Issuance Contract v1.1  
compiled with the documented toolchain and configuration.

Any modification to:

- Source code,
- Dependency versions,
- Compiler flags,
- Account layout,
- Arithmetic logic,

INVALIDATES static certification.

A new static analysis document version MUST be issued
for any modification.

---

## 7.5 Structural Finality

If all static checks pass:

The implementation is certified as structurally deterministic,
memory-safe, arithmetic-strict, and invariant-preserving.

This certification is binary:

PASS → All checks satisfied  
FAIL → At least one violation detected  

No partial certification is permitted.

---

## 7.6 Static Analysis Completion Statement

This document defines the complete static analysis framework
for Lockrion Issuance Contract v1.1.

When all verification steps are executed and documented,
the static safety boundary of the contract is formally established.
