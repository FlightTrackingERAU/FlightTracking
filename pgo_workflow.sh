#!/bin/bash
if ! command -v llvm-profdata &> /dev/null
then
    echo "llvm-profdata is required to use Program Guided Optimization"
    echo "Either install llvm tools from your package manager or see: rustup component add llvm-tools-preview"
    echo "See: https://doc.rust-lang.org/rustc/profile-guided-optimization.html for more detials"
    exit
fi

rm -rf tmp/se300-pgo-data/

echo "Building binary with embedded PGO instrumentation..."
RUSTFLAGS="-Cprofile-generate=/tmp/se300-pgo-data" CARGO_TARGET_DIR="./target/embedded-pgo" \
    cargo build --release --target=x86_64-unknown-linux-gnu

echo "Running the binary: Scroll around and use as normal to gather data"
./target/embedded-pgo/x86_64-unknown-linux-gnu/release/flight_tracking_erau_se300

echo "Mergeing & post processing llvm data..."
llvm-profdata merge -o /tmp/se300-pgo-data/merged.profdata /tmp/se300-pgo-data

echo "Rebuilding the optimized binary, taking PGO data into account..."
rm -rf ./target/optimized-pgo
RUSTFLAGS="-Cprofile-use=/tmp/se300-pgo-data/merged.profdata" CARGO_TARGET_DIR="./target/optimized-pgo" \
    cargo build --release --target=x86_64-unknown-linux-gnu

echo "PGO optimized binary now available at: \"./target/optimized-pgo/x86_64-unknown-linux-gnu/release/flight_tracking_erau_se300\""

