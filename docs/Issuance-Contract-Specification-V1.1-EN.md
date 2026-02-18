# Lockrion Issuance Contract — Specification v1.1 (Clean)
Status: Draft  
Standard: Lockrion Issuance Contract v1  
Network Target: Solana  
Profile: Lockrion v1 (Fixed USDC reserve, fully funded pre-start, no reserve increases)

---

## 1. Purpose, Scope, and Conformance

### 1.1 Purpose

This Specification defines the normative on-chain behavior of a single Lockrion Issuance Contract instance.

An issuance contract is an autonomous, non-upgradeable, immutable program that:

- accepts participant deposits of `lock_mint` during a fixed participation window,
- accounts participation using discrete daily periods (86400 seconds),
- distributes a fixed and fully pre-funded reward reserve of `reward_mint = USDC`,
- enables post-maturity claims within a fixed claim window,
- enables post-maturity deposit withdrawals without time limit,
- transfers unclaimed rewards to `platform_treasury` after claim window expiration,
- enables issuer reclaim of the full reward reserve only in the zero-participation state.

This contract is an execution layer only.
It provides no discretionary controls and no post-deployment governance authority.

---

### 1.2 Scope

This Specification covers:

- immutable parameters and invariants,
- reserve funding rules,
- deposit eligibility and accounting rules,
- accumulator-based weight accounting model,
- deterministic reward formula,
- settlement phase state transitions:
  - claim,
  - withdraw deposit,
  - sweep unclaimed rewards,
  - zero-participation reclaim.

This Specification does not define:

- UI behavior,
- off-chain governance procedures,
- issuer admission policy,
- compensation policy,
- external dispute processes.

---

### 1.3 RCT Conformance and Lockrion v1 Profile

The contract is an implementation of RCT under a stricter Lockrion v1 Implementation Profile.

The following are mandatory:

- fixed commitment: reward reserve is finite and bounded by `reserve_total`,
- reserve non-extractability: reward reserve cannot be withdrawn prior to maturity,
- absolute immutability: parameters and logic cannot change after deployment,
- temporal discreteness: accounting period is 86400 seconds (calendar-day model),
- proportionality: rewards depend only on amount and time (linear weight),
- no bonuses: no multipliers, priorities, or nonlinear coefficients,
- reproducibility: all results are independently computable from public on-chain data.

Lockrion v1 profile constraints:

- `reward_mint` is fixed to USDC,
- `reserve_total` is fixed at deployment and must be fully funded before `start_ts`,
- reserve increases are prohibited after deployment,
- `fund_reserve()` is callable at most once and must transfer exactly `reserve_total`,
- all arithmetic is deterministic integer arithmetic (fixed-width, checked),
- settlement is a post-maturity claim phase with a fixed `claim_window`,
- no administrative, governance, pause, or emergency paths exist after deployment.

---

### 1.4 Authority and Immutability Model

After deployment:

- the issuance contract is non-upgradeable,
- no administrative keys remain that can alter logic or parameters,
- all state transitions are governed exclusively by on-chain deterministic rules.

No actor (platform, issuer, or participant) may:

- modify issuance parameters,
- modify accounting rules,
- withdraw reward reserve before maturity (except zero-participation reclaim after maturity),
- access participant deposits prior to maturity.

---

### 1.5 Determinism and Reproducibility Requirements

The contract MUST be deterministic.

- All calculations MUST use integer arithmetic only.
- Division MUST use deterministic floor rounding.
- Overflow MUST cause transaction failure (checked arithmetic).
- The canonical reward formula is:

  reward = reserve_total * user_weight_accum / total_weight_accum

Any participant MUST be able to reproduce:

- day index progression,
- accumulator updates,
- per-user weight accumulation,
- final reward amounts,
using only public on-chain data.

---

## 2. Immutable Parameters and State Model

### 2.1 Immutable Deployment Parameters

The following parameters are fixed at deployment and SHALL NOT be modified under any circumstances:

- issuer_address
- lock_mint
- reward_mint (USDC, fixed by Lockrion v1 profile)
- reserve_total
- start_ts
- maturity_ts
- accounting_period = 86400 seconds
- claim_window
- platform_treasury

Deployment MUST fail if:

- start_ts is not aligned to 00:00:00 UTC,
- maturity_ts <= start_ts,
- (maturity_ts - start_ts) mod 86400 != 0,
- reserve_total == 0,
- reward_mint != USDC.

These parameters define the full structural commitment of the issuance.

---

### 2.2 Global State Variables

### 2.2 Global State Variables

The contract maintains the following global state variables:

- reserve_funded (bool)
- total_locked (u128)
- total_weight_accum (u128)
- last_day_index (u64)
- final_day_index (u64)
- sweep_executed (bool)
- reclaim_executed (bool)

Definitions:

- final_day_index = (maturity_ts - start_ts) / 86400
- last_day_index represents the last accounting day fully accumulated.

Global state MUST NOT accumulate weight beyond final_day_index.

The variables sweep_executed and reclaim_executed enforce
single-execution guarantees for settlement-phase operations.

---

### 2.3 Per-User State Variables

For each participant address, the contract maintains:

- locked_amount (u128)
- user_weight_accum (u128)
- user_last_day_index (u64)
- reward_claimed (bool)

Per-user state MUST remain isolated and MUST NOT depend on other participants.

---

### 2.4 Escrow Accounts

Each issuance SHALL maintain two independent escrow accounts:

1. Deposit Escrow Account
   - Holds lock_mint tokens.
   - Receives deposits.
   - Releases tokens only via withdraw_deposit().

2. Reward Escrow Account
   - Holds USDC reserve.
   - Receives funding via fund_reserve().
   - Transfers rewards via claim_reward().
   - Transfers unclaimed rewards via sweep().
   - Transfers full reserve to issuer via zero-participation reclaim.

Escrow accounts SHALL NOT be commingled.
No other token transfer paths are permitted.

---

### 2.5 State Invariants (Final — Escrow Bound Clarification)

The following invariants MUST hold at all times:

1. total_locked equals the sum of all user.locked_amount.
2. total_weight_accum is monotonically non-decreasing.
3. last_day_index <= final_day_index.
4. Weight accumulation never occurs beyond final_day_index.
5. Sum of all claimed rewards ≤ reserve_total.
6. Deposit escrow balance MUST be ≥ sum of all user.locked_amount.
   Any excess lock_mint tokens directly transferred into the deposit escrow
   outside contract logic are ignored and do not affect accounting.
7. The reward distribution base is strictly bounded by reserve_total.
   The contract MUST NOT distribute more than reserve_total under any circumstances.
   If excess USDC is externally transferred into the reward escrow,
   such excess MUST NOT increase distributable rewards.
8. Reward escrow balance after zero-participation reclaim MUST equal zero.

Violation of any invariant MUST cause transaction failure.

---

## 3. Reserve Funding Rules

### 3.1 Funding Requirement

Each issuance defines a fixed reward reserve:

- reserve_total (USDC)

The reward reserve MUST be fully funded before participation begins.

Funding is performed via fund_reserve().

Rules:

- fund_reserve() MUST transfer exactly reserve_total.
- Any amount not equal to reserve_total MUST cause failure.
- fund_reserve() MAY be executed only once.
- Funding MUST occur strictly before start_ts.
- reserve_funded is set to true only after successful transfer verification.

Partial funding is strictly prohibited.

---

### 3.2 Deposit Eligibility Conditions

Deposits are permitted only if ALL of the following are true:

- reserve_funded == true
- block_timestamp >= start_ts
- block_timestamp < maturity_ts

If any condition is not satisfied, deposit MUST fail.

---

### 3.3 Prohibition of Reserve Modification

After successful funding:

- reserve_total SHALL NOT change.
- No additional funding is permitted.
- No reserve withdrawal is permitted before maturity.
- No reserve reweighting is permitted.

The reward reserve is immutable for the entire lifecycle of the issuance.

---

## 4. Participation and Deposit Rules

### 4.1 Participation Window

Participation is permitted strictly within:

start_ts <= block_timestamp < maturity_ts

Deposits outside this window MUST fail.

---

### 4.2 Multiple Deposits

A participant MAY submit multiple deposits during the participation window.

Each deposit:

- increases user.locked_amount,
- increases global.total_locked,
- does not reset prior accumulated weight,
- begins participation at the next accounting day.

---

### 4.3 No Early Withdrawal

Before maturity_ts:

- withdraw_deposit() MUST fail.
- Partial withdrawal is prohibited.
- Locked tokens remain inaccessible.

This rule is absolute and unconditional.

---

### 4.4 Canonical Deposit Execution Order (Clarified — Validation Phase)

The canonical execution order for deposit() SHALL be:

Validation Phase (no state mutation):

- reserve_funded == true
- start_ts <= block_timestamp < maturity_ts
- amount > 0
- correct token accounts and mint

Execution Phase:

1. Update global accumulator (bounded to final_day_index).
2. Update user accumulator.
3. Modify state:
   - user.locked_amount += amount
   - global.total_locked += amount
4. Execute SPL transfer of lock_mint to the Deposit Escrow Account.

Validation MUST precede accumulator updates.
Accumulator updates MUST precede state mutation.
State mutation MUST precede token transfer.

Deviation from this order is not permitted in v1.

---

### 4.5 Discrete Accounting Model

Accounting is based on fixed periods of 86400 seconds.

Day index is defined as:

day_index = floor((block_timestamp - start_ts) / 86400)

A deposit made during accounting day D begins participation at day D + 1.

Intraday transaction ordering has no effect on weight accumulation.

---

## 5. Accumulator-Based Accounting Model

### 5.1 Global Accumulator Update

Before any state-changing operation, the contract MUST update the global accumulator.

Procedure:

1. Compute raw_day_index:

   raw_day_index = floor((block_timestamp - start_ts) / 86400)

2. Compute current_day_index:

   current_day_index = min(raw_day_index, final_day_index)

3. Compute days_elapsed:

   days_elapsed = current_day_index - last_day_index

4. If days_elapsed > 0:

   total_weight_accum += total_locked * days_elapsed  
   last_day_index = current_day_index

Weight accumulation MUST NOT occur beyond final_day_index.

---

### 5.2 Per-User Accumulator Update (Revised — Strict Context Binding)

Before modifying user.locked_amount, the contract MUST update the user accumulator.

The current_day_index used here MUST be the value computed in Section 5.1.

Procedure:

1. Let current_day_index be the value defined in 5.1.
2. Compute days_elapsed_user:

   days_elapsed_user = current_day_index - user_last_day_index

3. If days_elapsed_user > 0:

   user_weight_accum += locked_amount * days_elapsed_user  
   user_last_day_index = current_day_index

User accumulation MUST always occur after global accumulation.

---

### 5.3 Finalization at Maturity

If block_timestamp >= maturity_ts:

- current_day_index MUST equal final_day_index.
- No weight accumulation beyond final_day_index is permitted.

All accumulation is strictly bounded to the participation window.

---

### 5.4 Monotonicity Requirements

The following properties MUST always hold:

- total_weight_accum is monotonically non-decreasing.
- user_weight_accum is monotonically non-decreasing.
- last_day_index is monotonically non-decreasing.
- user_last_day_index is monotonically non-decreasing.

Violation of these properties MUST cause transaction failure.

---

### 5.5 Initialization Rules

At deployment:

- total_locked = 0
- total_weight_accum = 0
- last_day_index = 0
- final_day_index = (maturity_ts - start_ts) / 86400
- reserve_funded = false
- sweep_executed = false
- reclaim_executed = false

For a new participant (first deposit):

- user.locked_amount = 0
- user.user_weight_accum = 0
- user.user_last_day_index = current_day_index (as computed in 5.1 at first interaction)
- user.reward_claimed = false

Initialization MUST ensure no unintended retroactive weight accumulation.

---

## 6. Settlement Logic and State Transitions

### 6.1 Maturity Condition

An issuance reaches maturity when:

block_timestamp >= maturity_ts

At this moment:

- No new deposits are permitted.
- Weight accumulation is bounded to final_day_index.
- Settlement functions become available.

Maturity cannot be extended or postponed.

---

### 6.2 Canonical Claim Execution Order (v1)

The canonical execution order for claim_reward() SHALL be:

1. Update global accumulator (finalize to final_day_index).
2. Update user accumulator.
3. Verify:
   - block_timestamp >= maturity_ts
   - block_timestamp < maturity_ts + claim_window
   - reward_claimed == false
   - total_weight_accum > 0
4. Compute reward:

   reward = reserve_total * user_weight_accum / total_weight_accum

5. Set user.reward_claimed = true.
6. Execute SPL transfer from Reward Escrow Account.

Accumulator finalization MUST precede reward calculation.

---

### 6.3 Reward Claim Window

The claim window is defined as:

maturity_ts <= block_timestamp < maturity_ts + claim_window

Outside this interval:

- claim_reward() MUST fail.

Claim window expiration permanently disables reward claims.

---

### 6.4 Deposit Withdrawal (Revised — Defensive Order)

Canonical execution order for withdraw_deposit() SHALL be:

1. Verify block_timestamp >= maturity_ts.
2. Let amount = user.locked_amount.
3. Decrease global.total_locked by amount.
4. Set user.locked_amount = 0.
5. Execute SPL transfer of amount from Deposit Escrow Account.

State mutation MUST precede token transfer.

Deposit withdrawal is not time-limited.
Withdrawal does not depend on reward claim status.

---

### 6.5 Unclaimed Reward Sweep (Revised — Defensive Order Alignment)

Sweep becomes available strictly when:

block_timestamp >= maturity_ts + claim_window

Conditions:

- total_weight_accum > 0
- reward escrow balance > 0
- sweep_executed == false

Canonical execution order:

1. Set sweep_executed = true.
2. Transfer entire remaining reward escrow balance to platform_treasury.

Sweep does not modify:

- user_weight_accum
- total_weight_accum
- deposit escrow balances

State mutation MUST precede token transfer.

---

### 6.6 Zero-Participation State (Final — Single-Execution Flag Enforcement)

If total_weight_accum == 0 at or after maturity_ts:

- claim_reward() MUST fail.
- sweep() MUST fail.

In this state, issuer_address MAY execute zero-participation reclaim.

Conditions:

- total_weight_accum == 0
- reward escrow balance > 0
- reclaim_executed == false

Execution:

1. Set reclaim_executed = true.
2. Transfer full reward escrow balance to issuer_address.

After execution:

- reward escrow balance MUST equal zero.
- Subsequent reclaim attempts MUST fail regardless of escrow balance.

Zero-participation reclaim does not affect deposit escrow.

---

## 7. Reward Distribution Model

### 7.1 Participation Weight Definition

Participation weight is defined as the product of:

- locked_amount
- number of full accounting days participated

Weight is accumulated exclusively through the accumulator mechanism defined in Section 5.

No alternative weight calculation methods are permitted.

---

### 7.2 Total Participation Weight

At maturity, total participation weight is:

total_weight_accum

This value is finalized by accumulator bounding at final_day_index.

No further modification of total_weight_accum is permitted after maturity finalization.

---

### 7.3 Canonical Reward Formula

The reward for a participant is strictly defined as:

reward = reserve_total * user_weight_accum / total_weight_accum

Properties:

- Division MUST use deterministic floor rounding.
- All arithmetic MUST use fixed-width checked integers.
- No floating-point operations are permitted.
- No rounding compensation adjustments are allowed.

Total distributed rewards MUST NOT exceed reserve_total.

---

### 7.4 Bounded Distribution Guarantee

The contract MUST guarantee:

- Sum of all claimed rewards ≤ reserve_total.
- No over-distribution is possible.
- Reward escrow balance never becomes negative.
- Reward escrow cannot exceed reserve_total.

Any violation MUST cause transaction failure.

---

### 7.5 Absence of Bonuses and Non-Linear Modifiers

The distribution model is strictly linear.

The following mechanisms are prohibited:

- Early participation bonuses.
- Volume multipliers.
- Address-based priority rules.
- Referral systems.
- Governance-based reward adjustments.
- Time decay multipliers.
- Manual reward overrides.

Reward entitlement depends exclusively on:

- locked_amount
- participation duration

---

### 7.6 Reproducibility Requirement

Any participant MUST be able to independently reconstruct:

- day indices,
- global accumulator progression,
- user accumulator progression,
- final reward amount,

using only:

- public on-chain timestamps,
- immutable issuance parameters,
- publicly visible state variables.

Reward computation must be fully deterministic and reproducible.

---

## 8. Deterministic Arithmetic and Safety Constraints

### 8.1 Integer Arithmetic Requirement

All arithmetic operations within the contract MUST use fixed-width unsigned integers.

- Multiplication MUST be performed using u128.
- Addition and subtraction MUST be checked.
- Division MUST use deterministic floor rounding.
- Any overflow or underflow MUST cause transaction failure.
- Floating-point arithmetic is strictly prohibited.

No implicit type casting is permitted.

---

### 8.2 Overflow Protection

The contract MUST enforce checked arithmetic for:

- total_locked updates,
- total_weight_accum updates,
- user_weight_accum updates,
- reward calculation.

If any intermediate calculation exceeds type bounds, the transaction MUST revert.

Overflow MUST NOT result in silent truncation.

---

### 8.3 Division Safety

Before computing reward:

- total_weight_accum MUST be greater than zero.

If total_weight_accum == 0:

- claim_reward() MUST fail.
- reward calculation MUST NOT be executed.

Division by zero is strictly prohibited.

---

### 8.4 Accumulator Bounding Guarantee

Weight accumulation MUST always be bounded by:

current_day_index = min(raw_day_index, final_day_index)

No accumulation is permitted beyond final_day_index.

This rule applies to:

- deposit(),
- claim_reward(),
- withdraw_deposit(),
- any state-changing instruction.

---

### 8.5 Deterministic State Transitions

All state transitions MUST be:

- deterministic,
- reproducible,
- independent of transaction ordering within a single accounting day.

Within the same accounting day:

- weight accumulation MUST remain unchanged.
- participation timing inside the day MUST NOT affect reward share.

---

### 8.6 Irreversibility

Once executed:

- reward_claimed = true SHALL NOT revert.
- sweep_executed = true SHALL NOT revert.
- zero-participation reclaim SHALL NOT be reversible.
- Deposit withdrawal SHALL permanently reduce user.locked_amount to zero.

No rollback or recalculation mechanism exists.

All settlement state transitions are final.

---

## 9. Security and Structural Integrity Requirements

### 9.1 Immutability Enforcement

The issuance contract MUST be deployed as non-upgradeable.

After deployment:

- Program upgrade authority MUST be revoked.
- No administrative authority MUST remain.
- No pause functionality MUST exist.
- No emergency override paths MUST exist.
- No proxy or upgrade pattern MUST be used.

Contract logic and parameters are permanently immutable.

---

### 9.2 Escrow Isolation Guarantee

The contract MUST maintain strict separation between:

- Deposit Escrow Account (lock_mint)
- Reward Escrow Account (USDC)

The following MUST be enforced:

- Deposit escrow funds MUST NOT be used for reward distribution.
- Reward escrow funds MUST NOT be used for deposit withdrawal.
- No commingling of funds is permitted.
- No hidden transfer paths are allowed.

All token transfers MUST occur only through explicitly defined instructions.

---

### 9.3 No Administrative Privilege

After deployment:

- The platform MUST NOT have authority to modify state.
- The issuer MUST NOT have authority to modify logic.
- No address MAY override execution paths.
- No discretionary intervention is permitted.

The only privileged operation permitted is zero-participation reclaim, callable exclusively by issuer_address and only under defined conditions.

---

### 9.4 Timestamp Model

The contract relies exclusively on:

block_timestamp provided by the Solana runtime.

The following MUST hold:

- start_ts MUST align to 00:00:00 UTC.
- maturity_ts MUST align to full accounting periods.
- day index MUST be computed deterministically.
- Minor timestamp drift MUST NOT affect reproducibility.

No alternative time sources are permitted.

---

### 9.5 Isolation Between Issuances

Each issuance contract instance MUST be fully isolated.

The following are prohibited:

- Shared escrow accounts.
- Cross-issuance state references.
- Cross-issuance reserve pooling.
- Cross-issuance dependency of weight calculation.

Failure of one issuance MUST NOT affect another.

---

### 9.6 Structural Integrity Principle

The issuance contract MUST guarantee:

- Bounded obligations.
- Deterministic execution.
- Immutable rules.
- Fully reproducible accounting.
- Strict separation of authority.
- Absence of discretionary intervention.

System trust derives exclusively from structural impossibility of rule deviation.

---

## 10. Versioning and Finality

### 10.1 Version Identity

This document defines:

Lockrion Issuance Contract — Specification v1.1 (Clean)

All contracts deployed under the Lockrion Issuance Contract v1 standard:

- are permanently governed by v1 logic,
- cannot adopt future versions,
- cannot be upgraded,
- cannot be migrated.

Each deployed issuance is permanently bound to its deployment version.

---

### 10.2 Forward Version Policy

Future changes to issuance logic require:

- creation of a new contract version,
- independent security validation,
- explicit version designation,
- deployment of new issuance instances.

Existing issuances remain unaffected.

No automatic migration mechanism exists.

---

### 10.3 No Backward Modification

After deployment:

- Parameters cannot change.
- Accounting logic cannot change.
- Reward formula cannot change.
- Settlement logic cannot change.
- Escrow structure cannot change.

Any deviation invalidates RCT conformance.

---

### 10.4 Architectural Finality

The following properties are permanent for v1:

- Fully pre-funded fixed reserve.
- No reserve increases.
- Discrete daily accounting (86400 seconds).
- Accumulator-based weight model.
- Deterministic integer arithmetic.
- No administrative intervention after deployment.
- Strict escrow isolation.
- Bounded reward distribution.

These properties define the canonical Lockrion Issuance Contract v1 model.

---

### 10.5 Normative Status

This Specification is normative.

If any other document (Design, commentary, marketing materials, or interface description) conflicts with this Specification:

- This Specification prevails.
- Implementation MUST conform to this Specification.
- Deviations require formal version increment.

This document defines the canonical execution behavior of Lockrion Issuance Contract v1.

---

## 11. Structural Conclusion

### 11.1 Deterministic Commitment Model

The Lockrion Issuance Contract v1 defines a deterministic, time-bound, reserve-backed commitment.

The contract guarantees:

- Full reserve pre-funding before participation.
- Immutable execution rules.
- Linear proportional reward distribution.
- Discrete daily accounting.
- Bounded obligations.
- Strict escrow isolation.

No external authority can alter execution after deployment.

---

### 11.2 Separation of Risk and Execution

The contract enforces strict separation between:

- Market risk (borne entirely by participants),
- Economic design responsibility (borne by issuer),
- Execution integrity (enforced by immutable code).

The contract does not:

- Guarantee yield,
- Guarantee profitability,
- Manage strategies,
- Provide advisory services,
- Intervene in economic outcomes.

Execution integrity is independent of market performance.

---

### 11.3 Structural Trust Principle

Trust in the system derives exclusively from:

- Immutability,
- Determinism,
- Reproducibility,
- Bounded obligations,
- Structural impossibility of rule deviation.

The contract does not rely on:

- Governance discretion,
- Reputation,
- Manual intervention,
- Administrative authority.

Correctness is enforced by code.

---

### 11.4 Canonical v1 Baseline

Specification v1.1 establishes the canonical execution baseline for:

Lockrion Issuance Contract v1.

All implementations claiming v1 compliance MUST:

- Preserve canonical execution order,
- Preserve accumulator-based accounting,
- Preserve escrow isolation,
- Preserve bounded reserve guarantees,
- Preserve settlement determinism,
- Preserve post-deployment immutability.

Deviation requires a new version.

This document defines the complete and authoritative behavioral model of Lockrion Issuance Contract v1.