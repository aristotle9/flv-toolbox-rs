#!/bin/bash
# https://chr4.org/blog/2017/03/15/cross-compile-and-link-a-static-binary-on-macos-for-linux-with-cargo-and-rust/
cargo build --release --target x86_64-unknown-linux-musl --example flv-aac-fix
cargo build --release --target x86_64-unknown-linux-musl --example timestamp-normalization
file target/x86_64-unknown-linux-musl/release/examples/flv-aac-fix