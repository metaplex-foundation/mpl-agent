#!/usr/bin/env bash
set -euo pipefail

# End-to-end test for Bubblegum -> MPL Core migration with Agent Registry
#
# Prerequisites:
#   - Solana CLI configured with a funded devnet keypair
#   - Programs deployed to devnet (or use mainnet program IDs)
#
# Usage:
#   ./scripts/test-bubblegum-e2e.sh [keypair] [rpc_url] [count]

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MIGRATOR_DIR="$(dirname "$SCRIPT_DIR")"

KEYPAIR="${1:-~/.config/solana/id.json}"
RPC="${2:-https://api.devnet.solana.com}"
COUNT="${3:-3}"

echo "============================================"
echo "  Bubblegum E2E Migration Test"
echo "============================================"
echo ""
echo "  Keypair:  $KEYPAIR"
echo "  RPC:      $RPC"
echo "  Count:    $COUNT"
echo ""

cd "$MIGRATOR_DIR"

# Step 1: Mint a test Bubblegum collection
echo "--- Step 1: Minting test Bubblegum collection ---"
echo ""

MINT_OUTPUT=$(npx ts-node src/index.ts mint-test \
  -s bubblegum \
  -k "$KEYPAIR" \
  --rpc "$RPC" \
  --count "$COUNT" \
  --concurrency 1 \
  --delay 2000 \
  --name "Bubblegum E2E Test" 2>&1 | tee /dev/stderr)

# Extract collection address from mint output
COLLECTION=$(echo "$MINT_OUTPUT" | grep "Collection:" | tail -1 | awk '{print $NF}')

if [ -z "$COLLECTION" ]; then
  echo "ERROR: Failed to extract collection address from mint output"
  exit 1
fi

echo ""
echo "  Collection: $COLLECTION"
echo ""

# Step 2: Wait for DAS indexing
echo "--- Step 2: Waiting for DAS indexing (30s) ---"
echo ""
sleep 30

# Step 3: Fetch assets to verify they are visible via DAS
echo "--- Step 3: Fetching assets via DAS ---"
echo ""

npx ts-node src/index.ts fetch \
  -c "$COLLECTION" \
  -s bubblegum \
  --rpc "$RPC"

echo ""

# Step 4: Dry run migration
echo "--- Step 4: Dry run migration ---"
echo ""

npx ts-node src/index.ts migrate \
  -c "$COLLECTION" \
  -s bubblegum \
  -k "$KEYPAIR" \
  --rpc "$RPC" \
  --batch-size 1 \
  --delay 2000 \
  --agent-uri "https://example.com/test-agent-registration.json"

echo ""

# Step 5: Execute migration
echo "--- Step 5: Executing migration ---"
echo ""

npx ts-node src/index.ts migrate \
  -c "$COLLECTION" \
  -s bubblegum \
  -k "$KEYPAIR" \
  --rpc "$RPC" \
  --batch-size 1 \
  --delay 2000 \
  --agent-uri "https://example.com/test-agent-registration.json" \
  --execute

echo ""

# Step 6: Check status
echo "--- Step 6: Checking migration status ---"
echo ""

npx ts-node src/index.ts status \
  -c "$COLLECTION" \
  --rpc "$RPC"

echo ""
echo "============================================"
echo "  Bubblegum E2E test complete!"
echo "  Manifest: ${COLLECTION}-migration.json"
echo "============================================"
