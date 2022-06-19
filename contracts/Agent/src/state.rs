use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Timestamp};
use cw_storage_plus::{Item, Map};

const STATE_KEY: &str = "state";
const STAKED_ACCOUNT_INFOS_KEY: &str = "staked_account_infos";
const THRESH_HOLD_KEY: &str = "threshold";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum DestroyThreshold {
    LightAir,
    StrongBreeze,
    Hurricane,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Histories {
    pub action: String,
    pub amount: u128,
    pub time: Timestamp,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StakedAccountInfo {
    pub total_staked_amount: u128,
    pub pending_rewards: u128,
    pub last_claim: Timestamp,
    pub histories: Vec<Histories>,
    pub destroy_threshold: DestroyThreshold,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub rand_addr: String,
    pub cw20_addr: String,
    pub cw721_address: String,
    pub owner: Addr,
    pub start_time: Timestamp,
}

pub const STATE: Item<State> = Item::new(STATE_KEY);
pub const STAKED_ACCOUNT_INFOS: Map<&Addr, StakedAccountInfo> = Map::new(STAKED_ACCOUNT_INFOS_KEY);
pub const THREHOLD: Map<&Addr, u64> = Map::new(THRESH_HOLD_KEY);