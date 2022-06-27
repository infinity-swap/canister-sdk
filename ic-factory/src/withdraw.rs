use candid::{CandidType, Principal};
use ic_helpers::management::{Canister, InstallCodeMode};
use serde::{Serialize,Deserialize};
use ic_canister_macros::{canister_call, virtual_canister_call};

use crate::error::FactoryError;

const WITHDRAW_MODULE: &'static [u8] = include_bytes!("cycles-withdraw.wasm");

// #[derive(Serialize, Deserialize, CandidType, Clone, Debug)]
// struct CyclesWalletInitPayload {
//     withdraw_to: Principal,
// }

pub fn get_bytes() -> Vec<u8> {
    Vec::from(WITHDRAW_MODULE)
}

pub async fn withdraw_cycles(withdraw_to: Principal, canister: &Canister) -> Result<(), FactoryError> {
    let wasm_module = Vec::from(WITHDRAW_MODULE);

    // let arg = CyclesWalletInitPayload {withdraw_to};
    canister
        .install_code(InstallCodeMode::Upgrade, wasm_module, (withdraw_to,))
        .await
        .map_err(|(_, e)| FactoryError::ManagementError(e))?;

    canister
        .start()
        .await
        .map_err(|(_, e)| FactoryError::ManagementError(e))?;

    let result = virtual_canister_call!(canister.id(), "withdraw_cycles", (), Result<String, String>).await;

    canister.stop().await;

    Ok(())
}