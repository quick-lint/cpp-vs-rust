#!/usr/bin/env bash

set -e
set -u

cd "$(dirname "${0}")/.."

build_dir() {
    local dir="${1}"
    (
        cd "${dir}"
        cargo clean --profile quick-build-incremental
        cargo test --profile quick-build-incremental
        cargo clean --profile quick-build-incremental
        cargo build --profile quick-build-incremental
        cargo test --profile quick-build-incremental
    )
}

build_dir rust
build_dir rust-twocrate-cratecargotest
build_dir rust-workspace-crateunotest
