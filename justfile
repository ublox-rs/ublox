# Just recipes for running development tasks with ease
# https://github.com/casey/just
# install:  cargo install just
# run:      just

alias ba := build-all
alias ca := check-all
alias ta := test-all
alias l := lint

@_default:
    just --list

# Run all CI checks (except semver)
ci: typos lint build-all test-all lint-examples build-examples doc msrv

# Check all feature combinations
check-all *ARGS: (cmd-for-all-features "cargo check" ARGS)

# Build all feature combinations
build-all *ARGS: (cmd-for-all-features "cargo build" ARGS)

# Test all feature combinations
test-all *ARGS: (cmd-for-all-features "cargo test" ARGS)

# Build examples
build-examples:
    cargo build --release --all

# Format and lint examples
lint-examples:
    cargo fmt --all -- --check
    cargo clippy --all-targets --all -- -D warnings

# Run formatting and clippy lints
lint *ARGS:
    cargo fmt --all
    cargo clippy --all-targets -- -D warnings

# Build docs
doc $RUSTDOCFLAGS="--cfg docrs":
    cd ublox_derive && cargo +nightly doc --no-deps
    cd ublox        && cargo +nightly doc --no-deps

# Run MSRV checks
msrv:
    cargo hack check --rust-version --workspace

# Typo checking
typos:
    typos .

# Run `CMD` for all feature combinations
[no-exit-message]
cmd-for-all-features CMD *ARGS:
    #!/usr/bin/env bash
    set -euo pipefail
    
    feature_combinations=(
    '--features "alloc std ubx_proto23"'
    '--no-default-features --features "alloc ubx_proto23 sfrbx-gps"'
    '--no-default-features --features ubx_proto14'
    '--no-default-features --features ubx_proto23'
    '--no-default-features --features "ubx_proto23 std"'
    '--no-default-features --features "ubx_proto23 std serde"'
    '--no-default-features --features ubx_proto27'
    '--no-default-features --features ubx_proto31'
    '--no-default-features --features "ubx_proto31 std"'
    '--no-default-features --features "ubx_proto31 std serde"'
    )
    
    # Loop through each feature combination
    for feat in "${feature_combinations[@]}"; do
        tmp_cmd="{{CMD}} ${feat} {{ARGS}}"
        echo "{{YELLOW}}{{BOLD}}${tmp_cmd}{{NORMAL}}"
        set +e
        eval "${tmp_cmd}"
        rc=$?
        set -e
        if [[ rc -ne 0 ]]; then
            echo "{{RED}}{{BOLD}}Command failed: {{NORMAL}}{{YELLOW}}${tmp_cmd}{{NORMAL}}"
            exit 1
        fi
    done
