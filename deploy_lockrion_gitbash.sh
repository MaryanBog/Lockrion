#!/bin/bash
set -e

# ==============================
# Settings
# ==============================
PROJECT_DIR="$(pwd)"
SO_FILE="$PROJECT_DIR/target/deploy/lockrion_issuance_v1_1.so"
PROGRAM_KEYPAIR="$PROJECT_DIR/target/deploy/lockrion_issuance_v1_1_program-keypair.json"
TEST_WALLET="$PROJECT_DIR/target/deploy/test-wallet.json"
RPC_URL="http://127.0.0.1:8899"
AIRDROP_TEST_WALLET=10

# ==============================
# 0) Создаём тестовый кошелек если ещё нет
# ==============================
if [ ! -f "$TEST_WALLET" ]; then
    echo "Creating test wallet for airdrop..."
    solana-keygen new -o "$TEST_WALLET" --no-bip39-passphrase
fi

# ==============================
# 1) Проверка доступности RPC
# ==============================
echo "Checking RPC at $RPC_URL ..."
until curl -s $RPC_URL > /dev/null; do
    sleep 1
done
echo "RPC is ready."

# ==============================
# 2) Airdrop на тестовый кошелек
# ==============================
solana config set --url "$RPC_URL"
solana config set --keypair "$TEST_WALLET"
echo "Airdropping $AIRDROP_TEST_WALLET SOL to test wallet..."
solana airdrop $AIRDROP_TEST_WALLET
echo "Balance after airdrop:"
solana balance

# ==============================
# 3) Проверка и deploy Lockrion v1.1
# ==============================
# Если keypair уже существует, создаём новый, чтобы avoid "already in use"
if [ -f "$PROGRAM_KEYPAIR" ]; then
    echo "Overwriting old Program keypair to avoid 'already in use'..."
    rm "$PROGRAM_KEYPAIR"
fi
echo "Creating new Program keypair for upgradeable account..."
solana-keygen new -o "$PROGRAM_KEYPAIR" --no-bip39-passphrase

solana config set --keypair "$PROGRAM_KEYPAIR"

echo "Deploying Lockrion v1.1 as upgradeable program..."
solana program deploy "$SO_FILE" --program-id "$PROGRAM_KEYPAIR"

echo "Lockrion deployed successfully."
solana program show "$PROGRAM_KEYPAIR"

echo "✅ Done. Test wallet funded, upgradeable Program account created, Lockrion deployed."
