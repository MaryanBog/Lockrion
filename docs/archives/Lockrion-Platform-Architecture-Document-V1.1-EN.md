# Lockrion
## Platform Architecture Document v1.1

Status: Draft
Version: 1.1
Network Target: Solana
Standard: RCT Issuance Contract v1

---

# 1. Purpose and Scope

Lockrion is a commercially governed but executionally autonomous platform for deploying time-locked reserve issuances in accordance with Reserve Commitment Theory (RCT).

This document defines the structural architecture of the platform, the separation of responsibilities, the on-chain issuance mechanics, the economic model, and the compliance positioning.

Lockrion operates under a strict separation principle:
- Governance controls access prior to deployment.
- Smart contracts control execution after deployment.

Once an issuance contract is deployed, its logic and parameters cannot be modified.

---

# 1.1 RCT Conformance and Implementation Profile

Lockrion is an implementation of Reserve Commitment Theory (RCT) operating under a defined Implementation Profile.

Lockrion fully adheres to all constitutional axioms of RCT, including:

- Fixed Commitment
- Reserve Non-Extractability
- Absolute Issuance Immutability
- Temporal Discreteness
- Proportional Distribution
- Issuance Independence
- Reproducibility
- Compensation Boundaries

Lockrion v1 introduces a stricter Implementation Profile with the following constraints:

## Fixed-Reserve Profile

Although RCT permits reserve increases prior to maturity,  
Lockrion v1 enforces a strictly fixed-reserve model:

- The full reward reserve must be deposited prior to `start_ts`.
- Reserve increases are not permitted after deployment.
- Reserve amount is immutable throughout the issuance lifecycle.

This restriction strengthens predictability and eliminates dynamic reserve reweighting.

## Fixed Reward Form

Lockrion v1 enforces a fixed reward asset:

- `reward_mint` is USDC.
- Reward asset is independent from `lock_mint`.
- No alternative reward forms are supported.

This represents a platform-level safety constraint and does not modify RCT principles.

## Settlement Phase Model

Settlement occurs after `maturity_ts` through a post-maturity claim phase.

Lockrion defines:

- A 90-day reward claim window.
- User-initiated `claim_reward()` instruction.
- Permissionless execution of reward transfers.
- Deterministic and immutable reward calculation.

Unclaimed rewards after the claim window are transferred to `platform_treasury`  
strictly according to pre-declared immutable rules.

This settlement structure complies with RCT Clarification on Settlement Phase.

## Deterministic Arithmetic Model

Lockrion v1 defines:

- Fixed-width integer arithmetic.
- Checked overflow behavior.
- Deterministic division rounding (floor).
- Canonical on-chain state as sole calculation source.

Independent reward reproducibility is guaranteed using public on-chain data only.

## No Discretion After Deployment

After contract deployment:

- No administrative keys exist.
- No pause or emergency withdrawal mechanisms exist.
- No governance intervention is possible.
- No parameter modification is permitted.

Lockrion governance authority exists exclusively prior to deployment.

Once deployed, an issuance becomes an autonomous RCT-compliant commitment  
governed solely by immutable smart-contract logic.

---

# 2. Structural Separation of Layers

Lockrion operates through three strictly separated layers:

1. Governance Layer (Off-Chain)
2. Issuance Layer (On-Chain)
3. Interface Layer (UI)

The Governance Layer operates exclusively prior to deployment. It performs issuer review, risk screening, and manual contract deployment.

The Issuance Layer consists of autonomous, non-upgradeable smart contracts deployed on Solana. Each issuance is independent and immutable.

The Interface Layer provides user interaction through a web application. It does not possess authority over contract execution and serves purely as a convenience interface.

After deployment, no off-chain layer may alter issuance behavior.

---

# 3. Governance Layer (Off-Chain)

The Governance Layer operates exclusively prior to issuance deployment.

It includes:

- Issuer application intake
- Soft KYC verification
- Token due diligence
- Economic model assessment
- Risk screening
- Determination of platform fee
- Manual deployment of issuance contract

Lockrion reserves full discretion to approve or reject any issuance application.

## 3.1 Issuer Review

Issuer identity data is:

- Collected off-chain
- Stored privately
- Not published publicly
- Not embedded in smart contracts

Soft KYC is required for issuers. Participants are not subject to KYC.

Issuer approval does not constitute endorsement, investment advice, or guarantee of economic outcome.

## 3.2 Platform Fee

Lockrion charges a fixed, individually determined fee per issuance.

The platform fee:

- Is paid prior to deployment
- Is not part of the issuance reserve
- Is not deducted from participant funds
- Is not stored in the issuance contract

Once an issuance contract is deployed, the Governance Layer has no authority to modify issuance parameters or execution logic.

---

# 4. Issuance Layer (On-Chain)

Each issuance is deployed as an independent, non-upgradeable smart contract on Solana.

Each issuance operates autonomously and does not depend on other issuances.

---

## 4.1 Immutability Guarantees

At deployment, the issuance contract enforces strict immutability of execution logic and parameters.

The following guarantees apply:

- Program upgrade authority is permanently revoked.
- No platform administrative keys are retained.
- No pause mechanisms exist.
- No emergency withdrawal mechanisms exist.
- No proxy upgrade patterns are used.
- No governance-controlled execution paths are present.

Issuance logic cannot be modified after deployment under any circumstances.

The only privileged operation defined within the issuance contract is the zero-participation reclaim described in Section 4.7.

This operation:

- Is callable exclusively by the immutable `issuer_address`.
- Is executable only if `total_weight_accum == 0` at or after `maturity_ts`.
- Does not allow modification of issuance parameters.
- Does not allow withdrawal of participant deposits.
- Does not allow alteration of reward distribution rules.

No platform authority exists after deployment.

No discretionary intervention in execution logic is possible.

All state transitions are strictly deterministic and governed solely by immutable smart-contract code.

---

## 4.2 Immutable Issuance Parameters

Each issuance contract defines the following immutable parameters:

- `issuer_address` — immutable issuer authority address
- `lock_mint` — SPL token locked by participants
- `reward_mint` — USDC (fixed for v1)
- `reserve_total` — full USDC reserve
- `start_ts`
- `maturity_ts`
- `accounting_period = 86400 seconds`
- `claim_window = 90 days`
- `platform_treasury` — immutable treasury address

These parameters are fixed at deployment and cannot be altered.

---

### UTC Alignment Requirement

To preserve the constitutional principle of calendar-day accounting under RCT, the following rules apply:

- `start_ts` MUST be aligned to 00:00:00 UTC.
- `accounting_period` is strictly defined as 86400 seconds.
- Day indexing is computed deterministically using on-chain `block_timestamp`.

Day index is defined as:

day_index = floor((block_timestamp - start_ts) / 86400)

Each accounting period represents a 86400-second interval
starting from the aligned `start_ts`.

The system does not rely on external time sources.
All time calculations are based exclusively on Solana `block_timestamp`
as provided by the runtime.

Minor timestamp drift inherent to blockchain systems
does not affect deterministic reproducibility,
as all participants rely on the same canonical on-chain time source.

If `start_ts` is not aligned to 00:00:00 UTC at deployment,
the issuance MUST be rejected.

No alternative alignment, offset-based period calculation,
or dynamic time adjustment is permitted.

---

## 4.3 Reserve Requirement

The issuance reserve must be fully funded prior to the beginning of the participation window.

The issuance contract defines an immutable parameter:

- `reserve_total` — the exact USDC amount required to fully fund the reward reserve.

Reserve funding is performed exclusively through the on-chain instruction `fund_reserve()`.

The following rules apply:

- `fund_reserve()` MUST transfer exactly `reserve_total`.
- Any amount other than `reserve_total` MUST be rejected.
- `fund_reserve()` MUST be executable at most once.
- Reserve funding MUST be completed strictly before `start_ts`.

A boolean state variable `reserve_funded` is set to `true`
only when the full `reserve_total` has been successfully deposited.

Deposit eligibility is strictly determined by on-chain state:

- `deposit()` is permitted only if:
    - `reserve_funded == true`
    - `block_timestamp ≥ start_ts`
    - `block_timestamp < maturity_ts`
- If `reserve_funded == false`, all deposit attempts MUST be rejected.

No partial funding is permitted.
No discretionary approval is involved.
No off-chain confirmation is required.

Issuance participation is enabled exclusively by the on-chain condition
`reserve_funded == true`.

No separate activation mechanism exists.

---

### 4.3.1 Reserve Funding and Activation Gate

At deployment, the issuance defines an immutable parameter:

- `reserve_total` — the exact USDC amount required to fully fund the reward reserve.

The issuer must fund the reserve by transferring USDC directly to the reward escrow account via a dedicated on-chain instruction `fund_reserve()`.

Reserve funding is atomic and non-partial:

- `fund_reserve()` MUST transfer exactly `reserve_total`.
- Any amount other than `reserve_total` MUST be rejected.
- `fund_reserve()` MUST be executable at most once.

A boolean state variable `reserve_funded` is set to `true` only when the full `reserve_total` has been deposited.

Deposit acceptance is strictly gated by reserve funding:

- `deposit()` is allowed only if `reserve_funded == true`
- If `reserve_funded == false`, all deposit attempts are rejected.

Reserve funding must be completed prior to `start_ts`.

No partial activation is permitted.
No discretionary approval is involved.
No off-chain confirmation is required.

Issuance activation is purely deterministic and enforced entirely by on-chain state.

---

### 4.3.2 Funding Deadline and Permanent Inactivation

Reserve funding must be completed prior to `start_ts`.

If reserve funding has not been completed by `start_ts` (`reserve_funded == false`), the issuance becomes permanently inactive.

In this state:

- `fund_reserve()` is permanently disabled.
- `deposit()` is permanently disabled.
- No activation is possible at any later time.

No refund, recovery, or reactivation mechanism exists.

This rule is absolute and enforced entirely on-chain.

---

## 4.4 Deposit Rules

Deposits are permitted only within the active issuance window:

start_ts ≤ block_timestamp < maturity_ts

The following conditions apply:

- Multiple deposits per address are permitted
- Partial withdrawals are prohibited
- Deposits after maturity are rejected

Participation is calculated in discrete UTC-based accounting days.

Day index is defined as:

day_index = floor((block_timestamp - start_ts) / 86400)

A deposit made during day D begins participation at day D + 1.

Intraday timing has no effect on reward calculation.

---

# 4.5 Accounting Model (Accumulator-Based)

Lockrion uses an accumulator-based weight model to ensure deterministic,
gas-efficient, and iteration-free participation accounting.

All accounting updates are executed lazily and triggered exclusively
by state-changing user interactions.

The contract maintains:

Global State:
- `total_locked`
- `total_weight_accum`
- `last_day_index`

Per-User State:
- `locked_amount`
- `user_weight_accum`
- `user_last_day_index`
- `reward_claimed`

---

## 4.5.1 Day Index Definition

Day index is defined as:

day_index = floor((block_timestamp - start_ts) / 86400)

If `block_timestamp < start_ts`, deposits are rejected.

All weight accumulation is based strictly on integer day indices.

Intraday timing has no effect.

---

## 4.5.2 Global Weight Update Rule

Before any state-changing operation
(`deposit()`, `withdraw_deposit()`, `claim_reward()`),
the contract MUST perform a global weight update.

Let:

current_day_index = computed day index  
days_elapsed = current_day_index - last_day_index

If `days_elapsed > 0`, then:

total_weight_accum += total_locked × days_elapsed
last_day_index = current_day_index

If `days_elapsed == 0`, no global update occurs.

This update MUST be executed before modifying `total_locked`.

---

## 4.5.3 Per-User Weight Update Rule

Before modifying a user's `locked_amount`,
the contract MUST update the user accumulator.

Let:

days_elapsed_user = current_day_index - user_last_day_index

If `days_elapsed_user > 0`, then:

user_weight_accum += locked_amount × days_elapsed_user
user_last_day_index = current_day_index

If `days_elapsed_user == 0`, no user update occurs.

This update MUST be executed before modifying `locked_amount`.

---

## 4.5.4 Deposit Behavior

When `deposit(amount)` is executed:

1. Global accumulator is updated.
2. User accumulator is updated.
3. `locked_amount += amount`
4. `total_locked += amount`

A deposit made during day D begins participation at day D + 1,
since the accumulator logic only increases weight
for completed day intervals.

---

## 4.5.5 Withdraw Behavior (Post-Maturity Only)

When `withdraw_deposit()` is executed after `maturity_ts`:

1. Global accumulator is updated.
2. User accumulator is updated.
3. `total_locked -= locked_amount`
4. `locked_amount = 0`

Withdrawals are prohibited before maturity.

---

## 4.5.6 Claim Behavior

When `claim_reward()` is executed:

1. Global accumulator is updated.
2. User accumulator is updated.
3. Reward is calculated deterministically.
4. `reward_claimed` flag is set.

Claim does not modify `locked_amount`.

---

## 4.5.7 Deterministic Properties

The accumulator model guarantees:

- No iteration over participant set.
- Deterministic weight accumulation.
- Identical results regardless of transaction ordering within a single day.
- Reproducibility from on-chain state only.

All arithmetic uses `u128` as defined in the Deterministic Arithmetic Specification.

No floating-point operations are permitted.

---

# 4.6 Settlement Structure

After `maturity_ts`, the issuance transitions into the settlement phase.

Settlement consists of reward distribution and deposit withdrawal operations,
executed strictly according to immutable on-chain state.

Two independent user actions are available:

1. claim_reward()

   - Available only if `total_weight_accum > 0`
   - Available only within the reward claim window
   - Transfers proportional USDC reward
   - Does not automatically withdraw deposit
   - Executable only if:
       - `maturity_ts ≤ block_timestamp < maturity_ts + claim_window`
       - `total_weight_accum > 0`
   - MUST fail if `total_weight_accum == 0`
   - MUST fail if called outside the claim window
   - MUST fail if reward already claimed

   Reward calculation:

   reward = reserve_total × user_weight_accum / total_weight_accum

   Division by zero is strictly prohibited.
   If `total_weight_accum == 0`, reward calculation is undefined
   and claim operations are permanently disabled.

2. withdraw_deposit()

   - Available indefinitely after `maturity_ts`
   - Returns full locked token amount
   - Not time-limited
   - Independent from reward claim

Reward claims are strictly bounded by the claim window:

`maturity_ts ≤ block_timestamp < maturity_ts + claim_window`

After expiration of the 90-day claim window:

- `claim_reward()` is permanently disabled
- Unclaimed USDC rewards become sweepable
- `withdraw_deposit()` remains permanently available

If `total_weight_accum == 0` at `maturity_ts`,
no participant has acquired reward entitlement.
In such case, `claim_reward()` is permanently disabled,
and settlement proceeds according to Section 4.7 (Zero Participation Reclaim).

Settlement logic is fully deterministic,
executed exclusively on-chain,
and cannot be modified after deployment.

---

### 4.6.1 Unclaimed Reward Sweep

After expiration of the reward claim window, any unclaimed reward funds may be transferred to `platform_treasury` via a permissionless on-chain instruction.

The claim window is defined as:

`maturity_ts ≤ block_timestamp < maturity_ts + claim_window`

The sweep phase begins strictly at:

`block_timestamp ≥ maturity_ts + claim_window`

The two phases do not overlap under any circumstances.

The sweep mechanism operates under the following rules:

- Callable by any address.
- Executable only if:
    - `block_timestamp ≥ maturity_ts + claim_window`
    - Reward claim window has fully expired.
    - Reward escrow account balance > 0.
- Executable only once.
- Protected by an immutable state flag preventing re-execution.
- Transfers only the remaining balance of the reward escrow account.
- Does not affect deposit escrow accounts.
- Does not modify participation accounting state.
- Does not alter previously claimed rewards.

If the sweep instruction is never executed, unclaimed reward funds remain in the reward escrow account indefinitely.

Sweep execution is purely mechanical and does not require governance approval.

No discretionary authority is involved.

---

## 4.7 Zero Participation Reclaim

If `total_weight_accum == 0` at `maturity_ts`, no participant has acquired any reward entitlement.

In this case, the issuer is entitled to reclaim the full reward reserve.

The reclaim mechanism operates under the following rules:

- Callable only by the immutable `issuer_address` defined at deployment.
- Executable only if:
  - `block_timestamp ≥ maturity_ts`
  - `total_weight_accum == 0`
- Executable only once.
- Protected by an immutable state flag preventing re-execution.
- Transfers the full balance of the reward escrow account to `issuer_address`.
- Does not affect deposit escrow accounts.
- Does not modify participation accounting state.

If at least one valid deposit was recorded during the issuance lifecycle (`total_weight_accum > 0`), reclaim is permanently disabled and standard settlement rules apply.

No discretionary authority is involved in this process.

---

# 5. Escrow Structure

Each issuance maintains segregated escrow token accounts:

- Escrow account for `lock_mint`
- Escrow account for USDC reserve

The following principles apply:

- Deposits and reward reserves are separated
- No commingling of funds occurs
- Escrow accounts are controlled exclusively by the issuance contract
- Funds cannot be accessed outside contract-defined rules

Escrow design ensures that:

- Participant deposits remain fully returnable
- Reward distribution remains bounded by declared reserve
- Platform governance cannot access issuance funds

---

# 6. Interface Layer (UI)

Lockrion provides a web-based interface to facilitate interaction with issuance contracts.

The Interface Layer enables:

- Public viewing of issuance parameters
- Wallet connection
- Deposit submission
- Real-time reward estimation
- Reward claim execution
- Deposit withdrawal

The UI does not possess administrative authority over contracts.

Direct interaction with issuance contracts through blockchain tools remains unrestricted.

The Interface Layer is a convenience mechanism and does not alter execution logic.

---

# 7. Economic Model

## 7.1 Platform Revenue

Lockrion revenue derives from two sources:

1. Fixed issuance deployment fee
   - Determined individually during issuer review
   - Paid prior to deployment
   - Not deducted from participant funds
   - Not included in reserve

2. Unclaimed rewards
   - Transferred to `platform_treasury` after expiration of claim window
   - Executed strictly according to pre-declared issuance rules
   - Not discretionary

Lockrion does not charge percentage-based fees from participants.

## 7.2 Risk Allocation

Participants bear:

- Market risk of locked token
- Liquidity risk
- Volatility risk
- Opportunity cost risk

Lockrion bears:

- Correct execution responsibility
- Contract integrity responsibility

Lockrion does not guarantee yield, profit, or financial performance.

---

# 8. Security Model

Lockrion Issuance Contracts v1 are designed with strict immutability and minimal attack surface.

The following security properties apply:

- Non-upgradeable program deployment
- Upgrade authority permanently revoked
- No administrative keys retained
- No pause functionality
- No emergency withdrawal mechanisms
- No hidden control paths
- No off-chain settlement dependencies

All reward calculations are deterministic and executed on-chain.

All accounting data required for independent verification is publicly accessible.

Security assumptions rely solely on:

- Solana network integrity
- SPL token program integrity
- Correct contract implementation

---

No discretionary trust in Lockrion governance is required after deployment.

Once an issuance contract is deployed, no administrative authority can alter its parameters, execution logic, or fund flows.

System correctness depends solely on:

- the integrity of the Solana network,
- the integrity of the SPL token program,
- the correctness of the deployed contract code.

No post-deployment governance intervention is possible.

---

# 9. Compliance Position

Lockrion operates as a deployment and governance platform for autonomous smart-contract issuances.

The following compliance position applies:

- Soft KYC required for issuers
- No KYC required for participants
- Lockrion does not provide investment advice
- Lockrion does not manage participant assets
- Lockrion does not promise yield or financial returns
- Lockrion does not guarantee profitability

Issuances represent fixed, time-bound smart-contract commitments.

Economic outcomes depend exclusively on predefined rules and market conditions.

---

# 9.1 Compensation Handling Policy

Lockrion implements compensation strictly in accordance with the constitutional provisions of Reserve Commitment Theory (RCT).

Compensation applies only in cases of a proven violation of declared issuance rules.

The following principles govern compensation handling:

- Compensation does not modify, pause, or alter any active or completed issuance.
- No recalculation or redistribution of issuance funds is permitted.
- Compensation is executed exclusively outside the issuance contract.
- Compensation is limited by the pre-declared compensation cap of the specific issuance.
- Market risks, price volatility, and unrealized profit are not subject to compensation.

Compensation review is performed off-chain by the Governance Layer.

If a violation is confirmed:

- Compensation is paid from platform-controlled funds or designated insurance reserves.
- The issuance contract remains immutable.
- No precedent is created for other issuances.

No discretionary intervention in issuance logic is permitted under any circumstances.

---

# 10. Versioning Policy

Current standard:

Lockrion Issuance Contract v1

Future modifications require:

- Deployment of new contract versions
- No alteration of existing issuances
- No retroactive rule changes

Each issuance remains permanently governed by the version deployed at its creation.

---

# 11. Structural Conclusion

Lockrion combines commercially governed access with execution-level immutability.

Governance determines eligibility to launch.
Smart contracts determine execution mechanics.

Trust is established through structural impossibility of rule deviation.

Immutability protects participants.
Transparency enables verification.
Layer separation ensures systemic integrity.

Lockrion v1 establishes the foundational architecture for controlled yet autonomous time-locked reserve issuances.
