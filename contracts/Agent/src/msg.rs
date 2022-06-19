use cosmwasm_std::{Timestamp, Addr};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::state::{Histories,DestroyThreshold};
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub rand_addr: String,
    pub cw20_addr: String,
    pub cw721_address: String,
    pub start_time: Timestamp,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Stake {amount: u128},
    Unstake {amount: u128},
    Claim {},
    Withdraw {},
    SetStartTime {new_start_time: Timestamp}
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
    GetStakedInfo {account: Addr},
    GetTimeStart {},
    GetDestroyRate { account: Addr},
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StakedInfoResponse {
    pub total_staked_amount: u128,
    pub pending_rewards: u128,
    pub last_claim: Timestamp,
    pub histories: Vec<Histories>,
    pub destroy_threshold: DestroyThreshold,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TimeResponse {
    pub time: Timestamp,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RateResponse {
    pub rate: u64,
}

