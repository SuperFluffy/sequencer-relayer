#!/bin/sh

set -o errexit -o nounset -o pipefail

# change ports that we know metro metro will not receive messages on
# so they won't interfere with celestia-app ports:
dasel put -r toml '.rpc.pprof_laddr' -t string -v "127.0.0.1:60000" -f "$home_dir/config/config.toml"
dasel put -r toml '.rpc.laddr' -t string -v "tcp://0.0.0.0:60001" -f "$home_dir/config/config.toml"
dasel put -r toml '.p2p.laddr' -t string -v "tcp://0.0.0.0:60002" -f "$home_dir/config/config.toml"
