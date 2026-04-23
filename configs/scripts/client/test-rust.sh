#!/bin/bash

set -o pipefail

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)
PROGRAMS_OUTPUT="./programs/.bin"
# go to parent folder
cd "$(dirname "$(dirname "$(dirname "${SCRIPT_DIR}")")")" || exit 1

# command-line input
ARGS=$*

WORKING_DIR=$(pwd)
SOLFMT="solfmt"
export SBF_OUT_DIR="${WORKING_DIR}/${PROGRAMS_OUTPUT}"
SBF_TOOLS_VERSION="${SBF_TOOLS_VERSION:-v1.53}"

# Run tests for all Rust client crates from the workspace root.
RUST_CLIENTS=("rust-identity" "rust-reputation" "rust-validation" "rust-tools")

for CLIENT in "${RUST_CLIENTS[@]}"; do
    echo "Testing clients/${CLIENT}..."
    cd "${WORKING_DIR}/clients/${CLIENT}" || exit 1

    if [ ! "$(command -v $SOLFMT)" = "" ]; then
        CARGO_TERM_COLOR=always cargo test-sbf --tools-version ${SBF_TOOLS_VERSION} --sbf-out-dir ${WORKING_DIR}/${PROGRAMS_OUTPUT} ${ARGS} 2>&1 | ${SOLFMT} -- --nocapture
    else
        cargo test-sbf --tools-version ${SBF_TOOLS_VERSION} --sbf-out-dir ${WORKING_DIR}/${PROGRAMS_OUTPUT} ${ARGS} -- --nocapture
    fi

    if [ $? -ne 0 ]; then
        echo "Tests failed for clients/${CLIENT}"
        exit 1
    fi
done
