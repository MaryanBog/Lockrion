# Lockrion Issuance Contract — Compliance Matrix v1.1

Status: Draft  
Standard: Lockrion Issuance Contract v1  
Scope: Specification → Verification Method → Evidence → Status  

This matrix maps each normative requirement in Specification v1.1
to a concrete verification method and a completion status.

Legend:
- PASS = verified and evidenced
- FAIL = verified and violated
- TBD = verification not yet executed / evidence missing

---

## 1. Global System Requirements (Normative)

| ID | Requirement (Specification v1.1) | Verification Method | Evidence Artifact | Status |
|---:|---|---|---|---|
| G-01 | Contract is non-upgradeable and immutable after deployment | Deployment checklist + on-chain loader inspection | Deployment Report v1.1 | TBD |
| G-02 | reserve_total is fixed and immutable | Spec/Design review + state immutability check | Code Review Notes v1.1 | TBD |
| G-03 | reward escrow MUST be fully funded before start_ts | Integration test: fund_reserve before start_ts | Integration Logs v1.1 | TBD |
| G-04 | Deposits MUST be rejected unless reserve_funded == true | Integration test: deposit before funding | Integration Logs v1.1 | TBD |
| G-05 | No floating-point arithmetic may exist | Static scan + clippy policy | Static Analysis Report v1.1 | TBD |
| G-06 | All arithmetic MUST be checked (no overflow wrap) | Static inspection + code review | Static Analysis Report v1.1 | TBD |
| G-07 | All instructions MUST be deterministic | Theoretical validation + replay integration test | Theoretical Validation v1.1 + Replay Logs | TBD |

---

## 2. Time Model and Discrete Accounting

| ID | Requirement (Specification v1.1) | Verification Method | Evidence Artifact | Status |
|---:|---|---|---|---|
| T-01 | accounting_period fixed at 86400 seconds | Spec/Design/Impl cross-check | Code Review Notes v1.1 | TBD |
| T-02 | Day index uses floor((t-start_ts)/86400) | Unit tests + code inspection | Auto-Test Suite v1.1 | TBD |
| T-03 | Accumulation bounded by final_day_index | Unit tests + proof mapping | Auto-Test Suite v1.1 + Theoretical Validation v1.1 | TBD |
| T-04 | No accumulation beyond maturity_ts | Integration test across maturity boundary | Integration Logs v1.1 | TBD |
| T-05 | Same-day transactions do not change weight | Integration scenario: multiple deposits same day | Integration Logs v1.1 | TBD |

---

## 3. State and Invariants

| ID | Requirement (Specification v1.1) | Verification Method | Evidence Artifact | Status |
|---:|---|---|---|---|
| S-01 | total_locked tracks sum of user.locked_amount | Unit tests + invariant assertions | Auto-Test Suite v1.1 | TBD |
| S-02 | total_weight_accum monotonic non-decreasing | Unit tests + code inspection | Auto-Test Suite v1.1 | TBD |
| S-03 | last_day_index never exceeds final_day_index | Unit tests + bounds checks | Auto-Test Suite v1.1 | TBD |
| S-04 | reward_claimed is irreversible | Unit tests + flag gating | Auto-Test Suite v1.1 | TBD |
| S-05 | sweep_executed is irreversible | Unit tests + flag gating | Auto-Test Suite v1.1 | TBD |
| S-06 | reclaim_executed is irreversible | Unit tests + flag gating | Auto-Test Suite v1.1 | TBD |

---

## 4. Instruction Requirements

### 4.1 fund_reserve()

| ID | Requirement | Verification Method | Evidence Artifact | Status |
|---:|---|---|---|---|
| F-01 | Only issuer may fund | Integration test: non-issuer attempt | Integration Logs v1.1 | TBD |
| F-02 | Must occur before start_ts | Integration test: call after start_ts | Integration Logs v1.1 | TBD |
| F-03 | Funding amount must equal reserve_total exactly | Integration test: wrong amount | Integration Logs v1.1 | TBD |
| F-04 | reserve_funded set true only on success | Integration test + state inspection | Integration Logs v1.1 | TBD |

### 4.2 deposit(amount)

| ID | Requirement | Verification Method | Evidence Artifact | Status |
|---:|---|---|---|---|
| D-01 | Deposits only within [start_ts, maturity_ts) | Integration tests (before/after) | Integration Logs v1.1 | TBD |
| D-02 | amount > 0 enforced | Unit test + integration test | Auto-Test Suite v1.1 | TBD |
| D-03 | Accumulator invoked before state mutation | Code review + unit test | Code Review Notes v1.1 | TBD |
| D-04 | State mutation before CPI transfer | Code review + integration | Code Review Notes v1.1 | TBD |
| D-05 | Mint validation enforced | Integration test: wrong mint | Integration Logs v1.1 | TBD |

### 4.3 claim_reward()

| ID | Requirement | Verification Method | Evidence Artifact | Status |
|---:|---|---|---|---|
| C-01 | Only within claim window | Integration tests (early/late) | Integration Logs v1.1 | TBD |
| C-02 | Fails if total_weight_accum == 0 | Integration test: zero participation | Integration Logs v1.1 | TBD |
| C-03 | reward_claimed gates repeat | Unit test: double claim | Auto-Test Suite v1.1 | TBD |
| C-04 | Uses canonical formula floor(R*Wi/W) | Unit tests with known vectors | Auto-Test Suite v1.1 | TBD |
| C-05 | Checked arithmetic on numerator | Static inspection + unit test overflow vectors | Static Analysis Report v1.1 + Auto-Test Suite v1.1 | TBD |

### 4.4 withdraw_deposit()

| ID | Requirement | Verification Method | Evidence Artifact | Status |
|---:|---|---|---|---|
| W-01 | Only after maturity_ts | Integration test pre-maturity | Integration Logs v1.1 | TBD |
| W-02 | Accumulator finalization occurs before clearing locked_amount | Code review + unit test | Code Review Notes v1.1 + Auto-Test Suite v1.1 | TBD |
| W-03 | State mutation before CPI transfer | Code review + integration | Code Review Notes v1.1 | TBD |
| W-04 | Repeat withdraw fails (locked_amount == 0) | Unit test | Auto-Test Suite v1.1 | TBD |

### 4.5 sweep()

| ID | Requirement | Verification Method | Evidence Artifact | Status |
|---:|---|---|---|---|
| SW-01 | Only after maturity_ts + claim_window | Integration test early/late | Integration Logs v1.1 | TBD |
| SW-02 | Fails if total_weight_accum == 0 | Integration test zero participation | Integration Logs v1.1 | TBD |
| SW-03 | sweep_executed gates repeat | Unit test + integration | Auto-Test Suite v1.1 | TBD |
| SW-04 | Canonical order: set flag before transfer | Code review | Code Review Notes v1.1 | TBD |

### 4.6 zero_participation_reclaim()

| ID | Requirement | Verification Method | Evidence Artifact | Status |
|---:|---|---|---|---|
| Z-01 | Only if total_weight_accum == 0 | Integration tests | Integration Logs v1.1 | TBD |
| Z-02 | Only issuer may reclaim | Integration test non-issuer attempt | Integration Logs v1.1 | TBD |
| Z-03 | reclaim_executed gates repeat | Unit test + integration | Auto-Test Suite v1.1 | TBD |
| Z-04 | Canonical order: set flag before transfer | Code review | Code Review Notes v1.1 | TBD |

---

## 5. Account and Security Requirements

| ID | Requirement | Verification Method | Evidence Artifact | Status |
|---:|---|---|---|---|
| A-01 | PDA derivation validated in every instruction | Code review + integration substitution tests | Integration Logs v1.1 + Code Review Notes v1.1 | TBD |
| A-02 | Token program ID validated | Integration test with wrong token program | Integration Logs v1.1 | TBD |
| A-03 | Escrow authority equals issuance PDA | Integration substitution tests | Integration Logs v1.1 | TBD |
| A-04 | platform_treasury bound to immutable parameter | Integration substitution tests | Integration Logs v1.1 | TBD |
| A-05 | Mints validated for every transfer | Integration tests wrong mint | Integration Logs v1.1 | TBD |

---

## 6. Completion Gate

The Issuance Contract v1.1 may be declared compliant only when:

- All items are PASS
- Evidence artifacts exist and are reproducible
- No FAIL or TBD remains

Current Status: TBD (evidence execution not yet attached)
