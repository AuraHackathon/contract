use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, Uint128, Addr};
use cw_storage_plus::{ Item, Map};

const CONFIG_KEY: &str = "config";
const STATE_KEY: &str = "state";
const STAKED_ACCOUNT_INFOS_KEY: &str = "staked_account_infos";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum TypeNFT {
    House,
    Building
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: CanonicalAddr,
    pub cw721_addr: Option<CanonicalAddr>,
    pub cw20_addr: Option<CanonicalAddr>,
    pub rand_addr: Option<CanonicalAddr>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]

pub struct StakedAccountInfo {
    pub token_id: String,
    pub owner: CanonicalAddr,
    pub value: Uint128,
    pub type_nft: TypeNFT,
    pub ternant_rating: u8
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub tax_rate: u8,
    pub amount_tax: Uint128,
    pub unaccounted_reward: Uint128,
    pub total_building_stake: u16
}

pub const CONFIG: Item<Config> = Item::new(CONFIG_KEY);
pub const STATE: Item<State> = Item::new(STATE_KEY);
pub const STAKED_ACCOUNT_INFOS: Map<&[u8], Vec<StakedAccountInfo>> = Map::new(STAKED_ACCOUNT_INFOS_KEY);
