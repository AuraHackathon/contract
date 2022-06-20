use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub base_token_uri: String,
    pub max_tokens: u32,
    pub max_tokens_per_batch_mint: u32,
    pub max_tokens_per_batch_transfer: u32,
    pub cw721_code_id: u64,
    pub cw721_address: Option<Addr>,
    pub rand_address: Option<Addr>,
    pub name: String,
    pub symbol: String,
    pub house_infos: Vec<HouseInfo>,
    pub rarities: Vec<Vec<u128>>,
    pub aliases: Vec<Vec<u128>>,
    pub house_max_tokens: u128,
    pub house_paid_tokens: u128,
    pub house_minted: u128,
    pub house_cost_mint: u128,
    pub building_max_tokens: u128,
    pub building_paid_tokens: u128,
    pub building_minted: u128,
    pub building_cost_mint: u128,
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum Model {
    TREEHOUSE,
    TRAILERHOUSE,
    CABIN,
    ONESTORYHOUSE,
    TWOSTORYHOUSE,
    MANSION,
    PALACE,
    UTILITYBUILDING,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RandomData {
    pub seed: u8,
    pub entropy: u8,
    pub round: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const MINTABLE_TOKEN_IDS: Map<u32, bool> = Map::new("mintable_ids");
pub const MINTABLE_NUM_TOKENS: Item<u32> = Item::new("mintable_num_tokens");
pub const CW721_ADDRESS: Item<Addr> = Item::new("cw721_address");
pub const HOUSEBUILDING: Item<HouseBuilding> = Item::new("house_building");
