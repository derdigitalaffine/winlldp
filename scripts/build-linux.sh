#!/usr/bin/env sh
set -eu
cd "$(dirname "$0")/.."
export WINLLDP_SOURCE_REV="${GITHUB_SHA:-$(git rev-parse HEAD)}"
cargo fmt --check
cargo test --locked --all-targets
cargo clippy --locked --all-targets -- -D warnings
cargo build --locked --release --bins
mkdir -p build/linux-x64
cp target/release/winlldp build/linux-x64/WinLLDP-Linux-x64
cp target/release/winlldp-cli build/linux-x64/WinLLDP-CLI-Linux-x64
chmod 755 build/linux-x64/WinLLDP-Linux-x64 build/linux-x64/WinLLDP-CLI-Linux-x64
rust_version="$(rustc --version)"
source_rev="${WINLLDP_SOURCE_REV}"
printf 'WinLLDP 1.2.0 (Linux x64)\nSource commit: %s\nRust: %s\nTarget: x86_64-unknown-linux-gnu\nLive capture: Windows only\n' "$source_rev" "$rust_version" > build/linux-x64/BUILD_INFO-Linux.txt
(cd build/linux-x64 && sha256sum WinLLDP-Linux-x64 WinLLDP-CLI-Linux-x64 > SHA256SUMS-Linux.txt)
