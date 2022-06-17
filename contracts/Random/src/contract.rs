use crate::error::ContractError;
use crate::msg::{InstantiateMsg, QueryMsg};
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
    let state = State { owner: msg.owner };

    config(deps.storage).save(&state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Random {
            seed,
            entropy,
            round,
        } => to_binary(&query_random(seed, entropy, round)?),
        QueryMsg::GetOwner {} => to_binary(&query_owner(deps)?),
    }
}
fn query_owner(deps: Deps) -> StdResult<Addr> {
    let state = config_read(deps.storage).load()?;
    Ok(state.owner)
}
fn query_random(seed: &[u8], entropy: &[u8], round: u64) -> StdResult<u64> {
    let mut hasher = Sha256::new();

    // write input message
    hasher.update(&seed);
    hasher.update(&entropy);

    let hash = hasher.finalize();

    let mut hash_bytes = [0u8; 32];
    hash_bytes.copy_from_slice(hash.as_slice());

    let mut rng: ChaChaRng = ChaChaRng::from_seed(hash_bytes);

    let mut bytes = [0u8; 32];
    for _ in 1..round {
        rng.fill_bytes(&mut bytes);
    }
    let mut sum: u64 = 0;
    bytes.map(|f| sum += u64::from(f));
    Ok(sum)
}

#[cfg(test)]
mod tests {

    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary, Api};

    // fn pubkey_loe_mainnet() -> Binary {
    //     vec![
    //         134, 143, 0, 94, 184, 230, 228, 202, 10, 71, 200, 167, 124, 234, 165, 48, 154, 71, 151,
    //         138, 124, 113, 188, 92, 206, 150, 54, 107, 93, 122, 86, 153, 55, 197, 41, 238, 218,
    //         102, 199, 41, 55, 132, 169, 64, 40, 1, 175, 49,
    //     ]
    //     .into()
    // }

    /// This test checks that the rng is stateful and generates
    /// different random bytes every time it is called.
    #[test]
    fn test_rng() {
        let mut deps = mock_dependencies();
        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = InstantiateMsg {
            owner: deps.api.addr_validate("human").unwrap(),
        };

        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(res.messages.len(), 0);
        let seed = b"foo";
        let entropy = b"bar!";
        let mut round: u64 = 5;
        let response1: u64 = from_binary(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::Random {
                    seed,
                    entropy,
                    round,
                },
            )
            .unwrap(),
        )
        .unwrap();

        // let r1: [u8; 32] = [
        //     68, 196, 114, 205, 225, 64, 201, 179, 18, 77, 216, 197, 211, 13, 21, 196, 11, 102, 106,
        //     195, 138, 250, 29, 185, 51, 38, 183, 0, 5, 169, 65, 190,
        // ];
        assert_eq!(response1, 3918);

        round += 5;
        let response2: u64 = from_binary(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::Random {
                    seed,
                    entropy,
                    round,
                },
            )
            .unwrap(),
        )
        .unwrap();
        // let r2: [u8; 32] = [
        //     83, 72, 156, 8, 87, 226, 57, 74, 107, 126, 225, 119, 173, 193, 73, 64, 52, 235, 203, 6,
        //     7, 206, 68, 88, 29, 149, 247, 35, 8, 229, 174, 164,
        // ];
        assert_eq!(response2, 3743);
    }
}
