mod api;

#[cfg(any(target_arch = "wasm32", test))]
fn main() {}

#[cfg(not(any(target_arch = "wasm32", test)))]
fn main() {
    use candid::Principal;
    use crate::api::TestFactoryCanister;
    use ic_factory::api::FactoryCanister;

    let canister_idl = ic_canister::generate_idl!();
    let mut factory_idl = <TestFactoryCanister as FactoryCanister>::get_idl();
    factory_idl.merge(&canister_idl);

    let result = candid::bindings::candid::compile(&factory_idl.env.env, &Some(factory_idl.actor));
    println!("{result}");
}
