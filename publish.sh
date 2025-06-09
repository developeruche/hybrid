#!/bin/bash

# we cant publish to crate.io for now, some lib cannot be resolved using crate.io
# stop on error
set -e

cargo publish --package hybrid-compile
cargo publish --package hybrid-evm
cargo publish --package hybrid-vm
cargo publish --package hybrid-node
cargo publish --package cargo-hybrid
