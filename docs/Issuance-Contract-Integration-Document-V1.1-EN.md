# Lockrion Issuance Contract — Integration Document v1.1

Status: Draft  
Standard: Lockrion Issuance Contract v1  
Scope: API integrity, account wiring correctness, input/output purity  

---

## 1. Purpose

This document defines the integration-level verification framework
for Lockrion Issuance Contract v1.1.

The objective is to validate that:

- All instruction APIs are structurally clean.
- Account inputs are minimal and deterministic.
- No hidden side effects exist.
- No implicit dependencies on off-chain behavior exist.
- Account relationships are correctly enforced.
- CPI interactions are properly isolated.

This document verifies integration correctness,
not arithmetic or invariant correctness (covered in Static Analysis).

Integration validation ensures that the contract behaves correctly
when invoked by:

- Wallet clients
- Frontend applications
- SDK integrations
- Automated settlement scripts
- On-chain composability environments

---

## 2. Instruction Interface Model

This section defines the external interface surface of the Issuance Contract v1.1.

The objective is to ensure:

- Minimal and deterministic account inputs
- No redundant account requirements
- No implicit state dependencies
- No cross-instruction hidden coupling
- Clean API boundaries

Each instruction MUST define:

- Required accounts
- Required signers
- Required mutable accounts
- Expected account ownership
- Expected mint constraints
- Explicit input parameters

No instruction may rely on:

- Implicit global state
- Unchecked optional accounts
- Off-chain computed assumptions
- Hidden context from prior transactions

---

### 2.1 fund_reserve() — Interface Contract

Required Accounts:

- Issuance State (mutable)
- Reward Escrow token account (mutable)
- Issuer token account (mutable)
- Issuer signer
- SPL Token Program

Validation Requirements:

- Issuer signer == issuer_address
- Reward Escrow mint == reward_mint
- Issuer token account mint == reward_mint
- Reward Escrow authority == issuance PDA

No additional accounts are permitted.

---

### 2.2 deposit(amount) — Interface Contract

Required Accounts:

- Issuance State (mutable)
- User State (mutable)
- Deposit Escrow token account (mutable)
- Participant token account (mutable)
- Participant signer
- SPL Token Program

Validation Requirements:

- Participant signer matches User State owner
- Deposit Escrow mint == lock_mint
- Participant token account mint == lock_mint
- Deposit Escrow authority == issuance PDA

Input Parameters:

- amount: u64 or u128 (implementation-defined, must match state type)

No external treasury or reward accounts may be passed.

---

### 2.3 claim_reward() — Interface Contract

Required Accounts:

- Issuance State (mutable)
- User State (mutable)
- Reward Escrow token account (mutable)
- Participant reward token account (mutable)
- Participant signer
- SPL Token Program

Validation Requirements:

- Participant signer matches User State owner
- Reward Escrow mint == reward_mint
- Participant reward token account mint == reward_mint
- Reward Escrow authority == issuance PDA

No deposit escrow account required.

---

### 2.4 withdraw_deposit() — Interface Contract

Required Accounts:

- Issuance State (mutable)
- User State (mutable)
- Deposit Escrow token account (mutable)
- Participant token account (mutable)
- Participant signer
- SPL Token Program

Validation Requirements:

- Participant signer matches User State owner
- Deposit Escrow mint == lock_mint
- Participant token account mint == lock_mint
- Deposit Escrow authority == issuance PDA

No reward escrow account required.

---

### 2.5 sweep() — Interface Contract

Required Accounts:

- Issuance State (mutable)
- Reward Escrow token account (mutable)
- platform_treasury token account (mutable)
- SPL Token Program

Validation Requirements:

- platform_treasury matches immutable platform_treasury parameter
- Reward Escrow mint == reward_mint
- Reward Escrow authority == issuance PDA

No external signer required unless platform policy mandates.

---

### 2.6 zero_participation_reclaim() — Interface Contract

Required Accounts:

- Issuance State (mutable)
- Reward Escrow token account (mutable)
- Issuer reward token account (mutable)
- Issuer signer
- SPL Token Program

Validation Requirements:

- Issuer signer == issuer_address
- Reward Escrow mint == reward_mint
- Issuer reward token account mint == reward_mint
- Reward Escrow authority == issuance PDA

No deposit escrow account required.

---

### 2.7 Interface Isolation Guarantee

Each instruction MUST:

- Depend only on explicitly passed accounts.
- Perform full validation of all account relationships.
- Avoid accessing any implicit global state.
- Avoid reading unrelated accounts.

There MUST be no hidden cross-instruction dependencies.

The interface surface MUST remain stable across builds.

---

## 3. Account Wiring and Validation Integrity

This section verifies that account relationships between:

- Issuance State
- User State
- Deposit Escrow
- Reward Escrow
- platform_treasury
- issuer_address

are structurally enforced at integration level.

No instruction may rely on implicit trust of caller-provided accounts.

---

### 3.1 Issuance State Binding

Every instruction MUST verify:

- The Issuance State account matches the expected PDA seeds.
- The Issuance State owner is the program ID.
- The immutable parameters inside Issuance State are used as the single source of truth.

No instruction may accept a generic Issuance State without seed validation.

Cross-issuance invocation MUST be impossible.

---

### 3.2 User State Binding

For all participant instructions:

- User State MUST be derived from:
  (issuance_state_pubkey, participant_pubkey)

The program MUST:

- Recompute PDA seeds inside the handler.
- Compare derived PDA to provided User State account.
- Ensure the participant signer matches the expected user key.

User State accounts must not be interchangeable across issuances.

---

### 3.3 Escrow Account Authority Validation

For both Deposit and Reward escrows:

The program MUST verify:

- Account owner == SPL Token Program.
- Mint matches expected mint.
- Authority == issuance PDA.

Escrow authority MUST NOT be externally controlled.

If authority mismatch is detected, execution MUST abort.

---

### 3.4 platform_treasury Binding

For sweep():

- platform_treasury token account MUST match immutable parameter.
- Mint MUST match reward_mint.
- platform_treasury MUST NOT be caller-specified arbitrarily.

This prevents treasury redirection attacks.

---

### 3.5 issuer_address Binding

For:

- fund_reserve()
- zero_participation_reclaim()

The program MUST verify:

- signer == issuer_address
- issuer_address matches immutable parameter in Issuance State

Caller substitution MUST be impossible.

---

### 3.6 Token Program Binding

All instructions performing CPI transfers MUST:

- Validate token_program.key() equals canonical SPL Token Program ID.

Arbitrary token programs MUST be rejected.

This prevents malicious CPI redirection.

---

### 3.7 Mint Consistency Enforcement

Before every token transfer, the program MUST validate:

- Source mint == expected mint
- Destination mint == expected mint

Cross-mint transfers MUST fail.

Reward escrow may never transfer lock_mint.
Deposit escrow may never transfer reward_mint.

---

### 3.8 Mutability Scope Enforcement

Only accounts that require mutation MUST be marked mutable.

The integration model MUST ensure:

- No unnecessary writable accounts.
- No extraneous signer accounts.
- No implicit mutation of unrelated accounts.

Minimal mutability reduces integration risk.

---

### 3.9 Account Isolation Conclusion

Integration-level validation guarantees:

- No account substitution attacks.
- No escrow redirection.
- No cross-issuance contamination.
- No unauthorized treasury modification.
- No authority bypass.

All account wiring is deterministic and explicit.

---

## 4. Input / Output Purity and Side-Effect Model

This section verifies that each instruction of the Issuance Contract v1.1:

- Produces only explicitly defined state changes.
- Has no hidden side effects.
- Does not mutate unrelated accounts.
- Does not depend on off-chain behavior.
- Has deterministic and bounded outputs.

The integration surface must be clean and composable.

---

### 4.1 Pure Input Model

Each instruction MUST depend exclusively on:

- Explicit instruction parameters
- Provided accounts
- On-chain state stored in Issuance State and User State
- block_timestamp

No instruction may:

- Read arbitrary accounts not passed explicitly
- Depend on global registries
- Depend on external program state
- Depend on transaction metadata beyond signer and timestamp

All required data MUST be present in the instruction context.

---

### 4.2 Explicit Output Model

Each instruction MUST produce only:

- Deterministic state mutations inside Issuance State and/or User State
- Explicit CPI token transfers (if applicable)
- Explicit error codes on failure

No instruction may:

- Emit dynamic external events (beyond standard Solana logs)
- Modify unrelated token accounts
- Create or destroy arbitrary accounts
- Write to accounts outside defined contract scope

All effects MUST be structurally predictable.

---

### 4.3 No Hidden Cross-Instruction Dependencies

The execution of any instruction MUST NOT require:

- Prior execution of unrelated instructions (except defined state transitions)
- Off-chain coordination
- UI-triggered ordering assumptions

Valid dependencies are limited to:

- reserve_funded gating deposit()
- reward_claimed gating claim_reward()
- sweep_executed gating sweep()
- reclaim_executed gating zero_participation_reclaim()

All gating conditions MUST be visible in state.

---

### 4.4 No Implicit Escrow State Usage

Escrow balances MUST NOT be used as accounting variables.

Escrow accounts act only as:

- Transfer constraints
- Value containers

Accounting variables MUST remain:

- total_locked
- total_weight_accum
- user_weight_accum

This ensures integration purity even if external tokens are sent to escrow.

---

### 4.5 Deterministic Error Surface

For identical:

- Inputs
- Accounts
- State
- Timestamp

The instruction MUST:

- Produce identical state mutation
- Produce identical transfer amount
- Produce identical error (if any)

Errors MUST NOT depend on:

- Transaction ordering within same day
- Non-deterministic compute paths
- External CPI side effects

---

### 4.6 Idempotency Model

Instructions are not universally idempotent,
but repeated execution MUST be deterministically gated.

Specifically:

- claim_reward() MUST fail after reward_claimed == true
- sweep() MUST fail after sweep_executed == true
- zero_participation_reclaim() MUST fail after reclaim_executed == true
- withdraw_deposit() MUST fail if locked_amount == 0

Repeated calls must not alter state beyond defined transitions.

---

### 4.7 No State Leakage

The contract MUST NOT:

- Store ephemeral integration metadata
- Cache intermediate reward buffers
- Store per-day snapshots
- Maintain hidden per-call counters

All persistent data MUST be defined in documented structures.

---

### 4.8 Side-Effect Isolation Conclusion

Integration-level purity guarantees:

- Deterministic instruction behavior
- Explicit state mutation boundaries
- Clean composability
- Safe integration with wallets and SDKs
- Absence of unintended state propagation

The Issuance Contract v1.1 exposes a minimal and strictly bounded integration surface.

---

## 5. Composability and Cross-Program Interaction Model

This section defines how the Issuance Contract v1.1 behaves
when invoked from other on-chain programs or composed
within larger transaction flows.

The objective is to ensure:

- Safe composability
- No hidden assumptions about caller origin
- No re-entrancy risks
- No cross-program state contamination
- Deterministic behavior in CPI contexts

---

### 5.1 CPI Invocation Safety

The Issuance Contract MAY be invoked:

- Directly by wallets
- Via SDK abstractions
- Via other on-chain programs (CPI)

The contract MUST NOT assume that:

- The caller is an EOA-style wallet
- The caller is a UI frontend
- The caller is trusted

All validation MUST rely exclusively on:

- Signer verification
- Account validation
- Immutable parameter checks

Caller identity beyond signer requirements MUST NOT influence behavior.

---

### 5.2 Re-Entrancy Model

Although Solana's execution model prevents classic EVM-style re-entrancy,
the contract MUST still:

- Mutate state before outbound CPI calls
- Not rely on post-transfer assumptions
- Not perform logic after CPI that assumes transfer success without checking result

All outbound transfers MUST occur as the final operation
after all state mutation is committed.

If CPI fails, the entire instruction MUST revert.

---

### 5.3 Cross-Program Isolation

The Issuance Contract MUST NOT:

- Invoke arbitrary external programs
- Depend on callbacks
- Perform nested CPI chains beyond SPL Token transfers

Permitted CPI interactions:

- SPL Token Program only

No other program interaction is permitted in v1.1.

This guarantees limited cross-program attack surface.

---

### 5.4 Atomic Multi-Instruction Transactions

The contract MUST behave correctly when:

- Multiple instructions are bundled in one transaction
- deposit() and claim_reward() are attempted in sequence
- withdraw_deposit() and claim_reward() are attempted in same transaction

Because accumulator logic is deterministic and bounded,
execution order within the same accounting day MUST NOT:

- Change reward proportionality
- Cause double accumulation
- Break invariants

Transaction atomicity MUST preserve internal consistency.

---

### 5.5 Parallel Execution Model

Solana runtime may execute transactions in parallel.

The design MUST ensure that:

- No global iteration occurs
- No shared mutable state outside Issuance State exists
- User State is isolated per participant

Race conditions are prevented by:

- Account-level write locks
- Deterministic accumulator logic
- Explicit state gating flags

Parallel deposits from different users MUST not corrupt accounting.

---

### 5.6 Flash Loan and Same-Slot Attack Resistance

Because weight accumulation is discrete per day:

- Same-day deposits generate zero additional accumulated weight
- Rapid deposit-withdraw patterns within same day yield no extra reward

Accumulator compression ensures:

- No intra-day exploitation
- No slot-level manipulation
- No timestamp micro-arbitrage

Weight increases only when day index increments.

---

### 5.7 Composability with DeFi Protocols

The contract MAY be composed with:

- Lending protocols
- Vault strategies
- DAO treasury automation
- Settlement bots

Provided:

- Required signer constraints are satisfied
- Required token accounts are correctly passed
- PDA and mint validation succeeds

No special composability hooks exist.
Integration is generic and permissionless.

---

### 5.8 No Implicit External Dependencies

The contract MUST NOT depend on:

- Off-chain price feeds
- External liquidity pools
- Governance signals
- DAO proposals
- Oracles

All economic logic is self-contained.

Composability does not introduce economic coupling.

---

### 5.9 Cross-Program Integrity Conclusion

The Issuance Contract v1.1:

- Is CPI-safe
- Is re-entrancy-resistant
- Is cross-program isolated
- Has minimal external interaction surface
- Preserves invariants under composability

Integration with external systems does not weaken structural guarantees.

---

## 6. Integration Test Scenarios Matrix

This section defines mandatory integration-level test scenarios
that validate correct behavior when the contract is invoked
through real transaction flows.

The objective is to verify:

- Account wiring correctness
- Instruction gating correctness
- Cross-instruction sequencing safety
- CPI correctness
- Deterministic outputs under realistic flows

All scenarios must execute against a local validator or devnet
with full account validation enabled.

---

### 6.1 Reserve Funding Flow

Scenario: Correct reserve funding before start_ts.

Steps:

1. Deploy issuance instance.
2. Call fund_reserve() with exact reserve_total.
3. Verify:
   - reserve_funded == true
   - Reward Escrow balance == reserve_total
4. Attempt second fund_reserve() call → MUST fail.

Edge Case:

- Call fund_reserve() after start_ts → MUST fail.
- Call fund_reserve() with incorrect amount → MUST fail.

---

### 6.2 Deposit Flow

Scenario: Valid deposit during participation window.

Steps:

1. reserve_funded == true.
2. block_timestamp within [start_ts, maturity_ts).
3. Call deposit(amount).

Verify:

- total_locked increased.
- user.locked_amount increased.
- Deposit Escrow balance increased.
- Accumulator updated deterministically.

Edge Cases:

- Deposit before reserve_funded → MUST fail.
- Deposit after maturity_ts → MUST fail.
- Deposit with wrong mint → MUST fail.

---

### 6.3 Multiple Deposits Same Day

Scenario: User deposits twice within same accounting day.

Verify:

- days_elapsed == 0 for second deposit.
- No extra weight accumulation occurs.
- total_weight_accum unchanged by second same-day deposit.

Ensures same-day determinism.

---

### 6.4 Claim Reward Flow

Scenario: Valid claim within claim window.

Steps:

1. Wait until block_timestamp >= maturity_ts.
2. Call claim_reward().

Verify:

- Accumulator finalizes to final_day_index.
- reward_claimed == true.
- Reward Escrow balance decreases by computed reward.
- Reward matches recomputed algebraic expectation.

Edge Cases:

- Claim before maturity_ts → MUST fail.
- Claim after claim_window → MUST fail.
- Claim twice → MUST fail.

---

### 6.5 Withdraw Deposit Flow

Scenario: Withdraw after maturity.

Steps:

1. block_timestamp >= maturity_ts.
2. Call withdraw_deposit().

Verify:

- Accumulator finalized before withdrawal.
- total_locked decreased.
- user.locked_amount == 0.
- Deposit Escrow balance decreased.
- Subsequent withdraw → MUST fail.

---

### 6.6 Sweep Flow

Scenario: Sweep after claim_window expiration.

Steps:

1. block_timestamp >= maturity_ts + claim_window.
2. Call sweep().

Verify:

- sweep_executed == true.
- Reward Escrow balance == 0.
- Further sweep() → MUST fail.
- claim_reward() → MUST fail.

Edge Case:

- sweep() when total_weight_accum == 0 → MUST fail.

---

### 6.7 Zero Participation Reclaim

Scenario: No deposits occurred.

Steps:

1. reserve funded.
2. No user deposits.
3. block_timestamp >= maturity_ts.
4. Call zero_participation_reclaim().

Verify:

- reclaim_executed == true.
- Reward Escrow balance == 0.
- sweep() → MUST fail.
- claim_reward() → MUST fail.

---

### 6.8 Cross-Instruction Atomicity

Scenario: deposit + withdraw in same transaction.

Expected:

- If block_timestamp < maturity_ts → withdraw fails.
- If block_timestamp >= maturity_ts → deposit fails.

Scenario: claim_reward() + withdraw_deposit() in same transaction.

Expected:

- Both succeed if preconditions satisfied.
- Reward amount unaffected by withdrawal order.

---

### 6.9 Account Substitution Attack Tests

Attempt:

- Passing wrong Deposit Escrow account.
- Passing wrong Reward Escrow account.
- Passing incorrect PDA.
- Passing wrong platform_treasury.
- Passing wrong issuer_address.

All attempts MUST fail deterministically.

---

### 6.10 Escrow Contamination Test

Scenario:

1. External actor transfers extra tokens into escrow.
2. Execute claim_reward() or sweep().

Verify:

- reward distribution remains bounded by reserve_total.
- Excess tokens are not distributed beyond defined logic.
- Accounting variables remain correct.

---

### 6.11 Deterministic Replay Test

Given identical:

- Initial state
- Deposits
- Timestamps
- Claim order

Replay entire flow on fresh deployment.

Verify:

- Identical final balances.
- Identical total_weight_accum.
- Identical user rewards.
- Identical settlement flags.

Determinism must hold exactly.

---

### 6.12 Integration Certification Result

After executing all scenarios:

Integration Status:

PASS → All scenarios behave as specified  
FAIL → Any deviation detected  

Integration certification requires full PASS status.

---

## 7. Integration Certification Statement

This section defines the formal certification boundary
for the Integration phase of Lockrion Issuance Contract v1.1.

Integration certification confirms that:

- All instruction interfaces are clean and deterministic.
- All account wiring is structurally validated.
- No hidden side effects exist.
- CPI interactions are properly bounded.
- Cross-instruction flows behave correctly.
- Account substitution attacks fail deterministically.
- Escrow isolation holds under realistic transaction flows.
- Deterministic behavior is preserved under replay.

---

### 7.1 Certification Preconditions

Integration certification may be granted only if:

- All scenarios defined in Section 6 pass.
- No unexpected state mutation occurs.
- No unexpected token transfer occurs.
- No instruction succeeds under invalid account configuration.
- No instruction bypasses required signer validation.
- No escrow authority mismatch is tolerated.

All tests MUST execute against:

- Local validator
or
- Devnet test environment

with full account validation enabled.

---

### 7.2 Certification Scope

Integration certification applies strictly to:

- Issuance Contract v1.1
- The documented instruction interface
- The documented account structure
- The documented PDA derivation model

Any change to:

- Account layout
- Instruction parameters
- PDA seed model
- Escrow authority logic
- Arithmetic model
- Execution order

INVALIDATES Integration certification.

A new Integration document version MUST be issued.

---

### 7.3 Certified Properties

If all integration tests pass, the contract is certified as:

- Interface-clean
- Account-safe
- CPI-bounded
- Cross-program isolated
- Deterministically composable
- Side-effect minimal
- Escrow-consistent
- Replay-safe

Integration certification does NOT guarantee:

- Economic viability
- Market behavior
- Token liquidity
- Off-chain UX correctness

It guarantees on-chain structural integration correctness only.

---

### 7.4 Failure Criteria

Integration certification MUST be denied if:

- Any scenario fails.
- Any account substitution succeeds.
- Any unauthorized caller succeeds.
- Any instruction produces non-deterministic behavior.
- Any escrow balance inconsistency is detected.
- Any settlement operation can execute twice.

Partial certification is NOT permitted.

Result must be binary:

PASS  
or  
FAIL

---

### 7.5 Integration Completion Declaration

Upon successful completion of all integration checks,
Lockrion Issuance Contract v1.1 is declared:

Integration-Certified.

This confirms readiness to proceed to:

- Theoretical Validation
- Compliance Matrix
- Code Review
- Auto-Test Suite
- Production deployment

Integration certification establishes that
the external interface surface is structurally sound.

---

