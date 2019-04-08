#!/bin/bash
# https://chr4.org/blog/2017/03/15/cross-compile-and-link-a-static-binary-on-macos-for-linux-with-cargo-and-rust/
cargo build --release --example flv-info
cargo build --release --example flv-config
cargo build --release --example flv-split
cp ./target/release/examples/flv-* ./bin/
cargo build --release --target x86_64-unknown-linux-musl --example flv-info
cargo build --release --target x86_64-unknown-linux-musl --example flv-aac-fix
cargo build --release --target x86_64-unknown-linux-musl --example timestamp-normalization
file target/x86_64-unknown-linux-musl/release/examples/flv-aac-fix
cd target/x86_64-unknown-linux-musl/release/examples/
zip -u ../../../../x86_64-unknown-linux-musl.zip flv-info flv-aac-fix timestamp-normalization
cd ~-