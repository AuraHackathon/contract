#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{ to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, from_binary, Uint128, StdError};

use crate::error::ContractError;
use crate::msg::{StakedInfoResponse, ExecuteMsg, InstantiateMsg, QueryMsg, Cw721Hook};
use crate::state::{State, Config, CONFIG, STATE, STAKED_ACCOUNT_INFOS, TypeNFT, StakedAccountInfo};
use cw721::{Cw721ReceiveMsg};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let sender = info.sender;
    let sndr_raw = deps.api.addr_canonicalize(sender.as_str())?;

    let config = Config {
        owner: sndr_raw,
        cw721_addr: None,
        cw20_addr: None,
        rand_addr: None
    };
    CONFIG.save(deps.storage, &config)?;
    
    let state = State {
        tax_rate: msg.tax_rate.clone(),
        amount_tax: msg.amount_tax.clone(),
        unaccounted_reward: msg.unaccounted_reward.clone(),
        total_building_stake: msg.total_building_stake.clone()
    };
    STATE.save(deps.storage, &state)?;

    let res = Response::new();
    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::ReceiveNft(msg) => handle_stake(deps, _env, info, msg),
        ExecuteMsg::Unstake {amount} => handle_unstake(deps, amount),
        ExecuteMsg::Claim {} => handle_claim(deps),
        ExecuteMsg::Withdraw {} => handle_withdraw(deps), 
    }
}

pub fn handle_stake(deps: DepsMut, env: Env, info: MessageInfo, rcv_msg: Cw721ReceiveMsg) -> StdResult<Response> {
    let contract_address = deps.api.addr_canonicalize(info.sender.as_str())?;

    // only token contract can execute this message
    let conf = CONFIG.load(deps.storage)?;

    let cw721_contract_addr = if let Some(a) = conf.cw721_addr {
        a
    } else {
        return Err(StdError::generic_err("the nft token contract must have been registered"));
    };
    
    if contract_address != cw721_contract_addr {
        return Err(StdError::generic_err("Unauthorize"));
    }
  
    match from_binary(&rcv_msg.msg)? {
        Cw721Hook::StakeHouse{} => staking_house(deps, env, rcv_msg.token_id, rcv_msg.sender, TypeNFT::House),
        Cw721Hook::StakeBuiliding{} => staking_building(deps,rcv_msg.token_id, rcv_msg.sender, TypeNFT::Building),
    };

    Ok(Response::new().add_attribute("method", "stake"))
}
pub fn handle_unstake(_deps: DepsMut, _amount: Uint128) -> StdResult<Response> {

    Ok(Response::new().add_attribute("method", "unstake"))
}


pub fn handle_claim(_deps: DepsMut) -> StdResult<Response> {
    Ok(Response::new().add_attribute("method", "claim"))
}

pub fn handle_withdraw(_deps: DepsMut) -> StdResult<Response> {
    Ok(Response::new().add_attribute("method", "withdraw"))
}

pub(crate) fn staking_house(deps: DepsMut, env: Env, token_id: String, owner: String, type_nft: TypeNFT) -> StdResult<Response>{
    let owner_addr = deps.api.addr_canonicalize(owner.as_str())?;

    let stake_info = StakedAccountInfo{
        owner: owner_addr.clone(),
        token_id: token_id.clone(),
        value: Uint128::from(env.block.time.seconds()),
        type_nft,
        ternant_rating: 0u8
    };
    
    let mut staked_account_infos = STAKED_ACCOUNT_INFOS.load(deps.storage, &owner_addr)?;
    staked_account_infos.push(stake_info);
    STAKED_ACCOUNT_INFOS.save(deps.storage, &owner_addr, &staked_account_infos)?;

    Ok(Response::new().add_attribute("method", "staking_house"))
}

pub(crate) fn staking_building(deps: DepsMut, token_id: String, owner: String, type_nft: TypeNFT) -> StdResult<Response>{
    let owner_addr = deps.api.addr_canonicalize(owner.as_str())?;

    let state = STATE.load(deps.storage)?;
    let stake_info = StakedAccountInfo{
        owner: owner_addr.clone(),
        token_id: token_id.clone(),
        value:state.amount_tax,
        type_nft,
        ternant_rating: 0u8
    };

    let mut staked_account_infos = STAKED_ACCOUNT_INFOS.load(deps.storage, &owner_addr)?;
    staked_account_infos.push(stake_info);
    STAKED_ACCOUNT_INFOS.save(deps.storage, &owner_addr, &staked_account_infos)?;
    Ok(Response::new().add_attribute("method", "staking_house"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetStakedInfo {account} => to_binary(&query_staked_info(deps, account)?),
    }
}

fn query_staked_info(deps: Deps, account: String) -> StdResult<Vec<StakedInfoResponse>> {
    let raw_account = &deps.api.addr_canonicalize(account.as_str())?;
    let account_infos  = STAKED_ACCOUNT_INFOS.load(deps.storage, raw_account.as_slice())?;

    let mut res_account_infos: Vec<StakedInfoResponse> = vec![];
    for account_info in account_infos{
        let stake_info_res = StakedInfoResponse{
            token_id: account_info.token_id,
            owner: account.clone(),
            value: account_info.value,
            type_nft: account_info.type_nft,
            ternant_rating: account_info.ternant_rating
        };
        res_account_infos.push(stake_info_res);
    }

    Ok(res_account_infos)
}

