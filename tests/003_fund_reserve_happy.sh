#!/bin/bash
export NODE_OPTIONS="--no-deprecation"
set -euo pipefail

RPC_URL="http://127.0.0.1:8899"
DEPLOY_SCRIPT="./deploy_lockrion_gitbash.sh"
PAYER_WALLET="target/deploy/test-wallet.json"
TOKEN_PROGRAM="TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
FEE_PAYER="$PAYER_WALLET"

die(){ echo "TEST FAIL: $*" 1>&2; exit 1; }
need(){ command -v "$1" >/dev/null 2>&1 || die "Missing command: $1"; }

need solana
need spl-token
need node.exe
need awk
need grep
need curl

curl -s "$RPC_URL" >/dev/null || die "RPC not reachable"
solana config set --url "$RPC_URL" >/dev/null
solana config set --keypair "$PAYER_WALLET" >/dev/null

OUT="$("$DEPLOY_SCRIPT" 2>&1)"
PROGRAM_ID="$(printf "%s\n" "$OUT" | awk '/Program ID \(will be deployed to\):/ {getline; gsub(/\r/,""); print; exit}')"
[ -n "${PROGRAM_ID:-}" ] || die "Could not parse Program ID"

ISSUER="$(solana address -k "$PAYER_WALLET" | tr -d '\r')"

read -r START_TS MATURITY_TS <<EOF
$(node.exe -e "const now=Math.floor(Date.now()/1000); const start=Math.floor((now+86400+86399)/86400)*86400; const mat=start+86400*30; console.log(start, mat);")
EOF

RESERVE_TOTAL=1000

ISSUANCE_PDA="$(PROGRAM_ID="$PROGRAM_ID" PAYER="$ISSUER" START_TS="$START_TS" RESERVE_TOTAL="$RESERVE_TOTAL" node.exe -e "const{PublicKey}=require('@solana/web3.js');const programId=new PublicKey(process.env.PROGRAM_ID);const payer=new PublicKey(process.env.PAYER);const startTs=BigInt(process.env.START_TS);const reserve=BigInt(process.env.RESERVE_TOTAL);const s1=Buffer.from('issuance');const s2=payer.toBuffer();const s3=Buffer.alloc(8);s3.writeBigInt64LE(startTs);const s4=Buffer.alloc(16);s4.writeBigUInt64LE(reserve,0);s4.writeBigUInt64LE(0n,8);const[pda]=PublicKey.findProgramAddressSync([s1,s2,s3,s4],programId);process.stdout.write(pda.toBase58());")"

REWARD_MINT="$(spl-token create-token --decimals 0 --fee-payer "$FEE_PAYER" | awk '/Creating token/ {print $3}')"
REWARD_ESCROW="$(spl-token create-account "$REWARD_MINT" --owner "$ISSUANCE_PDA" --fee-payer "$FEE_PAYER" | awk '/Creating account/ {print $3}')"

LOCK_MINT="$(spl-token create-token --decimals 0 --fee-payer "$FEE_PAYER" | awk '/Creating token/ {print $3}')"
DEPOSIT_ESCROW="$(spl-token create-account "$LOCK_MINT" --owner "$ISSUANCE_PDA" --fee-payer "$FEE_PAYER" | awk '/Creating account/ {print $3}')"

PLATFORM_TREASURY="$ISSUER"

PROGRAM_ID="$PROGRAM_ID" START_TS="$START_TS" MATURITY_TS="$MATURITY_TS" RESERVE_TOTAL="$RESERVE_TOTAL" \
LOCK_MINT="$LOCK_MINT" REWARD_MINT="$REWARD_MINT" DEPOSIT_ESCROW="$DEPOSIT_ESCROW" REWARD_ESCROW="$REWARD_ESCROW" \
PLATFORM_TREASURY="$PLATFORM_TREASURY" node.exe tests/js/init_issuance.js >/dev/null

ISSUER_REWARD_ATA="$(spl-token create-account "$REWARD_MINT" --owner "$ISSUER" --fee-payer "$FEE_PAYER" 2>&1 | awk '/Creating account/ {print $3}')"
spl-token mint "$REWARD_MINT" "$RESERVE_TOTAL" "$ISSUER_REWARD_ATA" --fee-payer "$FEE_PAYER" >/dev/null

AMOUNT="$RESERVE_TOTAL" ISSUER_REWARD_ATA="$ISSUER_REWARD_ATA" REWARD_ESCROW="$REWARD_ESCROW" \
PROGRAM_ID="$PROGRAM_ID" ISSUANCE_PDA="$ISSUANCE_PDA" node.exe tests/js/fund_reserve.js >/dev/null

DUMP="$(ISSUANCE_PDA="$ISSUANCE_PDA" node tests/js/dump_issuance_state.js)"
echo "$DUMP" | grep -q "reserve_funded 1" || die "reserve_funded not set"

ESCROW_BAL="$(spl-token balance --address "$REWARD_ESCROW" | tr -d '\r')"
[ "$ESCROW_BAL" = "$RESERVE_TOTAL" ] || die "escrow balance=$ESCROW_BAL expected=$RESERVE_TOTAL"

echo "✅ TEST PASS: fund_reserve"
