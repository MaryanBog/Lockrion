#!/bin/bash
set -euo pipefail

############################################################
# 🔧 === ПЕРЕМЕННЫЕ ДЛЯ ЗАПОЛНЕНИЯ (КАЖДЫЙ ВЫПУСК) ===
############################################################

PROGRAM_ID="PUT_PROGRAM_ID_HERE"
# Адрес задеплоенной программы (после deploy)

LOCK_MINT="PUT_LOCK_MINT_HERE"
# Mint токена, который пользователи будут блокировать

RESERVE_TOTAL="1000000"
# Сколько USDC распределяется (в целых USDC, НЕ в lamports)
# Например: 1000000 = 1,000,000 USDC

START_DATE="2026-03-01"
# Дата старта (UTC), формат YYYY-MM-DD

DURATION_DAYS="30"
# Сколько дней длится выпуск


############################################################
# 🔒 === ЗАФИКСИРОВАНО (НЕ ТРОГАТЬ) ===
############################################################

# USDC mainnet mint
REWARD_MINT="EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"

# Казна проекта
PLATFORM_TREASURY="B9xmmg2zPMSwPg7iX7a9J2j6SK5LcopZ8abRDj9ughxw"

# RPC mainnet
RPC_URL="https://api.mainnet-beta.solana.com"

# Wallet платформы (тот же что treasury)
WALLET_FILE="platform-authority.json"


############################################################
# 🧠 ВАЛИДАЦИЯ
############################################################

die(){ echo "ERROR: $*" >&2; exit 1; }

[[ "$PROGRAM_ID" != "PUT_PROGRAM_ID_HERE" ]] || die "Set PROGRAM_ID"
[[ "$LOCK_MINT" != "PUT_LOCK_MINT_HERE" ]] || die "Set LOCK_MINT"
[[ -f "$WALLET_FILE" ]] || die "Missing platform-authority.json"

############################################################
# 🕒 КОНВЕРТАЦИЯ ДАТЫ В TIMESTAMP (UTC)
############################################################

START_TS=$(date -u -d "$START_DATE 00:00:00" +%s)
MATURITY_TS=$(( START_TS + 86400 * DURATION_DAYS ))

echo "START_TS=$START_TS"
echo "MATURITY_TS=$MATURITY_TS"

############################################################
# 🔢 USDC УЧЁТ DECIMALS (6 знаков)
############################################################

# Переводим USDC в минимальные единицы
RESERVE_TOTAL_LAMPORTS=$(( RESERVE_TOTAL * 1000000 ))

echo "RESERVE_TOTAL_LAMPORTS=$RESERVE_TOTAL_LAMPORTS"

############################################################
# 🔐 НАСТРОЙКА SOLANA CLI
############################################################

solana config set --url "$RPC_URL" >/dev/null
solana config set --keypair "$WALLET_FILE" >/dev/null

PAYER=$(solana address -k "$WALLET_FILE" | tr -d '\r\n')

############################################################
# 🧠 DERIVE ISSUANCE PDA
############################################################

ISSUANCE_PDA="$(
PROGRAM_ID="$PROGRAM_ID" \
PAYER="$PAYER" \
START_TS="$START_TS" \
RESERVE_TOTAL="$RESERVE_TOTAL_LAMPORTS" \
node - <<'NODE'
const {PublicKey} = require("@solana/web3.js");

const programId = new PublicKey(process.env.PROGRAM_ID);
const payer = new PublicKey(process.env.PAYER);

const startTs = BigInt(process.env.START_TS);
const reserveTotal = BigInt(process.env.RESERVE_TOTAL);

const s1 = Buffer.from("issuance");
const s2 = payer.toBuffer();

const s3 = Buffer.alloc(8);
s3.writeBigInt64LE(startTs);

const s4 = Buffer.alloc(16);
s4.writeBigUInt64LE(reserveTotal, 0);
s4.writeBigUInt64LE(0n, 8);

const [pda] = PublicKey.findProgramAddressSync([s1,s2,s3,s4], programId);
process.stdout.write(pda.toBase58());
NODE
)"

echo "ISSUANCE_PDA=$ISSUANCE_PDA"

############################################################
# 🪙 CREATE ESCROWS (owned by ISSUANCE_PDA)
############################################################

extract_pubkey() {
  tr -d '\r' | tr ' ' '\n' | grep -E '^[1-9A-HJ-NP-Za-km-z]{32,44}$' | head -n 1
}

REWARD_ESCROW="$(spl-token create-account "$REWARD_MINT" --owner "$ISSUANCE_PDA" | extract_pubkey)"
DEPOSIT_ESCROW="$(spl-token create-account "$LOCK_MINT" --owner "$ISSUANCE_PDA" | extract_pubkey)"

echo "REWARD_ESCROW=$REWARD_ESCROW"
echo "DEPOSIT_ESCROW=$DEPOSIT_ESCROW"

############################################################
# 🚀 CALL JS INIT
############################################################

export PROGRAM_ID LOCK_MINT REWARD_MINT DEPOSIT_ESCROW REWARD_ESCROW PLATFORM_TREASURY
export START_TS MATURITY_TS
export RESERVE_TOTAL="$RESERVE_TOTAL_LAMPORTS"

node tests/js/init_issuance.js

echo "✅ MAINNET ISSUANCE CREATED"