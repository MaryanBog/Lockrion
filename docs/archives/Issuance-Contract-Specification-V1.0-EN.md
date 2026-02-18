# Issuance-Contract-Specification-V1.0-EN
## Section 1 — Purpose and Contract Scope

### 1.1 Objective

The Lockrion Issuance Contract v1.0 is a non-upgradeable Solana program implementing a fixed, reserve-backed, time-bound issuance commitment.

The contract SHALL:

- Enforce full reserve funding prior to participation.
- Maintain immutable issuance parameters.
- Implement discrete daily weight accumulation.
- Distribute rewards proportionally.
- Preserve strict escrow segregation.
- Prevent post-deployment modification.
- Operate deterministically using on-chain state only.

The contract SHALL NOT:

- Permit parameter mutation after deployment.
- Permit reserve increase after deployment.
- Allow partial reserve funding.
- Provide governance override paths.
- Use floating-point arithmetic.
- Depend on external oracles.
- Implement bonus or nonlinear reward mechanisms.

---

### 1.2 Execution Environment

The contract SHALL:

- Be deployed as a Solana BPF program.
- Revoke program upgrade authority at deployment.
- Use SPL Token Program for token transfers.
- Maintain escrow accounts as Program Derived Addresses (PDAs).
- Use Solana `block_timestamp` as the sole time reference.
- Use fixed-width unsigned integer arithmetic (u128).

No alternative execution environment SHALL be supported.

---

### 1.3 Contract Boundary

The Issuance Contract v1.0 governs exclusively:

- On-chain state management.
- Escrow fund custody.
- Weight accumulation.
- Reward calculation.
- Settlement logic.
- Terminal state transitions.

The contract SHALL NOT govern:

- Issuer screening.
- Deployment fee collection.
- Off-chain compensation.
- User interface behavior.
- Marketing or economic presentation.

All off-chain governance SHALL terminate at deployment.

---

### 1.4 Immutable Deployment Parameters

The following parameters SHALL be fixed at deployment:

- issuer_address : Pubkey
- lock_mint : Pubkey
- reward_mint : Pubkey (USDC)
- reserve_total : u128
- start_ts : i64 (UTC aligned)
- maturity_ts : i64
- accounting_period : 86400 seconds
- claim_window : i64
- platform_treasury : Pubkey

If any parameter violates defined constraints, deployment MUST fail.

After deployment, these parameters SHALL be immutable.

---

### 1.5 Core Functional Guarantee

The contract SHALL guarantee:

- total distributed rewards ≤ reserve_total.
- Deposits are fully withdrawable after maturity.
- Reward distribution is strictly proportional to accumulated weight.
- No weight accumulation occurs after maturity.
- No escrow commingling occurs.
- No cross-issuance interaction occurs.

Correctness SHALL derive from deterministic execution and invariant enforcement.

---

### 1.6 Deterministic Execution Requirement

All contract behavior SHALL:

- Be fully reproducible from on-chain state.
- Be independent of execution ordering within the same accounting day.
- Use checked arithmetic.
- Abort on overflow.
- Use floor division for reward calculation.
- Avoid nondeterministic branching.

Identical state history SHALL produce identical results.

---

### 1.7 Terminal Behavior Requirement

Each issuance SHALL:

- Reach a deterministic maturity state.
- Enter claim phase.
- Conclude in either sweep path or zero-participation reclaim path.
- Preserve deposit withdrawal availability permanently.
- Remain permanently immutable after settlement.

No execution path SHALL allow reactivation or modification post-settlement.

---

## Section 2 — State Model and Account Structure

### 2.1 Global State Account

Each issuance SHALL maintain exactly one Global State Account containing:

- issuer_address : Pubkey
- lock_mint : Pubkey
- reward_mint : Pubkey
- reserve_total : u128
- reserve_funded : bool
- total_locked : u128
- total_weight_accum : u128
- last_day_index : u64
- final_day_index : u64
- start_ts : i64
- maturity_ts : i64
- accounting_period : u64 (fixed = 86400)
- claim_window : i64
- platform_treasury : Pubkey
- sweep_executed : bool
- reclaim_executed : bool

The Global State Account SHALL:

- Be initialized at deployment.
- Be immutable with respect to deployment parameters.
- Be mutable only for runtime accounting variables.
- Be uniquely derived per issuance.

---

### 2.2 Per-User State Account

Each participant SHALL have a dedicated User State Account containing:

- owner : Pubkey
- locked_amount : u128
- user_weight_accum : u128
- user_last_day_index : u64
- reward_claimed : bool

The User State Account SHALL:

- Be created upon first deposit.
- Be uniquely associated with a single issuance.
- Not reference other participants.
- Not reference other issuances.
- Never allow negative balances.

---

### 2.3 Escrow Accounts

Each issuance SHALL maintain two escrow Program Derived Addresses (PDAs):

1. Deposit Escrow PDA
   - SPL Token account
   - mint == lock_mint
   - authority == Escrow Authority PDA
   - account.owner == SPL Token Program

2. Reward Escrow PDA
   - SPL Token account
   - mint == reward_mint (USDC)
   - authority == Escrow Authority PDA
   - account.owner == SPL Token Program

Escrow token accounts SHALL:

- Be created during initialize().
- Be deterministically derived from issuance seed.
- Be controlled exclusively by a program-derived Escrow Authority PDA.
- Reject any transfer not executed via program logic.
- Reject any mint mismatch.
- Reject any incorrect authority.

Escrow Authority PDA SHALL:

- Be derived from Global State PDA.
- Sign transfers only via invoke_signed.
- Not be externally accessible.

No escrow token account SHALL be:

- Owned directly by an externally controlled keypair.
- Controlled by issuer_address.
- Controlled by platform_treasury.
- Shared across issuances.

Escrow integrity SHALL derive from PDA authority control,
not from program ownership of the token account itself.

---

### 2.4 PDA Derivation Requirements

All PDAs SHALL:

- Include issuance-specific seed.
- Prevent cross-issuance collision.
- Prevent spoofed account substitution.
- Be verified on every instruction.

If PDA derivation fails validation, instruction MUST fail.

---

### 2.5 Immutable vs Mutable Fields

The following fields SHALL be immutable after successful initialize():

- issuer_address
- lock_mint
- reward_mint
- reserve_total
- start_ts
- maturity_ts
- accounting_period
- claim_window
- platform_treasury
- final_day_index

Immutability SHALL be enforced structurally by:

1. Writing immutable fields only during initialize().
2. Never assigning to immutable fields in any other instruction handler.
3. Validating expected immutable parameters on every instruction that
   depends on external accounts.

For every instruction execution, the program SHALL:

- Validate that provided lock_mint account matches stored lock_mint.
- Validate that provided reward_mint account matches stored reward_mint.
- Validate that provided issuer account matches stored issuer_address.
- Validate that provided platform treasury account matches stored platform_treasury.

Any mismatch SHALL cause immediate rejection.

No instruction SHALL:

- Reassign immutable fields.
- Accept alternative mint addresses.
- Accept alternative treasury.
- Modify final_day_index.
- Modify reserve_total.

Immutability SHALL be enforced both:

- By absence of mutation logic in handlers, and
- By explicit runtime validation of dependent accounts.

Any mutation of immutable fields SHALL constitute
a contract violation and MUST revert.

---

### 2.6 State Initialization Constraints

At deployment:

- reserve_funded == false
- total_locked == 0
- total_weight_accum == 0
- last_day_index == 0
- sweep_executed == false
- reclaim_executed == false

User accounts SHALL NOT exist prior to deposit.

---

### 2.7 Weight Accumulator Properties

The following SHALL hold at all times:

- total_weight_accum ≥ 0
- user_weight_accum ≥ 0
- last_day_index ≤ final_day_index
- user_last_day_index ≤ final_day_index
- Accumulators are monotonic non-decreasing

No decrement operation SHALL exist.

---

### 2.8 Conservation Constraints

Deposit Conservation:

At all times:

Σ locked_amount_i = deposit_escrow_balance

Where:

- locked_amount_i is stored in each User State account.
- deposit_escrow_balance is the SPL token balance of the Deposit Escrow PDA.

The contract SHALL NOT maintain an aggregate sum derived by iterating
over user accounts.

However, the Global State SHALL store:

total_locked

This field SHALL act as a mirrored accounting value that MUST satisfy:

total_locked = deposit_escrow_balance

total_locked SHALL:

- Be increased only after successful deposit escrow transfer.
- Be decreased only after successful withdrawal escrow transfer.
- Never be modified independently of escrow movement.

Deposit conservation SHALL therefore be enforced structurally by:

- Escrow isolation via PDA authority,
- Atomic SPL transfer semantics,
- Symmetric state updates tied directly to escrow transfers.

Reward Conservation:

Reward conservation SHALL be enforced using the structural escrow model
as defined in Section 5.10.

No aggregate reward distribution totals SHALL be stored.

---

### 2.9 Isolation Guarantee

The Global State Account and all related PDAs SHALL:

- Be unique per issuance.
- Not reference any external issuance.
- Not share state with other contracts.
- Not depend on global registry state.

Isolation SHALL be structural and cryptographically enforced.

---

### 2.10 Account Size and Layout Determinism

Account layouts SHALL:

- Be fixed-size.
- Use explicit field ordering.
- Avoid dynamic allocation.
- Avoid variable-length structures.

Serialization SHALL be deterministic and version-bound.

---

## Section 3 — Instruction Set and Execution Semantics

### 3.1 Instruction Set Overview

The Issuance Contract v1.0 SHALL expose exactly the following instructions:

1. initialize()
2. fund_reserve()
3. deposit(amount)
4. claim_reward()
5. withdraw_deposit()
6. sweep()
7. zero_participation_reclaim()

No additional state-mutating instruction SHALL exist.

---

### 3.2 initialize()

Purpose:

- Create Global State Account.
- Create escrow PDAs.
- Store immutable parameters.
- Compute final_day_index.
- Set initial runtime variables.
- Revoke upgrade authority (enforced at deployment level).

Preconditions:

- Called only once at deployment.
- start_ts aligned to 00:00:00 UTC.
- maturity_ts > start_ts.
- (maturity_ts - start_ts) mod 86400 == 0.
- reserve_total > 0.
- reward_mint == USDC.

Postconditions:

- Global State initialized.
- Escrow PDAs created.
- reserve_funded == false.
- No user accounts exist.

---

### 3.3 fund_reserve()

Purpose:

- Transfer exactly reserve_total USDC into Reward Escrow PDA.

Preconditions:

- signer == issuer_address.
- reserve_funded == false.
- block_timestamp < start_ts.
- transferred_amount == reserve_total.
- reward escrow balance == 0.

Postconditions:

- reward escrow balance == reserve_total.
- reserve_funded == true.

Failure SHALL revert fully.

---

### 3.4 deposit(amount)

Purpose:

- Lock participant tokens.
- Update accumulators.
- Increase total_locked.

Preconditions:

- reserve_funded == true.
- start_ts <= block_timestamp < maturity_ts.
- amount > 0.
- Valid lock_mint token account provided.

Execution Steps:

1. Update global accumulator.
2. Update user accumulator.
3. Transfer lock_mint tokens to Deposit Escrow PDA.
4. Increase locked_amount.
5. Increase total_locked.

Postconditions:

- State updated deterministically.
- No weight accrued for current day.
- Deposit escrow balance increases.

---

### 3.5 claim_reward()

Purpose:

- Distribute participant’s proportional reward.

Preconditions:

- maturity_ts <= block_timestamp < maturity_ts + claim_window.
- total_weight_accum > 0.
- reward_claimed == false.
- locked_amount MAY be zero or non-zero.
- Reward escrow balance sufficient.

Execution Steps:

1. Update global accumulator to final_day_index.
2. Update user accumulator to final_day_index.
3. Compute reward:
   reward = floor(reserve_total × user_weight_accum / total_weight_accum)
4. Transfer reward to user.
5. Set reward_claimed = true.

Postconditions:

- reward escrow balance decreases.
- reward_claimed prevents re-entry.

---

### 3.6 withdraw_deposit()

Purpose:

- Return full locked_amount to participant.
- Preserve accounting correctness and accumulator finalization.

Preconditions:

- block_timestamp >= maturity_ts.
- locked_amount > 0.

Execution Steps:

1. Update global accumulator to final_day_index.
2. Update user accumulator to final_day_index.
3. Transfer locked_amount from Deposit Escrow PDA.
4. Set locked_amount = 0.
5. Decrease total_locked by the withdrawn amount.

Postconditions:

- Deposit escrow decreases by withdrawn amount.
- User deposit fully unlocked.
- Accumulators remain finalized and unchanged after maturity.

Withdrawal SHALL NOT:

- Modify user_weight_accum beyond final_day_index.
- Modify total_weight_accum beyond final_day_index.
- Affect reward calculation formula.

Withdrawal MUST NOT execute any reward transfer logic.

---

### 3.7 sweep()

Purpose:

- Transfer unclaimed reward remainder to platform_treasury.

Preconditions:

- block_timestamp >= maturity_ts + claim_window.
- total_weight_accum > 0.
- sweep_executed == false.
- reclaim_executed == false.
- reward escrow balance > 0.

Execution Steps:

1. Transfer full reward escrow balance.
2. Set sweep_executed = true.

Postconditions:

- reward escrow balance == 0.
- No further reward claims possible.

---

### 3.8 zero_participation_reclaim()

Purpose:

- Return full reserve to issuer if no participation occurred.

Preconditions:

- block_timestamp >= maturity_ts.
- total_weight_accum == 0.
- reclaim_executed == false.
- sweep_executed == false.
- reward escrow balance > 0.
- signer == issuer_address.

Execution Steps:

1. Transfer full reward escrow balance to issuer_address.
2. Set reclaim_executed = true.

Postconditions:

- reward escrow balance == 0.
- sweep permanently disabled.

---

### 3.9 Instruction Atomicity Requirement

Each instruction SHALL:

- Fully succeed or fully revert.
- Not partially modify state.
- Not partially move tokens.
- Preserve invariants under failure.

Atomicity SHALL rely on Solana runtime guarantees.

---

### 3.10 Instruction Determinism Requirement

Given identical state and inputs:

- Instruction result SHALL be identical.
- No nondeterministic branching SHALL occur.
- No randomness SHALL influence state.
- No hidden side effects SHALL occur.

Execution SHALL remain fully deterministic.

---

## Section 4 — Time Model and Weight Accumulation Semantics

### 4.1 Canonical Time Source

The contract SHALL use exclusively:

- Solana `block_timestamp` as provided by the runtime.

The contract SHALL NOT:

- Accept user-supplied timestamps.
- Use external time oracles.
- Use slot numbers for accounting.
- Apply manual time offsets.

All time-dependent logic SHALL reference block_timestamp only.

---

### 4.2 Discrete Day Index Definition

Let:

accounting_period = 86400 seconds

Define:

raw_day_index = floor((block_timestamp - start_ts) / 86400)

current_day_index = min(raw_day_index, final_day_index)

Where:

final_day_index = floor((maturity_ts - start_ts) / 86400)

Constraints:

- start_ts MUST be aligned to 00:00:00 UTC.
- maturity_ts MUST align to full accounting periods.
- raw_day_index SHALL NOT be negative during participation phase.

No fractional-day participation SHALL be permitted.

---

### 4.3 Accumulator Invocation Ordering Requirement

For every instruction that may affect weight accounting,
the following strict ordering SHALL be enforced:

1. Validate lifecycle phase.
2. Compute current_day_index.
3. Update Global Accumulator:
     days_elapsed = current_day_index - last_day_index
     if days_elapsed > 0:
         total_weight_accum += total_locked × days_elapsed
         last_day_index = current_day_index
4. Update User Accumulator (if user involved):
     days_elapsed_user = current_day_index - user_last_day_index
     if days_elapsed_user > 0:
         user_weight_accum += locked_amount × days_elapsed_user
         user_last_day_index = current_day_index
5. Only after both accumulator updates:
     Perform core state mutation (e.g., increase locked_amount).
6. Perform escrow transfer.
7. Commit state.

For deposit():

- Global accumulator MUST be updated before increasing total_locked.
- User accumulator MUST be updated before increasing locked_amount.

This ordering guarantees:

- Deposits on day D SHALL NOT accrue weight for day D.
- Weight reflects only fully completed accounting days.
- Accumulators remain monotonic and deterministic.
- No retroactive weight assignment occurs.

For claim_reward():

- Accumulators MUST be updated to final_day_index
  before computing reward.

No instruction SHALL modify locked_amount
or total_locked before accumulator update completes.

Violation of this ordering SHALL constitute
mathematical inconsistency and MUST revert.

---

### 4.4 Per-User Accumulator Update Rule

Before modifying locked_amount, the contract SHALL:

1. Compute current_day_index.
2. Compute days_elapsed_user = current_day_index - user_last_day_index.
3. If days_elapsed_user > 0:
   user_weight_accum += locked_amount × days_elapsed_user
   user_last_day_index = current_day_index

Constraints:

- user_last_day_index ≤ final_day_index.
- user_weight_accum SHALL NOT overflow.
- Accumulation SHALL stop at final_day_index.

No user SHALL accumulate weight beyond maturity.

---

### 4.5 Deposit Participation Semantics

If a deposit occurs on accounting day D:

- The deposit SHALL NOT accrue weight for day D.
- Participation SHALL begin at day D + 1.
- If D == final_day_index - 1:
  Deposit MAY accrue weight for one day only.
- If D ≥ final_day_index:
  Deposit SHALL be rejected.

Participation SHALL reflect full-day exposure only.

---

### 4.6 Maturity Finalization Rule (Accumulator Freeze)

When:

block_timestamp ≥ maturity_ts

The accumulator engine SHALL finalize permanently.

The following SHALL hold:

1. current_day_index SHALL equal final_day_index.
2. last_day_index SHALL be updated to final_day_index.
3. No further increase of total_weight_accum SHALL be possible.
4. No further increase of user_weight_accum SHALL be possible.
5. final_day_index SHALL remain immutable.

After finalization:

- update_global_accumulator() SHALL produce days_elapsed = 0.
- update_user_accumulator() SHALL produce days_elapsed_user = 0.
- total_weight_accum SHALL remain constant.
- user_weight_accum SHALL remain constant.

No instruction SHALL:

- Modify last_day_index beyond final_day_index.
- Modify user_last_day_index beyond final_day_index.
- Increase total_weight_accum after maturity.
- Increase user_weight_accum after maturity.

This freeze SHALL apply regardless of:

- Claim execution
- Withdrawal execution
- Sweep execution
- Zero-participation reclaim

Accumulator freeze SHALL be:

- Deterministic
- Idempotent
- Irreversible
- Independent of reward settlement path

Post-maturity, weight state SHALL be fully finalized
and mathematically stable.

---

### 4.7 Order Independence Guarantee

Within the same accounting day:

- Deposit order SHALL NOT affect weight.
- Claim order SHALL NOT affect proportionality.
- Withdrawal order SHALL NOT affect reward distribution.

Weight accumulation SHALL depend only on full completed days.

---

### 4.8 Zero Participation Condition

Zero Participation SHALL be defined as:

total_weight_accum == 0
AND
block_timestamp ≥ maturity_ts

Under Zero Participation condition, the following SHALL hold:

1. No user SHALL have user_weight_accum > 0.
2. claim_reward() SHALL be permanently disabled.
3. sweep() SHALL be permanently disabled.
4. zero_participation_reclaim() SHALL be the only valid
   reward transfer instruction.
5. Division in reward computation SHALL NOT be attempted.

Zero Participation SHALL be derived exclusively from:

- total_weight_accum
- block_timestamp
- reclaim_executed
- sweep_executed

No separate boolean flag SHALL be stored.

Execution guarantees:

If total_weight_accum == 0 at maturity:

- No user SHALL be able to claim reward.
- Reward escrow balance SHALL equal reserve_total.
- zero_participation_reclaim() SHALL transfer full balance.
- reclaim_executed SHALL become true.
- sweep_executed SHALL remain permanently false.

After reclaim_executed == true:

- sweep() SHALL revert.
- claim_reward() SHALL revert.
- No reward transfer SHALL be possible.

Withdrawal of deposits SHALL remain available.

Zero Participation path SHALL be:

- Deterministic
- Mutually exclusive with sweep()
- Fully reserve-returning
- Irreversible

---

### 4.9 Monotonicity Conditions

The following SHALL be monotonic non-decreasing:

- total_weight_accum
- user_weight_accum
- last_day_index
- user_last_day_index

No instruction SHALL decrease these values.

---

### 4.10 Deterministic Time Invariance

Given identical:

- start_ts
- maturity_ts
- deposit history
- block_timestamp history

The following SHALL be identical across all nodes:

- total_weight_accum
- user_weight_accum
- final reward allocation

Time semantics SHALL be deterministic and reproducible.

---

## Section 5 — Reward Calculation and Settlement Semantics

### 5.1 Reward Eligibility Condition

A participant SHALL be eligible to claim reward only if:

- block_timestamp ≥ maturity_ts
- block_timestamp < maturity_ts + claim_window
- total_weight_accum > 0
- reward_claimed == false

If any condition fails, claim_reward() MUST revert.

---

### 5.2 Canonical Reward Formula

For each participant i:

reward_i = floor(reserve_total × user_weight_accum_i / total_weight_accum)

Constraints:

- Multiplication SHALL occur before division.
- All arithmetic SHALL use u128.
- Division SHALL use deterministic floor rounding.
- reward_i ≤ reserve_total.

No alternative reward formula SHALL exist.

---

### 5.3 Reward Remainder and Rounding Semantics

Reward calculation SHALL use integer floor division:

reward_i = floor(reserve_total × user_weight_accum_i / total_weight_accum)

Because floor division is used, it is mathematically possible that:

Σ reward_i < reserve_total

The remainder SHALL be defined as:

remainder = reserve_total − Σ reward_i

The contract SHALL NOT:

- Redistribute the remainder proportionally.
- Perform fractional rounding.
- Adjust reward_i values post-calculation.
- Store fractional values.

The remainder SHALL remain in the Reward Escrow PDA
until terminal settlement.

Terminal handling of remainder:

If total_weight_accum > 0:

- Remainder SHALL be transferred during sweep()
  together with any unclaimed rewards.

If total_weight_accum == 0:

- Full reserve_total SHALL be transferred
  via zero_participation_reclaim().

Under no condition SHALL:

- Σ reward_i exceed reserve_total.
- Any participant receive more than mathematically entitled.
- Remainder be lost or burned.
- Remainder be redistributed via implicit rounding.

Rounding behavior SHALL be:

- Deterministic
- Floor-based
- Identical across all nodes
- Independent of claim order

Remainder handling SHALL preserve
reward conservation invariant.

---

### 5.4 Reward Claim Finalization

Upon successful claim_reward():

- reward_i SHALL be transferred to participant.
- reward_claimed SHALL be set to true.
- Reward Escrow balance SHALL decrease.
- No recalculation SHALL occur.

A participant SHALL NOT claim more than once.

---

### 5.5 Withdrawal Independence

withdraw_deposit() SHALL:

- Not modify user_weight_accum.
- Not modify total_weight_accum.
- Not affect reward eligibility.
- Not affect reward_i value.

Reward calculation SHALL remain independent of withdrawal timing after maturity.

---

### 5.6 Claim Window Termination

The claim window SHALL be interpreted as a half-open interval.

ClaimReward eligibility interval:

[maturity_ts, maturity_ts + claim_window)

Therefore:

- claim_reward() SHALL be permitted only if:
  maturity_ts ≤ block_timestamp < maturity_ts + claim_window

- claim_reward() MUST revert if:
  block_timestamp ≥ maturity_ts + claim_window

The exact boundary timestamp:

block_timestamp == maturity_ts + claim_window

SHALL be treated as:

- Claim window ended
- claim_reward() disabled

This rule SHALL be applied consistently across all nodes
and across all handlers.

Post-claim behavior:

If block_timestamp ≥ maturity_ts + claim_window:

- claim_reward() SHALL be permanently disabled
- sweep() MAY be executed only if total_weight_accum > 0
- zero_participation_reclaim() MAY be executed only if total_weight_accum == 0

No alternative interpretation of claim window boundaries SHALL exist.

---

### 5.7 Sweep Semantics

If:

- total_weight_accum > 0
- sweep_executed == false
- block_timestamp ≥ maturity_ts + claim_window

Then:

- Entire remaining reward escrow balance SHALL transfer to platform_treasury.
- sweep_executed SHALL be set to true.
- No further reward transfers SHALL be possible.

Sweep SHALL be executable only once.

---

### 5.8 Zero Participation Reclaim Semantics

If:

- total_weight_accum == 0
- reclaim_executed == false
- block_timestamp ≥ maturity_ts
- signer == issuer_address

Then:

- Entire reward escrow balance SHALL transfer to issuer_address.
- reclaim_executed SHALL be set to true.
- sweep SHALL remain permanently disabled.

Reclaim SHALL be executable only once.

---

### 5.9 Mutual Exclusivity Constraint

The contract SHALL enforce:

If total_weight_accum == 0:
  sweep SHALL be permanently disabled.

If total_weight_accum > 0:
  zero_participation_reclaim SHALL be permanently disabled.

Only one terminal reward path SHALL be reachable.

---

### 5.10 Reward Conservation

Reward conservation SHALL be enforced structurally,
not via stored aggregate counters.

The contract SHALL guarantee:

1. reward_escrow_balance ≤ reserve_total
2. reward_escrow_balance ≥ 0

No reward SHALL ever be created or destroyed.

The only permitted outgoing reward transfers SHALL be:

- claim_reward()
- sweep()
- zero_participation_reclaim()

The contract SHALL NOT:

- Mint reward tokens.
- Burn reward tokens.
- Store cumulative distribution counters.
- Allow alternative transfer paths.

Each reward transfer SHALL:

1. Read current reward_escrow_balance.
2. Compute deterministic transfer_amount.
3. Validate transfer_amount ≤ reward_escrow_balance.
4. Abort on violation.

At Terminal State:

reward_escrow_balance SHALL equal 0.

Conservation SHALL derive from:

- Escrow isolation via PDA authority.
- Atomic SPL transfer semantics.
- Absence of mint/burn logic.
- Absence of alternative reward transfer instructions.
- Terminal flags preventing further reward movement.

No algebraic identity SHALL be relied upon
to define conservation correctness.

---

### 5.11 Deterministic Settlement Property

Given identical:

- user_weight_accum values
- total_weight_accum
- reserve_total

All nodes SHALL compute identical reward_i values.

Settlement SHALL be deterministic and reproducible.

---

### 5.12 Terminal Reward State

After either:

- sweep() executed, or
- zero_participation_reclaim() executed

Then:

- reward_escrow_balance == 0
- No further reward transfers possible
- Reward distribution state SHALL be final and immutable.

---

## Section 6 — Deposit Custody and Withdrawal Semantics

### 6.1 Deposit Acceptance Conditions

deposit(amount) SHALL succeed only if:

- reserve_funded == true
- start_ts ≤ block_timestamp < maturity_ts
- amount > 0
- lock_mint matches Global State
- Valid SPL token account provided
- Arithmetic bounds not exceeded

If any condition fails, transaction MUST revert.

---

### 6.2 Deposit Execution Semantics

Upon valid deposit:

1. Global accumulator SHALL be updated.
2. User accumulator SHALL be updated.
3. amount SHALL be transferred to Deposit Escrow PDA.
4. locked_amount SHALL increase by amount.
5. total_locked SHALL increase by amount.

Deposit SHALL NOT:

- Affect reserve_total.
- Affect reward_mint.
- Affect immutable parameters.

---

### 6.3 Pre-Funding and Funding Phase Separation

The lifecycle before Participation SHALL consist of a strictly defined Pre-Funding Phase.

Pre-Funding Phase SHALL be defined as:

- reserve_funded == false
- block_timestamp < start_ts

During Pre-Funding Phase:

Allowed:
- fund_reserve()

Disallowed:
- deposit()
- claim_reward()
- withdraw_deposit()
- sweep()
- zero_participation_reclaim()

Funding Completion SHALL occur only when:

- fund_reserve() transfers exactly reserve_total
- reward escrow balance becomes equal to reserve_total
- reserve_funded is set to true

Funding SHALL be:

- All-or-nothing
- Executed in a single transaction
- Equal to reserve_total
- Rejected if partial
- Rejected if block_timestamp ≥ start_ts

Once reserve_funded == true:

- reserve_funded SHALL never revert to false
- fund_reserve() SHALL permanently revert
- Participation Phase MAY begin only if:
    start_ts ≤ block_timestamp < maturity_ts

No funding SHALL be permitted:

- After start_ts
- After reserve_funded becomes true
- After maturity_ts

This separation guarantees:

- No partial reserve funding.
- No late funding after participation begins.
- Deterministic transition into Participation Phase.
- Alignment between funding completion and participation window.

---

### 6.4 No Early Withdrawal Constraint

Before maturity_ts:

- withdraw_deposit() MUST revert.
- No partial withdrawal allowed.
- No emergency withdrawal allowed.
- No governance override allowed.

Deposits SHALL remain locked until maturity.

---

### 6.5 Withdrawal Eligibility

withdraw_deposit() SHALL succeed only if:

- block_timestamp ≥ maturity_ts
- locked_amount > 0
- Valid user signer
- Valid deposit escrow PDA
- Valid user token account

If any condition fails, transaction MUST revert.

---

### 6.6 Withdrawal Execution Semantics

Upon successful withdrawal:

1. locked_amount SHALL be transferred from Deposit Escrow PDA.
2. locked_amount SHALL be set to 0.
3. total_locked SHALL decrease accordingly.

Withdrawal SHALL NOT:

- Modify user_weight_accum.
- Modify total_weight_accum.
- Affect reward eligibility.
- Affect reward calculation.

---

### 6.7 Withdrawal Idempotency

After withdrawal:

- locked_amount == 0
- Repeated withdraw_deposit() MUST revert.
- No negative balance SHALL occur.

Withdrawal SHALL be executable only once per user deposit state.

---

### 6.8 Deposit Conservation Invariant

At all times during contract execution:

Σ locked_amount_i = deposit_escrow_balance

Where:

- locked_amount_i is stored in each User State account.
- deposit_escrow_balance is the SPL token balance
  of the Deposit Escrow PDA.

The contract SHALL NOT store an aggregate sum of locked_amount values.
Instead, conservation SHALL be enforced structurally by:

1. Allowing deposit escrow transfers ONLY via:
   - deposit()
   - withdraw_deposit()

2. deposit() SHALL:
   - Transfer `amount` from user to Deposit Escrow PDA.
   - Increase locked_amount by `amount`.
   - Increase total_locked by `amount`.

3. withdraw_deposit() SHALL:
   - Transfer full locked_amount from Deposit Escrow PDA.
   - Decrease total_locked by locked_amount.
   - Set locked_amount to zero.

4. No instruction SHALL:
   - Modify locked_amount without escrow transfer.
   - Modify total_locked without corresponding escrow transfer.
   - Transfer lock_mint tokens from Deposit Escrow
     outside withdraw_deposit().

5. SPL transfer SHALL succeed before state mutation is committed.

If at any point:

deposit_escrow_balance ≠ total_locked

The instruction SHALL revert due to invariant violation.

Deposit escrow SHALL NEVER:

- Receive reward_mint.
- Be accessible by issuer.
- Be accessible by platform_treasury.
- Be used for any reward settlement logic.

After all users withdraw:

deposit_escrow_balance SHALL equal 0.

Deposit conservation SHALL be guaranteed by
strict escrow transfer symmetry and atomic execution.

---

### 6.9 Independence from Reward Path

Deposit custody SHALL remain independent of:

- claim_reward() execution.
- sweep() execution.
- zero_participation_reclaim() execution.

Withdrawal SHALL remain available regardless of reward settlement path.

---

### 6.10 Terminal Deposit State

After maturity and full withdrawal:

- User deposit state SHALL remain readable.
- locked_amount SHALL equal 0.
- user_weight_accum SHALL remain unchanged.
- reward_claimed SHALL remain as set.

Deposit lifecycle SHALL be finite and closed.

---

### 6.11 Deposit Custody Enforcement

Deposit custody SHALL follow the SPL Token security model.

For the Deposit Escrow token account:

1. account.owner MUST equal the SPL Token Program.
2. mint MUST equal lock_mint.
3. authority MUST equal Escrow Authority PDA.

Escrow Authority PDA SHALL:

- Be program-derived.
- Be deterministically validated on every instruction.
- Sign transfers only via invoke_signed.
- Not be externally signable.

Deposit tokens SHALL be moved ONLY through:

- deposit()
- withdraw_deposit()

The contract SHALL NOT:

- Assume program ownership of SPL token accounts.
- Treat account.owner as escrow authority.
- Permit externally controlled authority for escrow accounts.

Custody SHALL be enforced structurally by:

- SPL Token Program account ownership,
- Escrow Authority PDA as token authority,
- Deterministic PDA validation,
- Controlled transfer paths.

---

## Section 7 — Error Handling and Rejection Semantics

### 7.1 General Rejection Principle

For every instruction:

If any precondition is violated:

- Execution SHALL abort.
- No state mutation SHALL persist.
- No token transfer SHALL persist.
- All invariants SHALL remain intact.

Rejection SHALL be deterministic.

---

### 7.2 Initialization Rejection Conditions

initialize() MUST fail if:

- Called more than once.
- start_ts not aligned to 00:00:00 UTC.
- maturity_ts ≤ start_ts.
- (maturity_ts - start_ts) mod 86400 ≠ 0.
- reserve_total == 0.
- reward_mint ≠ USDC.
- Any arithmetic bound violated.
- PDA derivation mismatch.
- Immutable parameters inconsistent.

No partial initialization SHALL occur.

---

### 7.3 Reserve Funding Rejection Conditions

fund_reserve() MUST fail if:

- reserve_funded == true.
- signer ≠ issuer_address.
- block_timestamp ≥ start_ts.
- transferred_amount ≠ reserve_total.
- reward escrow already contains funds.
- SPL transfer fails.
- PDA validation fails.

Partial funding SHALL NOT be accepted.

---

### 7.4 Deposit Rejection Conditions

deposit(amount) MUST fail if:

- reserve_funded == false.
- block_timestamp < start_ts.
- block_timestamp ≥ maturity_ts.
- amount == 0.
- lock_mint mismatch.
- Arithmetic overflow in total_locked.
- SPL transfer fails.
- PDA validation fails.

No deposit SHALL partially modify accumulators.

---

### 7.5 Claim Rejection Conditions

claim_reward() MUST fail if:

- block_timestamp < maturity_ts.
- block_timestamp ≥ maturity_ts + claim_window.
- total_weight_accum == 0.
- reward_claimed == true.
- reward escrow balance < calculated reward.
- Arithmetic overflow.
- PDA validation fails.

Division by zero SHALL be impossible due to precondition.

---

### 7.6 Withdrawal Rejection Conditions

withdraw_deposit() MUST fail if:

- block_timestamp < maturity_ts.
- locked_amount == 0.
- lock_mint mismatch.
- SPL transfer fails.
- PDA validation fails.

No partial withdrawal SHALL occur.

---

### 7.7 Sweep Rejection Conditions

sweep() MUST fail if:

- block_timestamp < maturity_ts + claim_window.
- total_weight_accum == 0.
- sweep_executed == true.
- reclaim_executed == true.
- reward escrow balance == 0.
- PDA validation fails.

Sweep SHALL execute only once.

---

### 7.8 Zero Participation Reclaim Rejection Conditions

zero_participation_reclaim() MUST fail if:

- signer ≠ issuer_address.
- block_timestamp < maturity_ts.
- total_weight_accum > 0.
- reclaim_executed == true.
- sweep_executed == true.
- reward escrow balance == 0.
- PDA validation fails.

Reclaim SHALL execute only once.

---

### 7.9 Arithmetic Failure Handling

All arithmetic in Issuance Contract v1.0 SHALL be performed using checked
fixed-width integer operations.

The contract SHALL treat arithmetic overflow as a hard rejection condition.

For any instruction:

If any required arithmetic operation fails due to:

- u128 addition overflow
- u128 subtraction underflow
- u128 multiplication overflow
- division by zero attempt

Then:

- Execution SHALL abort.
- No state mutation SHALL persist.
- No token transfer SHALL persist.
- An explicit Overflow or DivisionByZero error SHALL be returned.

This applies explicitly to the reward computation numerator:

numerator = reserve_total × user_weight_accum

If checked multiplication overflows:

- claim_reward() MUST revert with Overflow.
- No partial reward distribution SHALL occur.

Safe-domain requirement:

The contract SHALL NOT introduce any alternative arithmetic path,
no floating-point approximations, and no widened intermediate types.

All arithmetic safety is enforced by:

- checked_* operations in all handlers and utility functions
- precondition guards that prevent division by zero
- invariant enforcement that prevents negative or inconsistent indices

Overflow rejection is a permitted and defined behavior
under this specification.

No instruction SHALL ever wrap silently.

---

### 7.10 Unauthorized Access Handling

Any instruction SHALL fail if:

- Required signer missing.
- Incorrect account owner.
- Incorrect PDA seed.
- Cross-issuance account provided.
- Incorrect token mint supplied.

Authorization SHALL be cryptographically enforced.

---

### 7.11 State Integrity Guarantee

If any invariant defined in prior sections would be violated:

- Instruction SHALL revert.
- No state SHALL be partially updated.
- No token SHALL move.

State integrity SHALL take precedence over instruction success.

---

## Section 8 — Security Model and Adversarial Constraints

### 8.1 Security Foundation

Security of the Issuance Contract v1.0 SHALL derive from:

- Program immutability.
- Deterministic execution.
- Escrow segregation via PDAs.
- Strict invariant enforcement.
- Checked arithmetic.

No discretionary administrative control SHALL exist post-deployment.

---

### 8.2 Immutability Enforcement

At deployment:

- Upgrade authority SHALL be permanently revoked.
- No proxy pattern SHALL be used.
- No delegated upgrade authority SHALL exist.
- No dynamic code loading SHALL occur.

Observers SHALL be able to verify program immutability on-chain.

---

### 8.3 Escrow Authority Constraint

Escrow token accounts SHALL:

- Be SPL Token accounts.
- Have account.owner == SPL Token Program.
- Have authority == Escrow Authority PDA.

The Escrow Authority PDA SHALL:

- Be derived deterministically from the issuance seed.
- Be controlled exclusively by program logic.
- Sign transfers only via invoke_signed.
- Not be externally accessible.

No externally controlled keypair SHALL:

- Act as escrow authority.
- Transfer tokens from escrow.
- Substitute escrow accounts.
- Override escrow constraints.

Escrow security SHALL derive from:

- PDA derivation guarantees,
- Explicit authority validation,
- Strict instruction routing through the Escrow Transfer Module.

Program ownership of escrow token accounts SHALL NOT be assumed.
Authority control SHALL be enforced explicitly.

---

### 8.4 Adversarial Participant Model

The contract SHALL assume:

- Participants may act strategically.
- Participants may attempt timing manipulation.
- Participants may split capital across addresses.
- Participants may spam transactions.

The contract SHALL ensure:

- No advantage from intraday timing.
- No advantage from address splitting.
- No advantage from transaction ordering within same day.

---

### 8.5 MEV Neutrality

The contract SHALL ensure:

- Execution ordering within same accounting day does not affect reward share.
- Claim order does not alter proportional allocation.
- Withdrawal order does not affect others’ rewards.

No miner-extractable structural arbitrage SHALL exist.

---

### 8.6 Arithmetic Attack Resistance

The contract SHALL:

- Use checked u128 arithmetic.
- Multiply before divide.
- Prevent division by zero.
- Abort on overflow.
- Avoid implicit type casting.

Arithmetic SHALL be safe under all valid inputs.

---

### 8.7 Timestamp Manipulation Boundaries

The contract SHALL rely solely on:

- Solana block_timestamp.

It SHALL NOT:

- Accept user-supplied timestamps.
- Use off-chain time.
- Allow manual override of time.

Minor block_timestamp drift SHALL NOT affect proportional fairness due to discrete day model.

---

### 8.8 Denial-of-Service Resistance

The contract SHALL ensure:

- No global iteration over participants.
- No unbounded loops.
- No state-dependent linear scans.
- All instructions execute in O(1) time.

Large participant counts SHALL NOT degrade claim performance.

---

### 8.9 Replay and Double-Execution Protection

The contract SHALL enforce:

- reward_claimed prevents duplicate claim.
- sweep_executed prevents duplicate sweep.
- reclaim_executed prevents duplicate reclaim.
- reserve_funded prevents duplicate funding.
- locked_amount == 0 prevents duplicate withdrawal.

State flags SHALL enforce single execution paths.

---

### 8.10 Cross-Issuance Isolation

The contract SHALL ensure:

- Unique PDAs per issuance.
- No shared escrow.
- No shared accounting state.
- No global reward variable.
- No cross-issuance dependency.

Compromise of one issuance SHALL NOT affect another.

---

### 8.11 Privilege Escalation Prohibition

The contract SHALL ensure:

- Only issuer_address may execute reclaim.
- Any address may execute sweep (subject to rules).
- No hidden authority exists.
- No emergency override exists.
- No role-based mutation exists.

Authority SHALL be strictly rule-bound.

---

### 8.12 Final Security Guarantee

Under correct Solana runtime behavior and correct SPL Token behavior:

- Escrow theft SHALL be impossible.
- Parameter mutation SHALL be impossible.
- Over-distribution SHALL be impossible.
- Unauthorized execution SHALL be impossible.
- Deterministic reproducibility SHALL hold.

Security SHALL derive from structural constraints, not discretionary oversight.

---

## Section 9 — Formal Invariants and Proof Obligations

### 9.1 Global Invariant Set

At all times during contract lifecycle, the following invariants MUST hold:

1. reserve_total > 0
2. total_locked ≥ 0
3. total_weight_accum ≥ 0
4. last_day_index ≤ final_day_index
5. reserve_funded ∈ {true, false}
6. sweep_executed ∈ {true, false}
7. reclaim_executed ∈ {true, false}
8. NOT (sweep_executed == true AND reclaim_executed == true)

Violation of any invariant SHALL constitute a contract failure.

---

### 9.2 User-Level Invariant Set

For each participant i:

1. locked_amount_i ≥ 0
2. user_weight_accum_i ≥ 0
3. user_last_day_index_i ≤ final_day_index
4. reward_claimed_i ∈ {true, false}
5. If reward_claimed_i == true → claim_reward() MUST revert
6. If locked_amount_i == 0 → withdraw_deposit() MUST revert

User invariants SHALL hold independently of other users.

---

### 9.3 Accumulator Monotonicity Invariant

The following SHALL be monotonic non-decreasing:

- total_weight_accum
- user_weight_accum_i
- last_day_index
- user_last_day_index_i

No instruction SHALL decrease these values.

---

### 9.4 Terminal Reward Conservation

At Terminal State:

reward_escrow_balance SHALL equal 0.

The contract SHALL guarantee that:

- All reward tokens initially funded into escrow
  are either:
  - Claimed proportionally by participants,
  - Swept to platform treasury (if participation > 0),
  - Reclaimed by issuer (if participation == 0).

No reward tokens SHALL remain locked permanently
after claim_window expiry.

No reward tokens SHALL be lost or duplicated.

Conservation SHALL be enforced structurally through:

- Escrow authority restrictions,
- Controlled transfer paths,
- Deterministic settlement logic,
- Terminal state flags preventing re-entry.

No cumulative reward counter SHALL be required
to verify conservation correctness.

---

### 9.5 Deposit Conservation Invariant

At all times:

Σ locked_amount_i = deposit_escrow_balance

After all withdrawals:

deposit_escrow_balance = 0

No instruction SHALL mint or burn lock_mint tokens.

---

### 9.6 Mutual Exclusivity Invariant

The contract SHALL enforce strict mutual exclusivity
between the two terminal reward paths:

- sweep()
- zero_participation_reclaim()

The following conditions SHALL always hold:

1. NOT (sweep_executed == true AND reclaim_executed == true)

2. If total_weight_accum > 0:
     reclaim_executed SHALL NEVER become true.

3. If total_weight_accum == 0:
     sweep_executed SHALL NEVER become true.

4. Once either flag becomes true:
     - reward_escrow_balance SHALL equal 0
     - The other terminal instruction SHALL permanently revert
     - claim_reward() SHALL permanently revert

Execution constraints:

Before executing sweep():

- total_weight_accum > 0
- sweep_executed == false
- reclaim_executed == false

Before executing zero_participation_reclaim():

- total_weight_accum == 0
- reclaim_executed == false
- sweep_executed == false

Flags SHALL be:

- Written only after successful escrow transfer
- Never reset
- Never toggled back to false

Terminal reward resolution SHALL be:

- Deterministic
- Structurally exclusive
- Irreversible
- Fully escrow-balanced

No instruction SHALL create a state
in which both sweep_executed and reclaim_executed are true.

---

### 9.7 Immutability Invariant

After initialize():

The following SHALL NEVER change:

- issuer_address
- lock_mint
- reward_mint
- reserve_total
- start_ts
- maturity_ts
- accounting_period
- claim_window
- platform_treasury
- final_day_index

Any mutation SHALL constitute contract violation.

---

### 9.8 Deterministic State Transition Invariant

Given identical:

- Global state
- User state
- Instruction input
- block_timestamp

The resulting state transition SHALL be identical across all nodes.

No nondeterministic branching SHALL exist.

---

### 9.9 Terminal State Invariant

After either:

- sweep_executed == true, or
- reclaim_executed == true

The following SHALL hold:

- reward_escrow_balance == 0
- claim_reward() SHALL revert
- No further reward transfers possible

Terminal state SHALL be final and irreversible.

---

### 9.10 Lifecycle Closure Property

For every issuance:

1. Terminal State SHALL be reachable deterministically.
2. No issuance SHALL remain in ambiguous reward state.
3. No reward SHALL remain permanently locked beyond claim window.

Closure Model:

If total_weight_accum > 0:
  Terminal State SHALL be reachable via sweep().

If total_weight_accum == 0:
  Terminal State SHALL be reachable via zero_participation_reclaim().

The contract SHALL NOT auto-execute terminal instructions.
Terminal settlement requires explicit invocation by an external actor.

However:

- No additional state transition SHALL be required.
- No hidden condition SHALL prevent terminal instruction execution.
- Terminal instruction SHALL remain valid indefinitely after eligibility.

Thus:

Lifecycle closure SHALL be structurally guaranteed,
while execution SHALL remain explicitly triggered.

---

### 9.11 Soundness Condition

If all invariants (Sections 9.1–9.10) hold for all reachable states:

Then:

- Reward distribution SHALL be mathematically correct.
- No over-distribution SHALL occur.
- No escrow breach SHALL occur.
- No authority escalation SHALL occur.
- Lifecycle SHALL remain finite and deterministic.

Soundness SHALL derive from invariant preservation.

---

## Section 10 — Acceptance Criteria and Implementation Readiness

### 10.1 Functional Acceptance Criteria

The contract SHALL be considered functionally complete when:

- All defined instructions are implemented.
- All preconditions are enforced.
- All state transitions conform to lifecycle model.
- All arithmetic constraints are enforced.
- All invariants hold under all valid execution paths.

No undocumented behavior SHALL exist.

---

### 10.2 Determinism Acceptance Criteria

The contract SHALL satisfy:

- Identical on-chain state → identical outputs.
- No randomness used.
- No floating-point arithmetic used.
- No external data dependencies used.
- No execution-order dependency within same accounting day.

Determinism SHALL be verifiable by replaying transaction history.

---

### 10.3 Escrow Integrity Acceptance Criteria

The contract SHALL demonstrate:

- Deposit escrow balance equals sum of locked_amount.
- Reward escrow balance equals reserve_total minus distributed amounts.
- No instruction allows escrow transfer outside defined logic.
- No underflow or overflow in escrow arithmetic.

Escrow integrity SHALL be testable.

---

### 10.4 Reward Correctness Acceptance Criteria

Given a defined deposit timeline:

- total_weight_accum SHALL equal Σ user_weight_accum.
- reward_i SHALL equal floor(reserve_total × W_i / W_total).
- Σ reward_i ≤ reserve_total.
- Remainder SHALL remain in escrow until sweep().

Reward distribution SHALL match formal model.

---

### 10.5 Lifecycle Acceptance Criteria

The contract SHALL demonstrate:

- Reserve funding must occur before participation.
- Deposits accepted only during participation window.
- Claims accepted only during claim window.
- Withdrawals accepted only after maturity.
- Only one terminal reward path reachable.
- Terminal state irreversible.

Lifecycle SHALL be finite and complete.

---

### 10.6 Failure Handling Acceptance Criteria

For each invalid input scenario:

- Instruction SHALL revert.
- No state mutation SHALL persist.
- No token transfer SHALL occur.
- All invariants SHALL remain intact.

Failure cases SHALL be explicitly testable.

---

### 10.7 Security Acceptance Criteria

The contract SHALL demonstrate:

- Upgrade authority revoked.
- PDA validation enforced.
- No admin override exists.
- No cross-issuance interaction possible.
- No hidden instruction exists.

Security SHALL derive from structural constraints.

---

### 10.8 Performance Acceptance Criteria

The contract SHALL demonstrate:

- O(1) execution complexity per instruction.
- No iteration over all users.
- No dynamic memory allocation.
- No state growth beyond fixed account structures.

Performance SHALL scale linearly only by number of participants (via separate accounts).

---

### 10.9 Compliance Readiness Criteria

This specification SHALL be sufficient to derive:

- Design Document
- Implementation Document
- Static Analysis Plan
- Integration Plan
- Auto-Test Suite
- Compliance Matrix

No additional behavior SHALL be introduced outside this document.

---

### 10.10 Final Conformance Statement

If Sections 1–10 are implemented exactly as specified:

Then Issuance Contract v1.0 SHALL be:

- Immutable after deployment.
- Fully reserve-backed.
- Deterministic in execution.
- Mathematically sound.
- Escrow-safe.
- Proportionally fair.
- Structurally isolated.
- Publicly auditable.

Contract correctness SHALL derive from invariant enforcement and deterministic state transitions.

---

## Section 11 — Explicit Non-Goals and Prohibited Behaviors

### 11.1 No Governance After Deployment

The contract SHALL NOT:

- Expose administrative override.
- Allow pause functionality.
- Allow parameter update.
- Allow forced settlement.
- Allow emergency withdrawal.

Governance authority SHALL terminate permanently at deployment.

---

### 11.2 No Dynamic Economic Adjustment

The contract SHALL NOT:

- Adjust reserve_total post-deployment.
- Introduce dynamic reward scaling.
- Introduce participation-based bonus.
- Adjust claim_window dynamically.
- Modify reward formula.

Economic structure SHALL remain fixed.

---

### 11.3 No Early Exit Mechanism

The contract SHALL NOT:

- Permit withdrawal before maturity.
- Permit partial withdrawal before maturity.
- Permit penalty-based early exit.
- Offer buyback mechanism.

Locked capital SHALL remain locked until maturity.

---

### 11.4 No Hidden Fees

The contract SHALL NOT:

- Deduct percentage-based participant fees.
- Deduct reward distribution fees.
- Transfer funds to undisclosed accounts.
- Implement protocol-level hidden extraction.

All token movement SHALL be explicit and verifiable.

---

### 11.5 No Cross-Issuance Coupling

The contract SHALL NOT:

- Share state with other issuances.
- Reference external issuance state.
- Pool reward reserves.
- Offset reward balances across issuances.

Each issuance SHALL remain fully isolated.

---

### 11.6 No Floating-Point Arithmetic

The contract SHALL NOT:

- Use floating-point types.
- Perform fractional reward allocation beyond integer precision.
- Rely on implicit rounding modes.

All arithmetic SHALL use fixed-width unsigned integers.

---

### 11.7 No Randomness

The contract SHALL NOT:

- Use random number generation.
- Use slot-based randomness.
- Use external entropy sources.
- Implement lottery-like distribution.

Execution SHALL be purely deterministic.

---

### 11.8 No Off-Chain Dependencies

The contract SHALL NOT:

- Depend on price feeds.
- Depend on oracle services.
- Depend on external APIs.
- Depend on off-chain settlement.

All behavior SHALL be derived solely from on-chain state.

---

### 11.9 No Retroactive Adjustment

The contract SHALL NOT:

- Recalculate rewards after claim.
- Reverse completed withdrawals.
- Modify historical weight.
- Change final_day_index.
- Alter terminal state.

All state transitions SHALL be final.

---

### 11.10 No Implicit Guarantees

The contract SHALL NOT:

- Guarantee profitability.
- Guarantee capital preservation.
- Guarantee minimum return.
- Guarantee comparative performance.

The contract SHALL operate strictly as deterministic reserve-backed issuance logic.

---

### 11.11 Specification Closure Condition

The contract SHALL NOT implement any behavior not explicitly defined in Sections 1–11.

Any additional behavior SHALL require:

- New version designation.
- Separate specification document.
- Independent deployment.

---

## Section 12 — Specification Closure and Conformance Boundary

### 12.1 Scope Finalization

This document defines the complete behavioral, mathematical, state, and security model of:

Lockrion Issuance Contract v1.0

The specification includes:

- State model
- Account structure
- Instruction set
- Time semantics
- Reward computation
- Settlement rules
- Deposit custody rules
- Error handling
- Security model
- Formal invariants
- Acceptance criteria
- Explicit non-goals

No additional functional domain exists beyond this document.

---

### 12.2 No Implicit Behavior Rule

The contract SHALL NOT:

- Implement undocumented logic.
- Include hidden instruction branches.
- Rely on unspecified state.
- Introduce undocumented authority.
- Derive behavior from assumptions outside this document.

All contract behavior MUST be explicitly described herein.

---

### 12.3 Conformance Requirement

An implementation SHALL be considered compliant only if:

- Every invariant is enforced.
- Every prohibited behavior is absent.
- Every instruction matches defined semantics.
- All arithmetic rules are respected.
- PDA validation is implemented on all instructions.
- Immutability is enforced at deployment.

Partial adherence SHALL NOT constitute compliance.

---

### 12.4 Downstream Document Authority

For subsequent documentation:

- Design Document MUST derive strictly from this Specification.
- Implementation Document MUST implement only defined behavior.
- Static Analysis MUST validate invariants herein.
- Test Suite MUST validate constraints herein.
- Compliance Matrix MUST reference sections herein.

This Specification SHALL serve as the single source of truth.

---

### 12.5 Conflict Resolution Rule

If any contradiction arises between:

- Design and Specification,
- Implementation and Specification,
- Tests and Specification,

This Specification SHALL prevail.

---

### 12.6 Version Isolation Statement

This document defines exclusively:

Issuance Contract v1.0

Any modification to:

- Arithmetic model,
- Reward formula,
- Lifecycle semantics,
- Escrow structure,
- Authority model,

SHALL require a new version identifier and a separate specification.

---

### 12.7 Deterministic Closure Statement

If Sections 1–12 are implemented exactly:

Then the contract SHALL be:

- Deterministic
- Immutable post-deployment
- Reserve-bounded
- Proportionally fair
- Escrow-secure
- Mathematically sound
- Structurally isolated
- Publicly auditable

Specification scope is formally closed.