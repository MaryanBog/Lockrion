# Issuance-Contract-Design-V1.0-EN
## Section 1 — Architectural Overview

### 1.1 Design Objective

This Design Document defines the internal architecture of the Lockrion Issuance Contract v1.0.

It translates the Specification into:

- Concrete account architecture
- PDA derivation model
- Instruction flow structure
- Internal module separation
- State transition control structure
- Arithmetic execution flow

No behavior beyond the Specification SHALL be introduced.

---

### 1.2 Architectural Principles

The contract architecture SHALL follow these principles:

1. Immutable after deployment
2. Deterministic execution
3. Strict escrow isolation
4. Accumulator-based accounting
5. Constant-time instruction complexity
6. Explicit authority validation
7. No global iteration
8. No dynamic allocation

Design SHALL enforce structural constraints defined in Specification.

---

### 1.3 Program Structure

The program SHALL consist of:

1. Entry module
2. Instruction dispatcher
3. State validation module
4. Accumulator module
5. Settlement module
6. Escrow transfer module
7. Invariant validation layer

Each module SHALL be logically isolated.

---

### 1.4 High-Level Execution Flow

For any instruction, execution SHALL follow a strict pipeline.

All state mutations SHALL occur in memory only until invariant verification
completes successfully.

No account data SHALL be committed before invariants pass.

Canonical execution pipeline:

1. Validate account ownership (program accounts and SPL token accounts).
2. Validate PDA derivation for all required PDAs.
3. Validate signer authority requirements.
4. Validate lifecycle phase eligibility for the instruction.
5. Validate arithmetic preconditions (overflow / division-by-zero impossibility).
6. Update accumulators (if required by the instruction).
7. Execute core instruction logic (in-memory state mutation only).
8. Execute escrow transfer (if required).
9. Verify full invariant set (conservation, monotonicity, exclusivity, immutability).
10. Commit state (write updated state back to account data).

If any step fails:

- Execution SHALL abort.
- No account data SHALL be written.
- No token transfer SHALL persist.
- All invariants SHALL remain intact.

This flow SHALL be followed by every handler with no bypass path.

---

### 1.5 Account Architecture

Per issuance, the design SHALL include:

- Global State Account
- Deposit Escrow PDA (SPL Token account)
- Reward Escrow PDA (SPL Token account)

Per participant:

- User State Account

No additional account types SHALL exist.

---

### 1.6 PDA Derivation Scheme (Deterministic Model)

All Program Derived Addresses (PDAs) SHALL be derived
directly and deterministically from immutable issuance parameters.

The Global State PDA SHALL be derived from:

Seeds:
- b"issuance"
- issuer_address
- lock_mint
- start_ts (little-endian bytes)

No intermediate hash-based seed SHALL be used.

All other PDAs SHALL be derived using the Global State PDA
as issuance anchor:

Escrow Authority PDA:
- b"escrow_authority"
- global_state_pubkey

Deposit Escrow PDA:
- b"deposit_escrow"
- global_state_pubkey

Reward Escrow PDA:
- b"reward_escrow"
- global_state_pubkey

User State PDA:
- b"user_state"
- global_state_pubkey
- user_pubkey

This derivation model guarantees:

- Deterministic uniqueness per issuance
- Cross-issuance isolation
- Stable seed ordering
- No dependency on mutable state
- No dependency on runtime values beyond immutable parameters

Seed literals SHALL remain frozen for v1.0.

Any change to seed ordering or composition SHALL require
a new contract version and redeployment.

---

### 1.7 Module Responsibilities

Entry Module:
- Receives instruction data
- Routes to dispatcher

Dispatcher:
- Maps instruction enum → handler

Validation Module:
- Verifies account ownership
- Verifies PDA derivation
- Verifies signer roles
- Verifies lifecycle constraints

Accumulator Module:
- Updates global weight
- Updates user weight
- Enforces monotonicity

Settlement Module:
- Computes reward
- Enforces claim window
- Enforces terminal path exclusivity

Escrow Transfer Module:
- Performs SPL transfers
- Verifies mint correctness
- Prevents unauthorized movement

Invariant Layer:
- Validates conservation equations
- Validates monotonic properties
- Validates mutual exclusivity

---

### 1.8 State Transition Control Model

The lifecycle SHALL be encoded as implicit state derived from:

- block_timestamp
- reserve_funded
- total_weight_accum
- sweep_executed
- reclaim_executed

No explicit enum-based lifecycle variable SHALL exist.

Lifecycle phase SHALL be derived from conditions.

---

### 1.9 Arithmetic Design Model

All arithmetic SHALL:

- Use u128 for financial values
- Use u64 for day indices
- Multiply before divide
- Use checked operations
- Abort on overflow

Arithmetic operations SHALL be encapsulated in utility functions.

---

### 1.10 Deterministic Execution Guarantee

Design SHALL ensure:

- No randomness
- No floating-point
- No branching on non-deterministic state
- No reliance on instruction ordering within same accounting day

All instruction handlers SHALL be pure state-transition functions.

---

### 1.11 Terminal State Enforcement

Design SHALL enforce:

- Only one terminal reward path reachable
- Post-terminal instructions revert
- No state reactivation possible
- Reward escrow balance reaches zero at termination

Terminal state SHALL be structurally enforced, not conditionally patched.

---

### 1.12 Isolation Model

Each issuance SHALL:

- Have independent Global State Account
- Have independent escrow PDAs
- Have independent user accounts
- Not reference external issuance state

Isolation SHALL be guaranteed at account level.

---

## Section 2 — Account Layout and Data Structures

### 2.1 Global State Layout

The Global State Account SHALL be a fixed-size structure.

Logical field order:

1. issuer_address : Pubkey
2. lock_mint : Pubkey
3. reward_mint : Pubkey
4. platform_treasury : Pubkey
5. reserve_total : u128
6. start_ts : i64
7. maturity_ts : i64
8. claim_window : i64
9. accounting_period : u64 (constant 86400)
10. final_day_index : u64
11. reserve_funded : bool
12. sweep_executed : bool
13. reclaim_executed : bool
14. total_locked : u128
15. total_weight_accum : u128
16. last_day_index : u64

Global State SHALL NOT contain variable-length fields.

Immutable fields MUST NOT be modified after initialization.

---

### 2.2 User State Layout

The User State Account SHALL be a fixed-size structure.

Logical field order:

1. owner : Pubkey
2. locked_amount : u128
3. user_weight_accum : u128
4. user_last_day_index : u64
5. reward_claimed : bool

User State SHALL NOT contain variable-length fields.

owner MUST match transaction signer.

---

### 2.3 Escrow Token Accounts

Deposit Escrow PDA token account:

- SPL Token account
- mint == lock_mint
- account.owner == SPL Token Program
- authority == Escrow Authority PDA (program-controlled)

Reward Escrow PDA token account:

- SPL Token account
- mint == reward_mint (USDC)
- account.owner == SPL Token Program
- authority == Escrow Authority PDA (program-controlled)

Escrow token accounts SHALL be created during initialize().

The Escrow Authority PDA SHALL be the only valid signing authority
for any outgoing transfer from escrow accounts via invoke_signed.

No externally controlled keypair SHALL be able to act as escrow authority.

---

### 2.4 Authority and Ownership Rules

Ownership constraints:

- Global State owner MUST be the program.
- User State owner MUST be the program.
- Escrow token accounts MUST be controlled by program-derived authority.

Signer constraints:

- fund_reserve() requires issuer_address signer.
- zero_participation_reclaim() requires issuer_address signer.
- deposit(), claim_reward(), withdraw_deposit() require user signer.
- sweep() requires no special signer beyond transaction signer.

All authority SHALL be validated via explicit checks.

---

### 2.5 PDA Validation Data Model

For each instruction, the program SHALL validate:

- Global State PDA matches expected derivation.
- Deposit Escrow PDA matches expected derivation.
- Reward Escrow PDA matches expected derivation.
- User State PDA matches expected derivation (for user instructions).

PDA validation SHALL occur before any state mutation.

---

### 2.6 Serialization and Determinism

All state accounts SHALL use:

- Fixed serialization format
- Explicit field ordering
- Stable type widths
- Deterministic encoding

No platform-dependent serialization SHALL be used.

---

### 2.7 Account Initialization Strategy

initialize() SHALL:

- Allocate Global State Account at exact fixed size.
- Create deposit escrow token account.
- Create reward escrow token account.
- Store immutable parameters.
- Initialize mutable variables to zero defaults.

User State accounts SHALL be created lazily on first deposit.

---

### 2.8 Data Separation Guarantees

The design SHALL ensure:

- No cross-user data stored in Global State.
- No participant list stored on-chain.
- No global arrays.
- No dynamic collection of users.

All participant data SHALL be stored per-user.

---

### 2.9 State Mutation Constraints

Only these instructions SHALL mutate Global State:

- fund_reserve()
- deposit()
- claim_reward() (accumulator updates only)
- sweep()
- zero_participation_reclaim()

Only these instructions SHALL mutate User State:

- deposit()
- claim_reward()
- withdraw_deposit()

No other mutation path SHALL exist.

---

### 2.10 Derived Values

The following are derived at runtime and NOT stored:

- current_day_index
- raw_day_index
- days_elapsed
- computed_reward

Derived values SHALL be computed deterministically from stored state and block_timestamp.

---

## Section 3 — Instruction Dispatch and Handler Architecture

### 3.1 Instruction Enum Design

The program SHALL define a fixed instruction enum:

1. Initialize
2. FundReserve
3. Deposit { amount: u128 }
4. ClaimReward
5. WithdrawDeposit
6. Sweep
7. ZeroParticipationReclaim

No additional instruction variants SHALL exist.

Instruction decoding SHALL be:

- Deterministic
- Fixed-width
- Free of variable-length payloads
- Stable across all nodes

The Deposit amount SHALL use u128 to maintain
type consistency with:

- reserve_total
- locked_amount
- total_locked
- user_weight_accum
- total_weight_accum

No implicit widening conversions SHALL occur
inside handlers.

All financial quantities SHALL use u128 uniformly.

---

### 3.2 Entry Point Structure

The entrypoint SHALL:

1. Parse instruction data.
2. Deserialize instruction enum.
3. Route to dispatcher.
4. Return result or error.

Entrypoint SHALL NOT:

- Perform business logic.
- Perform state mutation.
- Perform arithmetic.

Entrypoint SHALL only coordinate dispatch.

---

### 3.3 Dispatcher Model

The dispatcher SHALL:

- Match instruction enum.
- Call corresponding handler function.
- Enforce instruction-to-account mapping expectations.
- Ensure handler isolation.

Each handler SHALL be independent.

---

### 3.4 Handler Structure Template

Each instruction handler SHALL follow the exact execution model below.

All state mutations SHALL occur in memory only
until invariant verification completes.

No account data SHALL be committed before invariant validation succeeds.

Handler execution pipeline:

1. Account ownership validation
2. PDA derivation validation
3. Signer validation
4. Lifecycle phase validation
5. Arithmetic precondition validation
6. Accumulator updates (if required)
7. Core business logic (in-memory state mutation only)
8. Escrow transfer (if required)
9. Invariant verification (full invariant set)
10. State commit (write-back to account data)

Clarifications:

- Steps 6–7 may mutate in-memory representations of Global State
  and User State, but SHALL NOT write to account data yet.

- Escrow transfers SHALL execute before state commit,
  but after in-memory state mutation.

- Invariant verification SHALL validate:
  - Deposit conservation
  - Reward conservation
  - Monotonicity constraints
  - Mutual exclusivity
  - Immutability constraints

- Only after all invariants pass,
  updated state SHALL be written back to account data.

If any step fails:

- Execution SHALL abort.
- No account data SHALL be written.
- No token transfer SHALL persist.
- All invariants SHALL remain intact.

There SHALL be no intermediate partial commits.

State commit SHALL occur exactly once per successful instruction
and only after invariant validation succeeds.

---

### 3.5 Initialize Handler Architecture

initialize():

- Validate parameters.
- Derive PDAs.
- Allocate and write Global State.
- Create escrow token accounts.
- Compute final_day_index.
- Set reserve_funded = false.
- Set accumulators to zero.

Initialize SHALL NOT:

- Accept reserve funding.
- Create user accounts.
- Accept deposits.

---

### 3.6 FundReserve Handler Architecture

fund_reserve():

- Validate issuer signer.
- Validate reward escrow PDA.
- Validate zero balance.
- Validate amount == reserve_total.
- Execute SPL transfer.
- Set reserve_funded = true.

No accumulator logic SHALL execute.

---

### 3.7 Deposit Handler Architecture

deposit():

- Validate lifecycle window.
- Validate reserve_funded.
- Validate lock_mint.
- Update global accumulator.
- Update user accumulator.
- Execute SPL transfer to deposit escrow.
- Increase locked_amount.
- Increase total_locked.

Deposit SHALL NOT:

- Modify immutable parameters.
- Affect reward escrow.

---

### 3.8 ClaimReward Handler Architecture

claim_reward():

- Validate claim window.
- Validate reward_claimed == false.
- Update global accumulator to final_day_index.
- Update user accumulator to final_day_index.
- Compute reward deterministically.
- Execute SPL transfer from reward escrow.
- Set reward_claimed = true.

Claim SHALL NOT:

- Modify locked_amount.
- Modify total_locked.

---

### 3.9 WithdrawDeposit Handler Architecture

withdraw_deposit():

- Validate maturity reached.
- Validate locked_amount > 0.

Accumulator finalization requirement:

- Update global accumulator to final_day_index.
- Update user accumulator to final_day_index.

Execution steps:

1. update_global_accumulator() with block_timestamp (finalizes at maturity).
2. update_user_accumulator() (finalizes at maturity).
3. Execute SPL transfer from deposit escrow to user token account.
4. Set locked_amount = 0.
5. Decrease total_locked by withdrawn amount.

Withdraw SHALL NOT:

- Affect reward escrow.
- Perform reward computation.
- Modify reward_claimed flag.

Postconditions:

- Deposit custody remains conserved.
- Accumulators remain frozen post-maturity.
- Withdrawal remains independent from reward settlement path.

---

### 3.10 Sweep Handler Architecture

sweep():

- Validate claim window ended.
- Validate total_weight_accum > 0.
- Validate not executed.
- Transfer entire reward escrow balance.
- Set sweep_executed = true.

Sweep SHALL NOT:

- Modify user states.
- Modify accumulators.

---

### 3.11 ZeroParticipationReclaim Handler Architecture

zero_participation_reclaim():

- Validate maturity reached.
- Validate total_weight_accum == 0.
- Validate issuer signer.
- Transfer entire reward escrow balance.
- Set reclaim_executed = true.

Reclaim SHALL disable sweep permanently.

---

### 3.12 Invariant Enforcement Layer

After each handler:

- Validate conservation invariants.
- Validate monotonic invariants.
- Validate mutual exclusivity.
- Validate escrow balances.

If any invariant fails, instruction SHALL revert.

---

### 3.13 Error Propagation Model

Each handler SHALL:

- Return explicit error codes.
- Avoid panic.
- Avoid partial state writes.
- Rely on Solana atomic rollback.

Error propagation SHALL be deterministic.

---

### 3.14 Isolation Guarantee

Handlers SHALL NOT:

- Call other handlers recursively.
- Share mutable static state.
- Depend on global mutable variables.
- Use unsafe memory operations.

Instruction logic SHALL remain modular and isolated.

---

## Section 4 — Accumulator Engine Architecture

### 4.1 Accumulator Engine Purpose

The Accumulator Engine SHALL:

- Maintain discrete-time weight accounting.
- Enforce monotonicity.
- Prevent fractional-day participation.
- Guarantee deterministic finalization at maturity.

The engine SHALL operate identically across all instructions requiring state update.

---

### 4.2 Global Accumulator Update Function

The design SHALL include an internal function:

update_global_accumulator(global_state, block_timestamp)

This function SHALL:

1. Compute current_day_index.
2. Compute days_elapsed = current_day_index - last_day_index.
3. If days_elapsed > 0:
   total_weight_accum += total_locked × days_elapsed
   last_day_index = current_day_index

This function SHALL:

- Abort on overflow.
- Never decrease last_day_index.
- Stop accumulation at final_day_index.

---

### 4.3 User Accumulator Update Function

The design SHALL include:

update_user_accumulator(user_state, global_state)

This function SHALL:

1. Compute current_day_index (derived from global).
2. Compute days_elapsed_user = current_day_index - user_last_day_index.
3. If days_elapsed_user > 0:
   user_weight_accum += locked_amount × days_elapsed_user
   user_last_day_index = current_day_index

The function SHALL:

- Abort on overflow.
- Never decrease user_last_day_index.
- Stop accumulation at final_day_index.

---

### 4.4 Accumulator Invocation Rules

The following instructions SHALL invoke update_global_accumulator:

- deposit()
- claim_reward()
- withdraw_deposit()

The following instructions SHALL invoke update_user_accumulator:

- deposit()
- claim_reward()
- withdraw_deposit()

Rationale:

- deposit() requires accumulator updates prior to increasing locked_amount and total_locked.
- claim_reward() requires finalization to final_day_index prior to reward computation.
- withdraw_deposit() requires finalization to final_day_index to preserve the
  post-maturity freeze guarantees and ensure accounting stability independent of claim order.

The following instructions SHALL NOT invoke accumulator updates:

- sweep()
- zero_participation_reclaim()

Because they do not depend on per-user accounting values and SHALL NOT
perform reward computation.

Accumulator updates SHALL remain idempotent, and post-maturity calls SHALL
produce zero additional accumulation due to final_day_index clamping.

---

### 4.5 Order of Operations Constraint

For deposit():

1. update_global_accumulator()
2. update_user_accumulator()
3. Increase locked_amount
4. Increase total_locked

Weight SHALL reflect participation only for completed days.

---

### 4.6 Finalization at Maturity

If block_timestamp ≥ maturity_ts:

- current_day_index SHALL equal final_day_index.
- Accumulators SHALL finalize permanently.
- Further calls to update functions SHALL produce zero days_elapsed.

Accumulator engine SHALL be idempotent post-maturity.

---

### 4.7 Overflow Protection Model

All accumulator arithmetic SHALL:

- Use checked u128 multiplication.
- Use checked u128 addition.
- Abort on overflow.
- Prevent silent wraparound.

Overflow SHALL terminate instruction before state commit.

---

### 4.8 Monotonicity Enforcement

Accumulator engine SHALL enforce:

- last_day_index ≤ final_day_index
- user_last_day_index ≤ final_day_index
- total_weight_accum monotonic
- user_weight_accum monotonic

Decrement operations SHALL NOT exist.

---

### 4.9 Isolation of Accumulator Logic

Accumulator engine SHALL:

- Contain no escrow logic.
- Contain no reward logic.
- Contain no authorization logic.
- Contain no lifecycle branching beyond maturity boundary.

Accumulator SHALL remain pure accounting module.

---

### 4.10 Deterministic Recalculation Property

Given:

- Identical deposit timeline
- Identical block_timestamp history

The accumulator engine SHALL produce identical:

- total_weight_accum
- user_weight_accum

No dependency on execution order within same accounting day SHALL exist.

---

### 4.11 Zero Participation Safety

If total_locked remains zero for entire lifecycle:

- total_weight_accum SHALL remain zero.
- No division SHALL occur.
- Accumulator SHALL remain consistent.

Accumulator SHALL safely handle empty participation.

---

### 4.12 Idempotency Guarantee

Multiple calls within same accounting day SHALL:

- Produce days_elapsed = 0.
- Produce no accumulator mutation.
- Preserve determinism.

Accumulator engine SHALL be idempotent per day.

---

## Section 5 — Escrow Transfer Architecture

### 5.1 Escrow Transfer Module Purpose

The Escrow Transfer Module SHALL:

- Execute SPL token transfers.
- Enforce mint correctness.
- Enforce escrow authority.
- Prevent unauthorized token movement.
- Maintain conservation invariants.

All token transfers SHALL pass exclusively through this module.

---

### 5.2 Escrow Authority Model

Escrow token accounts SHALL be controlled by:

- A program-derived authority (Escrow Authority PDA).

The Escrow Authority PDA SHALL:

- Sign transfers via program logic.
- Be derived deterministically from issuance seed.
- Not be externally accessible.

No direct external signing SHALL be possible.

---

### 5.3 Deposit Transfer Flow

deposit():

1. Validate lock_mint.
2. Validate user token account.
3. Validate deposit escrow PDA.
4. Execute SPL transfer:
   from user token account
   to deposit escrow PDA
5. Verify transfer success before state update.

State mutation SHALL occur only after transfer success.

---

### 5.4 Reward Claim Transfer Flow

claim_reward():

1. Validate reward_mint.
2. Validate reward escrow PDA.
3. Validate user reward token account.
4. Compute reward.
5. Execute SPL transfer:
   from reward escrow PDA
   to user token account
6. Verify transfer success before setting reward_claimed.

Reward SHALL NOT be marked claimed before transfer success.

---

### 5.5 Sweep Transfer Flow

sweep():

1. Validate reward escrow PDA.
2. Validate platform_treasury token account.
3. Read full reward escrow balance.
4. Execute SPL transfer:
   from reward escrow PDA
   to platform_treasury
5. Set sweep_executed only after transfer success.

Sweep SHALL transfer entire remaining balance.

---

### 5.6 Zero Participation Reclaim Flow

zero_participation_reclaim():

1. Validate issuer signer.
2. Validate reward escrow PDA.
3. Validate issuer reward token account.
4. Read full reward escrow balance.
5. Execute SPL transfer:
   from reward escrow PDA
   to issuer reward account
6. Set reclaim_executed only after transfer success.

Reclaim SHALL transfer entire remaining balance.

---

### 5.7 Withdrawal Transfer Flow

withdraw_deposit():

1. Validate deposit escrow PDA.
2. Validate user token account.
3. Read locked_amount.
4. Execute SPL transfer:
   from deposit escrow PDA
   to user token account
5. Set locked_amount = 0 only after transfer success.

Withdrawal SHALL be all-or-nothing.

---

### 5.8 Transfer Validation Rules

Before every SPL token transfer, the following validations SHALL occur.

For escrow token accounts (Deposit and Reward):

1. account.owner MUST equal the SPL Token Program.
2. token_account.mint MUST match expected mint:
   - lock_mint for Deposit Escrow
   - reward_mint for Reward Escrow
3. token_account.authority MUST equal Escrow Authority PDA.
4. Escrow Authority PDA MUST match deterministic derivation.

For non-escrow token accounts (user accounts, treasury account):

1. token_account.mint MUST match expected mint.
2. token_account.owner MUST equal SPL Token Program.

The program SHALL NOT:

- Assume program ownership of SPL token accounts.
- Treat account.owner as authority.
- Permit external keypair control over escrow.

Any mismatch in:

- account.owner
- token_account.mint
- token_account.authority
- PDA derivation

SHALL cause immediate instruction rejection before any state mutation.

Escrow control SHALL derive strictly from:

- SPL Token Program ownership model,
- Escrow Authority PDA as token authority,
- Deterministic PDA validation on every instruction.

---

### 5.9 Conservation Enforcement

The Design SHALL enforce conservation using structural escrow invariants only.

No aggregate reward distribution totals SHALL be stored.

The following variables SHALL NOT exist in Global State:

- claimed_rewards_total
- swept_amount
- reclaimed_amount
- distributed_reward_total

Deposit Conservation SHALL be enforced as:

Σ locked_amount_i = deposit_escrow_balance
AND
total_locked = deposit_escrow_balance

Reward Conservation SHALL be enforced as:

reward_escrow_balance ≤ reserve_total
reward_escrow_balance ≥ 0

Escrow Structure Requirements:

For escrow token accounts:

1. account.owner MUST equal SPL Token Program.
2. token_account.authority MUST equal Escrow Authority PDA.
3. token_account.mint MUST match expected mint.

Outgoing reward transfers SHALL be permitted ONLY via:

- claim_reward()
- sweep()
- zero_participation_reclaim()

Each reward transfer SHALL:

1. Read current reward_escrow_balance.
2. Compute deterministic transfer amount.
3. Verify transfer_amount ≤ reward_escrow_balance.
4. Abort on violation.

At Terminal State:

reward_escrow_balance SHALL equal 0.

Conservation SHALL derive from:

- Escrow isolation via PDA authority,
- Strict instruction routing,
- Atomic SPL transfer semantics,
- Absence of mint/burn logic,
- Absence of aggregate distribution counters.

No invariant SHALL rely on stored cumulative reward totals.

---

### 5.10 Atomicity Requirement

Escrow transfers SHALL:

- Occur within single instruction execution.
- Abort fully if transfer fails.
- Never partially transfer.
- Never partially mutate state.

Atomic rollback SHALL preserve invariants.

---

### 5.11 No External Escrow Access

The design SHALL ensure:

- No external instruction can move escrow funds.
- No instruction bypasses Escrow Transfer Module.
- No alternative token movement path exists.
- No CPI call modifies escrow unexpectedly.

Escrow movement SHALL be single-path controlled.

---

### 5.12 Deterministic Transfer Semantics

Given identical:

- Escrow balance
- User state
- Instruction input

Transfer outcomes SHALL be identical across all nodes.

No nondeterministic branch SHALL affect token movement.

---

## Section 6 — Lifecycle Control and Phase Derivation

### 6.1 Implicit Lifecycle Model

The contract SHALL NOT store an explicit lifecycle enum.

Lifecycle phase SHALL be derived from:

- reserve_funded
- block_timestamp
- start_ts
- maturity_ts
- claim_window
- total_weight_accum
- sweep_executed
- reclaim_executed

Lifecycle SHALL be computed conditionally.

---

### 6.2 Lifecycle Phases

The following logical phases SHALL exist:

1. Pre-Funding Phase
2. Funding Phase
3. Participation Phase
4. Maturity Phase
5. Claim Phase
6. Terminal Phase

Transitions SHALL be deterministic and irreversible.

---

### 6.3 Pre-Funding Phase

Conditions:

- reserve_funded == false
- block_timestamp < start_ts

Allowed instructions:

- fund_reserve()

Disallowed:

- deposit()
- claim_reward()
- withdraw_deposit()
- sweep()
- zero_participation_reclaim()

---

### 6.4 Participation Phase

Conditions:

- reserve_funded == true
- start_ts ≤ block_timestamp < maturity_ts

Allowed instructions:

- deposit()

Disallowed:

- claim_reward()
- withdraw_deposit()
- sweep()
- zero_participation_reclaim()

---

### 6.5 Maturity Phase

Condition:

- block_timestamp ≥ maturity_ts

At first entry:

- Accumulators SHALL finalize to final_day_index.

Allowed instructions:

- claim_reward() (if within claim window)
- withdraw_deposit()
- zero_participation_reclaim() (if total_weight_accum == 0)

Disallowed:

- deposit()

---

### 6.6 Claim Phase

Conditions:

- maturity_ts ≤ block_timestamp < maturity_ts + claim_window
- total_weight_accum > 0

Allowed instructions:

- claim_reward()
- withdraw_deposit()

Disallowed:

- deposit()
- sweep()
- zero_participation_reclaim()

---

### 6.7 Post-Claim Terminal Phase

Condition:

- block_timestamp ≥ maturity_ts + claim_window

If total_weight_accum > 0:

Allowed:

- sweep()
- withdraw_deposit()

If total_weight_accum == 0:

Allowed:

- zero_participation_reclaim()
- withdraw_deposit()

No other reward transfer SHALL be possible.

---

### 6.8 Terminal State Determination

Terminal state SHALL be reached when:

- sweep_executed == true
OR
- reclaim_executed == true

After terminal state:

- reward_escrow_balance == 0
- claim_reward() SHALL revert
- sweep() SHALL revert
- zero_participation_reclaim() SHALL revert

Withdrawal SHALL remain allowed.

---

### 6.9 Irreversibility Guarantees

The lifecycle SHALL NOT:

- Transition backward.
- Reset reserve_funded.
- Reset sweep_executed.
- Reset reclaim_executed.
- Reactivate deposit phase.

All phase transitions SHALL be one-directional.

---

### 6.10 Mutual Exclusivity Enforcement

Design SHALL enforce:

If total_weight_accum == 0:
  sweep() permanently disabled.

If total_weight_accum > 0:
  zero_participation_reclaim() permanently disabled.

Both terminal reward paths SHALL NEVER be reachable.

---

### 6.11 Phase Validation Layer

Each instruction handler SHALL:

- Derive current lifecycle phase.
- Validate allowed instruction set.
- Reject invalid phase-instruction combinations.

Phase validation SHALL occur before accumulator updates.

---

### 6.12 Lifecycle Determinism

Given identical:

- start_ts
- maturity_ts
- claim_window
- block_timestamp history

Lifecycle phase transitions SHALL be identical across all nodes.

No nondeterministic branching SHALL exist.

---

### 6.13 Phase Independence from User Behavior

Lifecycle SHALL depend ONLY on:

- block_timestamp
- reserve_funded
- total_weight_accum
- terminal flags

Lifecycle SHALL NOT depend on:

- number of participants
- deposit size
- claim order
- withdrawal order

Lifecycle SHALL be globally deterministic.

---

## Section 7 — Validation Layer and Precondition Enforcement

### 7.1 Validation Layer Purpose

The Validation Layer SHALL enforce all structural and semantic constraints
before any state mutation or escrow transfer occurs.

For every instruction execution, the Validation Layer SHALL:

1. Enforce all instruction preconditions.
2. Enforce signer constraints.
3. Enforce PDA correctness.
4. Enforce lifecycle phase rules.
5. Enforce arithmetic safety.
6. Enforce immutable field revalidation.

Immutable Field Revalidation Requirement:

For every instruction that depends on externally provided accounts,
the program SHALL validate that immutable Global State fields
match the provided accounts exactly.

The following SHALL be revalidated on every instruction:

- lock_mint
- reward_mint
- issuer_address
- platform_treasury

Specifically:

- Provided lock_mint account MUST equal stored lock_mint.
- Provided reward_mint account MUST equal stored reward_mint.
- Provided issuer account MUST equal stored issuer_address (if required by instruction).
- Provided platform treasury account MUST equal stored platform_treasury (for sweep()).

No instruction SHALL:

- Accept alternative mint accounts.
- Accept alternative treasury.
- Accept alternative issuer.
- Derive behavior from mismatched immutable parameters.

Immutable fields SHALL:

- Be written only during initialize().
- Never be reassigned.
- Be validated structurally on every instruction.

If any immutable mismatch is detected:

- Instruction SHALL abort immediately.
- No state mutation SHALL occur.
- No escrow transfer SHALL occur.

The Validation Layer SHALL execute entirely before accumulator updates
and before any in-memory state mutation.

No instruction SHALL bypass this validation layer.

---

### 7.2 Account Ownership and Authority Validation

For every instruction, the program SHALL validate ownership and authority
semantics exactly as defined by the SPL Token model.

Global State Account:
- account.owner MUST equal program_id.

User State Account:
- account.owner MUST equal program_id.

Escrow Token Accounts (Deposit and Reward):

For each escrow token account:

1. account.owner MUST equal SPL Token Program.
2. token_account.mint MUST equal the expected mint:
   - lock_mint for Deposit Escrow
   - reward_mint for Reward Escrow
3. token_account.authority MUST equal Escrow Authority PDA.

Escrow Authority PDA:

- MUST be derived deterministically from Global State PDA.
- MUST NOT be externally signable.
- MUST only sign transfers via invoke_signed within program logic.

The program SHALL NOT assume:

- Program ownership of token accounts.
- Implicit authority.
- External keypair control over escrow.

Any mismatch in:

- account.owner
- token_account.mint
- token_account.authority
- PDA derivation

SHALL cause immediate instruction rejection before any state mutation.

Escrow security SHALL derive strictly from:

- SPL Token account ownership by the Token Program,
- Authority assignment to Escrow Authority PDA,
- Deterministic PDA validation on every instruction.

---

### 7.3 PDA Derivation Validation

The program SHALL verify for each provided PDA:

- Seeds match expected pattern.
- Bump seed matches derived PDA.
- Account public key equals derived address.

Validation SHALL occur before any state mutation.

---

### 7.4 Signer Validation

The Validation Layer SHALL enforce:

- fund_reserve() requires issuer_address signer.
- zero_participation_reclaim() requires issuer_address signer.
- deposit() requires user signer.
- claim_reward() requires user signer.
- withdraw_deposit() requires user signer.
- sweep() requires no privileged signer.

Missing required signer SHALL abort execution.

---

### 7.5 Mint Validation

Before any SPL transfer:

- Source mint MUST match expected mint.
- Destination mint MUST match expected mint.
- lock_mint and reward_mint MUST match Global State.

Mint mismatch SHALL abort execution.

---

### 7.6 Lifecycle Phase Validation

Each instruction SHALL validate allowed phase:

- deposit() allowed only during Participation Phase.
- claim_reward() allowed only during Claim Phase.
- withdraw_deposit() allowed only at or after Maturity.
- sweep() allowed only Post-Claim.
- zero_participation_reclaim() allowed only if total_weight_accum == 0 and at Maturity or later.

Invalid phase SHALL abort execution.

---

### 7.7 Arithmetic Precondition Validation

Before performing arithmetic:

- Verify operands within bounds.
- Verify no division by zero possible.
- Verify multiplication will not overflow.
- Verify addition will not overflow.

Failure SHALL abort execution before state write.

---

### 7.8 Accumulator Preconditions

Before updating accumulators:

- Ensure current_day_index ≥ last_day_index.
- Ensure current_day_index ≤ final_day_index.
- Ensure user_last_day_index ≤ final_day_index.

Invalid index state SHALL abort execution.

---

### 7.9 Terminal Path Validation

Before executing terminal instructions:

If sweep():

- total_weight_accum > 0
- sweep_executed == false
- reclaim_executed == false

If zero_participation_reclaim():

- total_weight_accum == 0
- reclaim_executed == false
- sweep_executed == false

Violation SHALL abort execution.

---

### 7.10 Invariant Pre-Commit Validation

Before final state commit:

- Validate deposit conservation equation.
- Validate reward conservation equation.
- Validate monotonicity invariants.
- Validate mutual exclusivity flags.
- Validate immutability constraints.

If any invariant fails, instruction SHALL revert.

---

### 7.11 Validation Ordering Guarantee

Validation SHALL occur in the following order:

1. Account ownership
2. PDA derivation
3. Signer checks
4. Lifecycle phase checks
5. Arithmetic bounds
6. Accumulator update
7. Core logic
8. Escrow transfer
9. Invariant verification
10. State commit

No state mutation SHALL occur before step 6.

---

### 7.12 Deterministic Rejection Guarantee

Given identical:

- Input accounts
- Instruction data
- block_timestamp
- On-chain state

Validation outcome SHALL be identical across all nodes.

Rejection SHALL be deterministic and reproducible.

---

### 7.13 No Silent Failure

The program SHALL:

- Return explicit error codes.
- Avoid panic.
- Avoid undefined behavior.
- Avoid partial writes.
- Avoid unchecked unwrap operations.

All failure paths SHALL be explicit and controlled.

---

## Section 8 — Determinism, Complexity, and Resource Model

### 8.1 Deterministic Execution Architecture

The contract SHALL guarantee that:

- Execution depends only on instruction input, on-chain state, and block_timestamp.
- No randomness exists.
- No floating-point arithmetic exists.
- No external data source exists.
- No hidden global mutable state exists.

All handlers SHALL behave as pure state-transition functions.

---

### 8.2 Instruction Complexity Constraints

Each instruction SHALL execute in O(1) time.

The design SHALL ensure:

- No iteration over participants.
- No iteration over deposits.
- No iteration over claim history.
- No scanning of historical logs.

All aggregate values SHALL be accumulator-based.

---

### 8.3 State Growth Model

State SHALL scale linearly with participants:

- 1 Global State Account per issuance.
- 1 Deposit Escrow PDA per issuance.
- 1 Reward Escrow PDA per issuance.
- 1 User State Account per participant.

No dynamic list of users SHALL be stored.

No global registry SHALL be stored in this contract.

---

### 8.4 Memory Layout Constraints

The design SHALL:

- Use fixed-size structs.
- Avoid heap allocation.
- Avoid dynamic vectors.
- Avoid variable-length fields.
- Avoid runtime resizing.

Account size SHALL be static and predictable.

---

### 8.5 Idempotency Properties

Within the same accounting day:

- Repeated calls to accumulator update SHALL produce zero change.
- Order of deposit() calls SHALL NOT affect weight.
- Order of claim_reward() calls SHALL NOT affect final proportional outcome.

Instruction handlers SHALL be idempotent within same day boundaries.

---

### 8.6 Execution Order Independence

If two valid transactions occur within the same accounting day:

Final state SHALL be independent of:

- Transaction ordering.
- Deposit order.
- Claim order.
- Withdrawal order.

Weight accumulation SHALL depend only on completed days.

---

### 8.7 Overflow Safety Model

All financial arithmetic SHALL:

- Use u128.
- Use checked_add.
- Use checked_mul.
- Use checked_sub.
- Abort on overflow.

No wrapping arithmetic SHALL be used.

---

### 8.8 Block Timestamp Dependence

The only time dependency SHALL be:

block_timestamp provided by Solana runtime.

The design SHALL:

- Not accept user-provided timestamps.
- Not use slot number.
- Not adjust timestamps manually.

Minor block drift SHALL NOT affect fairness due to discrete day model.

---

### 8.9 Deterministic Reward Computation

Given identical:

- reserve_total
- total_weight_accum
- user_weight_accum

Reward computation SHALL always produce identical result:

reward = floor(reserve_total × user_weight_accum / total_weight_accum)

No rounding ambiguity SHALL exist.

---

### 8.10 Atomic Rollback Guarantee

If any error occurs:

- Entire instruction SHALL revert.
- No state mutation SHALL persist.
- No token transfer SHALL persist.

Atomicity SHALL be guaranteed by Solana runtime.

---

### 8.11 Resource Exhaustion Resistance

The contract SHALL:

- Avoid state proportional to participation history.
- Avoid loops over participant count.
- Avoid cumulative gas growth per participant.
- Avoid storing historical event logs.

Resource usage SHALL remain bounded per instruction.

---

### 8.12 Deterministic Replay Property

Given identical transaction history:

Replaying the entire issuance SHALL reproduce identical:

- total_weight_accum
- user_weight_accum
- reward distribution
- terminal balances

Determinism SHALL be verifiable via chain replay.

---

## Section 9 — Terminal State Architecture and Finality Enforcement

### 9.1 Terminal State Definition

The issuance SHALL enter Terminal State when either:

- sweep_executed == true
OR
- reclaim_executed == true

Terminal State SHALL represent irreversible reward settlement completion.

---

### 9.2 Terminal Reward Conditions

Upon entering Terminal State:

- reward_escrow_balance SHALL equal 0.
- claim_reward() SHALL revert permanently.
- sweep() SHALL revert permanently.
- zero_participation_reclaim() SHALL revert permanently.

Reward settlement SHALL be closed.

---

### 9.3 Withdrawal Post-Terminal Guarantee

Even after Terminal State:

- withdraw_deposit() SHALL remain available.
- User locked_amount SHALL remain withdrawable.
- No reward logic SHALL depend on withdrawal timing.

Deposit custody SHALL remain independent from reward finality.

---

### 9.4 Mutual Exclusivity Structural Enforcement

The design SHALL enforce:

If total_weight_accum > 0:
  reclaim_executed SHALL NEVER become true.

If total_weight_accum == 0:
  sweep_executed SHALL NEVER become true.

The two terminal reward paths SHALL be structurally exclusive.

---

### 9.5 No Reactivation Constraint

After Terminal State:

The contract SHALL NOT:

- Reset sweep_executed.
- Reset reclaim_executed.
- Reopen claim window.
- Accept deposits.
- Modify reward balances.
- Modify immutable parameters.

Terminal State SHALL be irreversible.

---

### 9.6 Final Accumulator Freeze

Upon maturity and subsequent finalization:

- last_day_index SHALL equal final_day_index.
- user_last_day_index SHALL NOT exceed final_day_index.
- total_weight_accum SHALL remain fixed.
- user_weight_accum SHALL remain fixed.

No further weight accumulation SHALL occur.

---

### 9.7 Terminal Conservation Invariant

At Terminal State:

Deposit conservation SHALL hold:

Σ locked_amount_i = deposit_escrow_balance

And:

deposit_escrow_balance SHALL equal total_locked.

Reward conservation SHALL hold:

reward_escrow_balance = 0

No aggregate reward distribution totals SHALL be assumed or stored.

In particular, the following variables SHALL NOT exist:

- claimed_rewards_total
- swept_amount
- reclaimed_amount
- distributed_reward_total

Terminal conservation SHALL be enforced structurally by:

- Escrow isolation via PDA authority
- Restricting outgoing reward transfers to:
  claim_reward(), sweep(), zero_participation_reclaim()
- Atomic SPL transfer semantics
- Terminal flags preventing further reward movement

This invariant SHALL hold permanently after Terminal State is reached.

---

### 9.8 Final Instruction Rejection Rules

After Terminal State:

deposit() SHALL revert.
claim_reward() SHALL revert.
sweep() SHALL revert.
zero_participation_reclaim() SHALL revert.

Only withdraw_deposit() SHALL remain valid.

---

### 9.9 Terminal Determinism Guarantee

Given identical:

- deposit timeline
- claim behavior
- block_timestamp progression

Terminal State SHALL be identical across all nodes.

No nondeterministic termination SHALL occur.

---

### 9.10 Lifecycle Closure Property

For every issuance:

- Terminal State SHALL be reachable.
- No issuance SHALL remain indefinitely in ambiguous reward state.
- No reward SHALL remain permanently locked beyond claim window.

Lifecycle SHALL be finite and fully closed.

---

### 9.11 Final Structural Integrity Guarantee

At Terminal State:

- No invariant SHALL be violated.
- No escrow imbalance SHALL exist.
- No authority escalation SHALL be possible.
- No state mutation beyond withdrawal SHALL occur.

Finality SHALL be structurally enforced by design.

---

### 9.12 Design Closure Condition

The design of Issuance Contract v1.0 SHALL be considered complete when:

- All account structures defined.
- All instruction flows defined.
- All validation layers defined.
- All accumulator mechanics defined.
- All escrow flows defined.
- All lifecycle transitions defined.
- All invariants structurally enforceable.

No additional architectural component SHALL be required.

---

## Section 10 — Design Conformance and Implementation Interface Boundary

### 10.1 Design-to-Specification Mapping

This Design Document SHALL map directly to:

- Issuance-Contract-Specification-V1.0-EN

For every section in the Specification:

- A corresponding architectural component SHALL exist.
- No undefined behavior SHALL be introduced.
- No extension SHALL exceed Specification boundaries.

Specification SHALL remain the source of truth.

---

### 10.2 Implementation Boundary Definition

The Implementation stage SHALL:

- Translate modules into concrete code.
- Preserve account layout exactly.
- Preserve arithmetic model exactly.
- Preserve instruction semantics exactly.
- Preserve validation ordering exactly.

Implementation SHALL NOT:

- Introduce new instructions.
- Introduce new state fields.
- Modify invariant definitions.
- Alter lifecycle transitions.
- Add economic logic.

---

### 10.3 Static Analysis Interface

The Design SHALL enable Static Analysis to verify:

- No unchecked arithmetic.
- No unsafe memory access.
- No unreachable code.
- No authority bypass.
- No invariant violation path.

Design SHALL be structured to allow formal verification.

---

### 10.4 Test Suite Interface

The Design SHALL support derivation of:

- Unit tests per instruction.
- Lifecycle phase tests.
- Accumulator correctness tests.
- Reward proportionality tests.
- Terminal state tests.
- Failure-path tests.

Each module SHALL be testable in isolation.

---

### 10.5 Compliance Matrix Readiness

The Design SHALL allow direct mapping:

Requirement → Module → Validation Rule → Test Case

Each invariant SHALL map to:

- Specific code location.
- Specific validation function.
- Specific test assertion.

Traceability SHALL be preserved.

---

### 10.6 No Hidden State Guarantee

The Design SHALL ensure:

- No implicit global state.
- No static mutable variables.
- No cached external values.
- No state stored outside defined accounts.

All persistent state SHALL reside in defined accounts only.

---

### 10.7 Deterministic Interface Guarantee

The external interface SHALL:

- Accept fixed instruction enum.
- Accept fixed account structure.
- Reject malformed instruction data.
- Reject incorrect account ordering.

ABI SHALL remain stable for v1.0.

---

### 10.8 Version Isolation Statement

This Design applies exclusively to:

Issuance Contract v1.0

Any change to:

- Account layout
- Instruction enum
- Arithmetic logic
- Lifecycle rules
- Escrow model

SHALL require a new Design version.

---

### 10.9 Architectural Finality Statement

If implemented exactly as defined in Sections 1–10:

The architecture SHALL guarantee:

- Deterministic execution.
- Immutable behavior.
- Reserve-bounded reward.
- Strict escrow isolation.
- Formal invariant preservation.
- O(1) instruction complexity.
- Lifecycle finality.

Design SHALL be structurally sufficient for secure implementation.

---

### 10.10 Design Closure Declaration

All architectural components required to implement:

Lockrion Issuance Contract v1.0

are fully defined.

No additional structural elements are required.

Design scope is formally closed.
