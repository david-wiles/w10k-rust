#!/bin/bash

# Compile for linux from macOS

set -eo pipefail

mkdir -p target
cd target

if [ ! -d "usr" ]; then
  # Download libssl-dev for linking
  # https://packages.debian.org/buster/amd64/libssl-dev/download
  curl -O http://ftp.us.debian.org/debian/pool/main/o/openssl/libssl-dev_1.1.1n-0+deb10u3_amd64.deb
  ar p libssl-dev_1.1.1n-0+deb10u3_amd64.deb data.tar.xz | tar xvf -
  rm -rf libssl-dev_1.1.1n-0+deb10u3_amd64.deb
fi

cd ..

export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-unknown-linux-gnu-gcc

export OPENSSL_DIR="$(pwd)/target/usr/"
export OPENSSL_LIB_DIR="$(pwd)/target/usr/lib/x86_64-linux-gnu/"

cargo build --target=x86_64-unknown-linux-gnu
