#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Addr, Timestamp};
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{StakedInfoResponse, TimeResponse, RateResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE, STAKED_ACCOUNT_INFOS, THREHOLD, DestroyThreshold, StakedAccountInfo};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:renting-house";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        rand_addr: msg.rand_addr.clone(),                                                         
        cw721_address: msg.cw721_address.clone(),
        cw20_addr: msg.cw20_addr.clone(),
        owner: info.sender.clone(),
        start_time: msg.start_time.clone(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("cw721", msg.cw721_address.to_string())
        .add_attribute("cw20", msg.cw20_addr.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Stake {amount} => handle_stake(deps, amount),
        ExecuteMsg::Unstake {amount} => handle_unstake(deps, amount),
        ExecuteMsg::Claim {} => handle_claim(deps),
        ExecuteMsg::Withdraw {} => handle_withdraw(deps), 
        ExecuteMsg::SetStartTime { new_start_time} => handle_set_start_time(deps, new_start_time), 
    }
}

pub fn handle_stake(deps: DepsMut, amount: u128) -> Result<Response, ContractError> {
    // check agent pool is start ?
    // check sender have a NFT ?
    // transferFrom NFT for lock
    //                                                                                                                                                                                                                                                                                                                                                                                              
 

    Ok(Response::new().add_attribute("method", "stake"))
}
pub fn handle_unstake(deps: DepsMut, amount: u128) -> Result<Response, ContractError> {

    Ok(Response::new().add_attribute("method", "unstake"))
}


pub fn handle_claim(deps: DepsMut) -> Result<Response, ContractError> {

    Ok(Response::new().add_attribute("method", "claim"))
}

pub fn handle_withdraw(deps: DepsMut) -> Result<Response, ContractError> {
    // STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
    //     state.count += 1;
    //     Ok(state)
    // })?;

    Ok(Response::new().add_attribute("method", "withdraw"))
}

pub fn handle_set_start_time(deps: DepsMut, new_start_time: Timestamp) -> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        state.start_time = new_start_time;
        Ok(state)
    })?;
    Ok(Response::new().add_attribute("method", "set_start_time"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetStakedInfo {account} => to_binary(&query_staked_info(deps, account)?),
        QueryMsg::GetTimeStart {} => to_binary(&query_start_time(deps)?),
        QueryMsg::GetDestroyRate {account } => to_binary(&query_destroy_rate(deps, account)?),
    }
}

fn query_staked_info(deps: Deps, account: Addr) -> StdResult<StakedInfoResponse> {
    let account_info  = STAKED_ACCOUNT_INFOS.load(deps.storage, &account)?;
    Ok(StakedInfoResponse { 
        total_staked_amount: account_info.total_staked_amount,
        pending_rewards: account_info.pending_rewards, 
        last_claim: account_info.last_claim, 
        histories: account_info.histories, 
        destroy_threshold: account_info.destroy_threshold 
    })
}

fn query_start_time(deps: Deps) -> StdResult<TimeResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(TimeResponse { time: state.start_time })
}

fn query_destroy_rate(deps: Deps, account: Addr) -> StdResult<RateResponse> {
    let rate = THREHOLD.load(deps.storage, &account)?;
    Ok(RateResponse { rate })
}
