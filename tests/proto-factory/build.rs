use std::path::Path;
use std::process::Command;

macro_rules! w {
    ($($tokens: tt)*) => {
        println!("cargo:warning={}", format!($($tokens)*))
    }
}

fn main() {
    // Tricky relative dirs
    let target = "target-tests";
    let ws = Path::new("..");
    let ws_target = ws.join(target);

    w!("Installing ic-cdk-optimizer");
    let out = Command::new("cargo")
        .args(["install", "ic-cdk-optimizer"])
        .output()
        .expect("Failed to install optimizer");

    if !out.status.success() {
        w!("NOT able to install ic-cdk-optimizer");
        panic!("Enough (^O_o)╯︵ ┻━┻");
    }

    w!("Launching child cargo build");
    let out = Command::new("cargo")
        .args(["build", "-p", "canister-a", "--target", "wasm32-unknown-unknown", "--release"])
        .env("CARGO_TARGET_DIR", target)
        .current_dir(ws)
        .output()
        .expect("Cargo build was not started");

    if !out.status.success() {
        w!("Child cargo build failed with code");
        w!("STDERR: {}", String::from_utf8(out.stderr).unwrap());
        panic!("Enough (^O_o)╯︵ ┻━┻");
    }

    w!("Optimizing wasm");
    let out = Command::new("ic-cdk-optimizer")
        .arg(ws_target.join("wasm32-unknown-unknown").join("release").join("canister-a.wasm"))
        .arg("-o")
        .arg(Path::new("src").join("canister-a.wasm"))
        .output()
        .expect("Failed to optimize wasm");

    if !out.status.success() {
        w!("Could not optimize wasm");
        w!("STDERR: {}", String::from_utf8(out.stderr).unwrap());
        panic!("Failed at the last step (❛︵❛。)");
    }
}