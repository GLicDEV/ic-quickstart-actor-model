mod business_logic;
mod env;
mod lifetime;

use crate::env::{CanisterEnv, EmptyEnv, Environment};
use business_logic::{
    BusinessState, ColonyState, ExpeditionState, ExpeditionStep, Inventory, PlayerStatus,
    Resources, SystemSettings,
};
use candid::{candid_method, CandidType, Encode, Nat, Principal};

use env::MILLIS_TO_SECONDS;
use ic_cdk_macros::*;
use serde::Deserialize;

use std::{
    cell::{Ref, RefCell},
    collections::HashMap,
};

use sha2::{Digest, Sha256};

thread_local! {
    static RUNTIME_STATE: RefCell<RuntimeState> = RefCell::default();
}

struct RuntimeState {
    pub env: Box<dyn Environment>,
    pub data: Data,
}

impl Default for RuntimeState {
    fn default() -> Self {
        RuntimeState {
            env: Box::new(EmptyEnv {}),
            data: Data::default(),
        }
    }
}

#[derive(CandidType, Default, Deserialize)]
struct Data {
    business_state: BusinessState,
    system_settings: SystemSettings,
}

#[derive(CandidType, Deserialize)]
struct ColonyInfo {
    canister_id: Principal,
    generation: u8,
    taxes_percent: u8,
    rewards_per_second: HashMap<Resources, u8>,
    coffers: Inventory,
    player_count: usize,
    expeditions_count: u64,
}

#[candid_method(query, rename = "getColonyInfo")]
#[query(name = "getColonyInfo")]
fn get_colony_info() -> ColonyInfo {
    RUNTIME_STATE.with(|state| get_colony_info_impl(state.borrow()))
}

fn get_colony_info_impl(runtime_state: Ref<RuntimeState>) -> ColonyInfo {
    ColonyInfo {
        canister_id: runtime_state.env.canister_id(),
        generation: runtime_state.data.business_state.colony.generation,
        taxes_percent: runtime_state.data.business_state.colony.taxes_percent,
        rewards_per_second: runtime_state
            .data
            .business_state
            .colony
            .rewards_per_second
            .clone(),
        coffers: runtime_state.data.business_state.colony.coffers.clone(),
        player_count: runtime_state.data.business_state.player.len(),
        expeditions_count: runtime_state.data.business_state.expeditions_count,
    }
}

#[candid_method(query, rename = "getPlayerInventory")]
#[query(name = "getPlayerInventory")]
fn get_player_inventory() -> Inventory {
    RUNTIME_STATE.with(|state| get_player_inventory_impl(state.borrow()))
}

fn get_player_inventory_impl(runtime_state: Ref<RuntimeState>) -> Inventory {
    runtime_state
        .data
        .business_state
        .player
        .get(&runtime_state.env.caller())
        .unwrap()
        .get_inventory()
}

#[candid_method(update, rename = "startWork")]
#[update(name = "startWork")]
fn start_work() -> Result<(), String> {
    RUNTIME_STATE.with(|state| start_work_impl(&mut state.borrow_mut()))
}

fn start_work_impl(runtime_state: &mut RuntimeState) -> Result<(), String> {
    match runtime_state
        .data
        .business_state
        .player
        .get(&runtime_state.env.caller())
        .expect("Player not found in this world")
        .get_status()
    {
        PlayerStatus::Idle => runtime_state.data.business_state.work_set(
            runtime_state.env.caller(),
            None,
            runtime_state.env.now(),
        ),
        PlayerStatus::WorkingAll(_) | PlayerStatus::WorkingFocused(_, _) => {
            Err("Player is already working.".to_string())
        }

        PlayerStatus::Traveling => Err("Cannot start work when traveling".to_string()),
    }
}

#[candid_method(update, rename = "stopWork")]
#[update(name = "stopWork")]
fn stop_work() -> Result<(), String> {
    RUNTIME_STATE.with(|state| stop_work_impl(&mut state.borrow_mut()))
}

fn stop_work_impl(runtime_state: &mut RuntimeState) -> Result<(), String> {
    match runtime_state
        .data
        .business_state
        .player
        .get(&runtime_state.env.caller())
        .expect("Player not found in this world")
        .get_status()
    {
        PlayerStatus::WorkingAll(_) | PlayerStatus::WorkingFocused(_, _) => runtime_state
            .data
            .business_state
            .work_claim(runtime_state.env.caller(), runtime_state.env.now()),

        PlayerStatus::Idle | PlayerStatus::Traveling => {
            Err("Player is currently not working".to_string())
        }
    }
}

#[candid_method(query, rename = "getUnclaimedWork")]
#[query(name = "getUnclaimedWork")]
fn get_unclaimed_work() -> Result<Vec<(Resources, u64)>, String> {
    RUNTIME_STATE.with(|state| get_unclaimed_work_impl(state.borrow()))
}

fn get_unclaimed_work_impl(
    runtime_state: Ref<RuntimeState>,
) -> Result<Vec<(Resources, u64)>, String> {
    let seconds_elapsed;

    match runtime_state
        .data
        .business_state
        .player
        .get(&runtime_state.env.caller())
        .expect("Player not found in this world")
        .get_status()
    {
        PlayerStatus::WorkingAll(working_since)
        | PlayerStatus::WorkingFocused(working_since, _) => {
            seconds_elapsed = (runtime_state.env.now() - working_since) / MILLIS_TO_SECONDS
        }
        _ => return Err("The player is not currently working".to_string()),
    };

    Ok(runtime_state
        .data
        .business_state
        .available_unclaimed(seconds_elapsed))
}

#[candid_method(query, rename = "getExpeditions")]
#[query(name = "getExpeditions")]
fn get_expeditions() -> HashMap<u64, ExpeditionState> {
    RUNTIME_STATE.with(|state| get_expeditions_impl(state.borrow()))
}

fn get_expeditions_impl(runtime_state: Ref<RuntimeState>) -> HashMap<u64, ExpeditionState> {
    runtime_state.data.business_state.expeditions.clone()
}

#[candid_method(update, rename = "startExpedition")]
#[update(name = "startExpedition")]
fn start_expedition() -> Result<(), String> {
    RUNTIME_STATE.with(|state| start_expedition_impl(&mut state.borrow_mut()))
}

fn start_expedition_impl(runtime_state: &mut RuntimeState) -> Result<(), String> {
    runtime_state
        .data
        .business_state
        .propose_expedition(runtime_state.env.caller(), runtime_state.env.now())
}

#[candid_method(update, rename = "joinExpedition")]
#[update(name = "joinExpedition")]
fn join_expedition(expedition_id: u64) -> Result<(), String> {
    RUNTIME_STATE.with(|state| join_expedition_impl(&mut state.borrow_mut(), expedition_id))
}

fn join_expedition_impl(
    runtime_state: &mut RuntimeState,
    expedition_id: u64,
) -> Result<(), String> {
    runtime_state
        .data
        .business_state
        .join_expedition(&runtime_state.env.caller(), expedition_id)
}

#[candid_method(update, rename = "demoAddResourcesToExpedition")]
#[update(name = "demoAddResourcesToExpedition")]
fn demo_add_res() -> Result<(), String> {
    RUNTIME_STATE.with(|state| demo_add_res_impl(&mut state.borrow_mut()))
}

fn demo_add_res_impl(runtime_state: &mut RuntimeState) -> Result<(), String> {
    for (_, exp) in runtime_state.data.business_state.expeditions.iter_mut() {
        exp.add_resources(&HashMap::from([
            (Resources::Wood, 1000),
            (Resources::Stone, 1000),
            (Resources::Food, 1000),
            (Resources::Water, 1000),
        ]))?;
    }
    Ok(())
}

/// We can send arguments to the newly installed canister
#[derive(CandidType, Deserialize, Debug)]
struct CanisterInstallSendArgs {
    colony_state: ColonyState,
}

#[candid_method(update, rename = "expeditionNext")]
#[update(name = "expeditionNext")]
async fn expedition_next(expedition_id: u64) -> Result<(), String> {
    // RUNTIME_STATE.with(|state| expedition_next_impl(&mut state.borrow_mut(), expedition_id))

    let now = RUNTIME_STATE.with(|state| state.borrow().env.now());
    let self_canister_id = RUNTIME_STATE.with(|state| state.borrow().env.canister_id());

    let current_step = RUNTIME_STATE.with(|state| {
        state
            .borrow()
            .data
            .business_state
            .expeditions
            .get(&expedition_id)
            .expect("Can't find expedition")
            .clone()
            .get_step()
    });

    match current_step {
        ExpeditionStep::Proposed => {
            let success = RUNTIME_STATE.with(|state| {
                let mut s = state.borrow_mut();
                if s.data
                    .business_state
                    .expeditions
                    .get(&expedition_id)
                    .expect("Can't find expedition")
                    .has_enough_resources()
                {
                    s.data
                        .business_state
                        .expeditions
                        .get_mut(&expedition_id)
                        .expect("Can't find expedition")
                        .set_step(ExpeditionStep::Ready)
                        .unwrap();
                    true
                } else {
                    false
                }
            });

            if success {
                return Ok(());
            } else {
                return Err("Not enough resources to start the expedition".to_string());
            }
        }
        ExpeditionStep::Ready => {
            // First we set the step to starting, so we don't try to start the same expedition two times

            RUNTIME_STATE.with(|state| {
                state
                    .borrow_mut()
                    .data
                    .business_state
                    .expeditions
                    .get_mut(&expedition_id)
                    .expect("Can't find expedition")
                    .set_step(ExpeditionStep::Starting(now))
                    .unwrap();
            });

            // Async try to start the expedition
            let canister_id = call_canister_create(self_canister_id).await;

            ic_cdk::print(format!("Created canister {}", canister_id.to_string()));

            let canister_wasm =
                RUNTIME_STATE.with(|state| state.borrow().data.business_state.wasm_store.clone());

            let coffers = RUNTIME_STATE.with(|state| {
                state
                    .borrow()
                    .data
                    .business_state
                    .expeditions
                    .get(&expedition_id)
                    .expect("Can't find expedition")
                    .resources_pool
                    .clone()
            });

            let canister_install_args = Encode!(&CanisterInstallSendArgs {
                colony_state: ColonyState {
                    generation: 1,
                    taxes_percent: 5,
                    global_resources_multiplier: 1,
                    rewards_per_second: HashMap::from([
                        (Resources::Wood, 100),
                        (Resources::Stone, 100),
                        (Resources::Food, 100),
                        (Resources::Water, 100),
                        (Resources::Gold, 100),
                    ]),
                    coffers,
                },
            })
            .unwrap();

            let result =
                call_canister_install(&canister_id, canister_install_args, canister_wasm).await;

            if result {
                // If successful, we update the step again
                RUNTIME_STATE.with(|state| {
                    state
                        .borrow_mut()
                        .data
                        .business_state
                        .expeditions
                        .get_mut(&expedition_id)
                        .expect("Can't find expedition")
                        .set_step(ExpeditionStep::Started(canister_id))
                        .unwrap();
                });
            }

            return Ok(());

            // If unsuccessful, we update the step to "::Ready" so we can try again.
        }
        ExpeditionStep::Starting(_timestamp) => {
            // We can use the timestamp to implement some kind of timeout retry logic

            return Err("Retry logic not implemented".to_string());
        }
        ExpeditionStep::Started(canister_id) => {
            // A new colony has been started
            RUNTIME_STATE.with(|state| {
                state
                    .borrow_mut()
                    .data
                    .business_state
                    .remote_colonies
                    .push(canister_id)
            });

            RUNTIME_STATE.with(|state| {
                state
                    .borrow_mut()
                    .data
                    .business_state
                    .expeditions
                    .get_mut(&expedition_id)
                    .expect("Can't find expedition")
                    .set_step(ExpeditionStep::Done)
                    .unwrap();
            });
            return Ok(());
        }
        ExpeditionStep::Done => {
            return Err("This expedition cannot be changed anymore".to_string());
        }
    }

    // Err("[expedition_next_impl] This should be unreachable".to_string())
}

async fn call_canister_create(self_canister_id: Principal) -> Principal {
    ic_cdk::print("creating new colony...");

    #[derive(CandidType, Debug, Clone, Deserialize)]
    pub struct CreateCanisterSettings {
        pub controllers: Option<Vec<Principal>>,
        pub compute_allocation: Option<Nat>,
        pub memory_allocation: Option<Nat>,
        pub freezing_threshold: Option<Nat>,
    }

    #[derive(CandidType, Clone, Deserialize)]
    pub struct CreateCanisterArgs {
        pub cycles: u64,
        pub settings: CreateCanisterSettings,
    }

    #[derive(CandidType, Clone, Deserialize, Debug)]
    pub struct CanisterIdRecord {
        pub canister_id: Principal,
    }

    // Add your own principal as a controller, in case manual control is needed
    let create_args = CreateCanisterArgs {
        cycles: 100_000_000_000,
        settings: CreateCanisterSettings {
            controllers: Some(vec![self_canister_id]),
            compute_allocation: None,
            memory_allocation: None,
            freezing_threshold: None,
        },
    };

    #[derive(CandidType)]
    struct In {
        settings: Option<CreateCanisterSettings>,
    }

    let in_arg = In {
        settings: Some(create_args.settings),
    };

    let (create_result,): (CanisterIdRecord,) = match ic_cdk::api::call::call_with_payment(
        Principal::management_canister(),
        "create_canister",
        (in_arg,),
        create_args.cycles,
    )
    .await
    {
        Ok(x) => x,
        Err((code, msg)) => {
            ic_cdk::print(format!(
                "An error happened during the call: {}: {}",
                code as u8, msg
            ));

            (CanisterIdRecord {
                canister_id: Principal::anonymous(),
            },)
        }
    };

    ic_cdk::print(format!("{}", create_result.canister_id.to_text()));

    create_result.canister_id
}

async fn call_canister_install(
    canister_id: &Principal,
    canister_install_args: Vec<u8>,
    canister_wasm: Vec<u8>,
) -> bool {
    #[derive(CandidType, Deserialize)]
    enum InstallMode {
        #[serde(rename = "install")]
        Install,
        #[serde(rename = "reinstall")]
        Reinstall,
        #[serde(rename = "upgrade")]
        Upgrade,
    }

    #[derive(CandidType, Deserialize)]
    struct CanisterInstall {
        mode: InstallMode,
        canister_id: Principal,
        #[serde(with = "serde_bytes")]
        wasm_module: Vec<u8>,
        #[serde(with = "serde_bytes")]
        arg: Vec<u8>,
    }

    let install_config: CanisterInstall = CanisterInstall {
        mode: InstallMode::Install,
        canister_id: canister_id.clone(),
        wasm_module: canister_wasm,
        arg: canister_install_args,
    };

    match ic_cdk::api::call::call(
        Principal::management_canister(),
        "install_code",
        (install_config,),
    )
    .await
    {
        Ok(x) => x,
        Err((code, msg)) => {
            ic_cdk::print(format!(
                "An error happened during the call: {}: {}",
                code as u8, msg
            ));
            return false;
        }
    };

    true
}

#[candid_method(query, rename = "isPlayerHere")]
#[query(name = "isPlayerHere")]
fn is_player_here() -> bool {
    RUNTIME_STATE.with(|state| is_player_here_impl(state.borrow()))
}

fn is_player_here_impl(runtime_state: Ref<RuntimeState>) -> bool {
    runtime_state
        .data
        .business_state
        .is_player_in_world(runtime_state.env.caller())
}

#[candid_method(update, rename = "addPlayerToWorld")]
#[update(name = "addPlayerToWorld")]
fn add_player_to_world() -> Result<(), String> {
    RUNTIME_STATE.with(|state| add_player_to_world_impl(&mut state.borrow_mut()))
}

fn add_player_to_world_impl(runtime_state: &mut RuntimeState) -> Result<(), String> {
    runtime_state
        .data
        .business_state
        .add_player(runtime_state.env.caller())
}

#[candid_method(query, rename = "getRemoteColonies")]
#[query(name = "getRemoteColonies")]
fn get_remote_colonies() -> Vec<Principal> {
    RUNTIME_STATE.with(|state| get_remote_colonies_impl(state.borrow()))
}

fn get_remote_colonies_impl(runtime_state: Ref<RuntimeState>) -> Vec<Principal> {
    runtime_state.data.business_state.remote_colonies.clone()
}

#[candid_method(query)]
#[query]
fn greet(name: String) -> String {
    let now = RUNTIME_STATE.with(|state| state.borrow().env.now());
    let identity = RUNTIME_STATE.with(|state| state.borrow().env.caller());
    format!(
        "Hello, {}! Identity: {}. Timestamp: {}",
        name, identity, now
    )
}

#[update(name = "load_wasm")]
fn load_wasm(wasm: Vec<u8>) -> bool {
    ic_cdk::print(format!("Loaded wasm with length: {}", &wasm.len()));

    RUNTIME_STATE.with(|state| state.borrow_mut().data.business_state.wasm_store = wasm);

    true
}

#[candid_method(query, rename = "wasm_sha256")]
#[query(name = "wasm_sha256")]
fn wasm_sha256() -> String {
    RUNTIME_STATE.with(|state| {
        let mut hasher = Sha256::new();
        hasher.update(&state.borrow_mut().data.business_state.wasm_store);
        let result = hasher.finalize();

        format!("{:x}", result)
    })
}

// Auto export the candid interface
candid::export_service!();

#[query(name = "__get_candid_interface_tmp_hack")]
#[candid_method(query, rename = "__get_candid_interface_tmp_hack")]
fn __export_did_tmp_() -> String {
    __export_service()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_candid() {
        let expected =
            String::from_utf8(std::fs::read("ic_quickstart_actor_model.did").unwrap()).unwrap();

        let actual = __export_service();

        if actual != expected {
            println!("{}", actual);
        }

        assert_eq!(
            actual, expected,
            "Generated candid definition does not match expected did file"
        );
    }
}
