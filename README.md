# Lockrion v1.1

Lockrion is a Solana-based issuance protocol. This repository contains the smart contract (BPF program), deployment scripts, and instructions for running a local test environment using WSL2 or Linux.

## Table of Contents

- Requirements
- Setup
- Building the Program
- Running a Local Validator
- Deploying Lockrion
- Testing Instructions

## Requirements

- WSL2 (Windows) or native Linux environment
- Rust toolchain (rustup)
- Solana CLI v1.18.22
- Node.js & NPM (optional, for Agave 3.1.8 helpers)
- Git

## Setup

1. Clone the repository:
git clone https://github.com/YourUsername/Lockrion.git
cd Lockrion

2. Make sure Solana CLI is installed:
solana --version
# Expected: solana-cli 1.18.22 or compatible

3. Ensure Rust is installed and the BPF target is added:
rustup default stable
rustup target add bpfel-unknown-unknown

## Building the Program

To build the Lockrion BPF program:
cargo build-sbf
# Output: target/deploy/lockrion_issuance_v1_1.so

## Running a Local Validator

Start a local Solana validator to test the program:
solana-test-validator --reset

- Ledger is created in test-ledger/
- RPC URL: http://127.0.0.1:8899
- Keep this terminal running while testing

## Deploying Lockrion

Назвать файл deploy_lockrion.sh

Запускать только внутри WSL2, например:

cd ~/lockrion
chmod +x deploy_lockrion.sh
./deploy_lockrion.sh

потом запускаем команды на Git bash 

chmod +x deploy_lockrion_gitbash.sh
./deploy_lockrion_gitbash.sh

## Testing Instructions

Once deployed, you can test Lockrion operations:
- fund_reserve
- deposit
- claim_reward
- withdraw_deposit
- sweep
- zero_participation_reclaim

All operations are performed on the local validator RPC (http://127.0.0.1:8899) using the same keypairs from target/deploy/.

## Optional: Using Agave 3.1.8 Helpers

If you want to automate local testnets or program interactions:
npm install -g agave@3.1.8
agave local start

This will setup a local Solana testnet with pre-funded accounts for faster development.

## Notes

- Always use WSL2 or Linux for reliable validator operation on Windows.
- Git Bash on Windows cannot reliably run solana-test-validator due to file system and networking limitations.
- Keep ledger clean with --reset if errors occur during validator startup.

## License

MIT License
