# Lockrion Issuance Contract
## Instruction Execution Order v1.1

Status: Draft  
Applies To: Raw Solana Program  
Scope: Deterministic validation and execution order  

All instructions MUST follow the exact validation order
defined in this document.

Validation order is part of protocol definition.

Changing validation order may change error surface behavior
and is considered a breaking change.

---

# 1. initialize_issuance()

Purpose:
Creates and initializes IssuanceState.

Execution Order:

1. Verify IssuanceState account is uninitialized.
2. Verify correct account size (292 bytes).
3. Verify PDA derivation (issuance seed model).
4. Verify version == 1.
5. Verify start_ts < maturity_ts.
6. Verify claim_window > 0.
7. Verify reserve_total > 0.
8. Verify deposit_escrow owner == SPL Token Program.
9. Verify reward_escrow owner == SPL Token Program.
10. Verify deposit_escrow mint == lock_mint.
11. Verify reward_escrow mint == reward_mint.
12. Verify escrow authority == issuance PDA.
13. Compute final_day_index.
14. Write immutable fields.
15. Initialize mutable fields to zero.
16. Set reserve_funded = 0.
17. Set sweep_executed = 0.
18. Set reclaim_executed = 0.

Failure at any step MUST abort immediately.

---

# 2. fund_reserve()

Purpose:
Funds reward escrow exactly with reserve_total.

Execution Order:

1. Verify IssuanceState version.
2. Verify PDA.
3. Verify caller == issuer_address.
4. Verify reserve_funded == 0.
5. Verify current_timestamp < start_ts.
6. Verify transfer amount == reserve_total.
7. Perform SPL token transfer.
8. Set reserve_funded = 1.

Partial funding MUST be rejected.

---

# 3. deposit()

Purpose:
Locks participant tokens.

Execution Order:

1. Verify IssuanceState version.
2. Verify IssuanceState PDA.
3. Verify reserve_funded == 1.
4. Verify current_timestamp >= start_ts.
5. Verify current_timestamp < maturity_ts.
6. Verify deposit amount > 0.
7. Verify correct lock_mint.
8. Verify deposit_escrow authority.
9. Derive and verify UserState PDA.
10. If UserState uninitialized:
    a. Verify correct size (112 bytes).
    b. Initialize with zero state (locked_amount=0, user_weight_accum=0, reward_claimed=0).
    c. Set user_last_day_index = current_day_index (computed in Step 11).
11. Update global accumulator to current day (bounded; produces canonical current_day_index).
12. Update user accumulator to current day (using the same current_day_index).
13. Increase locked_amount (checked_add).
14. Increase total_locked (checked_add).
15. Perform SPL token transfer from participant to deposit_escrow.

Notes:
- State mutation MUST precede outbound token transfer.
- If CPI transfer fails, the entire instruction MUST revert.

---

# 4. claim_reward()

Purpose:
Distributes participant reward.

Execution Order:

1. Verify IssuanceState version.
2. Verify IssuanceState PDA.
3. Verify UserState PDA.
4. Verify reserve_funded == 1.
5. Verify current_timestamp >= maturity_ts.
6. Verify current_timestamp < maturity_ts + claim_window.
7. Verify reward_claimed == 0.
8. Update global accumulator to final day (finalize to final_day_index).
9. Update user accumulator to final day (using the same bounded current_day_index).
10. Verify total_weight_accum > 0.
11. Verify user_weight_accum > 0.
12. Compute reward = floor(reserve_total * user_weight_accum / total_weight_accum)
    using checked_mul + checked_div.
13. Set reward_claimed = 1.
14. Perform SPL transfer from reward_escrow to participant for computed reward.

If total_weight_accum == 0:
Return NoParticipation.

If user_weight_accum == 0:
Return NoParticipation.

Notes:
- reward_claimed MUST be set before the outbound transfer.
- Any failure MUST revert atomically.

---

# 5. sweep()

Purpose:
Transfers unclaimed rewards to platform_treasury.

Execution Order:

1. Verify IssuanceState version.
2. Verify IssuanceState PDA.
3. Verify current_timestamp >= maturity_ts + claim_window.
4. Verify sweep_executed == 0.
5. Verify reclaim_executed == 0.
6. Verify platform_treasury matches stored immutable value.
7. Update global accumulator to final day (bounded to final_day_index).
8. Verify total_weight_accum > 0.
9. Compute remaining reward escrow balance (on-chain).
10. Verify remaining reward escrow balance > 0.
11. Set sweep_executed = 1.
12. Perform SPL transfer of full remaining balance to platform_treasury.

Sweep is irreversible.

Notes:
- State mutation MUST precede outbound token transfer.
- If total_weight_accum == 0, sweep MUST fail with NoParticipation (no sweep in zero-participation state).

---

# 6. zero_participation_reclaim()

Purpose:
Returns reward escrow to issuer if no participation occurred (total_weight_accum == 0).

Execution Order:

1. Verify IssuanceState version.
2. Verify IssuanceState PDA.
3. Verify reserve_funded == 1.
4. Verify caller == issuer_address.
5. Verify current_timestamp >= maturity_ts.
6. Update global accumulator to final day (bounded to final_day_index).
7. Verify total_weight_accum == 0.
8. Verify reclaim_executed == 0.
9. Verify sweep_executed == 0.
10. Compute reward escrow balance (on-chain).
11. Verify reward escrow balance > 0.
12. Set reclaim_executed = 1.
13. Perform SPL transfer of full reward escrow balance to issuer_address.

After reclaim:

- sweep() MUST be rejected.
- claim_reward() MUST be rejected.

Notes:
- Reclaim is only permitted in the zero-participation state.
- State mutation MUST precede outbound token transfer.

---

# 7. Deterministic Ordering Rule

All instructions MUST:

- Validate identity and account binding before authority checks.
- Validate authority and mint correctness before any state mutation.
- Validate time windows and gating flags before computing economic amounts.
- Execute required accumulator updates before any mutation of:
  - total_locked
  - locked_amount
  - reward_claimed
  - sweep_executed
  - reclaim_executed
- Perform all state mutation (including irreversible flags) BEFORE outbound SPL transfers.

No instruction may:

- Perform transfer before all validations.
- Perform transfer before irreversible state flags are set (when applicable).
- Mutate state before passing all checks required for that instruction.
- Return different error codes for identical conditions.

Instruction order is part of protocol definition.
Changing it is a breaking change.

