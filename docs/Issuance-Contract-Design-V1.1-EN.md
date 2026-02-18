# Lockrion Issuance Contract — Design v1.1 (Clean)
Status: Draft  
Standard: Lockrion Issuance Contract v1  
Target Network: Solana  

---

## 1. Architectural Overview

### 1.1 Design Purpose

This document defines the structural and implementation-level design of the Lockrion Issuance Contract v1.

The Design document translates the normative rules defined in Specification v1.1 into:

- concrete state structures,
- execution flow patterns,
- instruction-level behavior,
- account relationships,
- data isolation guarantees.

This document does not redefine rules.
It describes how the Specification is implemented at the contract level.

---

### 1.2 Contract Model

Each issuance is deployed as:

- an independent Solana program instance,
- non-upgradeable,
- immutable after deployment,
- structurally isolated from other issuances.

There are no shared global contracts.

Each issuance maintains:

- its own state account,
- its own deposit escrow,
- its own reward escrow.

Isolation is enforced structurally.

---

### 1.3 Core Structural Components

The issuance contract consists of:

1. Immutable Parameters (stored in Issuance State account)
2. Global Mutable State
3. Per-User State Accounts
4. Deposit Escrow Token Account
5. Reward Escrow Token Account

No other state accounts are permitted.

All state transitions occur exclusively through defined instructions.

---

### 1.4 Execution Principles

The contract enforces the following design principles:

- Canonical execution order for all instructions.
- Accumulator-first state updates.
- State mutation before token transfer (defensive order).
- No external discretionary branching.
- No dynamic logic injection.

All instruction flows are deterministic.

---

### 1.5 Trust Boundary Model

The trust boundary of the issuance contract is strictly limited to:

- Solana runtime,
- SPL Token Program.

The contract does not depend on:

- Oracles,
- External price feeds,
- Off-chain signatures,
- Governance-controlled state.

Design integrity derives from structural determinism.

---

## 2. State Account Structures

### 2.1 Issuance State Account (Global)

Each issuance maintains exactly one Issuance State account that stores:

A) Immutable Deployment Parameters:

- issuer_address
- lock_mint
- reward_mint (USDC)
- reserve_total
- start_ts
- maturity_ts
- accounting_period = 86400
- claim_window
- platform_treasury

B) Global Mutable State:

- reserve_funded (bool)
- total_locked (u128)
- total_weight_accum (u128)
- last_day_index (u64)
- final_day_index (u64)
- sweep_executed (bool)
- reclaim_executed (bool)

The Issuance State account is the single authoritative source for global accounting state.

---

### 2.2 Per-User State Account

Each participant address maintains exactly one per-user state account scoped to the issuance.

The Per-User State account stores:

- locked_amount (u128)
- user_weight_accum (u128)
- user_last_day_index (u64)
- reward_claimed (bool)

Per-User State accounts are independent and isolated.

No per-user state data is stored inside the Issuance State account.

---

### 2.3 Deposit Escrow Token Account

Each issuance maintains exactly one Deposit Escrow token account:

- mint = lock_mint
- owner = issuance contract program-derived authority (PDA)

The Deposit Escrow token account:

- receives deposits via deposit() instruction,
- releases tokens only via withdraw_deposit() instruction.

Direct external transfers into the escrow account MAY occur.
Such excess tokens are ignored for accounting purposes.

---

### 2.4 Reward Escrow Token Account

Each issuance maintains exactly one Reward Escrow token account:

- mint = reward_mint (USDC)
- owner = issuance contract program-derived authority (PDA)

The Reward Escrow token account:

- receives funding only via fund_reserve(),
- releases rewards only via claim_reward(),
- releases remaining balance only via sweep(),
- releases full balance to issuer only via zero-participation reclaim.

Reward escrow balance MUST never exceed reserve_total.

---

### 2.5 Initialization Rules

At deployment, the Issuance State MUST be initialized as:

- reserve_funded = false
- total_locked = 0
- total_weight_accum = 0
- last_day_index = 0
- final_day_index = (maturity_ts - start_ts) / 86400
- sweep_executed = false
- reclaim_executed = false

For a new participant (first interaction), Per-User State MUST be initialized as:

- locked_amount = 0
- user_weight_accum = 0
- user_last_day_index = current_day_index (computed by the global update procedure at first interaction)
- reward_claimed = false

Initialization MUST prevent retroactive weight accumulation.

---

## 3. Instruction-Level Design

### 3.1 Instruction Set Overview

The Issuance Contract v1 exposes exactly the following instructions:

1. fund_reserve()
2. deposit(amount)
3. claim_reward()
4. withdraw_deposit()
5. sweep()
6. zero_participation_reclaim()

No additional instructions are permitted.

Each instruction MUST follow canonical execution order rules defined in the Specification.

---

### 3.2 fund_reserve()

Purpose:
Fully fund the reward escrow before participation begins.

Preconditions:

- reserve_funded == false
- block_timestamp < start_ts
- transferred_amount == reserve_total

Execution Flow:

1. Verify transferred_amount == reserve_total.
2. Transfer USDC from issuer to Reward Escrow Account.
3. Set reserve_funded = true.

Postconditions:

- Reward Escrow balance == reserve_total.
- reserve_funded == true.
- Further calls to fund_reserve() MUST fail.

---

### 3.3 deposit(amount)

Purpose:
Lock lock_mint tokens for participation.

Preconditions:

- reserve_funded == true
- start_ts <= block_timestamp < maturity_ts
- amount > 0

Execution Flow (Canonical Order):

1. Update global accumulator.
2. Update user accumulator.
3. Increase:
   - user.locked_amount += amount
   - total_locked += amount
4. Transfer lock_mint tokens from participant to Deposit Escrow Account.

Postconditions:

- Deposit Escrow balance increases by amount.
- Accounting state remains deterministic.

Failure at any stage MUST revert the entire transaction.

---

### 3.4 claim_reward()

Purpose:
Claim proportional share of reserve_total.

Preconditions:

- block_timestamp >= maturity_ts
- block_timestamp < maturity_ts + claim_window
- reward_claimed == false
- total_weight_accum > 0

Execution Flow (Canonical Order):

1. Update global accumulator (finalize to final_day_index).
2. Update user accumulator.
3. Compute reward using canonical formula.
4. Set reward_claimed = true.
5. Transfer reward from Reward Escrow Account.

Postconditions:

- reward_claimed == true.
- Reward Escrow balance decreases deterministically.

If total_weight_accum == 0, instruction MUST fail.

---

### 3.5 withdraw_deposit()

Purpose:
Return locked lock_mint tokens after maturity.

Preconditions:

- block_timestamp >= maturity_ts
- user.locked_amount > 0

Execution Flow (Defensive Order):

1. Let amount = user.locked_amount.
2. Decrease total_locked by amount.
3. Set user.locked_amount = 0.
4. Transfer amount from Deposit Escrow Account to participant.

Postconditions:

- Deposit Escrow balance decreases.
- user.locked_amount == 0.

Withdrawal remains available indefinitely.

---

### 3.6 sweep()

Purpose:
Transfer unclaimed rewards to platform_treasury after claim window expiration.

Preconditions:

- block_timestamp >= maturity_ts + claim_window
- total_weight_accum > 0
- reward escrow balance > 0
- sweep_executed == false

Execution Flow:

1. Transfer entire Reward Escrow balance to platform_treasury.
2. Set sweep_executed = true.

Postconditions:

- Reward Escrow balance == 0.
- sweep_executed == true.

Further sweep() calls MUST fail.

---

### 3.7 zero_participation_reclaim()

Purpose:
Allow issuer to reclaim full reserve when no participation occurred.

Preconditions:

- total_weight_accum == 0
- block_timestamp >= maturity_ts
- reward escrow balance > 0
- reclaim_executed == false
- caller == issuer_address

Execution Flow (Defensive Order):

1. Set reclaim_executed = true.
2. Execute SPL transfer (Reward Escrow → issuer_address).

Postconditions:

- Reward Escrow balance == 0.
- reclaim_executed == true.

Subsequent reclaim attempts MUST fail regardless of escrow balance.

---

## 4. Accumulator Algorithm Flow Design

### 4.1 Design Objective

The accumulator mechanism ensures:

- deterministic weight accumulation,
- discrete daily accounting,
- bounded participation window,
- reproducible reward calculation.

The algorithm MUST be invoked before any state-changing instruction.

---

### 4.2 Global Accumulator Flow

Global accumulator update MUST execute exactly as follows:

1. If block_timestamp < start_ts:
   - No accumulation.
   - Exit procedure.

2. Compute raw_day_index:

   raw_day_index = floor((block_timestamp - start_ts) / 86400)

3. Compute current_day_index:

   current_day_index = min(raw_day_index, final_day_index)

4. Compute days_elapsed:

   days_elapsed = current_day_index - last_day_index

5. If days_elapsed > 0:

   total_weight_accum += total_locked * days_elapsed  
   last_day_index = current_day_index

Global accumulator update MUST precede all per-user updates.

---

### 4.3 Per-User Accumulator Flow

Per-user accumulator update MUST execute after global update.

Procedure:

1. Let current_day_index be the value computed in 4.2.
2. Compute days_elapsed_user:

   days_elapsed_user = current_day_index - user_last_day_index

3. If days_elapsed_user > 0:

   user_weight_accum += locked_amount * days_elapsed_user  
   user_last_day_index = current_day_index

Per-user update MUST occur before modifying user.locked_amount.

---

### 4.4 Bounded Accumulation Guarantee

Accumulation MUST never exceed final_day_index.

This applies to:

- deposit()
- claim_reward()
- withdraw_deposit()
- sweep()
- zero_participation_reclaim()

No instruction may alter weight beyond maturity.

---

### 4.5 Same-Day Determinism

Within the same accounting day:

- raw_day_index remains constant,
- current_day_index remains constant,
- days_elapsed = 0,
- no additional weight is accumulated.

Transaction ordering inside a single day MUST NOT influence weight share.

---

### 4.6 Initialization Interaction

At deployment:

- last_day_index = 0
- total_weight_accum = 0

For a new user:

- user_last_day_index MUST be set to current_day_index at first interaction
- user_weight_accum MUST begin at 0

This prevents retroactive weight accumulation.

---

### 4.7 Deterministic Reproducibility

Given:

- immutable parameters,
- on-chain timestamps,
- state variables,

any observer MUST be able to recompute:

- total_weight_accum,
- user_weight_accum,
- final reward amount.

The accumulator design MUST remain free of:

- hidden counters,
- implicit offsets,
- dynamic multipliers,
- non-linear modifiers.

---

## 5. Account Validation and Security Model

### 5.1 Account Set Validation

Each instruction MUST validate that all provided accounts match the issuance configuration.

Mandatory validations:

- Issuance State account is the expected instance for this issuance.
- Deposit Escrow token account mint == lock_mint.
- Reward Escrow token account mint == reward_mint (USDC).
- platform_treasury matches the immutable platform_treasury parameter.
- issuer_address matches the immutable issuer_address parameter.

Any mismatch MUST cause failure.

---

### 5.2 Program-Derived Authority (PDA)

All escrow token accounts MUST be owned by a program-derived authority (PDA) controlled exclusively by the issuance contract.

Properties:

- No externally held private key controls escrow accounts.
- Escrow transfers occur only through contract instructions.
- PDA derivation MUST be deterministic and unique per issuance.

The PDA MUST be the sole authority permitted to sign token transfers out of escrow.

---

### 5.3 Token Program Constraints

All token movements MUST use the SPL Token Program via CPI.

The contract MUST:

- validate token program ID,
- validate source and destination token accounts,
- validate mint consistency for each transfer,
- reject token accounts with mismatched mint.

No transfer may occur without explicit mint validation.

---

### 5.4 Defensive State Mutation Order

For instructions that perform outbound transfers from escrow, the Design enforces:

- state mutation precedes token transfer.

This applies to:

- withdraw_deposit()
- claim_reward() (reward_claimed set before transfer)
- sweep() (sweep_executed set before transfer, if implemented as a state flag)
- zero_participation_reclaim() (no flag required if balance becomes zero, but may be used)

This order minimizes risk from unexpected CPI behavior.

---

### 5.5 Re-Entrancy and CPI Safety

The contract MUST assume that CPI calls are external boundaries.

Therefore:

- all critical state MUST be committed before token CPI execution,
- no post-transfer state assumptions are permitted,
- repeated calls must be gated by state flags (reward_claimed, sweep_executed) or zero-balance conditions.

The design does not rely on re-entrancy being impossible.

---

### 5.6 Excess Token Handling

Direct external transfers into escrow accounts MAY occur.

Design rules:

- Deposit escrow balance MAY exceed sum(user.locked_amount).
- Any excess lock_mint tokens are ignored and do not affect accounting.

Reward escrow balance MUST NOT exceed reserve_total through contract logic.
If excess USDC is externally transferred in, the contract:

- MUST ignore the excess for reward calculation,
- MAY sweep or leave the excess unchanged depending on implementation policy,
- MUST NOT distribute more than reserve_total.

---

### 5.7 Failure Atomicity

All instructions MUST be atomic.

If any validation, arithmetic check, or CPI transfer fails:

- the entire instruction MUST revert,
- no partial state updates may remain committed.

Atomicity is required for escrow integrity and accounting correctness.

---

## 6. Instruction Flow Diagrams (Logical Execution Sequences)

### 6.1 fund_reserve() — Logical Flow

1. Validate:
   - reserve_funded == false
   - block_timestamp < start_ts
   - transferred_amount == reserve_total
   - reward_mint == USDC
2. Execute SPL transfer (issuer → Reward Escrow).
3. Verify Reward Escrow balance == reserve_total.
4. Set reserve_funded = true.

Failure at any step MUST revert.

---

### 6.2 deposit(amount) — Logical Flow

1. Validate:
   - reserve_funded == true
   - start_ts <= block_timestamp < maturity_ts
   - amount > 0
   - correct token mint and accounts
2. Execute global accumulator update.
3. Execute per-user accumulator update.
4. Mutate state:
   - user.locked_amount += amount
   - total_locked += amount
5. Execute SPL transfer (participant → Deposit Escrow).

All validation MUST occur before state mutation.
All state mutation MUST occur before token transfer.

---

### 6.3 claim_reward() — Logical Flow

1. Validate:
   - block_timestamp >= maturity_ts
   - block_timestamp < maturity_ts + claim_window
   - reward_claimed == false
   - total_weight_accum > 0
2. Execute global accumulator finalization.
3. Execute per-user accumulator update.
4. Compute reward using canonical formula.
5. Set reward_claimed = true.
6. Execute SPL transfer (Reward Escrow → participant).

Reward calculation MUST use finalized accumulators.

---

### 6.4 withdraw_deposit() — Logical Flow

1. Validate:
   - block_timestamp >= maturity_ts
   - user.locked_amount > 0
2. Let amount = user.locked_amount.
3. Mutate state:
   - total_locked -= amount
   - user.locked_amount = 0
4. Execute SPL transfer (Deposit Escrow → participant).

Withdrawal remains permanently available.

---

### 6.5 sweep() — Logical Flow

1. Validate:
   - block_timestamp >= maturity_ts + claim_window
   - total_weight_accum > 0
   - reward escrow balance > 0
   - sweep_executed == false
2. Set sweep_executed = true.
3. Execute SPL transfer (Reward Escrow → platform_treasury).

Sweep MUST be executable only once.

---

### 6.6 zero_participation_reclaim() — Logical Flow

1. Validate:
   - total_weight_accum == 0
   - block_timestamp >= maturity_ts
   - reward escrow balance > 0
   - caller == issuer_address
2. Execute SPL transfer (Reward Escrow → issuer_address).

After execution:

- Reward Escrow balance MUST equal zero.
- Subsequent reclaim attempts MUST fail.

---

## 7. Formal Invariant Enforcement Mapping

### 7.1 Purpose

This section maps each Specification invariant to the concrete enforcement mechanism implemented in the contract design.

The objective is to guarantee that every normative invariant defined in Specification v1.1 is:

- structurally enforced,
- programmatically validated,
- impossible to violate through defined instruction flows.

---

### 7.2 Invariant: total_locked Consistency

Specification Invariant:
total_locked equals the sum of all user.locked_amount.

Design Enforcement:

- deposit() increases both user.locked_amount and total_locked in the same instruction.
- withdraw_deposit() decreases total_locked before transfer and sets user.locked_amount = 0.
- No other instruction modifies these variables.
- Failure atomicity guarantees no partial mutation.

Violation is structurally impossible through defined instruction set.

---

### 7.3 Invariant: Monotonic total_weight_accum

Specification Invariant:
total_weight_accum is monotonically non-decreasing.

Design Enforcement:

- total_weight_accum only increases inside global accumulator update.
- No instruction decreases total_weight_accum.
- Accumulator bounding prevents overflow beyond final_day_index.
- Checked arithmetic prevents overflow wraparound.

---

### 7.4 Invariant: last_day_index ≤ final_day_index

Specification Invariant:
last_day_index ≤ final_day_index at all times.

Design Enforcement:

- current_day_index = min(raw_day_index, final_day_index)
- last_day_index is set only to current_day_index.
- final_day_index is immutable after deployment.

Thus, last_day_index can never exceed final_day_index.

---

### 7.5 Invariant: Bounded Reward Distribution

Specification Invariant:
Sum of all claimed rewards ≤ reserve_total.

Design Enforcement:

- reward calculation uses floor division.
- reward escrow initial balance == reserve_total.
- claim_reward() transfers from escrow only.
- sweep() transfers remaining escrow balance.
- zero_participation_reclaim() transfers full escrow only if no participation.

No instruction can mint or increase reward escrow balance.

---

### 7.6 Invariant: Deposit Escrow Accounting

Specification Invariant:
Deposit escrow balance ≥ sum(user.locked_amount).

Design Enforcement:

- deposit() increases locked_amount before transfer.
- withdraw_deposit() decreases locked_amount before transfer.
- Direct external transfers into escrow do not affect accounting variables.
- Accounting is based exclusively on state variables, not raw escrow balance.

Excess tokens are ignored.

---

### 7.7 Invariant: No Weight Accumulation Beyond Maturity

Specification Invariant:
Weight accumulation never occurs beyond final_day_index.

Design Enforcement:

- current_day_index is bounded by final_day_index.
- Accumulator update always uses bounded current_day_index.
- claim_reward(), withdraw_deposit(), sweep(), and reclaim all call accumulator logic before state change.

No path bypasses bounding logic.

---

### 7.8 Invariant: Single-Execution Flags

Specification Invariant:
- reward_claimed irreversible
- sweep_executed irreversible
- zero-participation reclaim single execution

Design Enforcement:

- reward_claimed checked before claim.
- sweep_executed set before sweep transfer.
- zero-participation reclaim gated by reward escrow balance > 0.

Repeated calls fail deterministically.

---

### 7.9 Invariant: Deterministic Arithmetic

Specification Invariant:
All arithmetic deterministic, integer-based, checked.

Design Enforcement:

- All numeric types fixed-width unsigned integers.
- Checked arithmetic enforced.
- Floor division used for reward formula.
- Division by zero prevented by precondition checks.

---

### 7.10 Structural Conclusion

Each invariant defined in Specification v1.1 has a direct structural enforcement path in Design v1.1.

There are:

- No invariant-dependent external assumptions.
- No invariant enforced only by convention.
- No invariant relying on off-chain behavior.

Invariant preservation is guaranteed by instruction-level structural design.

---

## 8. Compute Budget and Complexity Model

### 8.1 Design Objective

The Issuance Contract v1 is designed to:

- operate within deterministic compute bounds,
- avoid unbounded loops,
- avoid dynamic iteration over participant sets,
- maintain O(1) instruction complexity.

The design ensures scalability independent of the number of participants.

---

### 8.2 Constant-Time Accounting

All accounting operations are constant-time.

Specifically:

- Global accumulator update uses only scalar arithmetic.
- Per-user accumulator update uses only scalar arithmetic.
- Reward calculation uses a single multiplication and division.
- No iteration over user accounts occurs.

No instruction depends on:

- total number of participants,
- size of deposit escrow,
- number of past transactions.

Compute complexity per instruction is O(1).

---

### 8.3 Instruction-Level Compute Characteristics

fund_reserve():
- Single CPI transfer.
- Constant validation checks.

deposit():
- One global accumulator update.
- One per-user accumulator update.
- One state mutation.
- One CPI token transfer.

claim_reward():
- One global accumulator finalization.
- One per-user accumulator update.
- One reward computation.
- One CPI token transfer.

withdraw_deposit():
- One state mutation.
- One CPI token transfer.

sweep():
- One state mutation.
- One CPI token transfer.

zero_participation_reclaim():
- One CPI token transfer.

All instructions remain bounded and deterministic.

---

### 8.4 Absence of Iterative Loops

The design explicitly prohibits:

- Iterating over all participants.
- Iterating over historical deposits.
- Iterating over daily records.
- Maintaining per-day storage arrays.

Weight accumulation uses mathematical aggregation, not stored per-day entries.

---

### 8.5 Storage Growth Model

Per issuance storage growth consists of:

- One Issuance State account (constant size).
- One Deposit Escrow token account.
- One Reward Escrow token account.
- One Per-User State account per participant.

There is no global participant registry stored inside the Issuance State.

Storage grows linearly with number of participants,
but compute complexity does not depend on participant count.

---

### 8.6 Deterministic Gas Behavior

Because:

- no loops exist,
- no dynamic memory allocation occurs,
- no unbounded recursion is possible,

the compute cost of each instruction is predictable and reproducible.

No instruction exhibits adversarial compute amplification risk.

---

### 8.7 Denial-of-Service Resistance

The design prevents DoS amplification by:

- eliminating per-user iteration,
- eliminating dynamic reserve rebalancing,
- eliminating governance-triggered emergency paths,
- preventing repeated sweep or claim execution.

An attacker cannot increase compute cost by:

- increasing participant count,
- increasing deposit frequency,
- performing same-day micro-deposits.

Accumulator compression ensures bounded behavior.

---

## 9. Specification–Design Traceability Matrix

### 9.1 Purpose

This section provides a direct mapping between Specification v1.1 requirements and their concrete implementation mechanisms in Design v1.1.

The objective is to demonstrate:

- full coverage of normative rules,
- absence of unimplemented requirements,
- absence of undocumented execution behavior.

Every normative rule must have a corresponding structural enforcement path.

---

### 9.2 Fixed Reserve Commitment

Specification Requirement:
reserve_total is fixed, fully funded before participation, and bounded.

Design Enforcement:

- fund_reserve() requires exact reserve_total.
- reserve_funded flag gates deposit().
- Reward Escrow initial balance == reserve_total.
- No instruction permits reserve increase.
- Reward distribution strictly uses escrow balance.

Coverage: Complete.

---

### 9.3 Temporal Discreteness (86400 seconds)

Specification Requirement:
Accounting period fixed at 86400 seconds.

Design Enforcement:

- raw_day_index = floor((block_timestamp - start_ts) / 86400)
- final_day_index = (maturity_ts - start_ts) / 86400
- current_day_index bounded via min(raw_day_index, final_day_index)

Coverage: Complete.

---

### 9.4 Accumulator-Based Weight Model

Specification Requirement:
Weight accumulation must be deterministic, linear, and bounded.

Design Enforcement:

- Global accumulator update executed before state mutation.
- Per-user accumulator update executed after global update.
- No alternative weight calculation paths exist.
- No multipliers or non-linear modifiers implemented.

Coverage: Complete.

---

### 9.5 Canonical Execution Order

Specification Requirement:
State mutation and token transfers must follow canonical order.

Design Enforcement:

- deposit(): state mutation before transfer.
- withdraw_deposit(): state mutation before transfer.
- claim_reward(): reward_claimed set before transfer.
- sweep(): sweep_executed set before transfer.

Coverage: Complete.

---

### 9.6 Bounded Reward Distribution

Specification Requirement:
Total distributed rewards ≤ reserve_total.

Design Enforcement:

- Floor division reward formula.
- Escrow-limited transfers.
- No minting capability.
- Sweep transfers only remaining balance.

Coverage: Complete.

---

### 9.7 Zero-Participation Handling

Specification Requirement:
If total_weight_accum == 0, issuer may reclaim reserve.

Design Enforcement:

- zero_participation_reclaim() gated by:
  - total_weight_accum == 0
  - reward escrow balance > 0
  - caller == issuer_address
- No alternative transfer paths.

Coverage: Complete.

---

### 9.8 Escrow Isolation

Specification Requirement:
Deposit and reward escrows strictly separated.

Design Enforcement:

- Two independent token accounts.
- Distinct mint validation.
- PDA-controlled authority.
- No shared balances.

Coverage: Complete.

---

### 9.9 Deterministic Arithmetic

Specification Requirement:
All arithmetic fixed-width, checked, floor division.

Design Enforcement:

- u128 arithmetic.
- Checked overflow enforcement.
- Division-by-zero precondition checks.
- No floating-point logic.

Coverage: Complete.

---

### 9.10 Irreversibility

Specification Requirement:
Settlement operations irreversible.

Design Enforcement:

- reward_claimed flag.
- sweep_executed flag.
- Zero-balance post-reclaim condition.
- No rollback logic implemented.

Coverage: Complete.

---

### 9.11 Structural Conclusion

All normative requirements defined in Specification v1.1 are fully mapped to deterministic structural enforcement in Design v1.1.

There are:

- No unmapped requirements.
- No undocumented execution paths.
- No discretionary or hidden mechanisms.

Design v1.1 is fully traceable to Specification v1.1.
