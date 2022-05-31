use std::{collections::HashMap, iter::FromIterator};

use candid::{CandidType, Deserialize, Principal};

use crate::env::{TimestampMillis, MILLIS_TO_SECONDS};

#[derive(CandidType, Deserialize, Debug, Default)]
pub struct BusinessState {
    pub colony: ColonyState,
    pub player: HashMap<Principal, PlayerState>,
    pub expeditions: HashMap<u64, ExpeditionState>,
    pub expeditions_count: u64,
    pub remote_colonies: Vec<Principal>,
    pub wasm_store: Vec<u8>,
}

#[derive(CandidType, Deserialize, Debug, Default)]
pub struct SystemSettings {}

#[derive(CandidType, Deserialize, Debug)]
pub struct ColonyState {
    /// Starts at 0 for the first colony and is incremented by 1 for each
    /// successful expedition that becomes a colony
    pub(crate) generation: u8,
    /// Taxes are being applied at the customs office once a player enters
    /// a colony. Each resource carried by the player is taxed with this rate.
    pub(crate) taxes_percent: u8,
    /// Easy way to create x10 or x100 colonies for testing
    pub(crate) global_resources_multiplier: u16,
    /// Each colony defines a rate at which resources are rewarded if a player
    /// works for that colony.
    pub(crate) rewards_per_second: HashMap<Resources, u8>,
    /// Taxes go here
    pub(crate) coffers: Inventory,
}

impl Default for ColonyState {
    fn default() -> Self {
        Self {
            generation: 0,
            taxes_percent: 10,
            global_resources_multiplier: 1,
            rewards_per_second: HashMap::from([
                (Resources::Wood, 10),
                (Resources::Stone, 10),
                (Resources::Food, 10),
                (Resources::Water, 10),
            ]),
            coffers: Default::default(),
        }
    }
}

#[derive(CandidType, Deserialize, Debug, Default, Clone)]
pub struct Inventory {
    /// Could be used to decide if resources can be added to the inventory (e.g. a Player can
    /// only carry so much resources at a time, or a colony needs to be upgraded to hold more resources)
    size: u32,
    contents: HashMap<Resources, u64>,
}

#[allow(dead_code)]
impl Inventory {
    pub fn has_available_resources(&self, required: &HashMap<Resources, u64>) -> bool {
        for (res, val) in required.iter() {
            if self.contents.get(res).lt(&Some(val)) {
                return false;
            }
        }
        true
    }

    fn subtract_resources(&mut self, requirements: &HashMap<Resources, u64>) {
        for (res, val) in requirements.iter() {
            self.contents.entry(*res).and_modify(|v| *v -= val);
        }
    }

    fn add_resources(&mut self, requirements: &HashMap<Resources, u64>) {
        for (res, val) in requirements.iter() {
            self.contents
                .entry(*res)
                .and_modify(|v| *v += val)
                .or_insert(*val);
        }
    }

    fn get_all(&self) -> Vec<(Resources, u64)> {
        self.contents
            .iter()
            .map(|(res, val)| (res.clone(), val.clone()))
            .collect()
    }

    fn get(&self, res: Resources) -> u64 {
        self.contents.get(&res).unwrap_or(&0).clone()
    }
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct ExpeditionState {
    id: u64,
    step: ExpeditionStep,
    proposed_by: Principal,
    proposed_at: TimestampMillis,
    resources_required: HashMap<Resources, u64>,
    pub(crate) resources_pool: Inventory,
    pub(crate) members: Vec<Principal>,
}

impl Default for ExpeditionState {
    fn default() -> Self {
        Self {
            id: Default::default(),
            step: ExpeditionStep::Proposed,
            proposed_by: Principal::anonymous(),
            proposed_at: Default::default(),
            resources_required: Default::default(),
            resources_pool: Default::default(),
            members: Default::default(),
        }
    }
}

impl ExpeditionState {
    pub fn add_resources(&mut self, resources: &HashMap<Resources, u64>) -> Result<(), String> {
        for (res, val) in resources.iter() {
            self.resources_pool
                .contents
                .entry(*res)
                .and_modify(|v| *v += val);
        }

        Ok(())
    }

    pub fn set_step(&mut self, step: ExpeditionStep) -> Result<(), String> {
        self.step = step;

        Ok(())
    }

    pub fn get_step(self) -> ExpeditionStep {
        self.step
    }

    pub fn has_enough_resources(&self) -> bool {
        let mut required = self.resources_required.clone();

        // TODO: this multiplier should be stored somewhere in the world state
        for (_, val) in required.iter_mut() {
            *val *= 10;
        }

        self.resources_pool.has_available_resources(&required)
    }
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub enum ExpeditionStep {
    /// This is the default state of an expedition. In this state we wait until the conditions
    /// are met. Players can join the expedition in this step.
    Proposed,
    /// This state indicates tha the conditions for the expedition have been met, and we are ready to
    /// start the expedition. Players cannot join the expedition at this point.
    Ready,
    /// The async process of starting a new expedition has started at "timestamp". We can later implement
    /// some retry logic based on the timestamp.
    Starting(TimestampMillis),
    /// The new expedition was started, a new world has been spawned and we got confirmation that
    /// the new world is ready.
    Started(Principal),
    /// The end of an expedition's lifecycle. We can hold on to the expedition as a log of sorts
    /// but for all intents and purposes this is a finished task.
    Done,
}

#[derive(CandidType, Deserialize, Debug, Hash, PartialEq, Eq, Copy, Clone, Ord, PartialOrd)]
pub enum Resources {
    Wood,
    Stone,
    /// Only available in colonies of generation > 0
    Gold,
    Food,
    Water,
}

#[derive(CandidType, Deserialize, Debug, Default, Clone)]
pub struct PlayerState {
    status: PlayerStatus,
    inventory: Inventory,
}

#[allow(dead_code)]
impl PlayerState {
    pub fn get_status(&self) -> PlayerStatus {
        self.status.clone()
    }

    pub fn set_status(&mut self, status: PlayerStatus) {
        self.status = status
    }

    pub fn get_inventory(&self) -> Inventory {
        self.inventory.clone()
    }
}

#[derive(CandidType, Deserialize, Debug, PartialEq, Clone)]
pub enum PlayerStatus {
    Idle,
    WorkingAll(TimestampMillis),
    WorkingFocused(TimestampMillis, Resources),
    Traveling,
}

impl Default for PlayerStatus {
    fn default() -> Self {
        PlayerStatus::Idle
    }
}

#[allow(dead_code)]
impl BusinessState {
    pub fn is_player_in_world(&self, principal: Principal) -> bool {
        if let Some(_) = self.player.get(&principal) {
            true
        } else {
            false
        }
    }
    pub fn work_set(
        &mut self,
        principal: Principal,
        focus: Option<Resources>,
        now: TimestampMillis,
    ) -> Result<(), String> {
        let mut p = self
            .player
            .get_mut(&principal)
            .expect("Principal not found");

        if let Some(res) = focus {
            p.status = PlayerStatus::WorkingFocused(now, res);
        } else {
            p.status = PlayerStatus::WorkingAll(now);
        }

        Ok(())
    }

    pub fn work_claim(&mut self, principal: Principal, now: TimestampMillis) -> Result<(), String> {
        let seconds_elapsed;

        match self
            .player
            .get(&principal)
            .expect("Principal not found")
            .status
        {
            PlayerStatus::WorkingAll(working_since)
            | PlayerStatus::WorkingFocused(working_since, _) => {
                seconds_elapsed = (now - working_since) / MILLIS_TO_SECONDS
            }
            _ => return Err("The player is not currently working".to_string()),
        }

        let available = self.available_unclaimed(seconds_elapsed);

        let p = self
            .player
            .get_mut(&principal)
            .expect("Principal not found");

        for (res, val) in available {
            *p.inventory.contents.entry(res).or_insert(0) += val;
        }

        p.status = PlayerStatus::Idle;

        Ok(())
    }

    pub fn available_unclaimed(&self, seconds_elapsed: TimestampMillis) -> Vec<(Resources, u64)> {
        self.colony
            .rewards_per_second
            .iter()
            .map(|(res, val)| (res.clone(), val.clone() as u64 * seconds_elapsed))
            .collect::<Vec<(Resources, u64)>>()
    }

    pub fn propose_expedition(
        &mut self,
        principal: Principal,
        now: TimestampMillis,
    ) -> Result<(), String> {
        let p = self
            .player
            .get_mut(&principal)
            .expect("Principal not found");

        let requirements = HashMap::from([
            (Resources::Wood, 60),
            (Resources::Stone, 60),
            (Resources::Food, 60),
            (Resources::Water, 60),
        ]);

        if p.inventory.has_available_resources(&requirements) {
            p.inventory.subtract_resources(&requirements);
        } else {
            return Err(
                "The player doesn't have enough resources to propose an expedition".to_string(),
            );
        }

        let id = self.expeditions_count;

        let proposed = ExpeditionState {
            step: ExpeditionStep::Proposed,
            proposed_by: principal,
            proposed_at: now,
            resources_required: requirements.clone(),
            resources_pool: Inventory {
                size: 0,
                contents: requirements,
            },
            members: Vec::from([principal]),
            id: self.expeditions_count,
        };

        self.expeditions.insert(id, proposed);
        self.expeditions_count += 1;

        Ok(())
    }

    pub fn join_expedition(
        &mut self,
        principal: &Principal,
        expedition_id: u64,
    ) -> Result<(), String> {
        let p = self
            .player
            .get_mut(&principal)
            .expect("Principal not found");

        if let false = self.expeditions.contains_key(&expedition_id) {
            return Err("Can't find expedition".to_string());
        }

        let entry = self
            .expeditions
            .get_mut(&expedition_id)
            .expect("Can't find expedition");
        let requirements = entry.resources_required.clone();

        if entry.members.contains(principal) {
            return Err("The player is already a member of this expedition".to_string());
        }

        // subtract resources from the player
        if p.inventory.has_available_resources(&requirements) {
            p.inventory.subtract_resources(&requirements);
        } else {
            return Err(
                "The player doesn't have enough resources to propose an expedition".to_string(),
            );
        }

        entry.add_resources(&requirements)?;
        entry.members.push(*principal);

        Ok(())
    }

    pub fn add_player(&mut self, principal: Principal) -> Result<(), String> {
        if self.player.contains_key(&principal) {
            return Err("The player already exists in this world".to_string());
        }

        self.player.insert(principal, PlayerState::default());

        Ok(())
    }

    /// Add a player that is traveling from another world.
    pub fn add_traveler(
        &mut self,
        principal: Principal,
        player_state: PlayerState,
    ) -> Result<(), String> {
        if self.player.contains_key(&principal) {
            return Err("The player already exists in this world".to_string());
        }

        // Customs office: apply this colony's taxes percent to the player's inventory
        let percentage = self.colony.taxes_percent;

        let taxed_inventory: Vec<(Resources, u64)> = player_state
            .inventory
            .contents
            .iter()
            .map(|(k, v)| (k.clone(), v.clone() - (v * percentage as u64 / 100)))
            .collect();

        self.player.insert(
            principal,
            PlayerState {
                inventory: Inventory {
                    size: 0,
                    contents: HashMap::from_iter(taxed_inventory),
                },
                ..player_state
            },
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_work() -> Result<(), String> {
        let mut business_state = BusinessState::default();

        let user1: Principal = Principal::from_slice(&[1]);

        business_state.player.insert(user1, PlayerState::default());

        assert_eq!(
            business_state.player.get(&user1).unwrap().status,
            PlayerStatus::Idle
        );

        business_state.work_set(user1, None, 1)?;

        assert_eq!(
            business_state.player.get(&user1).unwrap().status,
            PlayerStatus::WorkingAll(1)
        );

        Ok(())
    }

    #[test]
    fn test_available_unclaimed() -> Result<(), String> {
        let mut business_state = BusinessState::default();

        let user1: Principal = Principal::from_slice(&[1]);

        business_state.player.insert(user1, PlayerState::default());

        business_state.work_set(user1, None, 1)?;

        let mut res = business_state.available_unclaimed(5);

        assert_eq!(res.len(), 4);
        // println!("{:?}", res);
        assert_eq!(
            res.sort(),
            [
                (Resources::Stone, 5),
                (Resources::Water, 5),
                (Resources::Food, 5),
                (Resources::Wood, 5)
            ]
            .sort()
        );

        Ok(())
    }

    #[test]
    fn test_work_claim() -> Result<(), String> {
        let mut business_state = BusinessState::default();

        let user1: Principal = Principal::from_slice(&[1]);

        business_state.player.insert(user1, PlayerState::default());

        business_state.work_set(user1, None, 1)?;

        assert_eq!(
            business_state
                .player
                .get(&user1)
                .unwrap()
                .inventory
                .contents
                .len(),
            0
        );

        business_state.work_claim(user1, MILLIS_TO_SECONDS * 6 + 1)?;

        assert_eq!(
            business_state
                .player
                .get(&user1)
                .unwrap()
                .inventory
                .contents
                .len(),
            4
        );

        println!("{:?}", business_state.player.get(&user1).unwrap().inventory);

        business_state
            .player
            .get_mut(&user1)
            .unwrap()
            .set_status(PlayerStatus::Idle);

        assert_eq!(
            business_state.work_claim(user1, MILLIS_TO_SECONDS * 6 + 1),
            Err("The player is not currently working".to_string())
        );

        Ok(())
    }

    #[test]
    fn test_player_has_resources() -> Result<(), String> {
        let mut business_state = BusinessState::default();

        let user1: Principal = Principal::from_slice(&[1]);

        business_state.player.insert(user1, PlayerState::default());

        business_state.work_set(user1, None, 1)?;

        assert_eq!(
            business_state
                .player
                .get(&user1)
                .unwrap()
                .inventory
                .contents
                .len(),
            0
        );

        business_state.work_claim(user1, MILLIS_TO_SECONDS * 60 + 1)?;

        assert_eq!(
            business_state
                .player
                .get(&user1)
                .unwrap()
                .inventory
                .has_available_resources(&HashMap::from([(Resources::Wood, 100)])),
            true
        );

        assert_eq!(
            business_state
                .player
                .get(&user1)
                .unwrap()
                .inventory
                .has_available_resources(&HashMap::from([
                    (Resources::Wood, 600),
                    (Resources::Stone, 600),
                    (Resources::Food, 600),
                    (Resources::Water, 600),
                ])),
            true
        );

        assert_eq!(
            business_state
                .player
                .get(&user1)
                .unwrap()
                .inventory
                .has_available_resources(&HashMap::from([(Resources::Wood, 700)])),
            false
        );

        assert_eq!(
            business_state
                .player
                .get(&user1)
                .unwrap()
                .inventory
                .has_available_resources(&HashMap::from([
                    (Resources::Wood, 600),
                    (Resources::Stone, 700),
                    (Resources::Food, 600),
                    (Resources::Water, 600),
                ])),
            false
        );
        Ok(())
    }

    #[test]
    fn test_player_remove_resources() -> Result<(), String> {
        let mut business_state = BusinessState::default();

        let user1: Principal = Principal::from_slice(&[1]);

        business_state.player.insert(user1, PlayerState::default());

        business_state.work_set(user1, None, 1)?;

        assert_eq!(
            business_state
                .player
                .get(&user1)
                .unwrap()
                .inventory
                .contents
                .len(),
            0
        );

        business_state.work_claim(user1, MILLIS_TO_SECONDS * 60 + 1)?;

        business_state
            .player
            .get_mut(&user1)
            .unwrap()
            .inventory
            .subtract_resources(&HashMap::from([
                (Resources::Wood, 300),
                (Resources::Stone, 300),
                (Resources::Food, 300),
                (Resources::Water, 300),
            ]));

        assert_eq!(
            business_state
                .player
                .get(&user1)
                .unwrap()
                .inventory
                .has_available_resources(&HashMap::from([(Resources::Wood, 100)])),
            true
        );

        assert_eq!(
            business_state
                .player
                .get(&user1)
                .unwrap()
                .inventory
                .has_available_resources(&HashMap::from([(Resources::Wood, 350)])),
            false
        );

        assert_eq!(
            business_state
                .player
                .get(&user1)
                .unwrap()
                .inventory
                .has_available_resources(&HashMap::from([
                    (Resources::Wood, 300),
                    (Resources::Stone, 300),
                    (Resources::Food, 300),
                    (Resources::Water, 300),
                ])),
            true
        );

        assert_eq!(
            business_state
                .player
                .get(&user1)
                .unwrap()
                .inventory
                .has_available_resources(&HashMap::from([
                    (Resources::Wood, 600),
                    (Resources::Stone, 700),
                    (Resources::Food, 600),
                    (Resources::Water, 600),
                ])),
            false
        );

        Ok(())
    }

    #[test]
    fn test_propose_expedition() -> Result<(), String> {
        let mut business_state = BusinessState::default();

        let user1: Principal = Principal::from_slice(&[1]);

        business_state.player.insert(
            user1,
            PlayerState {
                status: PlayerStatus::Idle,
                inventory: Inventory {
                    size: 0,
                    contents: HashMap::from([
                        (Resources::Wood, 100),
                        (Resources::Stone, 100),
                        (Resources::Food, 100),
                        (Resources::Water, 100),
                    ]),
                },
            },
        );

        business_state.propose_expedition(user1, 2)?;

        assert_eq!(business_state.expeditions.len(), 1);

        assert_eq!(
            business_state.propose_expedition(user1, 2),
            Err("The player doesn't have enough resources to propose an expedition".to_string())
        );

        assert_eq!(business_state.expeditions.len(), 1);

        Ok(())
    }

    #[test]
    fn test_join_expedition() -> Result<(), String> {
        let mut business_state = BusinessState::default();

        let user1: Principal = Principal::from_slice(&[1]);

        business_state.player.insert(
            user1,
            PlayerState {
                status: PlayerStatus::Idle,
                inventory: Inventory {
                    size: 0,
                    contents: HashMap::from([
                        (Resources::Wood, 100),
                        (Resources::Stone, 100),
                        (Resources::Food, 100),
                        (Resources::Water, 100),
                    ]),
                },
            },
        );

        let user2: Principal = Principal::from_slice(&[2]);

        business_state.player.insert(
            user2,
            PlayerState {
                status: PlayerStatus::Idle,
                inventory: Inventory {
                    size: 0,
                    contents: HashMap::from([
                        (Resources::Wood, 100),
                        (Resources::Stone, 100),
                        (Resources::Food, 100),
                        (Resources::Water, 100),
                    ]),
                },
            },
        );

        business_state.propose_expedition(user1, 2)?;

        assert_eq!(business_state.expeditions.get(&0).unwrap().members.len(), 1);

        business_state.join_expedition(&user2, 0)?;

        assert_eq!(business_state.expeditions.get(&0).unwrap().members.len(), 2);

        Ok(())
    }

    #[test]
    fn test_expedition_has_enough_resources() -> Result<(), String> {
        let mut business_state = BusinessState::default();

        let user1: Principal = Principal::from_slice(&[1]);

        business_state.player.insert(
            user1,
            PlayerState {
                status: PlayerStatus::Idle,
                inventory: Inventory {
                    size: 0,
                    contents: HashMap::from([
                        (Resources::Wood, 100),
                        (Resources::Stone, 100),
                        (Resources::Food, 100),
                        (Resources::Water, 100),
                    ]),
                },
            },
        );

        business_state.propose_expedition(user1, 2)?;

        assert_eq!(
            business_state
                .expeditions
                .get(&0)
                .unwrap()
                .has_enough_resources(),
            false
        );

        business_state
            .expeditions
            .get_mut(&0)
            .unwrap()
            .add_resources(&HashMap::from([
                (Resources::Wood, 1000),
                (Resources::Stone, 1000),
                (Resources::Food, 1000),
                (Resources::Water, 1000),
            ]))?;

        assert_eq!(
            business_state
                .expeditions
                .get(&0)
                .unwrap()
                .has_enough_resources(),
            true
        );

        Ok(())
    }

    #[test]
    fn test_add_player() -> Result<(), String> {
        let mut business_state = BusinessState::default();

        let user1: Principal = Principal::from_slice(&[1]);

        assert_eq!(business_state.player.contains_key(&user1), false);

        business_state.add_player(user1)?;

        assert_eq!(business_state.player.contains_key(&user1), true);

        Ok(())
    }

    #[test]
    fn test_is_player_in_world() -> Result<(), String> {
        let mut business_state = BusinessState::default();
        let user1: Principal = Principal::from_slice(&[1]);

        assert_eq!(business_state.is_player_in_world(user1), false);

        business_state.add_player(user1)?;

        assert_eq!(business_state.is_player_in_world(user1), true);

        Ok(())
    }

    #[test]
    fn test_inventory_getters() -> Result<(), String> {
        let player_state = PlayerState {
            status: PlayerStatus::Idle,
            inventory: Inventory {
                size: 0,
                contents: HashMap::from([
                    (Resources::Wood, 100),
                    (Resources::Stone, 100),
                    (Resources::Food, 100),
                    (Resources::Water, 100),
                ]),
            },
        };

        let mut a = player_state.inventory.get_all();
        let mut b = vec![
            (Resources::Wood, 100),
            (Resources::Stone, 100),
            (Resources::Food, 100),
            (Resources::Water, 100),
        ];
        a.sort();
        b.sort();
        assert_eq!(a, b);

        assert_eq!(player_state.inventory.get(Resources::Wood), 100);

        Ok(())
    }

    #[test]
    fn test_tax_inbound_inventory() -> Result<(), String> {
        let mut business_state = BusinessState::default();

        let user1: Principal = Principal::from_slice(&[1]);
        let player_state = PlayerState {
            status: PlayerStatus::Idle,
            inventory: Inventory {
                size: 0,
                contents: HashMap::from([
                    (Resources::Wood, 100),
                    (Resources::Stone, 100),
                    (Resources::Food, 100),
                    (Resources::Water, 100),
                ]),
            },
        };

        assert_eq!(business_state.player.contains_key(&user1), false);

        business_state.add_traveler(user1, player_state)?;

        assert_eq!(business_state.player.contains_key(&user1), true);

        assert_eq!(
            business_state
                .player
                .get(&user1)
                .unwrap()
                .inventory
                .has_available_resources(&HashMap::from([
                    (Resources::Wood, 90),
                    (Resources::Stone, 90),
                    (Resources::Food, 90),
                    (Resources::Water, 90),
                ])),
            true
        );

        assert_eq!(
            business_state
                .player
                .get(&user1)
                .unwrap()
                .inventory
                .has_available_resources(&HashMap::from([
                    (Resources::Wood, 91),
                    (Resources::Stone, 91),
                    (Resources::Food, 91),
                    (Resources::Water, 91),
                ])),
            false
        );

        Ok(())
    }
}
