#!/bin/bash
set -euo pipefail

# tests/001_deploy_sanity.sh
# Purpose:
# - Проверить что валидатор жив (WSL2 уже запущен)
# - Задеплоить программу через deploy_lockrion_gitbash.sh
# - Проверить что solana program show возвращает ожидаемые поля

RPC_URL="http://127.0.0.1:8899"
DEPLOY_SCRIPT="./deploy_lockrion_gitbash.sh"

die() { echo "TEST FAIL: $*" 1>&2; exit 1; }
need_cmd() { command -v "$1" >/dev/null 2>&1 || die "Missing command: $1"; }

need_cmd solana
need_cmd curl
need_cmd grep
need_cmd awk

# 1) Validator must be running (WSL2)
echo "[1/3] Checking RPC at $RPC_URL ..."
curl -s "$RPC_URL" >/dev/null || die "RPC not reachable. Start validator in WSL2: solana-test-validator --reset"

# Ensure CLI points to local RPC (URL only)
solana config set --url "$RPC_URL" >/dev/null

# 2) Deploy (Git Bash)
echo "[2/3] Deploying via $DEPLOY_SCRIPT ..."
[ -f "$DEPLOY_SCRIPT" ] || die "Deploy script not found: $DEPLOY_SCRIPT"
chmod +x "$DEPLOY_SCRIPT" >/dev/null 2>&1 || true

OUT="$("$DEPLOY_SCRIPT" 2>&1 | tee /dev/tty)"

# Extract program id (line after "Program ID (will be deployed to):")
PROGRAM_ID="$(printf "%s\n" "$OUT" | awk '
  $0 ~ /Program ID \(will be deployed to\):/ {getline; gsub(/\r/,""); print $0; exit}
')"

[ -n "${PROGRAM_ID:-}" ] || die "Could not parse Program ID from deploy output"

echo "Program ID: $PROGRAM_ID"

# 3) Validate program show output
echo "[3/3] Validating program info ..."
INFO="$(solana program show "$PROGRAM_ID" 2>&1 | tee /dev/tty)"

printf "%s\n" "$INFO" | grep -q "Program Id:" || die "Missing 'Program Id' in program show"
printf "%s\n" "$INFO" | grep -q "Owner:" || die "Missing 'Owner' in program show"

# Since you deploy upgradeable in this workflow, expect upgradeable loader owner:
printf "%s\n" "$INFO" | grep -q "BPFLoaderUpgradeab1e" || die "Program is not upgradeable (unexpected for current deploy script)"

# Authority should be present (current workflow keeps it)
printf "%s\n" "$INFO" | grep -q "Authority:" || die "Missing 'Authority' (unexpected for current deploy script)"

echo "✅ TEST PASS: deploy sanity ok"