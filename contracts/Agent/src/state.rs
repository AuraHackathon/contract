use nft::state::Model;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, Uint128};
use cw_storage_plus::{ Item, Map};

const CONFIG_KEY: &str = "config";
const STATE_KEY: &str = "state";
const STAKED_ACCOUNT_INFOS_KEY: &str = "staked_account_infos";
const STAKED_INFOS_KEY: &str = "staked_infos";
const TOKEN_LAST_CLAIM_KEY: &str = "token_last_claim";


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
    pub minter: Option<CanonicalAddr>,
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
    pub total_building_staked: u128
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct HouseBuilding {
    pub is_house: bool,
    pub model: u8,
    pub image_id: u128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct HouseInfo {
    pub model: Model,
    pub income_per_day: u128,
    pub property_damage: u128,
}

pub const CONFIG: Item<Config> = Item::new(CONFIG_KEY);
pub const STATE: Item<State> = Item::new(STATE_KEY);
pub const STAKED_ACCOUNT_INFOS: Map<&[u8], Vec<StakedAccountInfo>> = Map::new(STAKED_ACCOUNT_INFOS_KEY);
pub const STAKED_INFOS: Map<&str, StakedAccountInfo> = Map::new(STAKED_INFOS_KEY);
pub const TOKEN_LAST_CLAIMED: Map<&str, u64> = Map::new(TOKEN_LAST_CLAIM_KEY);