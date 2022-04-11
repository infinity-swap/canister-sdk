use crate::factory::error::FactoryError;
use crate::factory::Factory;
use crate::ledger::LedgerPrincipalExt;
use ic_cdk::export::candid::{CandidType, Deserialize, Principal};
use ic_types::PrincipalId;
use ledger_canister::{Subaccount, TRANSACTION_FEE};
use std::future::Future;
use std::hash::Hash;
use std::pin::Pin;

/// This macro adds the following methods to the `$state` struct:
/// * `stable_save` - used to save the state to the stable storage
/// * `stable_restore` - used to load the state from the stable storage
/// * `reset` - used to replace the state in in-memory storage with the current one. This method
///   can be used in `init` method to set up the state.
///
/// It also provides `pre_upgrade` and `post_upgrade` functions.
///
/// IMPORTANT: This macro assumes that ths `$state` object is the only state used in the canister.
/// If this is not true, than this implementation cannot be used for state stable storage.
#[macro_export]
macro_rules! impl_factory_state_management {
    ( $state:ident, $bytecode:expr ) => {
        impl $state {
            pub fn stable_save(&self) {
                ::ic_kit::ic::stable_store((self,)).unwrap();
            }

            pub fn stable_restore() {
                let (mut loaded,): (Self,) = ::ic_kit::ic::stable_restore().unwrap();
                loaded.factory.restore($bytecode);
                loaded.reset();
            }

            pub fn reset(self) {
                let state = State::get();
                let mut state = state.borrow_mut();
                *state = self;
            }
        }

        #[::ic_cdk_macros::pre_upgrade]
        fn pre_upgrade() {
            $state::get().borrow().stable_save();
        }

        #[::ic_cdk_macros::post_upgrade]
        fn post_upgrade() {
            $state::stable_restore();
        }
    };
}

#[derive(Debug, CandidType, Deserialize)]
pub struct FactoryConfiguration {
    ledger_principal: Principal,
    icp_fee: u64,
    icp_to: Principal,
    controller: Principal,
}

impl FactoryConfiguration {
    pub fn new(
        ledger_principal: Principal,
        icp_fee: u64,
        icp_to: Principal,
        controller: Principal,
    ) -> Self {
        Self {
            ledger_principal,
            icp_fee,
            icp_to,
            controller,
        }
    }
}

/// This trait must be implemented by a factory state to make using of `init_factory_api` macro
/// possible.
pub trait FactoryState<K: Hash + Eq> {
    fn factory(&self) -> &Factory<K>;
    fn factory_mut(&mut self) -> &mut Factory<K>;

    fn configuration(&self) -> &FactoryConfiguration;
    fn configuration_mut(&mut self) -> &mut FactoryConfiguration;

    fn ledger_principal(&self) -> Principal {
        self.configuration().ledger_principal
    }

    fn controller(&self) -> Principal {
        self.configuration().controller
    }

    fn set_controller(&mut self, controller: Principal) -> Result<(), FactoryError> {
        self.check_controller_access()?;
        self.configuration_mut().controller = controller;

        Ok(())
    }

    fn icp_fee(&self) -> u64 {
        self.configuration().icp_fee
    }

    fn set_icp_fee(&mut self, fee: u64) -> Result<(), FactoryError> {
        self.check_controller_access()?;
        self.configuration_mut().icp_fee = fee;

        Ok(())
    }

    fn icp_to(&self) -> Principal {
        self.configuration().icp_to
    }

    fn set_icp_to(&mut self, to: Principal) -> Result<(), FactoryError> {
        self.check_controller_access()?;
        self.configuration_mut().icp_to = to;

        Ok(())
    }

    fn consume_provided_cycles_or_icp(
        &self,
        caller: Principal,
    ) -> Pin<Box<dyn Future<Output = Result<u64, FactoryError>>>> {
        Box::pin(consume_provided_cycles_or_icp(
            caller,
            self.ledger_principal(),
            self.icp_to(),
            self.icp_fee(),
            self.controller(),
        ))
    }

    fn check_controller_access(&self) -> Result<(), FactoryError> {
        let caller = ic_kit::ic::caller();
        if caller == self.controller() {
            Ok(())
        } else {
            Err(FactoryError::AccessDenied)
        }
    }

    fn forget_canister(&mut self, canister: &K) -> Result<(), FactoryError> {
        self.check_controller_access()?;
        self.factory_mut().forget(canister)
    }
}

// The canister creation fee is 10^11 cycles, so we require the provided amount to be a little larger.
// According to IC docs, 10^12 cycles should always cost 1 SDR, with is ~$1.
const MIN_CANISTER_CYCLES: u64 = 10u64.pow(12);

async fn consume_provided_cycles_or_icp(
    caller: Principal,
    ledger: Principal,
    icp_to: Principal,
    icp_fee: u64,
    controller: Principal,
) -> Result<u64, FactoryError> {
    if ic_kit::ic::msg_cycles_available() > 0 {
        consume_message_cycles()
    } else {
        if caller != controller {
            consume_provided_icp(caller, ledger, icp_to, icp_fee).await?;
        }

        Ok(MIN_CANISTER_CYCLES)
    }
}

fn consume_message_cycles() -> Result<u64, FactoryError> {
    let amount = ic_kit::ic::msg_cycles_available();
    if amount < MIN_CANISTER_CYCLES {
        return Err(FactoryError::NotEnoughCycles(amount, MIN_CANISTER_CYCLES));
    }

    Ok(ic_kit::ic::msg_cycles_accept(amount))
}

async fn consume_provided_icp(
    caller: Principal,
    ledger: Principal,
    icp_to: Principal,
    icp_fee: u64,
) -> Result<(), FactoryError> {
    let balance = ledger
        .get_balance(
            ic_kit::ic::id(),
            Some(Subaccount::from(&PrincipalId::from(caller))),
        )
        .await
        .map_err(FactoryError::LedgerError)?;

    if balance < icp_fee + TRANSACTION_FEE.get_e8s() {
        return Err(FactoryError::NotEnoughIcp(
            balance,
            icp_fee + TRANSACTION_FEE.get_e8s(),
        ));
    }

    consume_icp(caller, icp_fee, icp_to, ledger).await?;

    Ok(())
}

async fn consume_icp(
    from: Principal,
    amount: u64,
    icp_to: Principal,
    ledger: Principal,
) -> Result<(), FactoryError> {
    LedgerPrincipalExt::transfer(
        &ledger,
        icp_to,
        amount,
        Some(Subaccount::from(&PrincipalId::from(from))),
        None,
    )
    .await
    .map_err(FactoryError::LedgerError)?;

    Ok(())
}
