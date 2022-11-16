use cosmwasm_schema::cw_serde;
use kujira::denom::Denom;

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Item;

#[cw_serde]
pub struct State {
    pub entropy_beacon_addr: Addr,
    pub token: Denom,
    pub play_amount: Uint128,
    pub win_amount: Uint128,
    #[serde(default)]
    pub fee_amount: Uint128,
}

pub const STATE: Item<State> = Item::new("state");
