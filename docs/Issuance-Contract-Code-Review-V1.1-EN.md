# Lockrion Issuance Contract — Code Review Document v1.1

Status: Draft  
Standard: Lockrion Issuance Contract v1  
Scope: Spec/Design conformance, algorithm correctness, defensive order, security  

---

## 1. Purpose

This document defines the code review checklist and acceptance criteria
for Lockrion Issuance Contract v1.1.

The objective is to verify that the implementation:

- conforms 1:1 to Specification v1.1,
- conforms 1:1 to Design v1.1,
- preserves canonical execution order,
- enforces all invariants structurally,
- validates accounts and PDAs deterministically,
- uses checked arithmetic everywhere,
- performs outbound CPI transfers only after state mutation,
- contains no hidden logic, admin overrides, or dynamic behavior.

This document is normative for review outcomes.

---

## 2. Review Inputs and Scope

The review MUST cover:

- Instruction handlers:
  - fund_reserve()
  - deposit(amount)
  - claim_reward()
  - withdraw_deposit()
  - sweep()
  - zero_participation_reclaim()

- Core algorithms:
  - global accumulator update
  - per-user accumulator update
  - reward formula implementation

- Security surfaces:
  - PDA derivation and validation
  - mint validation
  - authority validation
  - token program ID validation
  - signer verification

- State and invariants:
  - total_locked consistency
  - monotonic total_weight_accum
  - settlement flag irreversibility

Out of scope:

- UI/frontend
- off-chain indexing
- economic marketing claims

---

## 3. Mandatory Review Rules (Hard Fail)

If any item below is violated, the review result MUST be FAIL.

### 3.1 No Unsafe Code

- No `unsafe` blocks allowed.
- No raw pointer logic.
- No transmute.

### 3.2 No Floating-Point Arithmetic

- No f32/f64 types.
- No float literals.
- No float math.

### 3.3 Checked Arithmetic Only

All arithmetic in economic logic MUST use checked_* operations.

Any use of:

- +
- -
- *
- /

in value/weight arithmetic without checked wrappers is a FAIL.

### 3.4 Canonical Execution Order for CPI Outbound Transfers

For all outbound transfers:

- state mutation MUST occur before CPI transfer.

This applies to:

- claim_reward()
- withdraw_deposit()
- sweep()
- zero_participation_reclaim()

### 3.5 Accumulator Invocation Before State Changes

All state-changing instructions MUST invoke:

1. global accumulator update
2. per-user accumulator update (if applicable)

before mutating:

- total_locked
- locked_amount
- reward_claimed
- sweep_executed
- reclaim_executed

### 3.6 PDA and Account Validation is Mandatory

Every instruction MUST verify:

- Issuance State is correct instance (seed model)
- User State is correct (issuance, user) PDA
- Escrow authority == issuance PDA
- Token program ID == canonical SPL token program
- Mint consistency for every transfer

Any missing validation is FAIL.

---

## 4. Instruction-by-Instruction Review Checklist

### 4.1 fund_reserve()

Reviewer MUST verify:

- Issuer signer matches issuer_address parameter.
- Called only before start_ts.
- Exact amount == reserve_total.
- reserve_funded set true only after successful transfer.
- Reward escrow mint == reward_mint.
- Reward escrow authority == issuance PDA.

Result: PASS / FAIL

---

### 4.2 deposit(amount)

Reviewer MUST verify:

- reserve_funded required.
- start_ts <= t < maturity_ts enforced.
- amount > 0 enforced.
- global accumulator update called first.
- per-user accumulator update called second.
- total_locked and user.locked_amount updated with checked_add.
- state mutation precedes CPI transfer.
- lock_mint validation for both token accounts.
- escrow authority validation.

Result: PASS / FAIL

---

### 4.3 claim_reward()

Reviewer MUST verify:

- maturity_ts <= t < maturity_ts + claim_window enforced.
- reward_claimed gating.
- total_weight_accum > 0 enforced.
- global accumulator finalization before reward calculation.
- per-user accumulator update before reward calculation.
- reward numerator uses checked_mul.
- division uses checked_div.
- reward_claimed set true before CPI transfer.
- reward_mint validation for token accounts.
- escrow authority validation.

Result: PASS / FAIL

---

### 4.4 withdraw_deposit()

Reviewer MUST verify:

- t >= maturity_ts enforced.
- user.locked_amount > 0 enforced.
- global accumulator finalization executed before withdrawal.
- per-user accumulator update executed before clearing locked_amount.
- total_locked decreased via checked_sub.
- locked_amount set to 0 before transfer.
- state mutation precedes CPI transfer.
- lock_mint validation for token accounts.
- escrow authority validation.

Result: PASS / FAIL

---

### 4.5 sweep()

Reviewer MUST verify:

- t >= maturity_ts + claim_window enforced.
- total_weight_accum > 0 enforced.
- escrow balance > 0 enforced.
- sweep_executed gating.
- sweep_executed set true before transfer.
- transfer amount equals escrow balance read on-chain.
- platform_treasury matches immutable parameter.
- reward_mint validation.
- escrow authority validation.

Result: PASS / FAIL

---

### 4.6 zero_participation_reclaim()

Reviewer MUST verify:

- t >= maturity_ts enforced.
- total_weight_accum == 0 enforced.
- escrow balance > 0 enforced.
- issuer signer required and validated.
- reclaim_executed gating.
- reclaim_executed set true before transfer.
- transfer amount equals escrow balance read on-chain.
- reward_mint validation.
- escrow authority validation.

Result: PASS / FAIL

---

## 5. Algorithm Review Checklist

### 5.1 Global Accumulator Update

Reviewer MUST verify:

- Uses bounded current_day_index = min(raw_day_index, final_day_index).
- days_elapsed computed safely.
- total_weight_accum increases by total_locked * days_elapsed using checked_mul and checked_add.
- last_day_index updated to current_day_index.
- No accumulation before start_ts.
- No accumulation beyond maturity_ts due to bounding.

Result: PASS / FAIL

---

### 5.2 Per-User Accumulator Update

Reviewer MUST verify:

- Uses the same current_day_index produced by global update.
- days_elapsed_user computed safely.
- user_weight_accum increases by locked_amount * days_elapsed_user with checked arithmetic.
- user_last_day_index updated deterministically.
- Update occurs before any change to locked_amount.

Result: PASS / FAIL

---

### 5.3 Reward Computation

Reviewer MUST verify:

- reward = floor(reserve_total * user_weight_accum / total_weight_accum)
- checked_mul used for numerator.
- checked_div used for denominator.
- total_weight_accum > 0 required.
- reward amount cannot exceed reserve_total under escrow constraint.

Result: PASS / FAIL

---

## 6. Review Output Requirements

The review MUST produce:

- A checklist with PASS/FAIL per section.
- A list of identified issues with severity:
  - Critical (blocks release)
  - Major
  - Minor
- For each issue:
  - file path / module
  - function name
  - violation description
  - required fix

The final review result MUST be binary:

PASS → ready for release gating  
FAIL → not ready  

No partial certification.

---

## 7. Code Review Completion Statement

If all sections are PASS:

Lockrion Issuance Contract v1.1 is Code-Review Certified.

This confirms readiness to proceed to:

- Auto-Test Suite
- Final release packaging
- Deployment evidence capture
