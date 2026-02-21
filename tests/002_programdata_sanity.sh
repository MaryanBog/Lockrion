#!/bin/bash
set -euo pipefail

# tests/002_programdata_sanity.sh
# FIX (silent exit reason):
# With `set -o pipefail`, this line can kill the script silently:
#   solana account ... | head -n 12
# because `head` closes the pipe early -> solana gets SIGPIPE -> nonzero -> pipefail -> exit.
#
# Solution:
# - Write full output to a temp file (no pipe)
# - Read only header from the file

RPC_URL="http://127.0.0.1:8899"
DEPLOY_SCRIPT="./deploy_lockrion_gitbash.sh"
PAYER_WALLET="target/deploy/test-wallet.json"

die() { echo "TEST FAIL: $*" 1>&2; exit 1; }
need_cmd() { command -v "$1" >/dev/null 2>&1 || die "Missing command: $1"; }

need_cmd solana
need_cmd curl
need_cmd awk
need_cmd head
need_cmd tr
need_cmd mktemp

echo "[1/4] Checking RPC at $RPC_URL ..."
curl -s "$RPC_URL" >/dev/null || die "RPC not reachable. Start validator in WSL2: solana-test-validator --reset"

solana config set --url "$RPC_URL" >/dev/null

echo "[2/4] Deploying via $DEPLOY_SCRIPT ..."
[ -f "$DEPLOY_SCRIPT" ] || die "Deploy script not found: $DEPLOY_SCRIPT"
chmod +x "$DEPLOY_SCRIPT" >/dev/null 2>&1 || true

OUT="$("$DEPLOY_SCRIPT" 2>&1)"

PROGRAM_ID="$(printf "%s\n" "$OUT" | awk '
  /Program ID \(will be deployed to\):/ {getline; gsub(/\r/,""); print; exit}
')"
[ -n "${PROGRAM_ID:-}" ] || die "Could not parse Program ID from deploy output"
echo "Program ID: $PROGRAM_ID"

echo "[3/4] Reading program info ..."
INFO="$(solana program show "$PROGRAM_ID" 2>&1)"

PROGRAMDATA_ADDR="$(printf "%s\n" "$INFO" | awk -F': ' '/ProgramData Address/ {gsub(/\r/,"",$2); print $2; exit}')"
AUTHORITY="$(printf "%s\n" "$INFO" | awk -F': ' '/^Authority/ {gsub(/\r/,"",$2); print $2; exit}')"

[ -n "${PROGRAMDATA_ADDR:-}" ] || die "Missing ProgramData Address"
[ -n "${AUTHORITY:-}" ] || die "Missing Authority"

# Authority must equal payer wallet pubkey (current workflow)
[ -f "$PAYER_WALLET" ] || die "Payer wallet not found: $PAYER_WALLET"
EXPECTED_AUTH="$(solana address -k "$PAYER_WALLET" | tr -d '\r')"
[ "$AUTHORITY" = "$EXPECTED_AUTH" ] || die "Authority mismatch. Expected=$EXPECTED_AUTH Got=$AUTHORITY"

echo "ProgramData Address: $PROGRAMDATA_ADDR"

echo "[4/4] Validating ProgramData account (header only) ..."

TMP="$(mktemp -t lockrion_pd_XXXX.txt)"
trap 'rm -f "$TMP"' EXIT

set +e
solana account "$PROGRAMDATA_ADDR" >"$TMP" 2>&1
RC=$?
set -e
[ $RC -eq 0 ] || die "solana account failed: $(head -n 20 "$TMP")"

ACC_HEAD="$(head -n 12 "$TMP")"

PD_OWNER="$(printf "%s\n" "$ACC_HEAD" | awk -F': ' '/^Owner/  {gsub(/\r/,"",$2); print $2; exit}')"
PD_LEN="$(printf "%s\n" "$ACC_HEAD" | awk -F': ' '/^Length/ {gsub(/\r/,"",$2); print $2; exit}')"

[ -n "${PD_OWNER:-}" ] || die "Could not parse ProgramData Owner. Header:\n$ACC_HEAD"
[ -n "${PD_LEN:-}" ] || die "Could not parse ProgramData Length. Header:\n$ACC_HEAD"

[ "$PD_OWNER" = "BPFLoaderUpgradeab1e11111111111111111111111" ] || die "Unexpected ProgramData owner: $PD_OWNER"

echo "ProgramData Owner: $PD_OWNER"
echo "ProgramData Length: $PD_LEN"
echo "✅ TEST PASS: programdata sanity ok"