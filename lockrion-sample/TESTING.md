# Lockrion v1.1 — Testing

This document describes how to run all tests for Lockrion v1.1.

Complete test catalog and current status:

TEST-STATUS.md

---

## Test Categories

Lockrion uses two test categories:

1) Script-based tests (Shell, against local validator)  
2) Rust ProgramTest / unit tests (cargo test)

---

## A) Script-Based Tests (Shell)

These tests run from Git Bash against a local validator.

### Requirements

- WSL2 validator running
- RPC: http://127.0.0.1:8899

### Start Validator (WSL2)

solana-test-validator --reset

Leave it running.

### Run Tests (Git Bash)

Make scripts executable:

chmod +x deploy_lockrion_gitbash.sh  
chmod +x tests/*.sh  

Run a single test:

bash tests/001_deploy_sanity.sh

Example sequence:

bash tests/001_deploy_sanity.sh  
bash tests/003_fund_reserve_happy.sh  
bash tests/005_deposit_happy.sh  

Notes:
- These tests deploy the program as part of the flow.
- They print ISSUANCE_PDA and transaction signatures for inspection.

---

## B) Rust Tests (ProgramTest + Unit)

These tests run via Rust test harness.

### Run all Rust tests

cargo test --features test-clock -- --nocapture

### Run unit tests only

cargo test --test accumulator_unit  
cargo test --test processor_unit  

### Production build check (must pass)

cargo build-sbf --no-default-features

---

## Environment Rules (Do Not Mix)

- Validator runs ONLY in WSL2
- Script tests run ONLY from Git Bash
- Rust tests run via cargo test
- Do NOT run validator in Git Bash
- Do NOT deploy from WSL2

---