use std::path::MAIN_SEPARATOR;

use candid::{CandidType, Principal};
use candid::types::{Serializer, Type};
use ic_cdk::api::{canister_balance128, trap};
use ic_cdk::api::call::{call_raw128, CallResult};

use ic_canister::{Canister, PreUpdate};
use ic_canister_macros::{init, query, update};
use ic_factory::api::FactoryCanister;

const A_BYTES: &'static [u8] = include_bytes!(concat!(env!("OUT_DIR"), "/canister-a.wasm"));

const CYCLES_TO_ADD: u64 = 100_000_000_000;

#[derive(Clone, Canister)]
pub struct TestFactoryCanister {
    #[id]
    principal: Principal,

}

impl PreUpdate for TestFactoryCanister {}

impl FactoryCanister for TestFactoryCanister {}

#[allow(dead_code)]
impl TestFactoryCanister {
    #[update]
    pub async fn create_asset(&self) -> Result<Principal, String> {
        let factory_state = self.factory_state();
        let fs = factory_state.borrow();

        let canister = fs.factory.create_with_cycles(A_BYTES, (), CYCLES_TO_ADD)
            .await
            .map_err(|e| format!("{:?} -> {}", e.0, e.1))?;

        let principal = canister.identity();
        self.factory_state()
            .borrow_mut()
            .factory
            .register(principal, canister);

        Ok(principal)
    }

    #[update]
    fn delete_asset(&self, principal: Principal) -> Result<(), String> {
        Ok(())
    }
}