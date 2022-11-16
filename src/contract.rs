use cosmwasm_schema::cw_serde;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Addr, BankMsg, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult, Uint128,
};

use cw_storage_plus::{Item, Map};
use cw_utils::one_coin;
use entropy_beacon_cosmos::EntropyRequest;
use kujira::denom::Denom;

use crate::error::ContractError;
use crate::msg::{EntropyCallbackData, ExecuteMsg, GameResponse, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE};

#[cw_serde]
struct Game {
    player: Addr,
    result: Option<[u8; 3]>,
}

impl Game {
    pub fn win(&self) -> bool {
        match self.result {
            None => false,
            Some(xs) => xs[0] / 16 == xs[1] / 16 && xs[1] / 16 == xs[2] / 16,
        }
    }
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
        fee_amount: msg.fee_amount,
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

            let mut msgs = vec![EntropyRequest {
                callback_gas_limit: 100_000u64,
                callback_address: env.contract.address,
                funds: vec![],
                callback_msg: EntropyCallbackData {
                    original_sender: info.sender,
                    game: idx,
                },
            }
            .into_cosmos(state.entropy_beacon_addr)?];

            if !state.fee_amount.is_zero() {
                msgs.push(CosmosMsg::Bank(BankMsg::Send {
                    to_address: kujira::utils::fee_address().to_string(),
                    amount: state.token.coins(&state.fee_amount),
                }))
            };

            Ok(Response::new()
                .add_attribute("game", idx)
                .add_attribute("player", game.player)
                .add_messages(msgs))
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
            let mut game = GAME.load(deps.storage, callback_data.game.u128())?;
            game.result = Some([entropy[0], entropy[1], entropy[2]]);
            GAME.save(deps.storage, callback_data.game.u128(), &game)?;

            if game.win() {
                Ok(Response::default()
                    .add_message(CosmosMsg::Bank(BankMsg::Send {
                        to_address: game.player.to_string(),
                        amount: state.token.coins(&state.win_amount),
                    }))
                    .add_attribute("game", callback_data.game)
                    .add_attribute("player", game.player)
                    .add_attribute("result", "win"))
            } else {
                Ok(Response::default()
                    .add_attribute("game", callback_data.game)
                    .add_attribute("player", game.player)
                    .add_attribute("result", "lose"))
            }
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Game { idx } => {
            let game = GAME.load(deps.storage, idx.u128())?;
            to_binary(&GameResponse {
                idx,
                player: game.player.clone(),
                result: game.result.map(|x| x.into()),
                win: game.win(),
            })
        }
    }
}
