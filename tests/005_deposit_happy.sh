#!/bin/bash
export NODE_OPTIONS="--no-deprecation"
set -euo pipefail

RPC_URL="http://127.0.0.1:8899"
DEPLOY_SCRIPT="./deploy_lockrion_gitbash.sh"
PAYER_WALLET="platform-authority.json"
FEE_PAYER="$PAYER_WALLET"

die(){ echo "TEST FAIL: $*" 1>&2; exit 1; }
need(){ command -v "$1" >/dev/null 2>&1 || die "Missing command: $1"; }

need solana
need spl-token
need node.exe
need awk
need curl
need tr
need sleep

curl -s "$RPC_URL" >/dev/null || die "RPC not reachable"
solana config set --url "$RPC_URL" >/dev/null
solana config set --keypair "$PAYER_WALLET" >/dev/null

[ -f "tests/js/init_issuance.js" ] || die "Missing tests/js/init_issuance.js"
[ -f "tests/js/fund_reserve.js" ] || die "Missing tests/js/fund_reserve.js"
[ -f "tests/js/deposit.js" ] || die "Missing tests/js/deposit.js"

# Deploy
OUT="$("$DEPLOY_SCRIPT" 2>&1)"
solana config set --keypair "$PAYER_WALLET" >/dev/null
PROGRAM_ID="$(printf "%s\n" "$OUT" | awk '/Program ID \(will be deployed to\):/ {getline; gsub(/\r/,""); print; exit}')"
[ -n "${PROGRAM_ID:-}" ] || die "Could not parse Program ID"

ISSUER="$(solana address -k "$PAYER_WALLET" | tr -d '\r')"

# === CHAIN TIME (NO DRIFT) ===
CHAIN_NOW="$(node.exe -e "
const {Connection}=require('@solana/web3.js');
(async()=>{
  const c=new Connection('$RPC_URL','confirmed');
  const slot=await c.getSlot('confirmed');
  const bt=await c.getBlockTime(slot);
  if(!bt){ console.error('NO_BLOCKTIME'); process.exit(2); }
  process.stdout.write(String(bt));
})().catch(e=>{ console.error(e?.message||e); process.exit(2); });
")"
[ "${CHAIN_NOW:-}" != "NO_BLOCKTIME" ] || die "Could not read chain time via getBlockTime"

# start in 30s from CHAIN time (so fund_reserve is allowed), maturity +30d
START_TS=$((CHAIN_NOW + 30))
MATURITY_TS=$((START_TS + 86400*30))

RESERVE_TOTAL=1000
DEPOSIT_AMOUNT=100

# Issuance PDA (canonical v1.1 seeds)
ISSUANCE_PDA="$(PROGRAM_ID="$PROGRAM_ID" PAYER="$ISSUER" START_TS="$START_TS" RESERVE_TOTAL="$RESERVE_TOTAL" node.exe -e "const{PublicKey}=require('@solana/web3.js');const programId=new PublicKey(process.env.PROGRAM_ID);const payer=new PublicKey(process.env.PAYER);const startTs=BigInt(process.env.START_TS);const reserve=BigInt(process.env.RESERVE_TOTAL);const s1=Buffer.from('issuance');const s2=payer.toBuffer();const s3=Buffer.alloc(8);s3.writeBigInt64LE(startTs);const s4=Buffer.alloc(16);s4.writeBigUInt64LE(reserve,0);s4.writeBigUInt64LE(0n,8);const[pda]=PublicKey.findProgramAddressSync([s1,s2,s3,s4],programId);process.stdout.write(pda.toBase58());")"
echo "ISSUANCE_PDA=$ISSUANCE_PDA"

# Create mints + escrows
REWARD_MINT="$(spl-token create-token --decimals 0 --fee-payer "$FEE_PAYER" | awk '/Creating token/ {print $3}')"
REWARD_ESCROW="$(spl-token create-account "$REWARD_MINT" --owner "$ISSUANCE_PDA" --fee-payer "$FEE_PAYER" | awk '/Creating account/ {print $3}')"

LOCK_MINT="$(spl-token create-token --decimals 0 --fee-payer "$FEE_PAYER" | awk '/Creating token/ {print $3}')"
DEPOSIT_ESCROW="$(spl-token create-account "$LOCK_MINT" --owner "$ISSUANCE_PDA" --fee-payer "$FEE_PAYER" | awk '/Creating account/ {print $3}')"

PLATFORM_TREASURY="$ISSUER"

# init_issuance
PROGRAM_ID="$PROGRAM_ID" START_TS="$START_TS" MATURITY_TS="$MATURITY_TS" RESERVE_TOTAL="$RESERVE_TOTAL" \
LOCK_MINT="$LOCK_MINT" REWARD_MINT="$REWARD_MINT" DEPOSIT_ESCROW="$DEPOSIT_ESCROW" REWARD_ESCROW="$REWARD_ESCROW" \
PLATFORM_TREASURY="$PLATFORM_TREASURY" node.exe tests/js/init_issuance.js >/dev/null

# fund_reserve (must be before start_ts)
ISSUER_REWARD_ATA="$(spl-token create-account "$REWARD_MINT" --owner "$ISSUER" --fee-payer "$FEE_PAYER" 2>&1 | awk '/Creating account/ {print $3}')"
spl-token mint "$REWARD_MINT" "$RESERVE_TOTAL" "$ISSUER_REWARD_ATA" --fee-payer "$FEE_PAYER" >/dev/null

AMOUNT="$RESERVE_TOTAL" ISSUER_REWARD_ATA="$ISSUER_REWARD_ATA" REWARD_ESCROW="$REWARD_ESCROW" \
PROGRAM_ID="$PROGRAM_ID" ISSUANCE_PDA="$ISSUANCE_PDA" node.exe tests/js/fund_reserve.js >/dev/null

# Wait until chain time reaches start_ts
while true; do
  NOW_CHAIN="$(node.exe -e "
  const {Connection}=require('@solana/web3.js');
  (async()=>{
    const c=new Connection('$RPC_URL','confirmed');
    const slot=await c.getSlot('confirmed');
    const bt=await c.getBlockTime(slot);
    process.stdout.write(String(bt||0));
  })().catch(()=>process.stdout.write('0'));
  ")"
  [ "$NOW_CHAIN" -gt 0 ] || die "Chain time query failed during wait"
  if [ "$NOW_CHAIN" -ge "$START_TS" ]; then
    break
  fi
  REM=$((START_TS - NOW_CHAIN))
  if [ $REM -gt 5 ]; then
    echo "[wait] remaining ${REM}s (chain time) until start_ts=${START_TS} ..."
    sleep 3
  else
    sleep 1
  fi
done

# --- deposit happy path ---
PARTICIPANT="$ISSUER"
PARTICIPANT_LOCK_ATA="$(spl-token create-account "$LOCK_MINT" --owner "$PARTICIPANT" --fee-payer "$FEE_PAYER" 2>&1 | awk '/Creating account/ {print $3}')"
spl-token mint "$LOCK_MINT" "$DEPOSIT_AMOUNT" "$PARTICIPANT_LOCK_ATA" --fee-payer "$FEE_PAYER" >/dev/null

ESCROW_BEFORE="$(spl-token balance --address "$DEPOSIT_ESCROW" | tr -d '\r')"

PROGRAM_ID="$PROGRAM_ID" ISSUANCE_PDA="$ISSUANCE_PDA" \
PARTICIPANT_LOCK_ATA="$PARTICIPANT_LOCK_ATA" DEPOSIT_ESCROW="$DEPOSIT_ESCROW" AMOUNT="$DEPOSIT_AMOUNT" \
node.exe tests/js/deposit.js >/dev/null

ESCROW_AFTER="$(spl-token balance --address "$DEPOSIT_ESCROW" | tr -d '\r')"
[ "$ESCROW_AFTER" = "$DEPOSIT_AMOUNT" ] || die "deposit escrow balance=$ESCROW_AFTER expected=$DEPOSIT_AMOUNT"
[ "$ESCROW_BEFORE" = "0" ] || true

# === Hard assert: issuance.total_locked == DEPOSIT_AMOUNT (read bytes from account) ===
# IssuanceState.total_locked offset = 242, u128 little-endian (16 bytes)
TOTAL_LOCKED_JSON="$(
  RPC="$RPC_URL" ISSUANCE_PDA="$ISSUANCE_PDA" node.exe - <<'NODE'
const {Connection, PublicKey} = require("@solana/web3.js");
const RPC = process.env.RPC;
const pda = new PublicKey(process.env.ISSUANCE_PDA);

function readU128LE(buf, off){
  const lo = buf.readBigUInt64LE(off);
  const hi = buf.readBigUInt64LE(off + 8);
  return (hi << 64n) + lo;
}

(async()=>{
  const c = new Connection(RPC, "confirmed");
  const ai = await c.getAccountInfo(pda);
  if(!ai){ console.error("ISSUANCE_NOT_FOUND"); process.exit(2); }
  const data = Buffer.from(ai.data);
  const totalLocked = readU128LE(data, 242);
  const reserveFunded = data.readUInt8(282);
  process.stdout.write(JSON.stringify({ reserve_funded: reserveFunded, total_locked: totalLocked.toString() }));
})();
NODE
)"
echo "[debug] issuance=$TOTAL_LOCKED_JSON"

echo "$TOTAL_LOCKED_JSON" | node.exe -e "
let s='';process.stdin.on('data',d=>s+=d);
process.stdin.on('end',()=>{
  const j=JSON.parse(s);
  const expected = BigInt($DEPOSIT_AMOUNT);
  if(j.reserve_funded!==1){ process.exit(3); }
  if(BigInt(j.total_locked)!==expected){ process.exit(4); }
});" || die "total_locked not updated to $DEPOSIT_AMOUNT"

echo "✅ TEST PASS: deposit happy"