# Just recipes for running development tasks with ease
# https://github.com/casey/just
# install:  cargo install just
# run:      just

alias ba := build-all
alias bae := build-all-embedded
alias ca := check-all
alias ta := test-all
alias l := lint

@_default:
    just --list

# Run all CI checks (except semver)
[group("all")]
ci: typos \
    lint-msrv \
    build-all \
    build-all-embedded \
    test-all \
    lint-examples \
    build-examples \
    doc \
    msrv

# Check all feature combinations
[group("all")]
check-all *ARGS: (cmd-for-all-features "cargo check" ARGS)

# Build all feature combinations
[group("all")]
build-all *ARGS: (cmd-for-all-features "cargo build" ARGS)

# Build all feature combinations for embedded
[group("all")]
build-all-embedded *ARGS: (cmd-for-all-features-embedded "cargo build" ARGS)

# Test all feature combinations
[group("all")]
test-all *ARGS: (cmd-for-all-features "cargo test" ARGS)

# Build examples
[group("examples")]
build-examples: (cmd-for-all-examples "cargo build --release")

# Format and lint examples
[group("examples")]
lint-examples:
    cargo fmt --all -- --check
    cargo clippy --all-targets --workspace -- -D warnings

# Run formatting and clippy lints
[group("misc")]
lint *ARGS:
    cargo fmt --all
    cargo clippy --all-features --all-targets -- -D warnings

# Run clippy lints with version 1.85.0 which we use in CI atm (we should use a newer one!)
[group("misc")]
lint-msrv *ARGS:
    cargo fmt --all
    cargo +1.85 clippy --all-features --all-targets -- -D warnings

# Build docs
[group("misc")]
doc $RUSTDOCFLAGS="--cfg docrs":
    cd ublox_derive && cargo +nightly doc --no-deps --all-features
    cd ublox        && cargo +nightly doc --no-deps --all-features

# Run MSRV checks
[group("misc")]
msrv:
    cargo hack check --rust-version --workspace

# Typo checking
[group("misc")]
typos:
    typos .

# Run `CMD` for all feature combinations
[no-exit-message, group("misc")]
cmd-for-all-features CMD *ARGS:
    #!/usr/bin/env bash
    set -euo pipefail
    
    feature_combinations=(
    '' # Default features
    '--features full'
    '--no-default-features --features "alloc std ubx_proto23"'
    '--no-default-features --features "alloc ubx_proto23 sfrbx-gps"'
    '--no-default-features --features ubx_proto14'
    '--no-default-features --features ubx_proto23'
    '--no-default-features --features "ubx_proto23 std"'
    '--no-default-features --features "ubx_proto23 std serde"'
    '--no-default-features --features ubx_proto27'
    '--no-default-features --features ubx_proto31'
    '--no-default-features --features "ubx_proto31 std"'
    '--no-default-features --features "ubx_proto31 std serde"'
    '--no-default-features --features "alloc std ubx_proto14 ubx_proto23"'
    '--no-default-features --features "alloc std ubx_proto14 ubx_proto23 ubx_proto27 ubx_proto31"'
    )
    
    # Loop through each feature combination
    for feat in "${feature_combinations[@]}"; do
        just run-cmd-verbose "{{CMD}} ${feat} {{ARGS}}"
    done

[no-exit-message, group("misc")]
cmd-for-all-features-embedded CMD *ARGS:
    #!/usr/bin/env bash
    set -euo pipefail

    feature_combinations=(
    '--no-default-features --features ubx_proto23'
    '--no-default-features --features alloc,ubx_proto23'
    '--no-default-features --features serde,ubx_proto23'
    '--no-default-features --features ubx_proto27'
    '--no-default-features --features alloc,ubx_proto27'
    '--no-default-features --features serde,ubx_proto27'
    )

    # Loop through each feature combination
    for feat in "${feature_combinations[@]}"; do
        just run-cmd-verbose "{{CMD}} ${feat} --target thumbv6m-none-eabi --target thumbv7m-none-eabi --target thumbv7em-none-eabihf {{ARGS}}"
    done

[no-exit-message, group("misc")]
cmd-for-all-examples CMD *ARGS:
    #!/usr/bin/env bash
    set -euo pipefail

    examples=(
        'ublox-device'
        'ublox-tui'
        'basic-cli'
        'dds'
        'send-receive'
        'simple-parse'
    )

    # Loop through each example
    for example in "${examples[@]}"; do
        just run-cmd-verbose "{{CMD}} --package ${example} {{ARGS}}"
    done

[private, no-exit-message]
run-cmd-verbose CMD:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "{{YELLOW}}{{BOLD}}{{CMD}}{{NORMAL}}"
    set +e
    {{CMD}}
    rc=$?
    set -e
    if [[ rc -ne 0 ]]; then
        echo "{{RED}}{{BOLD}}Command failed: {{NORMAL}}{{YELLOW}}{{CMD}}{{NORMAL}}"
        exit 1
    fi