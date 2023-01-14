#!/bin/bash

set -eo pipefail

if [ -z "$1" ]; then
  echo "Usage: ./deploy.sh [domain]";
  exit 1
else
  export TF_VAR_domain=$1
fi

# Build linux executables
./build.sh --release

# Create the VM
cd tf
terraform apply

# Copy executables to VM
cd ..
scp target/x86_64-unknown-linux-gnu/release/broadcast "w10k-rust.$1":broadcast
scp target/x86_64-unknown-linux-gnu/release/client2client "w10k-rust.$1":client2client
