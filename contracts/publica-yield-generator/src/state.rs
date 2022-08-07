use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_storage_plus::{Item, Map};

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

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Stake {
    pub amount: Uint128,
    pub rewards: Uint128,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const LAST_PAYMENT_BLOCK: Item<u64> = Item::new("last_payment_block");
pub const STAKE_DETAILS: Map<&Addr, Stake> = Map::new("stake_details");
