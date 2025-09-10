#!/bin/bash
#
# Run `cargo metadata` to figure out metadata from a Cargo package, which we
# use in external utilities. Print this on `stdout` and exit with 0.
#
# Paths to `cargo`, `jq`, and the manifest can be passed via environment
# variables.

set -eo pipefail

BIN_CARGO="${BIN_CARGO:-"cargo"}"
BIN_JQ="${BIN_JQ:-"jq"}"
PATH_CARGO_TOML="${PATH_CARGO_TOML:-"./Cargo.toml"}"

MD=$(${BIN_CARGO} \
        metadata \
                --format-version 1 \
                --frozen \
                --manifest-path "${PATH_CARGO_TOML}" \
                --no-deps \
)

${BIN_JQ} \
        <<<"${MD}" \
        -cer \
        '.packages | map(select(.name == "osi")) | .[0].version'

${BIN_JQ} \
        <<<"${MD}" \
        -cer \
        '.packages | map(select(.name == "sys")) | .[0].version'

${BIN_JQ} \
        <<<"${MD}" \
        -cer \
        '.packages | map(select(.name == "tmp")) | .[0].version'
