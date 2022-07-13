use cosmwasm_std::{Uint128, Addr};
use cw20::Expiration;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::state::{ TypeNFT };
use cw721::Cw721ReceiveMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub tax_rate: u8,
    pub amount_tax: Uint128,
    pub unaccounted_reward: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    ReceiveNft (Cw721ReceiveMsg),
    Claim {token_id: String, unstake: bool},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw721Hook{
    StakeHouse {},
    StakeBuiliding {}
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct StakeNFT {
    pub type_nft: TypeNFT
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
    GetConfig {},
    GetState {},
    GetStakedInfo {account: String},
    GetStakedAccountInfo {account: String},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: Addr,
    pub cw721_addr: Addr,
    pub cw20_addr: Addr,
    pub minter: Addr,
    pub rand_addr: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StateResponse { 
    pub tax_rate: u8,
    pub amount_tax: Uint128,
    pub unaccounted_reward: Uint128,
    pub total_building_staked: u128
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StakedInfoResponse {
    pub token_id: String,
    pub owner: String,
    pub value: Uint128,
    pub type_nft: TypeNFT,
    pub ternant_rating: u8
}



