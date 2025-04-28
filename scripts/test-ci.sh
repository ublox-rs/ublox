#!/usr/bin/env bash
set -euxo pipefail

need() {
    if ! command -v "$1" > /dev/null 2>&1; then
        echo "need $1 (command not found)"
        exit 1
    fi
}

cargo clippy -- -D warnings
cargo fmt --all -- --check

FEATURE_SETS=(
    "--features=alloc,std,ubx_proto23"
    "--no-default-features --features=alloc,ubx_proto23"
    "--no-default-features --features=ubx_proto14"
    "--no-default-features --features=ubx_proto23"
    "--no-default-features --features=ubx_proto27"
)

for features in "${FEATURE_SETS[@]}"; do
    cargo build ${features}
    cargo test ${features}
done

# Examples - using a subshell to isolate directory changes
(
    cd examples || exit 1
    cargo build --release
    cargo fmt --all -- --check
    cargo clippy --all-targets -- -D warnings
    cargo hack check --rust-version --workspace
)


need cargo-hack
cargo hack check --rust-version --workspace
(
    cd examples || exit 1
    cargo hack check --rust-version --workspace
)
