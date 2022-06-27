mod api;

#[cfg(any(target_arch = "wasm32", test))]
fn main() {}

#[cfg(not(any(target_arch = "wasm32", test)))]
fn main() {
    use candid::Principal;

    let canister_idl = ic_canister::generate_idl!();
    let result = candid::bindings::candid::compile(
        &canister_idl.env.env,
        &Some(canister_idl.actor)
    );
    println!("{result}");
}
