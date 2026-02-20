#!/bin/bash
set -e

# ==============================
# Settings
# ==============================
PROJECT_DIR="$(pwd)"
SO_FILE="$PROJECT_DIR/target/deploy/lockrion_issuance_v1_1.so"
PROGRAM_KEYPAIR="$PROJECT_DIR/target/deploy/lockrion_issuance_v1_1_program-keypair.json"
VALIDATOR_LEDGER="$HOME/lockrion-ledger"   # Логически Linux FS, чтобы не было ошибок доступа
RPC_URL="http://127.0.0.1:8899"
AIRDROP_AMOUNT=10

# ==============================
# 1) Clean old ledger
# ==============================
if [ -d "$VALIDATOR_LEDGER" ]; then
    echo "Removing old ledger..."
    rm -rf "$VALIDATOR_LEDGER"
fi
mkdir -p "$VALIDATOR_LEDGER"

# ==============================
# 2) Start validator in background
# ==============================
echo "Starting solana-test-validator..."
solana-test-validator --reset --ledger "$VALIDATOR_LEDGER" > validator.log 2>&1 &
VALIDATOR_PID=$!
echo "Validator PID: $VALIDATOR_PID"

# Show validator log in real time
tail -f validator.log &
TAIL_PID=$!

# ==============================
# 3) Wait until RPC is ready
# ==============================
echo "Waiting for validator RPC to be ready..."
until curl -s $RPC_URL > /dev/null; do
    sleep 1
done
echo "Validator RPC is ready at $RPC_URL"

# ==============================
# 4) Configure Solana CLI
# ==============================
solana config set --url "$RPC_URL"
solana config set --keypair "$PROJECT_DIR/target/deploy/lockrion_issuance_v1_1-keypair.json"
solana cluster-version

# ==============================
# 5) Airdrop SOL
# ==============================
echo "Requesting airdrop of $AIRDROP_AMOUNT SOL..."
solana airdrop $AIRDROP_AMOUNT

# ==============================
# 6) Deploy Lockrion v1.1
# ==============================
echo "Deploying Lockrion v1.1..."
solana program deploy --program-id "$PROGRAM_KEYPAIR" "$SO_FILE"

# ==============================
# 7) Display info
# ==============================
echo "Lockrion deployed successfully."
echo "Program Id:"
solana program show "$PROGRAM_KEYPAIR"

# ==============================
# 8) Instructions
# ==============================
echo "Validator is running in background (PID $VALIDATOR_PID)."
echo "Tail log process PID: $TAIL_PID"
echo "Keep validator running while testing instructions."
echo "To stop validator: kill $VALIDATOR_PID && kill $TAIL_PID"
