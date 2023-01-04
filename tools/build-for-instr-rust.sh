#!/usr/bin/env bash

set -e
set -u

cd "$(dirname "${0}")/.."

build_dir() {
    local dir="${1}"
    (
        cd "${dir}"

        for profile in quick-build-incremental quick-build-nonincremental; do
            cargo clean --profile "${profile}"
            cargo test --profile "${profile}"
            cargo clean --profile "${profile}"
            cargo build --profile "${profile}"
            cargo test --profile "${profile}"
        done
    )
}

build_dir rust
build_dir rust-threecrate-cratecargotest
build_dir rust-threecrate-crateunotest
build_dir rust-twocrate-cratecargotest
build_dir rust-twocrate-unittest
build_dir rust-workspace-cratecargotest-nodefaultfeatures
build_dir rust-workspace-crateunotest
