# Issuance-Contract-Implementation-V1.0-EN
## Section 1 — Codebase Structure and Module Layout

### 1.1 Implementation Objective

This document defines the concrete implementation plan for:

Lockrion Issuance Contract v1.0

The implementation SHALL:

- Follow Issuance-Contract-Specification-V1.0-EN exactly.
- Follow Issuance-Contract-Design-V1.0-EN architecture exactly.
- Introduce no new behavior.
- Preserve all invariants structurally.
- Use Rust for Solana BPF target.

---

### 1.2 Programming Language and Toolchain

The contract SHALL be implemented using:

- Rust (stable toolchain)
- Solana Program SDK
- SPL Token Program CPI
- No unsafe Rust
- No floating-point types

Compiler configuration SHALL:

- Enable overflow checks.
- Disallow warnings.
- Disallow unsafe blocks.
- Use explicit feature gating.

---

### 1.3 Project File Structure

The program directory SHALL follow:

src/
  ├── lib.rs
  ├── entrypoint.rs
  ├── instruction.rs
  ├── state.rs
  ├── error.rs
  ├── processor.rs
  ├── validation.rs
  ├── accumulator.rs
  ├── escrow.rs
  ├── lifecycle.rs
  ├── invariants.rs
  └── utils.rs

No additional logic modules SHALL exist.

---

### 1.4 Module Responsibilities

lib.rs:
- Declare program entry.
- Export modules.

entrypoint.rs:
- Define Solana entrypoint.
- Route to processor.

instruction.rs:
- Define Instruction enum.
- Handle instruction deserialization.

state.rs:
- Define GlobalState struct.
- Define UserState struct.
- Implement serialization.

error.rs:
- Define explicit error codes.

processor.rs:
- Instruction dispatcher.
- Call appropriate handler.

validation.rs:
- PDA validation.
- Signer validation.
- Account ownership validation.
- Mint validation.

accumulator.rs:
- update_global_accumulator()
- update_user_accumulator()

escrow.rs:
- SPL transfer logic.
- Escrow authority derivation.

lifecycle.rs:
- Phase derivation logic.
- Phase validation functions.

invariants.rs:
- Conservation validation.
- Monotonicity validation.
- Mutual exclusivity validation.

utils.rs:
- Checked arithmetic helpers.
- Day index computation.

---

### 1.5 Instruction Processing Flow (Concrete)

For each instruction:

1. Deserialize instruction enum.
2. Parse accounts.
3. Validate account ownership.
4. Validate PDA derivation.
5. Validate signers.
6. Validate lifecycle phase.
7. Update accumulators (if required).
8. Execute core logic.
9. Execute escrow transfer (if required).
10. Validate invariants.
11. Write state to accounts.
12. Return success.

Failure at any step SHALL abort execution.

---

### 1.6 Data Types

Financial values:

- reserve_total : u128
- total_locked : u128
- total_weight_accum : u128
- locked_amount : u128
- user_weight_accum : u128

Day indices:

- last_day_index : u64
- user_last_day_index : u64
- final_day_index : u64

Timestamps:

- start_ts : i64
- maturity_ts : i64
- claim_window : i64

Flags:

- reserve_funded : bool
- sweep_executed : bool
- reclaim_executed : bool
- reward_claimed : bool

No other numeric types SHALL be used for financial values.

---

### 1.7 Arithmetic Rules

All arithmetic SHALL:

- Use checked_add()
- Use checked_sub()
- Use checked_mul()
- Use checked_div()
- Abort on overflow
- Multiply before divide for reward calculation

Floor division SHALL be default behavior.

---

### 1.8 PDA Derivation Scheme

All Program Derived Addresses (PDAs) SHALL be derived deterministically
from immutable issuance parameters exactly as defined in the Design and Specification.

Global State PDA:

Seeds:
- b"issuance"
- issuer_address
- lock_mint
- start_ts (little-endian bytes)

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

All PDA derivations SHALL be validated on every instruction.

No alternative seed ordering, hashing scheme, role identifier seeds,
or runtime-derived seed components SHALL be used in v1.0.

Any change to seed ordering or seed composition SHALL require
a new version identifier and redeployment.

---

### 1.9 Serialization Implementation

State structs SHALL:

- Derive BorshSerialize / BorshDeserialize
OR
- Use fixed byte layout with manual pack/unpack

Serialization SHALL:

- Be deterministic.
- Preserve field order.
- Avoid dynamic fields.

Account size SHALL be computed explicitly.

---

### 1.10 Error Handling Implementation

All error conditions SHALL:

- Return explicit custom error enum.
- Avoid panic.
- Avoid unwrap().
- Avoid expect().
- Avoid unreachable!().

Error codes SHALL map directly to Specification rejection conditions.

---

### 1.11 No Unsafe Code Rule

The implementation SHALL NOT:

- Use unsafe blocks.
- Use raw pointers.
- Use unchecked indexing.
- Use manual memory manipulation.

Memory safety SHALL rely on Rust guarantees.

---

### 1.12 Implementation Boundary Condition

The implementation SHALL NOT:

- Introduce logging-based side effects.
- Introduce hidden state.
- Introduce debug-only logic altering behavior.
- Introduce compile-time feature branching affecting logic.

The code SHALL reflect Design exactly.

---

## Section 2 — Concrete State Struct Definitions and Serialization

### 2.1 Serialization Strategy

All on-chain state accounts SHALL use a deterministic fixed-layout serialization.

The implementation SHALL use:

- Borsh serialization for GlobalState and UserState
- Explicit fixed field ordering
- Explicit account size constants
- No variable-length fields
- No optional fields

Serialization format SHALL remain stable for v1.0.

---

### 2.2 GlobalState Struct (Concrete)

The implementation SHALL define:

GlobalState {
  issuer_address: Pubkey,
  lock_mint: Pubkey,
  reward_mint: Pubkey,
  platform_treasury: Pubkey,

  reserve_total: u128,

  start_ts: i64,
  maturity_ts: i64,
  claim_window: i64,
  accounting_period: u64,

  final_day_index: u64,

  reserve_funded: bool,
  sweep_executed: bool,
  reclaim_executed: bool,

  total_locked: u128,
  total_weight_accum: u128,
  last_day_index: u64,
}

Field order SHALL NOT change.

---

### 2.3 UserState Struct (Concrete)

The implementation SHALL define:

UserState {
  owner: Pubkey,

  locked_amount: u128,
  user_weight_accum: u128,
  user_last_day_index: u64,

  reward_claimed: bool,
}

Field order SHALL NOT change.

---

### 2.4 Account Size Constants

The implementation SHALL define explicit constants:

GLOBAL_STATE_LEN = serialized_size(GlobalState)
USER_STATE_LEN   = serialized_size(UserState)

The build SHALL include static assertions that:

- GLOBAL_STATE_LEN is constant
- USER_STATE_LEN is constant

Any mismatch SHALL fail build.

---

### 2.5 Immutable Field Enforcement

The implementation SHALL enforce immutability by:

- Writing immutable fields only in initialize()
- Never assigning to immutable fields in any other handler
- Validating expected values on every instruction:
  - lock_mint
  - reward_mint
  - issuer_address
  - platform_treasury

If a provided account mismatches immutable parameter expectations, instruction MUST fail.

---

### 2.6 Initialization Defaults

initialize() SHALL set:

reserve_funded = false
sweep_executed = false
reclaim_executed = false
total_locked = 0
total_weight_accum = 0
last_day_index = 0

No other default values SHALL be used.

---

### 2.7 User Account Initialization Defaults

On first deposit, UserState SHALL be initialized as:

owner = signer_pubkey
locked_amount = 0
user_weight_accum = 0
user_last_day_index = current_day_index
reward_claimed = false

user_last_day_index initialization SHALL ensure no weight accrues for day of first deposit.

---

### 2.8 Rent and Account Allocation

Account allocation SHALL:

- Allocate exactly GLOBAL_STATE_LEN bytes for Global State.
- Allocate exactly USER_STATE_LEN bytes for User State.
- Reject accounts with insufficient data length.
- Reject accounts with unexpected owners.

No extra padding beyond deterministic serialization SHALL be relied upon.

---

### 2.9 Deserialization Safety

All deserialization SHALL:

- Validate account data length.
- Reject malformed serialized data.
- Return explicit errors, not panic.

No unchecked casts SHALL be used.

---

### 2.10 Version Binding

State structs SHALL include no version field for v1.0.

Version binding SHALL be by:

- Program ID
- Fixed struct layout
- Design document version

Any future layout change SHALL require new program deployment and new document version.

---

## Section 3 — PDA Seed Scheme and Account Derivation

### 3.1 Deterministic PDA Strategy

All PDAs SHALL be derived deterministically and validated on every instruction.

The implementation SHALL derive:

- Global State PDA
- Escrow Authority PDA
- Deposit Escrow Token Account PDA
- Reward Escrow Token Account PDA
- User State PDA (per participant)

No other PDAs SHALL exist.

---

### 3.2 PDA Seed Design Rules

All PDAs SHALL be derived deterministically
from immutable issuance parameters.

Seed components SHALL:

- Be static literals (e.g., b"issuance", b"user_state").
- Include immutable issuance parameters.
- Exclude mutable state.
- Exclude runtime-derived values.

Permitted seed components in v1.0:

- issuer_address
- lock_mint
- start_ts (little-endian bytes)
- global_state_pubkey
- user_pubkey

The inclusion of start_ts as a seed component
is explicitly required to guarantee deterministic
cross-issuance uniqueness.

The program SHALL NOT include:

- block_timestamp
- mutable accumulators
- terminal flags
- runtime-calculated values
- post-initialization fields

Seed ordering SHALL remain frozen.

Any change to seed structure SHALL require
a new contract version and redeployment.

---

### 3.3 Global State PDA

Global State PDA SHALL be derived from:

Seeds:
- b"issuance"
- issuer_address (Pubkey bytes)
- lock_mint (Pubkey bytes)
- start_ts (i64 LE bytes)

Purpose:

- Ensure uniqueness per issuance under issuer and lock_mint with specific start.

The program SHALL validate that the provided Global State account equals the derived PDA.

---

### 3.4 Escrow Authority PDA

Escrow Authority PDA SHALL be derived from:

Seeds:
- b"escrow_authority"
- global_state_pubkey

Purpose:

- Provide a single signing authority for both escrow token accounts.

Escrow Authority PDA SHALL never be stored; it SHALL be derived on demand.

---

### 3.5 Ownership and Authority Constraints

Program-owned accounts:

- Global State account.owner MUST equal program_id.
- User State account.owner MUST equal program_id.

SPL token accounts (including escrow and user token accounts):

- token_account.account.owner MUST equal SPL Token Program.

Escrow token accounts (Deposit Escrow and Reward Escrow):

For each escrow token account:

1. token_account.account.owner MUST equal SPL Token Program.
2. token_account.mint MUST match expected mint:
   - lock_mint for Deposit Escrow
   - reward_mint for Reward Escrow
3. token_account.authority MUST equal Escrow Authority PDA.
4. Escrow Authority PDA MUST match deterministic derivation.

Escrow Authority PDA:

- SHALL be program-derived.
- SHALL sign escrow transfers only via invoke_signed.
- SHALL NOT be externally signable.

The program SHALL NOT treat token_account.account.owner as the escrow authority.

Any mismatch in:

- program account owner
- token account owner
- token mint
- token authority
- PDA derivation

MUST cause instruction rejection before any state mutation or transfer.

---

### 3.6 Escrow Authority and Token Account Semantics

Escrow custody SHALL follow the SPL Token security model.

For each escrow SPL token account:

- account.owner MUST equal SPL Token Program.
- authority MUST equal Escrow Authority PDA.
- mint MUST equal the expected mint.

Escrow Authority PDA semantics:

- The Escrow Authority PDA SHALL be derived deterministically from the issuance anchor.
- It SHALL NOT be an externally controlled keypair.
- It SHALL sign escrow transfers only through invoke_signed.
- It SHALL NOT be used as account.owner for SPL token accounts.

Escrow movement constraints:

- No outgoing transfer from escrow SHALL be possible unless:
  1. the escrow token account address matches the derived PDA,
  2. the escrow token account owner is the SPL Token Program,
  3. the escrow token account authority equals Escrow Authority PDA,
  4. the invoked transfer is performed with the PDA signature via invoke_signed.

Any deviation from these constraints MUST cause instruction rejection.

---

### 3.7 User State PDA

User State PDA SHALL be derived from:

Seeds:
- b"user_state"
- global_state_pubkey
- user_pubkey

Purpose:

- Ensure a single canonical user state per issuance per participant.

The program SHALL reject any mismatching user state account.

---

### 3.8 PDA Validation Function Set

The implementation SHALL provide:

- derive_global_state_pda()
- derive_escrow_authority_pda()
- derive_deposit_escrow_pda()
- derive_reward_escrow_pda()
- derive_user_state_pda()

Each function SHALL return:

- Derived Pubkey
- Bump seed

All handlers SHALL call these functions before mutation.

---

### 3.9 Cross-Issuance Spoof Protection

The program SHALL reject:

- Any escrow account not derived from provided Global State PDA.
- Any user state not derived from provided Global State PDA.
- Any token account with mismatched mint.

Seed binding ensures cross-issuance isolation.

---

### 3.10 PDA Seed Freeze Requirement

All seed literals SHALL be constants.

Changing any seed literal SHALL be treated as a breaking change requiring:

- New program deployment
- New Specification version
- New Design version
- New Implementation version

v1.0 seeds SHALL be frozen.

---

## Section 4 — Instruction Handlers: Concrete Implementation Logic

### 4.1 Initialize Handler — Concrete Steps

Function: process_initialize()

Steps:

1. Validate Global State PDA matches derived address.
2. Validate account is uninitialized.
3. Validate immutable deployment parameters:
   - start_ts aligned to 00:00:00 UTC
   - maturity_ts > start_ts
   - (maturity_ts - start_ts) % 86400 == 0
   - accounting_period == 86400
   - reserve_total > 0
   - reward_mint == USDC
4. Compute final_day_index:
   final_day_index = (maturity_ts - start_ts) / 86400
5. Initialize GlobalState fields.
6. Set:
   - reserve_funded = false
   - sweep_executed = false
   - reclaim_executed = false
   - total_locked = 0
   - total_weight_accum = 0
   - last_day_index = 0
7. Create Deposit Escrow token account.
8. Create Reward Escrow token account.
9. Write GlobalState to account.

Immutability Guarantees:

- final_day_index SHALL be written exactly once during initialize().
- final_day_index SHALL NEVER be recomputed.
- No instruction other than initialize() SHALL modify:
  - final_day_index
  - start_ts
  - maturity_ts
  - claim_window
  - accounting_period
  - reserve_total
  - lock_mint
  - reward_mint
  - issuer_address
  - platform_treasury

Any attempt to modify immutable fields SHALL abort execution.

No reserve funding SHALL occur in initialize().

---

### 4.2 FundReserve Handler — Concrete Steps

Function: process_fund_reserve()

Steps:

1. Validate signer == issuer_address.
2. Validate reserve_funded == false.
3. Validate block_timestamp < start_ts.
4. Validate reward escrow balance == 0.
5. Validate transfer amount == reserve_total.
6. Execute SPL transfer to reward escrow.
7. Set reserve_funded = true.
8. Write state.

No other fields SHALL be modified.

---

### 4.3 Deposit Handler — Concrete Steps

Function: process_deposit()

Steps:

1. Validate lifecycle phase (Participation Phase).
2. Validate reserve_funded == true.
3. Validate lock_mint matches Global State.
4. Validate user signer.
5. Validate deposit amount > 0.

Accumulator Phase:

6. update_global_accumulator(global_state, block_timestamp)
7. update_user_accumulator(user_state, global_state)

In-Memory State Mutation (no commit yet):

8. new_locked_amount = user_state.locked_amount + deposit_amount
9. new_total_locked = global_state.total_locked + deposit_amount
10. Validate no overflow.
11. Temporarily assign in-memory:
    - user_state.locked_amount = new_locked_amount
    - global_state.total_locked = new_total_locked

Escrow Transfer:

12. Execute SPL transfer:
    from user token account
    to deposit escrow PDA
13. Abort on transfer failure.

Invariant Verification:

14. Validate deposit conservation:
    global_state.total_locked == deposit_escrow_balance
15. Validate monotonicity invariants.
16. Validate immutability constraints.

State Commit:

17. Write updated Global State.
18. Write updated User State.

Deposit SHALL NOT:

- Modify immutable parameters.
- Affect reward escrow.
- Bypass accumulator update.

No state SHALL be committed before invariant validation succeeds.

---

### 4.4 ClaimReward Handler — Concrete Steps

Function: process_claim_reward()

Steps:

1. Validate maturity_ts ≤ block_timestamp < maturity_ts + claim_window.
2. Validate total_weight_accum > 0.
3. Validate reward_claimed == false.
4. Call update_global_accumulator().
5. Call update_user_accumulator().
6. Compute reward:

   reward = (reserve_total * user_weight_accum) / total_weight_accum

7. Validate reward escrow balance ≥ reward.
8. Execute SPL transfer to user reward account.
9. Set reward_claimed = true.
10. Write updated UserState.

GlobalState SHALL NOT modify total_weight_accum here.

---

### 4.5 withdraw_deposit() — Handler Logic

withdraw_deposit() SHALL enforce post-maturity accumulator finalization
before any deposit state mutation.

Preconditions:

- block_timestamp ≥ maturity_ts
- locked_amount > 0
- Valid user signer
- Correct Deposit Escrow PDA and lock_mint

Execution order:

1. Compute current_day_index (clamped to final_day_index).
2. update_global_accumulator(global_state, block_timestamp) to finalize at maturity.
3. update_user_accumulator(user_state, global_state) to finalize at maturity.
4. Execute SPL transfer of locked_amount:
     from Deposit Escrow PDA
     to user lock_mint token account
5. Decrease total_locked by locked_amount.
6. Set locked_amount = 0.

Constraints:

- No reward computation SHALL occur.
- No reward escrow transfer SHALL occur.
- Accumulators SHALL NOT increase beyond final_day_index.
- Any transfer failure SHALL revert with no state mutation persisted.

Postconditions:

- Deposit escrow balance decreases by withdrawn amount.
- total_locked decreases accordingly.
- User locked_amount becomes zero.
- Accounting state remains consistent with maturity freeze guarantees.

---

### 4.6 Sweep Handler — Concrete Steps

Function: process_sweep()

Steps:

1. Validate block_timestamp ≥ maturity_ts + claim_window.
2. Validate total_weight_accum > 0.
3. Validate sweep_executed == false.
4. Validate reclaim_executed == false.
5. Read full reward escrow balance.
6. Validate reward escrow balance > 0.
7. Execute SPL transfer of full reward escrow balance to platform_treasury.
8. Re-read reward escrow balance.
9. Validate reward escrow balance == 0.
10. Set sweep_executed = true.
11. Write GlobalState.

Reward escrow balance SHALL become zero.

---

### 4.7 Zero Participation Reclaim — Concrete Steps

Function: process_zero_participation_reclaim()

Steps:

1. Validate block_timestamp ≥ maturity_ts.
2. Validate total_weight_accum == 0.
3. Validate reclaim_executed == false.
4. Validate sweep_executed == false.
5. Validate issuer signer.
6. Read full reward escrow balance.
7. Validate reward escrow balance > 0.
8. Execute SPL transfer of full reward escrow balance to issuer reward token account.
9. Re-read reward escrow balance.
10. Validate reward escrow balance == 0.
11. Set reclaim_executed = true.
12. Write GlobalState.

Reward escrow balance SHALL become zero.

---

### 4.8 Handler Atomicity Enforcement

Each handler SHALL:

- Perform all validation before state mutation.
- Execute escrow transfer before final state write.
- Write state only after successful transfer.
- Abort on any error before commit.

No partial updates SHALL occur.

---

### 4.9 Shared Utility Constraints

All handlers SHALL:

- Use derive_*_pda() for account validation.
- Use checked arithmetic utilities.
- Use lifecycle validation helpers.
- Use invariant verification before commit.

Code duplication SHALL be minimized via shared modules.

---

### 4.10 No Cross-Handler Side Effects

Handlers SHALL NOT:

- Mutate unrelated fields.
- Modify immutable parameters.
- Call other handlers recursively.
- Access external state.
- Store transient data in static variables.

Each handler SHALL be isolated and deterministic.

---

## Section 5 — Arithmetic Utilities and Reward Computation Engine

### 5.1 Arithmetic Safety Model

All arithmetic operations SHALL use checked variants.

The implementation SHALL wrap:

- checked_add_u128(a, b)
- checked_sub_u128(a, b)
- checked_mul_u128(a, b)
- checked_div_u128(a, b)

Each function SHALL:

- Return explicit error on overflow.
- Return explicit error on division by zero.
- Never panic.
- Never wrap silently.

---

### 5.2 Day Index Computation

Utility function:

compute_current_day_index(global_state, block_timestamp)

Steps:

1. If block_timestamp < start_ts:
     return 0
2. raw = (block_timestamp - start_ts) / 86400
3. If raw > final_day_index:
     return final_day_index
4. Else:
     return raw

All operations SHALL use checked arithmetic.

---

### 5.3 Global Accumulator Utility

Function:

update_global_accumulator(global_state, block_timestamp)

Implementation:

1. current_day = compute_current_day_index(...)
2. If current_day > last_day_index:
     days_elapsed = current_day - last_day_index
     increment = total_locked * days_elapsed
     total_weight_accum += increment
     last_day_index = current_day

All operations SHALL use checked arithmetic.

---

### 5.4 User Accumulator Utility

Function:

update_user_accumulator(user_state, global_state)

Implementation:

1. current_day = global_state.last_day_index
2. If current_day > user_last_day_index:
     days_elapsed = current_day - user_last_day_index
     increment = locked_amount * days_elapsed
     user_weight_accum += increment
     user_last_day_index = current_day

All arithmetic SHALL be checked.

---

### 5.5 Reward Computation Function

Function:

compute_reward(global_state, user_state)

Preconditions:

- total_weight_accum > 0

Steps:

1. numerator = reserve_total * user_weight_accum
2. reward = numerator / total_weight_accum
3. Return reward

Multiplication SHALL occur before division.

Floor division SHALL be implicit via integer division.

---

### 5.6 Overflow Bound Reasoning

Given:

- reserve_total ≤ u128 max
- user_weight_accum ≤ u128 max
- total_weight_accum ≤ u128 max

Multiplication reserve_total * user_weight_accum MAY overflow.

Therefore:

Implementation SHALL use:

checked_mul(reserve_total, user_weight_accum)

Overflow SHALL abort execution.

---

### 5.7 Zero Participation Guard

compute_reward() SHALL NOT be called if:

total_weight_accum == 0

Validation layer SHALL enforce this before calling reward computation.

Division by zero SHALL be structurally impossible.

---

### 5.8 Deterministic Division Model

Integer division SHALL:

- Use default Rust integer division.
- Produce floor(result).
- Produce identical result across all nodes.

No rounding modes SHALL be configurable.

---

### 5.9 No Floating-Point Guarantee

The implementation SHALL NOT:

- Use f32
- Use f64
- Cast integers to floats
- Use decimal arithmetic libraries

All reward logic SHALL use integer arithmetic only.

---

### 5.10 Reward Conservation — Structural Enforcement

Reward conservation SHALL be enforced structurally.

The contract SHALL guarantee:

1. reward_escrow_balance ≤ reserve_total
2. reward_escrow_balance ≥ 0

No reward SHALL be created or destroyed.

The only permitted outgoing reward transfers SHALL be:

- claim_reward()
- sweep()
- zero_participation_reclaim()

The program SHALL NOT:

- Mint reward tokens.
- Burn reward tokens.
- Store cumulative distribution totals.
- Permit alternative reward transfer paths.

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
- Restricting reward movement to the defined instruction set.
- Terminal flags preventing further reward movement.

---

### 5.11 Monotonicity Verification Helper

Utility:

validate_monotonicity(global_state, user_state)

Ensure:

- total_weight_accum ≥ previous value
- user_weight_accum ≥ previous value
- last_day_index ≤ final_day_index
- user_last_day_index ≤ final_day_index

Violation SHALL abort execution.

---

### 5.12 Deterministic Arithmetic Guarantee

Given identical:

- GlobalState
- UserState
- block_timestamp

Arithmetic utilities SHALL produce identical results across:

- All validator nodes
- All replay executions
- All deterministic simulations

Arithmetic SHALL remain fully deterministic.

---

## Section 6 — Validation, Error Codes, and Invariant Enforcement

### 6.1 Error Enum Strategy

The implementation SHALL define a single custom error enum:

IssuanceError

Each error variant SHALL map to exactly one rejection class in the Specification.

Errors SHALL be returned via:

- ProgramError::Custom(code)

No panic-based failure SHALL be used.

---

### 6.2 Mandatory Error Categories

The error enum SHALL include categories for:

- InvalidInstructionData
- InvalidAccountOwner
- InvalidPdaAddress
- InvalidTokenProgram
- InvalidMint
- MissingRequiredSigner
- Unauthorized
- InvalidPhase
- NotInitialized
- AlreadyInitialized
- ReserveNotFunded
- ReserveAlreadyFunded
- FundingTooLate
- InvalidFundingAmount
- DepositTooEarly
- DepositTooLate
- InvalidDepositAmount
- ClaimTooEarly
- ClaimTooLate
- AlreadyClaimed
- NoParticipation
- InsufficientRewardEscrow
- WithdrawTooEarly
- NothingToWithdraw
- SweepTooEarly
- SweepAlreadyExecuted
- ReclaimAlreadyExecuted
- ReclaimNotAllowed
- SweepNotAllowed
- Overflow
- DivisionByZero
- InvariantViolation

Exact numeric codes SHALL be frozen for v1.0.

---

### 6.3 Account Owner Validation

validation.rs SHALL implement:

validate_account_owner(account, expected_owner)

Used for:

- Global State owner == program_id
- User State owner == program_id
- Token accounts owner == SPL Token Program (account.owner field)
- Token program id == SPL Token Program

Any mismatch SHALL return InvalidAccountOwner or InvalidTokenProgram.

---

### 6.4 PDA Validation

validation.rs SHALL implement:

validate_pda(provided_pubkey, derived_pubkey)

Applied to:

- Global State PDA
- Escrow Authority PDA (derived, used for signing)
- Deposit Escrow PDA
- Reward Escrow PDA
- User State PDA

Mismatch SHALL return InvalidPdaAddress.

---

### 6.5 Signer Validation

validation.rs SHALL implement:

- require_signer(account_info)
- require_issuer_signer(issuer_account, global_state.issuer_address)

Rules:

- fund_reserve() requires issuer signer
- zero_participation_reclaim() requires issuer signer
- deposit/claim/withdraw require user signer

Missing signer SHALL return MissingRequiredSigner.
Wrong signer SHALL return Unauthorized.

---

### 6.6 Escrow Token Account Validation

Before any escrow-related operation, the program SHALL validate
SPL token account semantics strictly according to the SPL Token model.

For escrow token accounts (Deposit Escrow and Reward Escrow):

1. token_account.account.owner MUST equal SPL Token Program.
2. token_account.mint MUST equal expected mint:
   - lock_mint for Deposit Escrow
   - reward_mint for Reward Escrow
3. token_account.authority MUST equal Escrow Authority PDA.
4. Escrow Authority PDA MUST match deterministic derivation.
5. Provided token program account MUST equal SPL Token Program.

The program SHALL NOT:

- Treat token_account.account.owner as escrow authority.
- Assume program ownership of SPL token accounts.
- Accept externally controlled authority for escrow accounts.
- Permit escrow substitution via spoofed accounts.

All escrow authority validation SHALL occur before:

- Any state mutation,
- Any accumulator update,
- Any SPL token transfer.

If any validation fails:

- Instruction SHALL abort.
- No state SHALL be written.
- No token SHALL move.

---

### 6.7 Phase Validation Helpers

lifecycle.rs SHALL implement:

- is_participation_phase(global_state, now_ts) -> bool
- is_claim_phase(global_state, now_ts) -> bool
- is_post_claim_phase(global_state, now_ts) -> bool
- is_matured(global_state, now_ts) -> bool

Each handler SHALL call the appropriate predicate before proceeding.

Invalid phase SHALL return InvalidPhase.

---

### 6.8 Invariant Layer Interface

The invariant layer SHALL enforce only structural invariants.

No aggregate reward distribution totals SHALL exist in state,
and no invariant function SHALL accept, compute, or depend on
any cumulative distribution counters.

The invariant layer SHALL expose the following checks:

1. validate_immutables(global_state, provided_accounts)
   - Confirms lock_mint, reward_mint, issuer_address, platform_treasury.

2. validate_deposit_conservation(global_state, deposit_escrow_balance)
   - Requires:
     global_state.total_locked == deposit_escrow_balance

3. validate_reward_bounds(global_state, reward_escrow_balance)
   - Requires:
     0 ≤ reward_escrow_balance ≤ global_state.reserve_total

4. validate_monotonicity(global_state, user_state)
   - last_day_index monotonic
   - user_last_day_index monotonic
   - total_weight_accum monotonic
   - user_weight_accum monotonic

5. validate_terminal_exclusivity(global_state)
   - sweep_executed and reclaim_executed SHALL NOT both be true.
   - If total_weight_accum == 0 then sweep_executed MUST remain false.
   - If total_weight_accum > 0 then reclaim_executed MUST remain false.

6. validate_terminal_reward_zero(global_state, reward_escrow_balance)
   - If sweep_executed == true OR reclaim_executed == true:
     reward_escrow_balance MUST equal 0.

Invariant checks SHALL be executed after any escrow transfer
and before final state commit.

The invariant layer SHALL NOT:

- Track claimed totals,
- Track swept totals,
- Track reclaimed totals,
- Store or accept any derived distributed_reward_total.

All conservation guarantees SHALL remain escrow-structural.

---

### 6.9 Immutable Field Guard

validation.rs SHALL implement:

validate_immutable_fields(global_state, expected_accounts)

Handlers SHALL re-check that:

- lock_mint matches provided lock_mint accounts
- reward_mint matches provided reward mint accounts
- platform_treasury matches provided treasury account
- issuer_address matches expected issuer account

Mismatch SHALL return Unauthorized or InvalidMint as appropriate.

---

### 6.10 Terminal Flag Validation

Before terminal actions:

- sweep() MUST ensure sweep_executed == false AND reclaim_executed == false
- reclaim() MUST ensure reclaim_executed == false AND sweep_executed == false

On violation:

- SweepAlreadyExecuted / ReclaimAlreadyExecuted
- SweepNotAllowed / ReclaimNotAllowed

---

### 6.11 Overflow and Division Safety Integration

All checked arithmetic helpers SHALL return:

- Overflow on checked_* failure
- DivisionByZero on denominator == 0 (defensive)

Handlers SHALL propagate these errors unchanged.

No arithmetic error SHALL be downgraded or ignored.

---

### 6.12 Validation and Execution Order

All instruction handlers SHALL follow the same deterministic execution pipeline.

The enforced order is:

1. Account ownership validation
2. PDA derivation validation
3. Signer / role validation
4. Lifecycle phase validation
5. Arithmetic precondition checks (including division-by-zero guards)
6. Accumulator update (if required by the instruction)
7. Core instruction logic
8. Escrow transfer (if required by the instruction)
9. Invariant verification
10. State commit

No state mutation SHALL occur before step 6.

Notes:

- Accumulator update MUST occur before any mutation of locked_amount or total_locked.
- Reward computation MUST occur only after accumulator finalization.
- Escrow transfers MUST be atomic and MUST precede any flag finalization
  (e.g., sweep_executed, reclaim_executed, reward_claimed).

Any deviation from this ordering SHALL be considered a conformance violation.

---

### 6.13 Deterministic Error Guarantee

Given identical:

- on-chain state
- instruction data
- account set
- block_timestamp

Validation SHALL:

- Fail with the same error code
- At the same stage
- Without partial state mutation

Error behavior SHALL be deterministic and reproducible.

---

## Section 7 — Integration with Solana Runtime and SPL Token CPI

### 7.1 Solana Program Entry Integration

entrypoint.rs SHALL implement:

- The canonical Solana program entrypoint.
- Instruction data forwarding to processor::process().

Entrypoint SHALL:

- Not contain business logic.
- Not mutate state.
- Not perform arithmetic.
- Not perform CPI.

All logic SHALL reside in processor module.

---

### 7.2 Account Parsing Model

processor.rs SHALL:

- Expect accounts in strict positional order per instruction.
- Validate account count.
- Reject extra or missing accounts.

Account order SHALL be frozen for v1.0.

---

### 7.3 CPI to SPL Token Program

escrow.rs SHALL implement CPI calls to:

- spl_token::instruction::transfer

All CPI calls SHALL:

- Use invoke_signed when Escrow Authority PDA signs.
- Use correct signer seeds and bump.
- Validate token program account key.

No unchecked CPI SHALL be used.

---

### 7.4 Escrow Authority Signing

For transfers from escrow accounts:

- Escrow Authority PDA SHALL sign via invoke_signed.
- Seeds used SHALL match derive_escrow_authority_pda().
- Bump SHALL be included in signer seeds.

If signer seeds mismatch, transfer SHALL fail.

---

### 7.5 Token Account Validation Before CPI

Before any transfer CPI:

The implementation SHALL verify:

- Source account mint matches expected mint.
- Destination account mint matches expected mint.
- Source account owner matches expected authority.
- Token account is initialized.

Failure SHALL abort before CPI.

---

### 7.6 CPI Failure Handling

If SPL Token CPI returns error:

- Entire instruction SHALL revert.
- No state mutation SHALL persist.
- No flags SHALL be set.

State writes SHALL occur only after successful CPI.

---

### 7.7 Rent Exemption Requirements

Global State and User State accounts SHALL:

- Be rent-exempt at creation.
- Reject underfunded accounts.
- Reject accounts with insufficient lamports.

Implementation SHALL check rent exemption at initialize() and user account creation.

---

### 7.8 Compute Budget Considerations

The implementation SHALL:

- Avoid dynamic loops.
- Avoid large stack allocations.
- Avoid deep recursion.
- Avoid expensive serialization operations.

Each instruction SHALL fit within standard compute limits.

---

### 7.9 Account Data Mutability Rules

processor.rs SHALL:

- Borrow account data mutably only when necessary.
- Avoid overlapping mutable borrows.
- Avoid double-borrow patterns.
- Use scoped borrowing for safety.

Borrow rules SHALL follow Rust ownership model strictly.

---

### 7.10 No Cross-Program Invocation Beyond SPL

The contract SHALL NOT:

- Invoke arbitrary external programs.
- Invoke governance programs.
- Invoke registry programs.
- Invoke oracles.
- Invoke system program for logic beyond account creation.

Only permitted CPIs:

- SPL Token transfers
- System Program for account creation during initialize()

---

### 7.11 Replay Determinism with CPI

Given identical:

- Escrow balances
- Instruction input
- Account states

CPI transfers SHALL produce identical:

- Balance deltas
- Error results

No nondeterministic behavior SHALL arise from CPI.

---

### 7.12 Integration Failure Containment

If integration with SPL Token fails:

- Instruction SHALL revert.
- No internal state SHALL commit.
- No partial flag update SHALL occur.

Integration SHALL be atomic and deterministic.

---

### 7.13 Runtime Safety Guarantees

The implementation SHALL rely on:

- Solana runtime atomic transaction model
- Account locking per transaction
- Deterministic execution environment
- No shared mutable state across transactions

No concurrency control SHALL be implemented manually.

---

### 7.14 Final Integration Guarantee

If:

- All PDA validations succeed,
- All SPL CPIs succeed,
- All invariants hold,

Then integration SHALL guarantee:

- Escrow safety,
- Deterministic token movement,
- Immutable state transitions,
- Full reproducibility under chain replay.

---

## Section 8 — Testability Hooks and Deterministic Verification Strategy

### 8.1 Test-Oriented Architecture Requirement

The implementation SHALL be structured such that:

- Core logic can be tested independently of Solana runtime.
- Accumulator logic can be unit tested without CPI.
- Reward computation can be tested as pure function.
- Lifecycle predicates can be tested deterministically.
- Validation helpers can be tested in isolation.

Business logic SHALL be decoupled from account I/O where possible.

---

### 8.2 Pure Logic Extraction

The following components SHALL be pure functions:

- compute_current_day_index()
- update_global_accumulator()
- update_user_accumulator()
- compute_reward()
- lifecycle phase predicates
- arithmetic utilities
- invariant validation helpers

These functions SHALL:

- Accept only explicit parameters.
- Return explicit results.
- Avoid account I/O.
- Avoid CPI.

---

### 8.3 Deterministic Scenario Simulation

The implementation SHALL allow simulation of:

- Multiple deposits across days.
- Maturity transition.
- Claim window transition.
- Zero participation scenario.
- Sweep scenario.
- Reclaim scenario.

Simulations SHALL reproduce exact on-chain behavior.

---

### 8.4 Edge Case Coverage Requirements

Tests SHALL cover:

- Deposit on first participation day.
- Deposit on final participation day.
- Claim on first claim day.
- Claim on last claim day.
- Withdrawal before claim.
- Withdrawal after claim.
- Zero participation full lifecycle.
- Maximum u128 boundary stress tests.
- Overflow rejection cases.

Edge conditions SHALL be explicitly tested.

---

### 8.5 Invariant Verification in Tests

Test suite SHALL validate:

- Deposit conservation.
- Reward conservation.
- Monotonicity.
- Mutual exclusivity.
- Terminal state finality.
- No double claim.
- No double withdraw.

Each invariant SHALL have at least one explicit test case.

---

### 8.6 Deterministic Replay Test

The implementation SHALL support:

- Deterministic replay of instruction sequence.
- Verification that final state equals expected snapshot.
- Replaying identical sequence produces identical results.

Replay determinism SHALL be validated.

---

### 8.7 Negative Path Testing

Tests SHALL confirm rejection for:

- Deposit before reserve funding.
- Partial reserve funding.
- Deposit after maturity.
- Claim before maturity.
- Claim after claim window.
- Sweep before claim window.
- Reclaim when participation exists.
- Arithmetic overflow attempts.
- Incorrect PDA usage.
- Incorrect signer.

All failure paths SHALL be explicitly validated.

---

### 8.8 No Hidden State Validation

Tests SHALL verify:

- No state mutation when instruction fails.
- No partial escrow transfer on failure.
- No flag mutation before CPI success.
- No mutation of immutable fields.

Failure atomicity SHALL be tested.

---

### 8.9 Boundary Condition Testing

Tests SHALL validate:

- reserve_total = 1
- reserve_total near u128 max (safe bounds)
- locked_amount near u128 max (safe bounds)
- Large participant count (logical isolation)
- Claim of smallest possible reward
- Rounding remainder behavior

Boundary cases SHALL not break invariants.

---

### 8.10 Deterministic Arithmetic Validation

Test suite SHALL confirm:

reward = floor(reserve_total × user_weight_accum / total_weight_accum)

Rounding behavior SHALL be:

- Deterministic
- Floor-based
- Independent of execution order

No floating-point rounding SHALL occur.

---

### 8.11 Terminal Path Coverage

Tests SHALL verify:

- Sweep path correct when participation exists.
- Reclaim path correct when no participation exists.
- Mutual exclusivity enforced.
- No reward possible after terminal state.
- Withdrawal possible after terminal state.

Terminal closure SHALL be complete.

---

### 8.12 Implementation Readiness for Auto-Test Suite

This structure SHALL allow derivation of:

- Unit tests
- Integration tests
- Deterministic replay tests
- Property-based invariant tests
- Failure-path coverage tests

No additional refactoring SHALL be required to support full test coverage.

---

## Section 9 — Static Analysis, Safety Guarantees, and Build Constraints

### 9.1 Compiler Configuration

The implementation SHALL:

- Compile with overflow checks enabled.
- Deny warnings (#![deny(warnings)]).
- Forbid unsafe code (#![forbid(unsafe_code)]).
- Avoid unused variables and dead code.
- Avoid unreachable patterns.

Build SHALL fail on violation.

---

### 9.2 Unsafe Code Prohibition

The codebase SHALL NOT contain:

- unsafe blocks
- raw pointers
- transmute
- unchecked indexing
- manual memory layout manipulation

Memory safety SHALL rely exclusively on Rust guarantees.

---

### 9.3 No Panics Policy

The implementation SHALL NOT:

- Use unwrap()
- Use expect()
- Use panic!()
- Use unreachable!()
- Use assert!() in production logic

All errors SHALL propagate via Result.

---

### 9.4 Overflow Safety Verification

All arithmetic SHALL:

- Use checked_* operations.
- Propagate Overflow error explicitly.
- Never rely on implicit wrapping.

Static analysis SHALL confirm no unchecked arithmetic remains.

---

### 9.5 Division Safety Guarantee

Division SHALL:

- Be performed only when denominator > 0.
- Be guarded by validation.
- Never allow implicit division by zero.

DivisionByZero error SHALL be unreachable under valid flow.

---

### 9.6 Account Data Bounds Checking

The implementation SHALL:

- Validate account.data.len() before deserialization.
- Reject undersized accounts.
- Reject oversized unexpected accounts.
- Avoid unchecked slice indexing.

Deserialization SHALL be safe and bounded.

---

### 9.7 No Dynamic Memory Allocation

The contract SHALL:

- Avoid heap allocations.
- Avoid dynamic Vec resizing for state.
- Avoid variable-length serialization.
- Use stack-local variables only.

Memory footprint SHALL be deterministic.

---

### 9.8 Dead Code Elimination

The codebase SHALL:

- Contain no unreachable branches.
- Contain no unused functions.
- Contain no unused enum variants.

All logic SHALL be reachable via defined instruction flow.

---

### 9.9 Lint and Clippy Enforcement

The project SHALL:

- Enable clippy lints.
- Deny common pitfalls (integer overflow, needless clones, etc.).
- Avoid unnecessary allocations.
- Avoid redundant pattern matching.

Static linting SHALL be part of build process.

---

### 9.10 Formal Invariant Coverage

Static analysis SHALL confirm:

- No path bypasses invariant verification.
- No path mutates immutable fields post-initialize.
- No handler returns success without full validation.
- No escrow transfer occurs before validation.

Control flow SHALL be auditable.

---

### 9.11 Deterministic Compilation Guarantee

The build SHALL:

- Produce identical bytecode given identical source.
- Avoid build-time randomness.
- Avoid conditional compilation that changes logic.

Feature flags SHALL NOT alter behavior.

---

### 9.12 No Hidden Debug Behavior

The implementation SHALL NOT:

- Contain debug-only branches affecting logic.
- Alter arithmetic under debug mode.
- Change validation behavior under feature flags.

Debug logging MAY exist but SHALL NOT affect logic.

---

### 9.13 Stack Depth Safety

The implementation SHALL:

- Avoid deep call stacks.
- Avoid recursion.
- Keep function call depth bounded and predictable.

Stack usage SHALL remain within Solana BPF constraints.

---

### 9.14 Final Static Safety Guarantee

If Sections 9.1–9.13 are satisfied:

The implementation SHALL guarantee:

- Memory safety
- Arithmetic safety
- Deterministic behavior
- No UB
- No hidden mutation paths
- No runtime panics
- Strict invariant preservation

Static safety SHALL be structurally enforced by code design.

---

## Section 10 — Implementation Closure and Conformance Certification

### 10.1 Implementation Completeness Criteria

The implementation SHALL be considered complete when:

- All instructions defined in Specification are implemented.
- All handlers follow Design ordering.
- All validation layers are implemented.
- All invariant checks are integrated.
- All arithmetic utilities are used exclusively.
- All PDA derivations are centralized and validated.
- No unsafe code exists.
- No unchecked arithmetic exists.

No missing logic SHALL remain.

---

### 10.2 Specification Conformance Checklist

The implementation SHALL demonstrably satisfy:

- Immutable deployment parameters
- Reserve funding before participation
- Discrete daily accumulation
- Strict proportional reward formula
- Deposit conservation
- Reward conservation
- Mutual exclusivity of terminal paths
- O(1) instruction complexity
- No dynamic iteration over users
- No governance override

Each item SHALL be verifiable in code.

---

### 10.3 Design Conformance Checklist

The implementation SHALL conform to Design by:

- Using defined module structure
- Preserving handler flow order
- Maintaining isolated modules
- Preserving account layouts exactly
- Preserving PDA seed scheme exactly
- Preserving lifecycle derivation logic

No architectural deviation SHALL exist.

---

### 10.4 Deterministic Execution Certification

The implementation SHALL guarantee:

- Identical execution given identical state
- No reliance on external state
- No floating-point arithmetic
- No randomness
- No nondeterministic branching

Determinism SHALL be provable via replay.

---

### 10.5 Security Certification Conditions

Security SHALL be certified when:

- Upgrade authority revoked
- Escrow authority exclusively PDA-based
- No external escrow access path
- All signer requirements enforced
- No unauthorized state mutation path
- No cross-issuance dependency

Security SHALL be structural, not procedural.

---

### 10.6 Lifecycle Closure Verification

The implementation SHALL demonstrate:

- Reserve funding cannot occur after start_ts
- Deposits cannot occur after maturity
- Claims cannot occur outside claim window
- Exactly one terminal reward path reachable
- Terminal state irreversible
- Withdrawal always available post-maturity

Lifecycle SHALL be finite and closed.

---

### 10.7 Invariant Preservation Guarantee

All reachable states SHALL satisfy:

- Deposit conservation invariant
- Reward conservation invariant
- Monotonicity invariant
- Immutability invariant
- Mutual exclusivity invariant

No valid execution SHALL violate invariants.

---

### 10.8 Integration Integrity Confirmation

Integration with Solana runtime and SPL Token SHALL ensure:

- CPI atomicity
- Escrow signing via PDA only
- No partial state commit on failure
- No race condition within instruction

Integration SHALL preserve contract invariants.

---

### 10.9 No Hidden Behavior Certification

The codebase SHALL NOT contain:

- Hidden branches
- Debug-only economic logic
- Feature-flag-dependent logic
- Experimental paths
- Undocumented instructions

Behavior SHALL match Specification exactly.

---

### 10.10 Implementation Finality Statement

If Sections 1–10 of this Implementation Document are satisfied:

Lockrion Issuance Contract v1.0 SHALL be:

- Fully specified
- Architecturally sound
- Deterministic
- Immutable post-deployment
- Reserve-bounded
- Escrow-secure
- Mathematically correct
- Fully auditable

Implementation scope is formally closed.