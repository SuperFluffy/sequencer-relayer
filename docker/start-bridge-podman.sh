#!/usr/bin/env bash

set -o errexit -o nounset

celestia bridge init --node.store "$home_dir/bridge"
# celestia_custom=$(<"$home_dir/genesis.hash")
export CELESTIA_CUSTOM=test:`cat $home_dir/genesis.hash`
echo $CELESTIA_CUSTOM
  # --p2p.network "test:$celestia_custom"
exec celestia bridge start \
  --node.store "$home_dir/bridge" --gateway \
  --core.ip 127.0.0.1 \
  --keyring.accname "$validator_key_name"