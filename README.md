# Lockrion v1.1

Lockrion v1.1 is a deterministic Solana-based issuance protocol.

This repository contains:

- Raw Solana smart contract (SBF/BPF, no Anchor)
- Deployment scripts
- Full local execution model
- Complete deterministic test suite (001–046 + unit tests)

All tests are passing.

---

## Protocol Status

Version: v1.1  
Architecture: Non-Anchor raw Solana program  
Execution Model: Deterministic  
Arithmetic: Checked u128  
Time Model: Discrete 86400-second day index  
Upgradeability: Upgrade authority intended to be revoked after deployment  

Full test coverage includes:

- Funding logic
- Deposit window boundaries
- Claim logic
- Sweep logic
- Zero-participation reclaim
- Arithmetic overflow guards
- Escrow substitution protection
- PDA seed mutation protection
- Deterministic replay guarantee

See `TEST-STATUS.md` for complete list.

---

## Requirements

- WSL2 (Windows) or native Linux
- Rust (rustup)
- Solana CLI 3.1.8 (Agave)
- Git

Verify:

solana --version  
Expected: solana-cli 3.1.8 (Agave)

Note: `solana-program = 1.18.22` is a Rust dependency (Cargo.toml), not the CLI version.

---

## Building the Program

Lockrion is a raw Solana program (no Anchor).

Build inside WSL2 or Linux:

cargo build-sbf

Expected artifact:

target/deploy/lockrion_issuance_v1_1.so

The `.so` must exist before deployment.

---

## Local Execution Model (FINAL)

Strict environment separation is enforced.

### 🟢 WSL2 / Linux

WSL2 runs ONLY the local validator.

Start validator:

solana-test-validator --reset

RPC endpoint:

http://127.0.0.1:8899

WSL2 does NOT:

- create wallets
- deploy programs
- run tests
- manage keypairs

WSL2 = blockchain node only.

---

### 🔵 Git Bash (Windows)

Git Bash handles ALL operational logic:

- Wallet creation
- Program-id generation
- Airdrop
- Program deployment
- Instruction execution
- All testing

All keypairs are stored in:

target/deploy/

All interactions use:

http://127.0.0.1:8899

Git Bash = operator layer.

---

## Deployment Flow

### Step 1 — Start Validator (WSL2)

Open WSL2:

solana-test-validator --reset

Leave it running.

---

### Step 2 — Deploy (Git Bash)

Open Git Bash in project directory:

chmod +x deploy_lockrion_gitbash.sh  
./deploy_lockrion_gitbash.sh  

This script:

- connects to local RPC
- creates payer wallet if missing
- airdrops SOL
- generates program-id keypair
- deploys upgradeable Lockrion program

After completion:

solana program show <PROGRAM_ID>

---

## Testing

All tests are executed from Git Bash  
against the local validator at:

http://127.0.0.1:8899

Validator must be running before tests.

Run Rust program tests:

cargo test --features test-clock -- --nocapture

Production build check:

cargo build-sbf --no-default-features

All tests must pass before release.

---

## Rules

- Do NOT deploy from WSL2
- Do NOT create wallets in WSL2
- Do NOT run validator in Git Bash
- Do NOT mix environments

---

## License

MIT License