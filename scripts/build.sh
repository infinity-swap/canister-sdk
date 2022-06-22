set -e
set +o noclobber

cargo build -p cycles-withdraw --target wasm32-unknown-unknown --release
cargo build -p canister_a --target wasm32-unknown-unknown --release
cargo build -p canister_b --target wasm32-unknown-unknown --release

ic-cdk-optimizer target/wasm32-unknown-unknown/release/canister_a.wasm -o target/wasm32-unknown-unknown/release/canister_a-opt.wasm
ic-cdk-optimizer target/wasm32-unknown-unknown/release/canister_b.wasm -o target/wasm32-unknown-unknown/release/canister_b-opt.wasm

ic-cdk-optimizer target/wasm32-unknown-unknown/release/cycles-withdraw.wasm -o target/wasm32-unknown-unknown/release/cycles-withdraw-opt.wasm
cargo run -p cycles-withdraw > target/wasm32-unknown-unknown/release/cycles-withdraw.did
