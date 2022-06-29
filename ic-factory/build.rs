use std::path::Path;
use std::process::{Command};


macro_rules! w {
    ($($tokens: tt)*) => {
        println!("cargo:warning={}", format!($($tokens)*))
    }
}

fn main() {
    // Tricky relative dirs
    let target_dir_env = std::env::var("CARGO_TARGET_DIR").unwrap_or("target".to_string());
    let target_dir = Path::new(&target_dir_env).join("target-withdraw");
    let ws = Path::new(".").join("..");
    let ws_target = ws.join(&target_dir);
    let in_wasm = ws_target.join("wasm32-unknown-unknown").join("release").join("cycles-withdraw.wasm");
    let out_dir_env = std::env::var("OUT_DIR").unwrap();
    let out_wasm = Path::new(&out_dir_env).join("cycles-withdraw.wasm");

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
        .args(["build", "-p", "cycles-withdraw", "--target", "wasm32-unknown-unknown", "--release"])
        .env("CARGO_TARGET_DIR", target_dir)
        .current_dir(ws)
        .output()
        .expect("Cargo build was not started");

    if ! out.status.success() {
        w!("Child cargo build failed with code");
        w!("STDERR: {}", String::from_utf8(out.stderr).unwrap());
        panic!("Enough (^O_o)╯︵ ┻━┻");
    }

    w!("Optimizing wasm");
    w!("{} -> {} ", in_wasm.to_str().unwrap(), out_wasm.to_str().unwrap());
    let out = Command::new("ic-cdk-optimizer")
        .arg(in_wasm)
        .arg("-o")
        .arg(out_wasm)
        .output()
        .expect("Failed to optimize wasm");

    if ! out.status.success() {
        w!("Could not optimize wasm");
        w!("STDERR: {}", String::from_utf8(out.stderr).unwrap());
        panic!("Failed at the last step (❛︵❛。)");
    }
}