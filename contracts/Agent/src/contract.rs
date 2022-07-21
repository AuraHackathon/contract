use crate::error::ContractError;
use crate::msg::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, StakedInfoResponse,
    StateResponse,
};
use crate::state::{
    Config, HouseBuilding, HouseInfo, StakedAccountInfo, State, TypeNFT, CONFIG,
    STAKED_ACCOUNT_INFOS, STAKED_INFOS, STATE, TOKEN_LAST_CLAIMED, PRNG_SEED,
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    attr, to_binary, Binary, CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, QueryRequest, Response, StdError, StdResult, WasmMsg, WasmQuery, Uint128
};
use cw20::Cw20ExecuteMsg;
use cw721::{Cw721ExecuteMsg, Cw721ReceiveMsg, OwnerOfResponse};
use nft::msg::QueryMsg as MinterNFTQueryMsg;
use rand_num::{sha_256, Prng};

// For randomization
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaChaRng;

const MINIMUM_TO_EXIT: u128 = 259200; // 3 days
const ONE_DAY: u128 = 86400; // 1 day
const BUILDING_CLAIM_TAX_PERCENTAGE: u128 = 25;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let sender = info.sender;
    let sndr_raw = deps.api.addr_canonicalize(sender.as_str())?;

    //generate seed to random
    let prng_seed: Vec<u8> = sha_256(base64::encode(msg.entropy).as_bytes()).to_vec(); 

    let config = Config {
        owner: sndr_raw,
        cw721_addr: None,
        cw20_addr: None,
        minter: None,
        rand_addr: None,
    };

    let state = State {
        tax_rate: msg.tax_rate.clone(),
        amount_tax: msg.amount_tax.clone(),
        unaccounted_reward: msg.unaccounted_reward.clone(),
        total_building_staked: 0,
    };
    
    //Save state
    CONFIG.save(deps.storage, &config)?;
    STATE.save(deps.storage, &state)?;
    PRNG_SEED.save(deps.storage, &prng_seed)?;

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
        ExecuteMsg::Claim { token_id, unstake } => {
            handle_claim(deps, _env, info, token_id, unstake)
        }
    }
}

pub fn handle_stake(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    rcv_msg: Cw721ReceiveMsg,
) -> StdResult<Response> {
    let contract_address = deps.api.addr_canonicalize(info.sender.as_str())?;

    // only token contract can execute this message
    let conf = CONFIG.load(deps.storage)?;

    let cw721_contract_addr = if let Some(a) = conf.cw721_addr {
        a
    } else {
        return Err(StdError::generic_err(
            "the nft token contract must have been registered",
        ));
    };

    let minter = if let Some(a) = conf.minter {
        a
    } else {
        return Err(StdError::generic_err(
            "the minter contract must have been registered",
        ));
    };

    if contract_address != cw721_contract_addr {
        return Err(StdError::generic_err("Unauthorize"));
    }

    // get data token trait
    let data: HouseBuilding = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: deps
            .api
            .addr_humanize(&minter)
            .unwrap()
            .into_string(),
        msg: to_binary(&MinterNFTQueryMsg::GetTokenTrait {
            token_id: rcv_msg.token_id.clone(),
        })?,
    }))?;

    if data.is_house {
        staking_house(
            deps,
            env,
            info.clone(),
            rcv_msg.token_id.clone(),
            rcv_msg.sender,
            TypeNFT::House,
        )?;
    } else {
        staking_building(
            deps,
            rcv_msg.token_id.clone(),
            rcv_msg.sender,
            TypeNFT::Building,
        )?;
    }

    Ok(Response::new()
        .add_attribute("method", "stake")
        .add_attribute("sender", info.sender)
        .add_attribute("token_id", rcv_msg.token_id))
}

pub fn handle_claim(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
    unstake: bool,
) -> StdResult<Response> {
    //only sender can execute this message
    let sender = deps.api.addr_canonicalize(info.sender.as_str())?;
    let conf = CONFIG.load(deps.storage)?;

    let minter = if let Some(a) = conf.minter {
        a
    } else {
        return Err(StdError::generic_err(
            "the minter contract must have been registered",
        ));
    };

    let data_owner: OwnerOfResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: deps
            .api
            .addr_humanize(&minter)
            .as_ref()
            .unwrap()
            .to_string(),
        msg: to_binary(&MinterNFTQueryMsg::OwnerOf {
            token_id: token_id.clone(),
            include_expired: None
        })?,
    }))?;

    if deps.api.addr_canonicalize(&data_owner.owner)? != sender {
        return Err(StdError::generic_err(
            "Unauthorize",
        ));
    }

    let data: HouseBuilding = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: deps
            .api
            .addr_humanize(&minter)
            .as_ref()
            .unwrap()
            .to_string(),
        msg: to_binary(&MinterNFTQueryMsg::GetTokenTrait {
            token_id: token_id.clone(),
        })?,
    }))?;

    if data.is_house {
        claim_house_from_agent(&mut deps, env, info, token_id, unstake)?;
    } else {
        claim_building_from_pack(deps, env, info, token_id, unstake)?;
    }

    Ok(Response::new().add_attribute("method", "unstake"))
}

pub fn staking_house(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
    owner: String,
    type_nft: TypeNFT,
) -> StdResult<Response> {
    let owner_addr = deps.api.addr_canonicalize(owner.as_str())?;

    let mut staked_account_infos = STAKED_ACCOUNT_INFOS.load(deps.storage, &owner_addr)?;

    //check staked_info not include token_id
    if STAKED_INFOS.has(deps.storage, &token_id) {
        return Err(StdError::generic_err("this house has been staked before"));
    }

    //generate seed to get random number
    let prng_seed: Vec<u8> = PRNG_SEED.load(deps.storage)?;
    let random_seed = new_entropy(&info, &env, prng_seed.as_ref(), prng_seed.as_ref());
    let mut rng = ChaChaRng::from_seed(random_seed);
    let seed = rng.next_u64();

    let _random = ((seed % 10) + 1) as u8;

    let stake_info = StakedAccountInfo {
        owner: owner_addr.clone(),
        token_id: token_id.clone(),
        value: Uint128::from(env.block.time.seconds()),
        type_nft,
        ternant_rating: _random,
    };

    //store staked info
    STAKED_INFOS.save(deps.storage, &token_id, &stake_info)?;

    //store staked info by staker
    staked_account_infos.push(stake_info);
    STAKED_ACCOUNT_INFOS.save(deps.storage, &owner_addr, &staked_account_infos)?;

    Ok(Response::new().add_attribute("method", "staking_house"))
}

pub fn staking_building(
    deps: DepsMut,
    token_id: String,
    owner: String,
    type_nft: TypeNFT,
) -> StdResult<Response> {
    let owner_addr = deps.api.addr_canonicalize(owner.as_str())?;

    let mut staked_account_infos = STAKED_ACCOUNT_INFOS.load(deps.storage, &owner_addr)?;
    let state = STATE.load(deps.storage)?;

    //check staked_info not include token_id
    if STAKED_INFOS.has(deps.storage, &token_id) {
        return Err(StdError::generic_err("token has been staked before"));
    }

    let stake_info = StakedAccountInfo {
        owner: owner_addr.clone(),
        token_id: token_id.clone(),
        value: state.amount_tax,
        type_nft,
        ternant_rating: 0,
    };

    //store staked info
    STAKED_INFOS.save(deps.storage, &token_id, &stake_info)?;

    //store staked info by staker
    staked_account_infos.push(stake_info);
    STAKED_ACCOUNT_INFOS.save(deps.storage, &owner_addr, &staked_account_infos)?;

    Ok(Response::new().add_attribute("method", "staking_building"))
}

pub fn claim_house_from_agent(
    deps: &mut DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
    unstake: bool,
) -> StdResult<Response> {
    //use to calculate reward for staker
    let mut owed: Uint128;

    let owner_addr = deps.api.addr_canonicalize(info.sender.as_str())?;

    let config = CONFIG.load(deps.storage)?;
    let mut staked_info = STAKED_INFOS.load(deps.storage, &token_id)?;
    let current_time = Uint128::from(env.block.time.seconds());

    let cw721_contract_addr = if let Some(a) = config.cw721_addr {
        a
    } else {
        return Err(StdError::generic_err(
            "the nft token contract must have been registered",
        ));
    };

    let cw20_contract_addr = if let Some(a) = config.cw20_addr {
        a
    } else {
        return Err(StdError::generic_err(
            "the cw20 token contract must have been registered",
        ));
    };

    if owner_addr != staked_info.owner {
        return Err(StdError::generic_err("Unauthorized"));
    }

    if !STAKED_INFOS.has(deps.storage, &token_id) {
        return Err(StdError::generic_err("Token has not been staked before"));
    }

    if !(unstake && ((Uint128::from(env.block.time.seconds()) - staked_info.value) < Uint128::from(MINIMUM_TO_EXIT)))
    {
        return Err(StdError::generic_err(
            "GONNA BE COLD WITHOUT THREE DAY'S CASH",
        ));
    }

    let data: HouseInfo = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: deps
            .api
            .addr_humanize(&cw721_contract_addr)
            .unwrap()
            .into_string(),
        msg: to_binary(&MinterNFTQueryMsg::GetHouseInfo {
            token_id: token_id.clone(),
        })?,
    }))?;

    owed = ((current_time - staked_info.value) * Uint128::from(data.income_per_day)).checked_div(Uint128::from(ONE_DAY))?;

    //select token trait and store it to mapping
    let prng_seed: Vec<u8> = PRNG_SEED.load(deps.storage)?;
    let random_seed = new_entropy(&info, &env, prng_seed.as_ref(), prng_seed.as_ref());
    let mut rng = ChaChaRng::from_seed(random_seed);
    let seed = rng.next_u64();

    let mut _random = (seed % 100) + 1;

    //calculate tax
    let tax = (owed * Uint128::from(BUILDING_CLAIM_TAX_PERCENTAGE)).checked_div(Uint128::from(100u128))?;
    pay_building_tax(deps, tax)?;
    owed -= tax;

    if _random > (staked_info.ternant_rating as u64 * 10) {
        owed = property_damage_tax(owed, Uint128::from(data.property_damage))?;
    }

    let mut messages: Vec<CosmosMsg> = vec![];

    if unstake {
        let mut staked_accounts_info = STAKED_ACCOUNT_INFOS.load(deps.storage, &owner_addr)?;

        let prng_seed: Vec<u8> = PRNG_SEED.load(deps.storage)?;
        let random_seed = new_entropy(&info, &env, prng_seed.as_ref(), prng_seed.as_ref());
        let mut rng = ChaChaRng::from_seed(random_seed);
        let seed = rng.next_u64();

        _random = (seed % 100) + 1;
        if _random > (staked_info.ternant_rating as u64 * 10) {
            pay_building_tax(deps, owed)?;
            owed = Uint128::zero();
        }

        //send back house
        let transfer_cw721_msg = Cw721ExecuteMsg::TransferNft {
            recipient: deps.api.addr_humanize(&staked_info.owner)?.into_string(),
            token_id: staked_info.token_id.clone(),
        };
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.addr_humanize(&cw721_contract_addr)?.to_string(),
            msg: to_binary(&transfer_cw721_msg)?,
            funds: vec![],
        }));

        //update state
        staked_accounts_info.remove(
            staked_accounts_info
                .iter()
                .position(|x| *x.token_id == token_id)
                .unwrap(),
        );
        STAKED_INFOS.remove(deps.storage, &token_id);
    } else {
        //save state after claim
        staked_info.value = current_time;
        
        STAKED_INFOS.save(deps.storage, &token_id, &staked_info)?;
        TOKEN_LAST_CLAIMED.save(deps.storage, &token_id, &env.block.time.seconds())?;
    }

    // create transfer cw20 msg
    let transfer_cw20_msg = Cw20ExecuteMsg::Transfer {
        recipient: deps.api.addr_humanize(&staked_info.owner)?.into_string(),
        amount: owed,
    };

    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.addr_humanize(&cw20_contract_addr)?.to_string(),
        msg: to_binary(&transfer_cw20_msg)?,
        funds: vec![],
    }));

    let res = Response::new()
        .add_messages(messages)
        .add_attributes(vec![attr("action", "claim_house")]);
    return Ok(res);
}

pub fn claim_building_from_pack(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
    unstake: bool,
) -> StdResult<Response> {
    let owner_addr = deps.api.addr_canonicalize(info.sender.as_str())?;

    // load state of contract
    let config = CONFIG.load(deps.storage)?;
    let mut state = STATE.load(deps.storage)?;
    let mut staked_info = STAKED_INFOS.load(deps.storage, &token_id)?;

    if owner_addr != staked_info.owner {
        return Err(StdError::generic_err("Unauthorized"));
    }

    //read cw721 contract address
    let cw721_contract_addr = if let Some(a) = config.cw721_addr {
        a
    } else {
        return Err(StdError::generic_err(
            "the nft token contract must have been registered",
        ));
    };

    //read cw20 contract address
    let cw20_contract_addr = if let Some(a) = config.cw20_addr {
        a
    } else {
        return Err(StdError::generic_err(
            "the cw20 token contract must have been registered",
        ));
    };

    if !STAKED_INFOS.has(deps.storage, &token_id) {
        return Err(StdError::generic_err("Token has not been staked before"));
    }

    if !(unstake && ((Uint128::from(env.block.time.seconds()) - staked_info.value) < Uint128::from(MINIMUM_TO_EXIT))){
        return Err(StdError::generic_err(
            "GONNA BE COLD WITHOUT THREE DAY'S CASH",
        ));
    }

    let owed = state.amount_tax - staked_info.value;

    let mut messages: Vec<CosmosMsg> = vec![];
    if unstake {
        let mut staked_accounts_info = STAKED_ACCOUNT_INFOS.load(deps.storage, &owner_addr)?;

        state.total_building_staked -= 1;

        //send back house
        let transfer_cw721_msg = Cw721ExecuteMsg::TransferNft {
            recipient: deps.api.addr_humanize(&staked_info.owner)?.into_string(),
            token_id: staked_info.token_id.clone(),
        };
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.addr_humanize(&cw721_contract_addr)?.to_string(),
            msg: to_binary(&transfer_cw721_msg)?,
            funds: vec![],
        }));

        //update state
        staked_accounts_info.remove(
            staked_accounts_info
                .iter()
                .position(|x| *x.token_id == token_id)
                .unwrap(),
        );
        STAKED_INFOS.remove(deps.storage, &token_id);
        STATE.save(deps.storage, &state)?;
    } else {
        staked_info.value = state.amount_tax;

        STAKED_INFOS.save(deps.storage, &token_id, &staked_info)?;
        TOKEN_LAST_CLAIMED.save(deps.storage, &token_id, &env.block.time.seconds())?;
    }

    // create transfer cw20 msg
    let transfer_cw20_msg = Cw20ExecuteMsg::Transfer {
        recipient: deps.api.addr_humanize(&staked_info.owner)?.into_string(),
        amount: owed,
    };

    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.addr_humanize(&cw20_contract_addr)?.to_string(),
        msg: to_binary(&transfer_cw20_msg)?,
        funds: vec![],
    }));

    let res = Response::new()
        .add_messages(messages)
        .add_attributes(vec![attr("action", "claim_building")]);
    return Ok(res);
}

pub fn pay_building_tax(deps: &mut DepsMut, amount: Uint128) -> StdResult<()> {
    let mut state = STATE.load(deps.storage)?;

    if state.total_building_staked == 0 {
        // if there's no staked building
        state.unaccounted_reward += amount; // keep track of $CASH due to building
        return Ok(());
    }
    // makes sure to include any unaccounted $CASH
    state.amount_tax += (amount + state.amount_tax).checked_div(Uint128::from(state.total_building_staked))?;
    state.unaccounted_reward = Uint128::zero();

    STATE.save(deps.storage, &state)?;
    Ok(())
}

pub fn property_damage_tax(amount: Uint128, property_damage: Uint128) -> StdResult<Uint128> {
    return Ok((amount * (Uint128::from(100u128) - property_damage)).checked_div(Uint128::from(100u128))?);
}

pub fn new_entropy(info: &MessageInfo, env: &Env, seed: &[u8], entropy: &[u8]) -> [u8; 32] {
    // 16 here represents the lengths in bytes of the block height and time.
    let entropy_len = 16 + info.sender.to_string().len() + entropy.len();
    let mut rng_entropy = Vec::with_capacity(entropy_len);
    rng_entropy.extend_from_slice(&env.block.height.to_be_bytes());
    rng_entropy.extend_from_slice(&env.block.time.seconds().to_be_bytes());
    rng_entropy.extend_from_slice(&info.sender.as_bytes());
    rng_entropy.extend_from_slice(entropy);

    let mut rng = Prng::new(seed, &rng_entropy);

    rng.rand_bytes()
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {  } => to_binary(&query_config(deps)?),
        QueryMsg::GetState {  } => to_binary(&query_state(deps)?),
        QueryMsg::GetStakedInfo { account } => to_binary(&query_staked_info(deps, account)?),
        QueryMsg::GetStakedAccountInfo { account } => to_binary(&query_staked_account_info(deps, account)?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;

    Ok(ConfigResponse {
        owner: deps.api.addr_humanize(&config.owner)?,
        cw721_addr: deps.api.addr_humanize(&config.cw721_addr.unwrap())?,
        cw20_addr: deps.api.addr_humanize(&config.cw20_addr.unwrap())?,
        minter: deps.api.addr_humanize(&config.minter.unwrap())?,
        rand_addr: deps.api.addr_humanize(&config.rand_addr.unwrap())?,
    })
}

pub fn query_state(deps: Deps) -> StdResult<StateResponse> {
    let state = STATE.load(deps.storage)?;

    Ok(StateResponse {
        tax_rate: state.tax_rate.clone(),
        amount_tax: state.amount_tax.clone(),
        unaccounted_reward: state.unaccounted_reward.clone(),
        total_building_staked: state.total_building_staked.clone(),
    })
}

pub fn query_staked_info(deps: Deps, account: String) -> StdResult<StakedInfoResponse> {
    let raw_account = &deps.api.addr_canonicalize(account.as_str())?;
    let account_info = STAKED_INFOS.load(deps.storage, &raw_account.to_string())?;

    let stake_info_res = StakedInfoResponse {
        token_id: account_info.token_id,
        owner: account.clone(),
        value: account_info.value,
        type_nft: account_info.type_nft,
        ternant_rating: account_info.ternant_rating,
    };

    Ok(stake_info_res)
}

pub fn query_staked_account_info(
    deps: Deps,
    account: String,
) -> StdResult<Vec<StakedInfoResponse>> {
    let raw_account = &deps.api.addr_canonicalize(account.as_str())?;
    let account_infos = STAKED_ACCOUNT_INFOS.load(deps.storage, raw_account.as_slice())?;

    let mut res_account_infos: Vec<StakedInfoResponse> = vec![];
    for account_info in account_infos {
        let stake_info_res = StakedInfoResponse {
            token_id: account_info.token_id,
            owner: account.clone(),
            value: account_info.value,
            type_nft: account_info.type_nft,
            ternant_rating: account_info.ternant_rating,
        };
        res_account_infos.push(stake_info_res);
    }

    Ok(res_account_infos)
}
