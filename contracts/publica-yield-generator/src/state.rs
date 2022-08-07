use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Decimal};
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub owner: Addr,
    pub staking_addr: Addr,
    pub team_commision: TeamCommision,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TeamCommision {
    Some(Decimal),
    None,
}

pub const CONFIG: Item<Config> = Item::new("config");

pub const LAST_PAYMENT_BLOCK: Item<u64> = Item::new("last_payment_block");
