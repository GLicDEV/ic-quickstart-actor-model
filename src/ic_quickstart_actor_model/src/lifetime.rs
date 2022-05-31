use crate::{CanisterEnv, CanisterInstallSendArgs, Data, RuntimeState, RUNTIME_STATE};

#[allow(unused_imports)]
use ic_cdk_macros::{heartbeat, init, post_upgrade, pre_upgrade};

#[init]
fn init() {
    let env = Box::new(CanisterEnv::new());
    let data = Data::default();
    let mut runtime_state = RuntimeState { env, data };

    let call_arg = ic_cdk::api::call::arg_data::<(Option<CanisterInstallSendArgs>,)>().0;

    if let Some(args) = call_arg {
        runtime_state.data.business_state.colony = args.colony_state;
    }

    RUNTIME_STATE.with(|state| *state.borrow_mut() = runtime_state);
}

#[pre_upgrade]
fn pre_upgrade() {
    RUNTIME_STATE.with(|state| ic_cdk::storage::stable_save((&state.borrow().data,)).unwrap());
}

#[post_upgrade]
fn post_upgrade() {
    let env = Box::new(CanisterEnv::new());
    let (data,): (Data,) = ic_cdk::storage::stable_restore().unwrap();
    let runtime_state = RuntimeState { env, data };

    RUNTIME_STATE.with(|state| *state.borrow_mut() = runtime_state);
}

// #[heartbeat]
// fn heartbeat() {
//     // print("Hello from heartbeat");
// }
