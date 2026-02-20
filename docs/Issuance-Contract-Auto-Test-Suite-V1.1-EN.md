# Lockrion Issuance Contract — Auto-Test Suite Document v1.1

Status: Draft  
Standard: Lockrion Issuance Contract v1  
Scope: Logic, invariants, determinism, negative paths, security substitution tests  

---

## 1. Purpose

This document defines the mandatory automated test suite for
Lockrion Issuance Contract v1.1.

The objective is to verify through reproducible automated tests that:

- All instruction logic conforms to Specification v1.1 and Design v1.1.
- All invariants hold under all valid execution paths.
- All invalid paths fail deterministically with correct errors.
- Determinism holds under replay with identical inputs.
- Settlement operations are irreversible and single-execution.
- Account substitution and mint mismatch attacks fail.

This document specifies WHAT must be tested.
It does not mandate a specific framework implementation,
but assumes a Solana-compatible automated environment.

Test results MUST be reproducible and attached as evidence artifacts.

---

## 2. Test Environment and Execution Model

This section defines the required test environment,
execution boundaries, and reproducibility constraints
for the Auto-Test Suite.

All tests MUST be executable in an automated environment.

---

### 2.1 Required Environment

Tests MUST run against:

- Solana local validator (preferred)
or
- Deterministic devnet test harness

The environment MUST:

- Provide controllable block_timestamp progression.
- Allow deterministic account creation.
- Allow explicit token mint setup.
- Allow full PDA derivation testing.

All tests MUST execute in isolation.

---

### 2.2 Deterministic Time Control

Tests MUST simulate timestamp progression explicitly.

Time-sensitive tests MUST:

- Control start_ts and maturity_ts values.
- Advance block_timestamp deterministically.
- Validate behavior before start_ts.
- Validate behavior during participation window.
- Validate behavior at maturity boundary.
- Validate behavior after claim_window.

Timestamp progression MUST be reproducible.

---

### 2.3 Account Setup Model

Each test case MUST explicitly create:

- Issuance State account
- Deposit Escrow token account
- Reward Escrow token account
- User State account (when required)
- Mint accounts for lock_mint and reward_mint
- Participant token accounts

No test may depend on shared global state from previous tests.

Each test MUST:

- Initialize fresh state.
- Execute scenario.
- Assert postconditions.
- Tear down or reset environment.

---

### 2.4 Token Mint Configuration

Tests MUST include:

- Correct lock_mint and reward_mint setup.
- Validation of mint mismatches.
- Tests where wrong mint is passed.
- Tests where escrow mint is incorrect.

Mint validation MUST be tested explicitly.

---

### 2.5 Error Verification Model

For every negative-path test:

- The expected error code MUST be asserted.
- No generic failure acceptance is allowed.
- Error MUST match documented error variant.

Tests MUST fail if:

- An unexpected error is returned.
- No error is returned when failure is expected.
- State mutation occurs when failure is expected.

---

### 2.6 Atomicity Verification

Tests MUST validate that:

- If CPI transfer fails, state remains unchanged.
- Partial state updates are not persisted.
- Flags are not set if transfer fails.

This may require simulation of CPI failure conditions.

---

### 2.7 Replay Determinism Model

A subset of tests MUST:

1. Execute full issuance lifecycle.
2. Record final balances and state.
3. Reset environment.
4. Replay identical sequence.
5. Compare final state and balances.

All final values MUST match exactly.

---

### 2.8 Isolation Requirement

No test may:

- Rely on global test ordering.
- Depend on previous test state.
- Share mutable accounts across tests.

The test suite MUST support parallel execution.

---

### 2.9 Coverage Requirement

The suite MUST include:

- Positive path tests (valid execution).
- Boundary tests (start/maturity/claim_window edges).
- Negative tests (invalid calls).
- Arithmetic boundary tests.
- Account substitution tests.
- Settlement irreversibility tests.
- Zero participation tests.

Test coverage MUST span all instructions and invariants.

---

### 2.10 Test Execution Output

Each test MUST output:

- Test name
- Scenario description
- Expected result
- Actual result
- PASS / FAIL

Suite completion requires:

- 100% PASS
- No ignored tests
- No skipped negative-path checks

---

## 3. Unit Tests — Core Arithmetic and Accumulator Logic

This section defines mandatory unit-level tests
for arithmetic correctness and accumulator behavior.

These tests validate:

- Discrete time progression
- Correct day index calculation
- Global accumulator correctness
- Per-user accumulator correctness
- Boundary conditions
- Overflow protection

These tests do NOT require CPI execution.
They validate pure logic components.

---

### 3.1 Raw Day Index Calculation Tests

Test ID: UT-TIME-01  
Scenario: block_timestamp < start_ts  
Expected:
- raw_day_index == 0
- No accumulation occurs

Test ID: UT-TIME-02  
Scenario: block_timestamp == start_ts  
Expected:
- raw_day_index == 0

Test ID: UT-TIME-03  
Scenario: block_timestamp = start_ts + 86400  
Expected:
- raw_day_index == 1

Test ID: UT-TIME-04  
Scenario: block_timestamp = start_ts + (n * 86400)  
Expected:
- raw_day_index == n

Test ID: UT-TIME-05  
Scenario: block_timestamp beyond maturity_ts  
Expected:
- current_day_index == final_day_index

All tests MUST verify floor division semantics.

---

### 3.2 Global Accumulator Update Tests

Test ID: UT-GACC-01  
Initial:
- total_locked = 100
- last_day_index = 0
- current_day_index = 3

Expected:
- total_weight_accum increases by 300
- last_day_index == 3

Test ID: UT-GACC-02  
Scenario: days_elapsed == 0  
Expected:
- total_weight_accum unchanged
- last_day_index unchanged

Test ID: UT-GACC-03  
Scenario: current_day_index > final_day_index  
Expected:
- bounded to final_day_index
- no overflow beyond final_day_index

Test ID: UT-GACC-04  
Overflow Protection Test  
Use large total_locked and days_elapsed  
Expected:
- checked_mul prevents overflow
- ArithmeticOverflow error returned

---

### 3.3 Per-User Accumulator Update Tests

Test ID: UT-UACC-01  
Initial:
- locked_amount = 50
- user_last_day_index = 1
- current_day_index = 4

Expected:
- user_weight_accum increases by 150
- user_last_day_index == 4

Test ID: UT-UACC-02  
Scenario: days_elapsed_user == 0  
Expected:
- user_weight_accum unchanged

Test ID: UT-UACC-03  
Scenario: user_last_day_index already equals final_day_index  
Expected:
- no further accumulation

Test ID: UT-UACC-04  
Overflow Protection  
Large locked_amount and large days_elapsed_user  
Expected:
- checked_mul prevents overflow
- ArithmeticOverflow error returned

---

### 3.4 Bounded Accumulation Tests

Test ID: UT-BOUND-01  
Scenario:
- block_timestamp > maturity_ts
Expected:
- accumulation bounded to final_day_index
- no weight added beyond final_day_index

Test ID: UT-BOUND-02  
Multiple calls after maturity  
Expected:
- total_weight_accum remains constant

---

### 3.5 Same-Day Determinism Tests

Test ID: UT-SAME-01  
Scenario:
- Two deposits executed within same day
Expected:
- days_elapsed == 0 on second call
- total_weight_accum unchanged

Test ID: UT-SAME-02  
Scenario:
- Multiple accumulator updates within same day
Expected:
- No weight change

---

### 3.6 Reward Formula Unit Tests

Test ID: UT-REWARD-01  
Simple proportional case:
- reserve_total = 1000
- total_weight_accum = 100
- user_weight_accum = 25
Expected:
- reward == 250

Test ID: UT-REWARD-02  
Rounding case:
- reserve_total = 1000
- total_weight_accum = 3
- user_weight_accum = 1
Expected:
- reward == floor(1000 / 3)

Test ID: UT-REWARD-03  
Division by zero case:
- total_weight_accum == 0
Expected:
- DivisionByZero or NoParticipation error

Test ID: UT-REWARD-04  
Overflow case:
- reserve_total and user_weight_accum near u128 max
Expected:
- checked_mul prevents overflow
- ArithmeticOverflow error returned

---

### 3.7 Arithmetic Safety Tests

Test ID: UT-ARITH-01  
Addition overflow in total_locked  
Expected:
- ArithmeticOverflow error

Test ID: UT-ARITH-02  
Subtraction underflow in withdraw  
Expected:
- ArithmeticUnderflow error

Test ID: UT-ARITH-03  
Casting negative timestamp to unsigned  
Expected:
- Proper validation prevents invalid cast

---

### 3.8 Unit Test Completion Criteria

Unit tests MUST:

- Cover all arithmetic paths
- Cover all accumulator branches
- Cover all boundary cases
- Validate overflow/underflow handling
- Pass deterministically across repeated runs

Unit Test Status: PASS / FAIL

---

## 4. Instruction-Level Functional Tests

This section defines full instruction execution tests.
These tests validate end-to-end instruction behavior,
including state mutation and CPI token transfers.

All tests MUST execute against a local validator.

---

# 4.1 fund_reserve() Functional Tests

### FT-FUND-01 — Successful Funding

Initial State:
- reserve_funded == false
- block_timestamp < start_ts

Action:
- Call fund_reserve() with exact reserve_total

Expected:
- reserve_funded == true
- Reward Escrow balance == reserve_total
- No other state modified

---

### FT-FUND-02 — Double Funding Attempt

Initial:
- reserve_funded == true

Action:
- Call fund_reserve() again

Expected:
- Error::ReserveAlreadyFunded
- No state mutation

---

### FT-FUND-03 — Incorrect Amount

Action:
- Call fund_reserve() with amount != reserve_total

Expected:
- Error::InvalidFundingAmount
- reserve_funded remains false

---

### FT-FUND-04 — Non-Issuer Attempt

Action:
- Non-issuer signer attempts funding

Expected:
- Error::UnauthorizedCaller
- No state mutation

---

# 4.2 deposit(amount) Functional Tests

### FT-DEP-01 — Valid Deposit

Initial:
- reserve_funded == true
- block_timestamp within participation window

Action:
- deposit(100)

Expected:
- user.locked_amount += 100
- total_locked += 100
- Deposit Escrow balance += 100

---

### FT-DEP-02 — Deposit Before Funding

Initial:
- reserve_funded == false

Action:
- deposit(100)

Expected:
- Error::ReserveNotFunded

---

### FT-DEP-03 — Deposit After Maturity

Initial:
- block_timestamp >= maturity_ts

Action:
- deposit(100)

Expected:
- Error::DepositWindowClosed

---

### FT-DEP-04 — Zero Amount Deposit

Action:
- deposit(0)

Expected:
- Error::InvalidAmount

---

### FT-DEP-05 — Mint Mismatch

Action:
- deposit with incorrect mint token account

Expected:
- Error::InvalidMint

---

# 4.3 claim_reward() Functional Tests

### FT-CLAIM-01 — Valid Claim

Initial:
- maturity reached
- total_weight_accum > 0
- reward_claimed == false

Action:
- claim_reward()

Expected:
- reward transferred
- reward_claimed == true
- Reward Escrow balance decreases

---

### FT-CLAIM-02 — Double Claim

Action:
- claim_reward() again

Expected:
- Error::AlreadyClaimed

---

### FT-CLAIM-03 — Claim Before Maturity

Initial:
- block_timestamp < maturity_ts

Expected:
- Error::ClaimWindowNotStarted

---

### FT-CLAIM-04 — Claim After Claim Window

Initial:
- block_timestamp >= maturity_ts + claim_window

Expected:
- Error::ClaimWindowClosed

---

### FT-CLAIM-05 — Zero Participation

Initial:
- total_weight_accum == 0

Expected:
- Error::NoParticipation

---

# 4.4 withdraw_deposit() Functional Tests

### FT-WITH-01 — Valid Withdrawal

Initial:
- block_timestamp >= maturity_ts
- user.locked_amount > 0

Action:
- withdraw_deposit()

Expected:
- locked_amount == 0
- total_locked reduced
- Deposit Escrow balance reduced

---

### FT-WITH-02 — Withdrawal Before Maturity

Initial:
- block_timestamp < maturity_ts

Expected:
- Error::DepositWindowNotClosed

---

### FT-WITH-03 — Double Withdrawal

Initial:
- locked_amount == 0

Expected:
- Error::InvalidAmount or equivalent

---

# 4.5 sweep() Functional Tests

### FT-SWEEP-01 — Valid Sweep

Initial:
- block_timestamp >= maturity_ts + claim_window
- total_weight_accum > 0
- sweep_executed == false

Action:
- sweep()

Expected:
- sweep_executed == true
- Reward Escrow balance == 0

---

### FT-SWEEP-02 — Double Sweep

Action:
- sweep() again

Expected:
- Error::SweepAlreadyExecuted

---

### FT-SWEEP-03 — Sweep with Zero Participation

Initial:
- total_weight_accum == 0

Expected:
- Error::NoParticipation

---

# 4.6 zero_participation_reclaim() Functional Tests

### FT-RECLAIM-01 — Valid Reclaim

Initial:
- total_weight_accum == 0
- block_timestamp >= maturity_ts

Action:
- zero_participation_reclaim()

Expected:
- reclaim_executed == true
- Reward Escrow balance == 0

---

### FT-RECLAIM-02 — Reclaim by Non-Issuer

Expected:
- Error::UnauthorizedCaller

---

### FT-RECLAIM-03 — Double Reclaim

Expected:
- Error::ReclaimAlreadyExecuted

---

## 4.7 Functional Test Completion Criteria

Functional test suite MUST:

- Cover all instruction success paths
- Cover all negative paths
- Verify all settlement gating flags
- Verify correct CPI transfers
- Verify state mutation order

Functional Test Status: PASS / FAIL

---

## 5. Invariant and Stress Tests

This section defines stress-level and invariant-preservation tests.
These tests validate that structural invariants hold
under extreme or adversarial conditions.

The objective is to ensure:

- No invariant drift under load
- No overflow under large values
- No race-condition side effects
- No weight distortion under complex flows

---

# 5.1 High-Value Arithmetic Stress Tests

### ST-ARITH-01 — Large Reserve and Long Duration

Initial:
- reserve_total near u128 upper bound (within safe test range)
- long duration (large final_day_index)
- multiple users with large locked_amount

Action:
- Execute full lifecycle: deposit → maturity → claim

Expected:
- No overflow
- Correct proportional reward
- total_weight_accum bounded
- No arithmetic panic

---

### ST-ARITH-02 — Large Locked Amount

Initial:
- locked_amount near u128 safe upper bound

Action:
- Accumulate across multiple days

Expected:
- checked_mul prevents overflow
- ArithmeticOverflow error if bounds exceeded

---

# 5.2 Multi-User Distribution Consistency

### ST-DIST-01 — Two Equal Users

Initial:
- Two users deposit equal amounts on same day

Expected:
- Equal weight accumulation
- Equal reward

---

### ST-DIST-02 — Early vs Late Deposit

Initial:
- User A deposits on day 0
- User B deposits on day 5

Expected:
- User A receives proportionally higher reward
- Mathematical ratio matches theoretical model

---

### ST-DIST-03 — Staggered Deposits

Initial:
- Multiple users deposit at different day indices

Expected:
- total_weight_accum equals sum of user_weight_accum
- No drift in total distribution

---

# 5.3 Boundary Condition Stress Tests

### ST-BOUND-01 — Exact Maturity Boundary

Initial:
- block_timestamp == maturity_ts

Expected:
- No further accumulation
- claim_reward allowed
- deposit disallowed

---

### ST-BOUND-02 — Exact Claim Window End

Initial:
- block_timestamp == maturity_ts + claim_window

Expected:
- claim_reward disallowed
- sweep allowed

---

### ST-BOUND-03 — Exact Day Rollover

Initial:
- block_timestamp transitions exactly at 86400-second boundary

Expected:
- Single-day increment
- No double increment

---

# 5.4 Repeated Invocation Resistance

### ST-REPEAT-01 — Repeated claim attempts

Expected:
- Only first claim succeeds
- All subsequent claims fail

---

### ST-REPEAT-02 — Repeated sweep attempts

Expected:
- Only first sweep succeeds
- Subsequent fail

---

### ST-REPEAT-03 — Repeated reclaim attempts

Expected:
- Only first reclaim succeeds
- Subsequent fail

---

# 5.5 Account Substitution and Seed Stability Stress Tests

These tests validate that canonical PDA derivation and seed stability
are enforced as part of protocol definition.

---

### ST-ACCT-01 — Substitute Deposit Escrow

Action:
- Pass incorrect deposit escrow account.

Expected:
- Instruction fails with InvalidEscrowAccount.

---

### ST-ACCT-02 — Substitute Reward Escrow

Expected:
- Instruction fails with InvalidEscrowAccount.

---

### ST-ACCT-03 — Substitute User State

Action:
- Pass a UserState account derived from different seeds.

Expected:
- Instruction fails with InvalidPDA.

---

### ST-ACCT-04 — Substitute Platform Treasury

Expected:
- sweep() fails with InvalidPlatformTreasury.

---

### ST-SEED-01 — Issuance Seed Order Mutation

Action:
- Attempt to derive issuance PDA with altered seed order.
- Attempt to execute instruction using mutated PDA.

Expected:
- PDA mismatch detected.
- Instruction fails with InvalidPDA.

Purpose:
- Enforces seed ordering as immutable protocol definition.

---

### ST-SEED-02 — Numeric Encoding Mutation

Action:
- Encode start_ts or reserve_total using non-little-endian format.
- Attempt to derive issuance PDA.

Expected:
- Derived PDA mismatch.
- Instruction fails with InvalidPDA.

Purpose:
- Enforces little-endian encoding rule in PDA derivation.

---

# 5.6 Escrow Contamination Stress Test

### ST-ESCROW-01 — External Token Injection

Action:
- External transfer sends extra tokens to Reward Escrow

Expected:
- reward calculation unchanged
- Only reserve_total considered in proportional logic
- sweep transfers full remaining balance

---

# 5.7 Deterministic Replay Stress Test

### ST-REPLAY-01 — Full Lifecycle Replay

Procedure:
1. Execute complete issuance scenario.
2. Capture final balances and state.
3. Reset environment.
4. Replay identical sequence.
5. Compare final state.

Expected:
- All balances identical
- total_weight_accum identical
- user_weight_accum identical
- Flags identical

---

## 5.8 Stress Test Completion Criteria

Stress tests MUST:

- Validate invariants under extreme values
- Confirm settlement irreversibility
- Confirm arithmetic safety
- Confirm account validation enforcement
- Confirm deterministic replay

Stress Test Status: PASS / FAIL

---

## 6. Final Auto-Test Certification Statement

This section defines the formal certification boundary
for the Auto-Test Suite of Lockrion Issuance Contract v1.1.

The Auto-Test Suite validates:

- Core arithmetic correctness
- Accumulator correctness
- Instruction-level functional correctness
- Invariant preservation
- Settlement irreversibility
- Account substitution resistance
- Escrow isolation
- Deterministic replay behavior
- Boundary and stress resilience

---

### 6.1 Certification Preconditions

Auto-Test certification may be granted only if:

- All Unit Tests PASS
- All Functional Tests PASS
- All Invariant/Stress Tests PASS
- No test is skipped
- No negative-path test is ignored
- No arithmetic overflow is unhandled
- No unexpected state mutation is observed

Partial pass is NOT permitted.

---

### 6.2 Evidence Requirements

Certification requires:

- Full test execution log
- Deterministic replay evidence
- Failure-case logs for negative tests
- Explicit PASS/FAIL summary
- Environment description (validator version, toolchain)

All artifacts MUST be reproducible.

---

### 6.3 Determinism Confirmation

The suite MUST demonstrate that:

- Identical inputs produce identical final state
- No randomness affects execution
- Same-day ordering does not alter reward outcome
- Replay produces byte-for-byte identical results (where applicable)

Any deviation invalidates certification.

---

### 6.4 Irreversibility Confirmation

Tests MUST confirm:

- reward_claimed irreversible
- sweep_executed irreversible
- reclaim_executed irreversible
- locked_amount cannot be restored after withdrawal
- No hidden reset paths exist

---

### 6.5 Escrow Integrity Confirmation

Tests MUST confirm:

- Deposit escrow and reward escrow are isolated
- No cross-mint transfers occur
- Escrow authority always equals issuance PDA
- No unauthorized withdrawal is possible
- Distribution never exceeds reserve_total

---

### 6.6 Certification Result

If all sections PASS:

Lockrion Issuance Contract v1.1 is declared:

Auto-Test Certified.

This confirms:

- Implementation correctness
- Invariant preservation
- Deterministic settlement logic
- Safe integration behavior
- Arithmetic safety
- Security constraint enforcement

---

### 6.7 Release Gate Condition

The contract may proceed to deployment only if:

- Static Analysis = PASS
- Integration = PASS
- Theoretical Validation = PASS
- Compliance Matrix = All PASS
- Code Review = PASS
- Auto-Test Suite = PASS

If any section FAILS:

Release is blocked.

---

### 6.8 Version Finalization Statement

This document completes the verification stack for:

Lockrion Issuance Contract v1.1

Any change to:

- State layout
- Arithmetic logic
- Execution order
- Instruction interface
- PDA derivation
- Settlement logic

Requires:

- Incrementing version
- Re-running full Auto-Test Suite
- Re-issuing certification

v1.1 verification is complete only after all certifications are attached.
