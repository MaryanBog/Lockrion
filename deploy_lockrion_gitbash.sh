#!/bin/bash
set -euo pipefail

# ============================================================
# deploy_lockrion_gitbash.sh
#
# Run (Git Bash):
#   chmod +x deploy_lockrion_gitbash.sh
#   ./deploy_lockrion_gitbash.sh
#
# Notes:
# - Validator must already be running on 127.0.0.1:8899
# - This script does NOT build .so (build in WSL2)
# - IMPORTANT:
#   * Airdrop goes to payer wallet ONLY
#   * Program-id keypair must NOT receive airdrop (otherwise it's "already in use")
# ============================================================

PROJECT_DIR="$(pwd)"
RPC_URL="http://127.0.0.1:8899"

SO_FILE="$PROJECT_DIR/target/deploy/lockrion_issuance_v1_1.so"

# Payer wallet (pays fees + rent for deploy)
PAYER_WALLET="$PROJECT_DIR/target/deploy/test-wallet.json"
AIRDROP_SOL=10

# Program id keypair (address of the program)
PROGRAM_KEYPAIR="$PROJECT_DIR/target/deploy/lockrion_issuance_v1_1_program-keypair.json"

die() { echo "ERROR: $*" 1>&2; exit 1; }
need_cmd() { command -v "$1" >/dev/null 2>&1 || die "Missing command: $1"; }

need_cmd curl
need_cmd solana
need_cmd solana-keygen

[ -f "$SO_FILE" ] || die "Program .so not found: $SO_FILE (build it in WSL2 first)"
mkdir -p "$PROJECT_DIR/target/deploy"

echo "Checking RPC at $RPC_URL ..."
until curl -s "$RPC_URL" >/dev/null; do
  sleep 1
done
echo "RPC is ready."

# Configure Solana CLI to local validator (URL only)
solana config set --url "$RPC_URL" >/dev/null
solana config get

# 1) Ensure payer wallet exists
if [ ! -f "$PAYER_WALLET" ]; then
  echo "Creating payer wallet (test-wallet)..."
  solana-keygen new -o "$PAYER_WALLET" --no-bip39-passphrase >/dev/null
fi

echo "Payer pubkey:"
solana address -k "$PAYER_WALLET"

# 2) Airdrop to payer ONLY
echo "Airdropping $AIRDROP_SOL SOL to payer..."
solana airdrop "$AIRDROP_SOL" -k "$PAYER_WALLET"
echo "Payer balance after airdrop:"
solana balance -k "$PAYER_WALLET"

# 3) Create NEW program-id keypair each run (optional but avoids collisions)
if [ -f "$PROGRAM_KEYPAIR" ]; then
  echo "Removing existing Program keypair (to avoid collisions)..."
  rm -f "$PROGRAM_KEYPAIR"
fi

echo "Creating new Program keypair (program-id)..."
solana-keygen new -o "$PROGRAM_KEYPAIR" --no-bip39-passphrase >/dev/null

echo "Program ID (will be deployed to):"
solana address -k "$PROGRAM_KEYPAIR"

# IMPORTANT: DO NOT airdrop to PROGRAM_KEYPAIR

# 4) Deploy as upgradeable program (payer pays, program-id is separate)
echo "Deploying Lockrion v1.1 as upgradeable program..."
solana program deploy "$SO_FILE" \
  --program-id "$PROGRAM_KEYPAIR" \
  --keypair "$PAYER_WALLET"

echo "Lockrion deployed successfully."
echo "Program info:"
solana program show "$(solana address -k "$PROGRAM_KEYPAIR")"

echo "✅ Done."