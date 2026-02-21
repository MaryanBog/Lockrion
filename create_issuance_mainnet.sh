#!/bin/bash
set -euo pipefail

############################################
# 🔴 NETWORK
############################################

RPC_URL="https://api.mainnet-beta.solana.com"
solana config set --url "$RPC_URL" >/dev/null

############################################
# 🔐 WALLET (ISSUER)
############################################

ISSUER_WALLET="mainnet-issuer.json"
solana config set --keypair "$ISSUER_WALLET" >/dev/null

ISSUER="$(solana address -k "$ISSUER_WALLET" | tr -d '\r')"

############################################
# 📦 PROGRAM
############################################

PROGRAM_ID="PUT_YOUR_PROGRAM_ID_HERE"

############################################
# 💰 ECONOMIC PARAMETERS
############################################

RESERVE_TOTAL=1000000        # total reward tokens
START_TS=PUT_START_TIMESTAMP
MATURITY_TS=PUT_MATURITY_TIMESTAMP

# example:
# START_TS=1775000000
# MATURITY_TS=1775000000+86400*30

############################################
# 🪙 TOKENS
############################################

LOCK_MINT="PUT_LOCK_MINT_ADDRESS"
REWARD_MINT="PUT_REWARD_MINT_ADDRESS"

############################################
# 🏦 TREASURY
############################################

PLATFORM_TREASURY="PUT_PROJECT_TREASURY_ADDRESS"

############################################
# 🧠 DERIVE ISSUANCE PDA
############################################

ISSUANCE_PDA="$(
PROGRAM_ID="$PROGRAM_ID" \
PAYER="$ISSUER" \
START_TS="$START_TS" \
RESERVE_TOTAL="$RESERVE_TOTAL" \
node -e "
const {PublicKey} = require('@solana/web3.js');
const programId = new PublicKey(process.env.PROGRAM_ID);
const payer = new PublicKey(process.env.PAYER);
const startTs = BigInt(process.env.START_TS);
const reserve = BigInt(process.env.RESERVE_TOTAL);

const s1 = Buffer.from('issuance');
const s2 = payer.toBuffer();

const s3 = Buffer.alloc(8);
s3.writeBigInt64LE(startTs);

const s4 = Buffer.alloc(16);
s4.writeBigUInt64LE(reserve,0);
s4.writeBigUInt64LE(0n,8);

const [pda] = PublicKey.findProgramAddressSync(
  [s1, s2, s3, s4],
  programId
);

process.stdout.write(pda.toBase58());
")"

echo "ISSUANCE_PDA=$ISSUANCE_PDA"

############################################
# 🪙 CREATE ESCROWS
############################################

REWARD_ESCROW="$(spl-token create-account "$REWARD_MINT" --owner "$ISSUANCE_PDA" | awk '/Creating account/ {print $3}')"
DEPOSIT_ESCROW="$(spl-token create-account "$LOCK_MINT" --owner "$ISSUANCE_PDA" | awk '/Creating account/ {print $3}')"

############################################
# 🚀 INIT ISSUANCE
############################################

PROGRAM_ID="$PROGRAM_ID" \
START_TS="$START_TS" \
MATURITY_TS="$MATURITY_TS" \
RESERVE_TOTAL="$RESERVE_TOTAL" \
LOCK_MINT="$LOCK_MINT" \
REWARD_MINT="$REWARD_MINT" \
DEPOSIT_ESCROW="$DEPOSIT_ESCROW" \
REWARD_ESCROW="$REWARD_ESCROW" \
PLATFORM_TREASURY="$PLATFORM_TREASURY" \
node tests/js/init_issuance.js

echo "✅ Issuance created successfully."