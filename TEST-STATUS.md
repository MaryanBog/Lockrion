## ✅ Implemented Tests

### 001_deploy_sanity.sh
Status: PASS  
Scope:
- Verify RPC availability
- Deploy via deploy_lockrion_gitbash.sh
- Verify `solana program show`
- Verify upgradeable loader
- Verify ProgramData header integrity

Run:
chmod +x tests/001_deploy_sanity.sh
bash tests/001_deploy_sanity.sh

---

### 002_programdata_sanity.sh
Status: PASS  
Scope:
- Verify ProgramData address
- Verify ProgramData owner (BPFLoaderUpgradeab1e)
- Verify ProgramData header can be read without failure

Run:
chmod +x tests/002_programdata_sanity.sh
bash tests/002_programdata_sanity.sh

---

### 003_fund_reserve_happy.sh
Status: PASS  
Scope:
- Deploy
- init_issuance
- fund_reserve()
- Verify reserve_funded == 1
- Verify reward escrow balance == reserve_total

Run:
chmod +x tests/003_fund_reserve_happy.sh
bash tests/003_fund_reserve_happy.sh

---

### 004_double_fund_reserve_rejected.sh
Status: PASS  
Scope:
- Deploy
- init_issuance
- fund_reserve() successful (first call)
- fund_reserve() second call must be rejected (double funding attempt)

Expected:
- Transaction fails with ReserveAlreadyFunded (program error path)

Run:
chmod +x tests/004_double_fund_reserve_rejected.sh
bash tests/004_double_fund_reserve_rejected.sh

---

### 005_deposit_happy.sh
Status: PASS  
Scope:
- Deploy
- init_issuance
- fund_reserve()
- Wait for start_ts using on-chain time (no solana warp)
- Mint lock tokens to participant
- deposit(amount)
- Verify deposit_escrow balance == amount
- Verify issuance.total_locked == amount (read from account bytes, u128 LE)

Expected:
- deposit succeeds
- escrow balance increases by amount
- total_locked updates deterministically

Run:
chmod +x tests/005_deposit_happy.sh
bash tests/005_deposit_happy.sh

---

### 006_deposit_before_funding_rejected.sh
Status: PASS  
Scope:
- Deploy
- init_issuance (reserve NOT funded)
- Wait for start_ts using on-chain time
- deposit(amount) must be rejected

Expected:
- Transaction fails with ReserveNotFunded
- deposit_escrow balance remains unchanged

Run:
chmod +x tests/006_deposit_before_funding_rejected.sh
bash tests/006_deposit_before_funding_rejected.sh

---

### 007_deposit_before_start_rejected.sh
Status: PASS  
Scope:
- Deploy
- init_issuance (start_ts set in the future)
- fund_reserve()
- deposit(amount) BEFORE start_ts must be rejected

Expected:
- Transaction fails with DepositWindowNotStarted
- deposit_escrow balance remains unchanged

Run:
chmod +x tests/007_deposit_before_start_rejected.sh
bash tests/007_deposit_before_start_rejected.sh

---

### 008_deposit_after_maturity_rejected.sh
Status: PASS  
Scope:
- Deploy
- init_issuance (short window start_ts → maturity_ts)
- fund_reserve()
- Wait for maturity_ts using on-chain time
- deposit(amount) AFTER maturity_ts must be rejected

Expected:
- Transaction fails with DepositWindowClosed
- deposit_escrow balance remains unchanged

Run:
chmod +x tests/008_deposit_after_maturity_rejected.sh
bash tests/008_deposit_after_maturity_rejected.sh

---

### 009_deposit_zero_amount_rejected.sh
Status: PASS  
Scope:
- Deploy
- init_issuance
- fund_reserve()
- Wait for start_ts using on-chain time
- deposit(0) must be rejected

Expected:
- Transaction fails with InvalidAmount
- deposit_escrow balance remains unchanged

Run:
chmod +x tests/009_deposit_zero_amount_rejected.sh
bash tests/009_deposit_zero_amount_rejected.sh

---

### 010_deposit_invalid_mint_rejected.sh
Status: PASS  
Scope:
- Deploy
- init_issuance (LOCK_MINT = A)
- fund_reserve()
- Wait for start_ts using on-chain time
- Participant uses ATA with a different mint (B)
- deposit(amount) must be rejected

Expected:
- Transaction fails with InvalidMint
- deposit_escrow balance remains unchanged

Run:
chmod +x tests/010_deposit_invalid_mint_rejected.sh
bash tests/010_deposit_invalid_mint_rejected.sh

---

## 011_claim_happy_pt.rs (solana-program-test)

Status: PASS

Scope:
- ProgramTest (in-process, no WSL / solana-test-validator)
- init_issuance
- fund_reserve() (before start_ts)
- deposit() (after start_ts)
- Warp to maturity_ts (using test-clock feature)
- claim_reward()
- Verify participant_reward balance increased

Run:
cargo test --features test-clock --test 011_claim_happy_pt -- --nocapture

Notes:
- test-clock feature is used only for tests
- test-clock is NOT enabled in production build
- Production build verification:
cargo build-sbf --no-default-features
PASS

---

### 012_claim_double_rejected_pt.rs
Status: PASS  
Scope:
- ProgramTest
- init_issuance
- fund_reserve (before start_ts)
- deposit (after start_ts)
- warp to maturity_ts (test-clock feature)
- claim_reward() first call succeeds
- claim_reward() second call must be rejected

Expected:
- Second claim fails with AlreadyClaimed

Run:
cargo test --features test-clock --test 012_claim_double_rejected_pt -- --nocapture

---

### 013_claim_before_maturity_rejected_pt.rs
Status: PASS  
Scope:
- ProgramTest
- init_issuance
- fund_reserve (before start_ts)
- deposit (after start_ts)
- claim_reward() BEFORE maturity_ts must be rejected

Expected:
- Transaction fails with ClaimWindowNotStarted

Run:
cargo test --features test-clock --test 013_claim_before_maturity_rejected_pt -- --nocapture

---

### 014_withdraw_deposit_happy_pt.rs
Status: PASS  
Scope:
- ProgramTest
- init_issuance
- fund_reserve (before start_ts)
- deposit (after start_ts)
- warp to maturity_ts (test-clock feature)
- withdraw_deposit() succeeds
- Verify deposit escrow decreases
- Verify participant_lock balance increases

Run:
cargo test --features test-clock --test 014_withdraw_deposit_happy_pt -- --nocapture

---

### 015_withdraw_before_maturity_rejected_pt.rs
Status: PASS  
Scope:
- ProgramTest
- init_issuance
- fund_reserve (before start_ts)
- deposit (after start_ts)
- withdraw_deposit() BEFORE maturity_ts must be rejected

Expected:
- Transaction fails with DepositWindowNotClosed

Run:
cargo test --features test-clock --test 015_withdraw_before_maturity_rejected_pt -- --nocapture

---

### 016_sweep_happy_pt.rs
Status: PASS  
Scope:
- ProgramTest
- init_issuance
- fund_reserve (before start_ts)
- deposit (after start_ts) to ensure participation exists
- claim_reward() at maturity_ts (to transfer a portion to the participant)
- warp to maturity_ts + claim_window + 10
- sweep() succeeds
- Verify: reward_escrow -> 0
- Verify: platform_treasury_token_acc increases by the remaining balance

Run:
cargo test --features test-clock --test 016_sweep_happy_pt -- --nocapture

---

### 017_zero_participation_reclaim_happy_pt.rs
Status: PASS  
Scope:
- ProgramTest
- init_issuance
- fund_reserve (before start_ts)
- No deposit (total_weight_accum == 0)
- warp to maturity_ts
- zero_participation_reclaim() succeeds
- Verify: reward_escrow -> 0
- Verify: issuer_reward_token_acc increases by the full remaining balance

Run:
cargo test --features test-clock --test 017_zero_participation_reclaim_happy_pt -- --nocapture

---

### 018_sweep_double_rejected_pt.rs
Status: PASS  
Expected:
- Second sweep fails with SweepAlreadyExecuted

Run:
cargo test --features test-clock --test 018_sweep_double_rejected_pt -- --nocapture

---

### 019_reclaim_double_rejected_pt.rs
Status: PASS  
Expected:
- Second reclaim fails with ReclaimAlreadyExecuted

Run:
cargo test --features test-clock --test 019_reclaim_double_rejected_pt -- --nocapture

---

### 020_sweep_before_claim_window_end_rejected_pt.rs
Status: PASS  
Expected:
- sweep() is rejected before sweep_start

Run:
cargo test --features test-clock --test 020_sweep_before_claim_window_end_rejected_pt -- --nocapture

---

### 021_reclaim_with_participation_rejected_pt.rs
Status: PASS  
Expected:
- Reclaim is rejected when participation exists (total_weight_accum > 0)

Run:
cargo test --features test-clock --test 021_reclaim_with_participation_rejected_pt -- --nocapture

---

### 022_claim_after_claim_window_rejected_pt.rs
Status: PASS  
Expected:
- Claim after claim_window fails with ClaimWindowClosed

Run:
cargo test --features test-clock --test 022_claim_after_claim_window_rejected_pt -- --nocapture

---

### 023_invalid_token_program_rejected_pt.rs
Status: PASS  
Expected:
- Transaction fails with InvalidTokenProgram

Run:
cargo test --features test-clock --test 023_invalid_token_program_rejected_pt -- --nocapture

---

### 024_substitute_deposit_escrow_rejected_pt.rs
Status: PASS  
Expected:
- Transaction fails with InvalidEscrowAccount

Run:
cargo test --features test-clock --test 024_substitute_deposit_escrow_rejected_pt -- --nocapture

---

### 025_substitute_reward_escrow_rejected_pt.rs
Status: PASS  
Expected:
- Transaction fails with InvalidEscrowAccount

Run:
cargo test --features test-clock --test 025_substitute_reward_escrow_rejected_pt -- --nocapture

---

### 026_substitute_user_state_rejected_pt.rs
Status: PASS  
Expected:
- Transaction fails with InvalidPDA

Run:
cargo test --features test-clock --test 026_substitute_user_state_rejected_pt -- --nocapture

---

### 027_deposit_atomicity_cpi_fail_pt.rs
Status: PASS  
Expected:
- State remains unchanged if the CPI transfer fails

Run:
cargo test --features test-clock --test 027_deposit_atomicity_cpi_fail_pt -- --nocapture

---

### 028_arithmetic_overflow_rejected_pt.rs
Status: PASS  
Expected:
- Transaction fails with ArithmeticOverflow

Run:
cargo test --features test-clock --test 028_arithmetic_overflow_rejected_pt -- --nocapture

---

### accumulator_unit.rs
Status: PASS  
Expected:
- Strict discrete day model (raw_day_index)
- Deterministic 86400 boundary handling
- Bounded day index clamp
- Overflow / underflow guards
- Same-day determinism

Run:
cargo test --test accumulator_unit

---

### processor_unit.rs
Status: PASS  
Expected:
- Global accumulator accumulation logic (UT-GACC-01..04)
- Per-user accumulator logic (UT-UACC-01..04)
- Bounded accumulation correctness
- Same-day determinism
- Reward formula correctness (floor division, overflow guard, division-by-zero guard)
- Arithmetic safety checks

Run:
cargo test --test processor_unit

---

### 029_claim_after_withdraw_should_fail.rs
Status: PASS  
Expected:
- Withdraw deposit first, then claim_reward must fail OR produce zero reward (protocol-dependent)
- No state corruption (no unexpected balance/state changes)

Run:
cargo test --features test-clock --test 029_claim_after_withdraw_should_fail -- --nocapture

---

### 030_double_withdraw_should_fail.rs
Status: PASS  
Expected:
- Second withdraw attempt must fail (locked_amount already zero)
- No state mutation (balances remain unchanged)

Run:
cargo test --features test-clock --test 030_double_withdraw_should_fail -- --nocapture

---

### 031_claim_then_claim_again.rs
Status: PASS  
Expected:
- reward_claimed flag prevents double claim
- Second claim fails (AlreadyClaimed)
- No double transfer (balances unchanged on second attempt)

Run:
cargo test --features test-clock --test 031_claim_then_claim_again -- --nocapture

---

### 032_claim_exact_maturity_boundary.rs
Status: PASS  
Expected:
- Claim at exact maturity_ts succeeds
- Accumulators finalized correctly

Run:
cargo test --features test-clock --test 032_claim_exact_maturity_boundary -- --nocapture

---

### 033_claim_exact_claim_window_end.rs
Status: PASS  
Expected:
- Claim at maturity_ts + claim_window - 1 succeeds
- Deterministic boundary behavior

Run:
cargo test --features test-clock --test 033_claim_exact_claim_window_end -- --nocapture

---

### 034_claim_after_window_end.rs
Status: PASS  
Expected:
- Claim at maturity_ts + claim_window fails
- ClaimWindowClosed error

Run:
cargo test --features test-clock --test 034_claim_after_window_end -- --nocapture

---

### 035_sweep_transfers_full_balance
Status: PASS   
Expected:
- Entire reward escrow balance transferred to platform treasury
- sweep_executed flag set

Run:
cargo test --features test-clock --test 035_sweep_transfers_full_balance -- --nocapture

---

### 036_sweep_twice_should_fail
Status: PASS  
Expected:
- Second sweep attempt fails
- sweep_executed guard enforced

Run:
cargo test --features test-clock --test 036_sweep_twice_should_fail -- --nocapture

---

### 037_reclaim_only_if_zero_participation
Status: PASS   
Expected:
- Reclaim succeeds only if total_weight_accum == 0
- Otherwise returns NoParticipation

Run:
cargo test --features test-clock --test 037_reclaim_only_if_zero_participation -- --nocapture

---

### 038_two_users_proportional_distribution
Status: PASS  
Expected:
- Two users with different locked_amount
- Rewards proportional to accumulated weights
- No rounding drift beyond floor division

Run:
cargo test --features test-clock --test 038_two_users_proportional_distribution -- --nocapture

---

### 039_same_sequence_same_result
Status: PASS  
Expected:
- Identical sequence of instructions produces identical final state
- Deterministic replay guarantee

Run:
cargo test --features test-clock --test 039_same_sequence_same_result -- --nocapture

---

### 040_fund_reserve_wrong_amount_rejected_pt
Status: PASS  
Expected:
- fund_reserve() called with amount != reserve_total
- Transaction fails with InvalidFundingAmount
- reserve_funded remains false
- Reward escrow balance remains unchanged

Run:
cargo test --features test-clock --test 040_fund_reserve_wrong_amount_rejected_pt -- --nocapture

---

### 041_fund_reserve_non_issuer_rejected_pt
Status: PASS  
Expected:
- fund_reserve() called by non-issuer signer
- Transaction fails with UnauthorizedCaller
- reserve_funded remains false
- Reward escrow balance remains unchanged

Run:
cargo test --features test-clock --test 041_fund_reserve_non_issuer_rejected_pt -- --nocapture

---

### 042_claim_zero_participation_rejected_pt
Status: PASS  
Expected:
- No deposits performed (total_weight_accum == 0)
- Warp to maturity_ts
- claim_reward() fails with NoParticipation
- reward_claimed remains false
- Reward escrow balance unchanged

Run:
cargo test --features test-clock --test 042_claim_zero_participation_rejected_pt -- --nocapture

---

### 043_sweep_zero_participation_rejected_pt
Status: PASS  
Expected:
- No deposits performed (total_weight_accum == 0)
- Warp to maturity_ts + claim_window
- sweep() fails with NoParticipation
- sweep_executed remains false
- Reward escrow balance unchanged

Run:
cargo test --features test-clock --test 043_sweep_zero_participation_rejected_pt -- --nocapture

---

### 044_substitute_platform_treasury_rejected_pt
Status: PASS  
Expected:
- Valid issuance with participation
- Warp to maturity_ts + claim_window
- sweep() called with incorrect platform_treasury account
- Transaction fails with InvalidPlatformTreasury
- sweep_executed remains false
- Reward escrow balance unchanged

Run:
cargo test --features test-clock --test 044_substitute_platform_treasury_rejected_pt -- --nocapture

---

### 045_seed_order_mutation_invalid_pda_pt
Status: PASS  
Expected:
- Issuance PDA derived using altered seed order
- Provided IssuanceState account does not match canonical PDA
- Any instruction referencing mutated PDA fails with InvalidPDA
- No state mutation occurs

Run:
cargo test --features test-clock --test 045_seed_order_mutation_invalid_pda_pt -- --nocapture

---

### 046_seed_endianness_mutation_invalid_pda_pt
Status: PASS  
Expected:
- Issuance PDA derived using non-little-endian encoding for start_ts or reserve_total
- Provided IssuanceState account does not match canonical PDA
- Instruction fails with InvalidPDA
- No state mutation occurs

Run:
cargo test --features test-clock --test 046_seed_endianness_mutation_invalid_pda_pt -- --nocapture