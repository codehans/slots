use cosmwasm_schema::cw_serde;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};
use cw2::set_contract_version;

use cw_storage_plus::{Item, Map};
use cw_utils::one_coin;
use entropy_beacon_cosmos::EntropyRequest;
use kujira::denom::Denom;

use crate::error::ContractError;
use crate::msg::{EntropyCallbackData, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{State, STATE};

#[cw_serde]
struct Game {
    player: Addr,
    result: Option<[u8; 3]>,
}

const GAME: Map<u128, Game> = Map::new("game");
const IDX: Item<Uint128> = Item::new("idx");

/// Our [`InstantiateMsg`] contains the address of the entropy beacon contract.
/// We save this address in the contract state.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        entropy_beacon_addr: msg.entropy_beacon_addr,
        token: msg.token,
        play_amount: msg.play_amount,
        win_amount: msg.win_amount,
    };
    STATE.save(deps.storage, &state)?;
    IDX.save(deps.storage, &Uint128::zero())?;
    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        // Here we handle requesting entropy from the beacon.
        ExecuteMsg::Pull {} => {
            let state = STATE.load(deps.storage)?;
            let coin = one_coin(&info)?;
            if Denom::from(coin.denom) != state.token || coin.amount != state.play_amount {
                return Err(ContractError::InsufficientFunds {});
            }
            let idx = IDX.load(deps.storage)?;
            let game = Game {
                player: info.sender.clone(),
                result: None,
            };
            GAME.save(deps.storage, idx.u128(), &game)?;
            IDX.save(deps.storage, &(idx + Uint128::one()))?;

            Ok(Response::new().add_message(
                EntropyRequest {
                    callback_gas_limit: 100_000u64,
                    callback_address: env.contract.address,
                    funds: vec![],
                    callback_msg: EntropyCallbackData {
                        original_sender: info.sender,
                        game: idx,
                    },
                }
                .into_cosmos(state.entropy_beacon_addr)?,
            ))
        }
        // Here we handle receiving entropy from the beacon.
        ExecuteMsg::ReceiveEntropy(data) => {
            let state = STATE.load(deps.storage)?;
            let beacon_addr = state.entropy_beacon_addr;
            // IMPORTANT: Verify that the callback was called by the beacon, and not by someone else.
            if info.sender != beacon_addr {
                return Err(ContractError::Unauthorized {});
            }

            // IMPORTANT: Verify that the original requester for entropy is trusted (e.g.: this contract)
            if data.requester != env.contract.address {
                return Err(ContractError::Unauthorized {});
            }

            // The callback data has 64 bytes of entropy, in a Vec<u8>.
            let entropy = data.entropy;
            // We can parse out our custom callback data from the message.
            let callback_data = data.msg;
            let callback_data: EntropyCallbackData = from_binary(&callback_data)?;
            let mut response = Response::new();

            response =
                response.add_attribute("flip_original_caller", callback_data.original_sender);

            // Now we can do whatever we want with the entropy as a randomness source!
            // We can seed a PRNG with the entropy, but here, we just check for even and odd:
            if entropy.last().unwrap() % 2 == 0 {
                response = response.add_attribute("flip_result", "heads");
            } else {
                response = response.add_attribute("flip_result", "tails");
            }
            Ok(response)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    unimplemented!()
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("action", "migrate"))
}
