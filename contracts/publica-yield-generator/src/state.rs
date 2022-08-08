use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Coin, Decimal, Uint128};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TeamCommision {
    Some(Decimal),
    None,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub owner: Addr,
    pub staking_addr: Addr,
    pub team_commision: TeamCommision,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Stake {
    pub amount: Coin,
    pub join_height: u64,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct StakeDetails {
    pub total: Uint128,
    pub partials: Vec<Stake>,
    pub rewards: Uint128,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const LAST_PAYMENT_BLOCK: Item<u64> = Item::new("last_payment_block");
pub const STAKE_DETAILS: Map<&Addr, StakeDetails> = Map::new("stake_details");
