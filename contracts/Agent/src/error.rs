use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},
    
    #[error("ContractIsPaused")]
    ContractIsPaused {},

    #[error("No data in ReceiveMsg")]
    NoData {},

    #[error("the nft token contract must have been registered")]
    InvalidNftAddress
}
