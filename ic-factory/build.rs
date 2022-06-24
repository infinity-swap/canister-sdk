use std::env;
use std::fs;
use std::path::Path;
use std::process::{Command, ExitStatus};


macro_rules! w {
    ($($tokens: tt)*) => {
        println!("cargo:warning={}", format!($($tokens)*))
    }
}

fn main() {
    // Tricky relative dirs
    let target = "target";
    let ws = Path::new(".").join("..");
    let child_cargo = ws.join("cycles-withdraw");
    // let child_cargo_target = Path::new(target);
    let ws_target = child_cargo.join(target);

    w!("Installing ic-cdk-optimizer");
    let out = Command::new("cargo")
        .args(["install", "ic-cdk-optimizer"])
        .output()
        .expect("Failed to install optimizer");

    if ! out.status.success() {
        w!("NOT able to install ic-cdk-optimizer");
        panic!("Enough (^O_o)╯︵ ┻━┻");
    }

    w!("Launching child cargo build");
    let out = Command::new("cargo")
        .args(["build", "--target", "wasm32-unknown-unknown", "--release"])
        .env("CARGO_TARGET_DIR", target)
        .current_dir(child_cargo)
        .output()
        .expect("Cargo build was not started");

    if ! out.status.success() {
        w!("Child cargo build failed");
        panic!("Enough (^O_o)╯︵ ┻━┻");
    }

    w!("Optimizing wasm");
    let out = Command::new("ic-cdk-optimizer")
        .arg(ws_target.join("wasm32-unknown-unknown").join("release").join("cycles-withdraw.wasm"))
        .arg("-o")
        .arg(Path::new("src").join("cycles-withdraw.wasm"))
        .output()
        .expect("Failed to optimize wasm");

    if ! out.status.success() {
        w!("Could not optimize wasm");
        panic!("Failed at the last step (❛︵❛。)");
    }
}