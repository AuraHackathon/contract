pub mod contract;
mod error;
pub mod msg;
pub mod state;

#[cfg(test)]
mod contract_tests;

pub use cw721_base::ContractError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// pub use crate::error::ContractError;

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct Trait {
    pub display_type: Option<String>,
    pub trait_type: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct Metadata {
   pub extension: Option<String>
}

pub type Extension = Option<Metadata>;
