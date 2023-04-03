#!/usr/bin/env bash

set -o errexit -o nounset

# Start the celestia-app
exec celestia-appd start --home ${home_dir}
