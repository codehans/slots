use cosmwasm_schema::cw_serde;
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
pub enum QueryMsg {}

#[cw_serde]
pub struct MigrateMsg {}
