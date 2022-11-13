use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};
use entropy_beacon_cosmos::EntropyCallbackMsg;
use kujira::denom::Denom;

#[cw_serde]
pub struct InstantiateMsg {
    pub entropy_beacon_addr: Addr,
    pub token: Denom,
    pub play_amount: Uint128,
    pub win_amount: Uint128,
}

#[cw_serde]
pub struct EntropyCallbackData {
    pub game: Uint128,
    pub original_sender: Addr,
}

#[cw_serde]
pub enum ExecuteMsg {
    Pull {},
    ReceiveEntropy(EntropyCallbackMsg),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(GameResponse)]
    Game { idx: Uint128 },
}

#[cw_serde]
pub struct GameResponse {
    pub idx: Uint128,
    pub player: Addr,
    pub result: Option<Vec<u8>>,
    pub win: bool,
}

#[cw_serde]
pub struct MigrateMsg {}
