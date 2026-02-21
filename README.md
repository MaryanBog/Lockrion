# Lockrion v1.1

Lockrion v1.1 is a deterministic Solana issuance protocol implemented as a raw SBF/BPF program (no Anchor).

This repository contains:

- On-chain program source code
- Deployment script
- Production issuance creation template

---

## Requirements

- Linux or WSL2
- Rust (rustup)
- Solana CLI 3.1.8 (Agave)
- Node.js (for issuance script)
- spl-token CLI

Verify Solana version:

solana --version

Expected:
solana-cli 3.1.8 (Agave)

Note:
`solana-program = 1.18.22` is a Cargo dependency and unrelated to the CLI version.

---

# 1. Build the Program

Build inside Linux or WSL2:

cargo build-sbf --no-default-features

Expected output:

target/deploy/lockrion_issuance_v1_1.so

This file is the deployable on-chain binary.

---

# 2. Deploy the Program to Blockchain

Deploy to mainnet:

solana config set --url https://api.mainnet-beta.solana.com
solana program deploy target/deploy/lockrion_issuance_v1_1.so

After deployment, you will receive:

Program Id: <PROGRAM_ID>

Verify:

solana program show <PROGRAM_ID>

---

## Optional: Make Program Immutable

After verifying everything is correct, you may finalize the program:

solana program set-upgrade-authority <PROGRAM_ID> --final

This permanently removes upgrade authority.

After this:
- Code cannot be modified
- Program becomes immutable
- No further upgrades are possible

---

# 3. Create a New Issuance

Deploying the program does NOT create an issuance.

Each issuance must be created manually using:

create_issuance_mainnet.sh

This script:

- Derives the canonical Issuance PDA
- Creates escrow accounts
- Calls init_issuance
- Writes immutable economic parameters on-chain

---

## Before Running Issuance Script

Open:

create_issuance_mainnet.sh

Replace the placeholders:

PROGRAM_ID="PUT_YOUR_PROGRAM_ID_HERE"

START_TS=PUT_START_TIMESTAMP
MATURITY_TS=PUT_MATURITY_TIMESTAMP

LOCK_MINT="PUT_LOCK_MINT_ADDRESS"
REWARD_MINT="PUT_REWARD_MINT_ADDRESS"

PLATFORM_TREASURY="PUT_PROJECT_TREASURY_ADDRESS"

Verify:

RPC_URL="https://api.mainnet-beta.solana.com"
ISSUER_WALLET="mainnet-issuer.json"

Ensure:

- The issuer wallet exists
- The wallet contains sufficient SOL
- All mint addresses are correct
- Parameters are reviewed carefully

Issuance parameters cannot be changed after creation.

---

## Run Issuance Script

chmod +x create_issuance_mainnet.sh
./create_issuance_mainnet.sh

If successful, the script will output:

ISSUANCE_PDA=<address>

That address represents the live issuance instance.

---

# Deployment Model

1. Deploy program (once)
2. Optionally finalize program
3. Create issuance instances as needed
4. Each issuance is independent and immutable

---

# License

MIT License