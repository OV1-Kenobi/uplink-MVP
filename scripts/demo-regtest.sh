#!/usr/bin/env bash
# demo-regtest.sh — Uplink Phase A8 demo on regtest
#
# Prerequisites:
#   - bitcoin-cli + bitcoind running in regtest mode
#   - Esplora (esplora_spk_server) listening on localhost:3000
#   - cargo build done: PATH="/home/circleci/.cargo/bin:$PATH" cargo build -p host-cli
#
# Usage:
#   ./scripts/demo-regtest.sh
#
# What this demonstrates:
#   1. Generate a new Nostr identity (BIP-39 mnemonic → npub + LDK seed)
#   2. Get an on-chain receive address from the LDK node
#   3. Fund the address with bitcoin-cli sendtoaddress (regtest)
#   4. Mine a block to confirm the UTXO
#   5. Show on-chain balance
#   6. (Interactive) Open a channel stub and demonstrate streaming intent output

set -euo pipefail

CARGO="PATH=/home/circleci/.cargo/bin:$PATH cargo"
CLI="$CARGO run -p host-cli --"
NETWORK="regtest"
ESPLORA="http://localhost:3000"
LDK_DIR="./demo-ldk-data"

echo ""
echo "╔══════════════════════════════════════════════════════════╗"
echo "║          Uplink MVP — Phase A8 Regtest Demo              ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""

# Clean previous demo state
rm -rf "$LDK_DIR" uplink.db

# Step 1: Generate identity
echo "▶ Step 1: Generating new Nostr identity..."
$CLI --network $NETWORK --esplora-url $ESPLORA --ldk-dir $LDK_DIR identity new
echo ""

# Step 2: Show the npub
echo "▶ Step 2: Identity info:"
$CLI --network $NETWORK --esplora-url $ESPLORA --ldk-dir $LDK_DIR identity show
echo ""

# Step 3: Get an on-chain address
echo "▶ Step 3: Getting on-chain receive address..."
ADDRESS=$($CLI --network $NETWORK --esplora-url $ESPLORA --ldk-dir $LDK_DIR wallet address 2>&1 | grep "On-chain Address:" | awk '{print $NF}')
echo "  Address: $ADDRESS"
echo ""

# Step 4: Fund via bitcoin-cli (if available)
echo "▶ Step 4: Funding address on regtest..."
if command -v bitcoin-cli &>/dev/null; then
  bitcoin-cli -regtest sendtoaddress "$ADDRESS" 0.01
  bitcoin-cli -regtest -generate 6
  echo "  Funded 0.01 BTC and mined 6 blocks."
else
  echo "  ⚠ bitcoin-cli not found — skipping automatic funding."
  echo "  Fund manually: bitcoin-cli -regtest sendtoaddress $ADDRESS 0.01 && bitcoin-cli -regtest -generate 6"
fi
echo ""

# Step 5: Show balance
echo "▶ Step 5: Checking balance (sync may take a few seconds)..."
sleep 3
$CLI --network $NETWORK --esplora-url $ESPLORA --ldk-dir $LDK_DIR wallet balance
echo ""

echo "═══════════════════════════════════════════════════"
echo "✓ Phase A8 native demo complete."
echo ""
echo "Next steps:"
echo "  • Open the PWA (cd web && npm run dev) to use the browser UI"
echo "  • Create a streaming-sats flow in the Streams tab"
echo "  • Add a relay in Settings to receive kind-9901 receipt events"
echo "═══════════════════════════════════════════════════"
