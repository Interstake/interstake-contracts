use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Decimal};
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub staking_addr: Addr,
    pub team_commision: Option<Decimal>,
}

pub const CONFIG: Item<Config> = Item::new("config");

pub const LAST_PAYMENT_BLOCK: Item<u64> = Item::new("last_payment_block");
