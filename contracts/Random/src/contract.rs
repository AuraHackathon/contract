use crate::error::ContractError;
use crate::msg::{InstantiateMsg, QueryMsg, RandResponse};
use crate::state::{config, config_read, State};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::Addr;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use rand_chacha::ChaChaRng;
use rand_core::{RngCore, SeedableRng};
use sha2::{Digest, Sha256};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        owner: msg.owner.clone(),
    };

    config(deps.storage).save(&state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Random { seed, entropy } => to_binary(&query_random(seed, entropy)?),
        QueryMsg::GetOwner {} => to_binary(&query_owner(deps)?),
    }
}
fn query_owner(deps: Deps) -> StdResult<Addr> {
    let state = config_read(deps.storage).load()?;
    Ok(state.owner)
}
fn query_random(seed: &[u8], entropy: &[u8]) -> StdResult<RandResponse> {
    let mut hasher = Sha256::new();

    // write input message
    hasher.update(&seed);
    hasher.update(&entropy);
    let hash = hasher.finalize();

    let mut hash_bytes = [0u8; 32];
    hash_bytes.copy_from_slice(hash.as_slice());

    let mut rng: ChaChaRng = ChaChaRng::from_seed(hash_bytes);

    let mut bytes = [0u8; 32];

    rng.fill_bytes(&mut bytes);

    Ok(RandResponse { bytes })
}
