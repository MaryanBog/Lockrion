#!/bin/bash
set -euo pipefail

############################################################
# 🔧 ЗАПОЛНИТЬ
############################################################

PROGRAM_ID="PUT_PROGRAM_ID_HERE"
# Адрес задеплоенной программы

ISSUANCE_PDA="PUT_ISSUANCE_PDA_HERE"
# PDA выпуска (который вывел init-скрипт)

REWARD_ESCROW="PUT_REWARD_ESCROW_HERE"
# Reward escrow (из init)

RESERVE_TOTAL="1000000"
# Сколько USDC заводим (в целых USDC)

############################################################
# 🔒 ФИКСИРОВАНО
############################################################

USDC_MINT="EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
WALLET_FILE="platform-authority.json"
RPC_URL="https://api.mainnet-beta.solana.com"

############################################################
# ВАЛИДАЦИЯ
############################################################

die(){ echo "ERROR: $*" >&2; exit 1; }

[[ "$PROGRAM_ID" != "PUT_PROGRAM_ID_HERE" ]] || die "Set PROGRAM_ID"
[[ "$ISSUANCE_PDA" != "PUT_ISSUANCE_PDA_HERE" ]] || die "Set ISSUANCE_PDA"
[[ "$REWARD_ESCROW" != "PUT_REWARD_ESCROW_HERE" ]] || die "Set REWARD_ESCROW"
[[ -f "$WALLET_FILE" ]] || die "Missing platform-authority.json"

############################################################
# ПЕРЕВОД В USDC MIN UNITS (6 decimals)
############################################################

AMOUNT=$(( RESERVE_TOTAL * 1000000 ))

echo "Funding amount (raw) = $AMOUNT"

############################################################
# SOLANA CONFIG
############################################################

solana config set --url "$RPC_URL" >/dev/null
solana config set --keypair "$WALLET_FILE" >/dev/null

PAYER=$(solana address -k "$WALLET_FILE" | tr -d '\r\n')

############################################################
# НАЙТИ USDC ATA ИССЮЕРА
############################################################

ISSUER_USDC_ATA=$(spl-token address --token "$USDC_MINT" --owner "$PAYER")

echo "Issuer USDC ATA: $ISSUER_USDC_ATA"

############################################################
# ВЫЗОВ FundReserve (через node)
############################################################

PROGRAM_ID="$PROGRAM_ID" \
ISSUANCE_PDA="$ISSUANCE_PDA" \
ISSUER="$PAYER" \
ISSUER_USDC_ATA="$ISSUER_USDC_ATA" \
REWARD_ESCROW="$REWARD_ESCROW" \
AMOUNT="$AMOUNT" \
node - <<'NODE'
const {
  Connection,
  PublicKey,
  Keypair,
  Transaction,
  TransactionInstruction,
  sendAndConfirmTransaction
} = require("@solana/web3.js");
const fs = require("fs");

const RPC = "https://api.mainnet-beta.solana.com";

const programId = new PublicKey(process.env.PROGRAM_ID);
const issuancePda = new PublicKey(process.env.ISSUANCE_PDA);
const issuer = new PublicKey(process.env.ISSUER);
const issuerUsdcAta = new PublicKey(process.env.ISSUER_USDC_ATA);
const rewardEscrow = new PublicKey(process.env.REWARD_ESCROW);
const amount = BigInt(process.env.AMOUNT);

const payer = Keypair.fromSecretKey(
  Uint8Array.from(JSON.parse(fs.readFileSync("platform-authority.json","utf8")))
);

// ===== instruction data =====
// DISCRIMINANT FundReserve = 1 (ПРОВЕРЬ если у тебя другой порядок!)
const data = Buffer.alloc(1+8);
data.writeUInt8(1,0);
data.writeBigUInt64LE(amount,1);

const keys = [
  {pubkey: issuancePda, isSigner:false, isWritable:true},
  {pubkey: issuer, isSigner:true, isWritable:false},
  {pubkey: issuerUsdcAta, isSigner:false, isWritable:true},
  {pubkey: rewardEscrow, isSigner:false, isWritable:true},
  {pubkey: new PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"), isSigner:false, isWritable:false},
];

(async()=>{
  const conn = new Connection(RPC,"confirmed");
  const ix = new TransactionInstruction({programId, keys, data});
  const tx = new Transaction().add(ix);
  const sig = await sendAndConfirmTransaction(conn, tx, [payer]);
  console.log("FundReserve tx:", sig);
})();
NODE

echo "✅ RESERVE FUNDED"