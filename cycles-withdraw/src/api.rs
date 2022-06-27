use std::cell::RefCell;
use std::rc::Rc;

use candid::{CandidType, Principal};
use candid::types::{Serializer, Type};
use ic_cdk::api;
use ic_cdk::api::{canister_balance128, trap};
use ic_cdk::api::call::{call_raw128, CallResult};
use serde::Deserialize;

use ic_canister::{Canister, PreUpdate};
use ic_canister_macros::{init, query, update};
use ic_storage::IcStorage;
use ic_storage::stable::Versioned;

/// Emperically estimated value
/// from DFX SDK src/dfx/src/commands/canister/delete.rs
const WITHDRAWAL_COST: u128 = 10_000_000_000;

#[derive(CandidType)]
struct DepositCyclesArgs {
    canister_id: Principal,
}

#[derive(CandidType, IcStorage, Deserialize)]
pub struct WalletState {
    /// Cycles receiver
    pub receiver: Principal,
}

impl Default for WalletState {
    fn default() -> Self {
        WalletState {
            receiver: Principal::anonymous()
        }
    }
}

impl Versioned for WalletState {
    type Previous = ();

    fn upgrade(_: Self::Previous) -> Self {
        Self::default()
    }
}

#[derive(Clone, Canister)]
pub struct WalletCanister {
    #[id]
    principal: Principal,

    #[state]
    state: Rc<RefCell<WalletState>>,
}

impl PreUpdate for WalletCanister {}

#[allow(dead_code)]
impl WalletCanister {
    #[init]
    pub fn init(&self, cycles_receiver: Principal) {
        if cycles_receiver == Principal::anonymous() {
            let msg = format!("Invalid cycles receiver {}", cycles_receiver);
            trap(&msg);
        }

        self.state.replace(WalletState { receiver: cycles_receiver });
    }

    #[query]
    pub fn info(&self) -> String {
        let receiver = self.state.borrow().receiver;
        let balance = canister_balance128();
        format!(
            "Cycles available: {} | Withdraw allowed: {} | Withdraw principal: {}",
            canister_balance128(),
            if balance > WITHDRAWAL_COST {
                "YES"
            } else {
                "NO"
            },
            receiver
        )
    }

    #[update]
    pub async fn withdraw_cycles(&self) -> Result<String, String> {
        let receiver = self.state.borrow().receiver;

        let balance = canister_balance128();
        if balance <= WITHDRAWAL_COST {
            let msg = format!("Insufficient cycles to withdraw. Have only {}", balance);
            return Ok(msg);
        }

        let withdraw_amount = balance - WITHDRAWAL_COST;

        let result: CallResult<()> = api::call::call_with_payment128(
            Principal::management_canister(),
            "deposit_cycles",
            (DepositCyclesArgs {
                canister_id: receiver,
            }, ),
            withdraw_amount,
        ).await;

        return match result {
            Ok(_) => {
                let refund = api::call::msg_cycles_refunded128();
                let msg = format!("Cycles withdrawed: {}. Cycles refunded: {}", withdraw_amount, refund);
                Ok(msg)
            }
            Err((code, msg)) => {
                let refund = api::call::msg_cycles_refunded128();
                let call_error = format!("An error happened during the call: {}: {}", code as u8, msg);
                let error = format!(
                    "Cycles withdrawed: {}\nCycles refunded: {}\n{}",
                    withdraw_amount, refund, call_error
                );
                return Err(error);
            }
        };
    }
}