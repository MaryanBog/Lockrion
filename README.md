# Lockrion v1.1

Lockrion v1.1 is a Solana-based issuance protocol.  
This repository contains the smart contract (SBF/BPF program), deployment scripts, and instructions for running a local test environment.

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

This project uses strict environment separation.

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

---

## Rules

- Do NOT deploy from WSL2
- Do NOT create wallets in WSL2
- Do NOT run validator in Git Bash
- Do NOT mix environments

---

## License

MIT License