#!/bin/sh

set -o errexit -o nounset -o pipefail

genesis_hash=`wget -q -O - "http://127.0.0.1:26657/block?height=1" | dasel -r json '.result.block_id.hash' | tr -d '"'`
echo "genesis hash received: $genesis_hash"
printf "%s" "$genesis_hash" >  "$home_dir/genesis.hash"
