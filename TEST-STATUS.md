## ✅ Implemented Tests

### 001_deploy_sanity.sh
Status: PASS  
Scope:
- Проверка доступности RPC
- Деплой через deploy_lockrion_gitbash.sh
- Проверка solana program show
- Проверка upgradeable loader
- Проверка ProgramData header

Run:
chmod +x tests/001_deploy_sanity.sh
bash tests/001_deploy_sanity.sh

---

### 002_programdata_sanity.sh
Status: PASS  
Scope:
- Проверка ProgramData Address
- Проверка владельца ProgramData (BPFLoaderUpgradeab1e)
- Проверка чтения ProgramData header (без падений)

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
- Проверка reserve_funded == 1
- Проверка reward escrow balance == reserve_total

Run:
chmod +x tests/003_fund_reserve_happy.sh
bash tests/003_fund_reserve_happy.sh

---

### 004_double_fund_reserve_rejected.sh
Status: PASS  
Scope:
- Deploy
- init_issuance
- fund_reserve() успешный (первый раз)
- fund_reserve() второй раз должен быть отклонён (double funding)

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
- Ожидание start_ts по chain-time (без solana warp)
- Mint lock tokens участнику
- deposit(amount)
- Проверка deposit_escrow balance == amount
- Проверка issuance.total_locked == amount (чтение по байтам аккаунта, u128 LE)

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
- init_issuance (reserve НЕ funded)
- Ожидание start_ts по chain-time
- deposit(amount) должен быть отклонён

Expected:
- Transaction fails with ReserveNotFunded (custom program error: 0xb / 0x0b)
- deposit_escrow balance не меняется

Run:
chmod +x tests/006_deposit_before_funding_rejected.sh
bash tests/006_deposit_before_funding_rejected.sh

---

### 007_deposit_before_start_rejected.sh
Status: PASS  
Scope:
- Deploy
- init_issuance (start_ts в будущем)
- fund_reserve()
- deposit(amount) ДО start_ts должен быть отклонён

Expected:
- Transaction fails with DepositWindowNotStarted (0x14)
- deposit_escrow balance не меняется

Run:
chmod +x tests/007_deposit_before_start_rejected.sh
bash tests/007_deposit_before_start_rejected.sh

---

### 008_deposit_after_maturity_rejected.sh
Status: PASS  
Scope:
- Deploy
- init_issuance (короткое окно start_ts → maturity_ts)
- fund_reserve()
- Ожидание maturity_ts по chain-time
- deposit(amount) ПОСЛЕ maturity_ts должен быть отклонён

Expected:
- Transaction fails with DepositWindowClosed (0x15)
- deposit_escrow balance не меняется

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
- Ожидание start_ts по chain-time
- deposit(0) должен быть отклонён

Expected:
- Transaction fails with InvalidAmount (0x17)
- deposit_escrow balance не меняется

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
- Ожидание start_ts по chain-time
- Участник использует ATA с другим mint (B)
- deposit(amount) должен быть отклонён

Expected:
- Transaction fails with InvalidMint (0x35)
- deposit_escrow balance не меняется

Run:
chmod +x tests/010_deposit_invalid_mint_rejected.sh
bash tests/010_deposit_invalid_mint_rejected.sh

---

## 011_claim_happy_pt.rs (solana-program-test)

Status: PASS

Scope:
- ProgramTest (in-process, без WSL / solana-test-validator)
- init_issuance
- fund_reserve() (до start_ts)
- deposit() (после start_ts)
- Warp по slot до maturity_ts (через feature test-clock)
- claim_reward()
- Проверка: participant_reward balance увеличился

Run:
cargo test --features test-clock --test 011_claim_happy_pt -- --nocapture

Notes:
- Добавлена feature test-clock (используется только для тестов)
- В прод-сборке test-clock НЕ включается
- Проверка прод-сборки:
cargo build-sbf --no-default-features
PASS

---

## ⏳ Planned Tests

### Claim / Withdraw / Sweep / Reclaim
- 012_claim_double_rejected.sh
- 013_claim_before_maturity_rejected.sh
- 014_withdraw_deposit_happy.sh
- 015_withdraw_before_maturity_rejected.sh
- 016_sweep_happy.sh
- 017_zero_participation_reclaim_happy.sh