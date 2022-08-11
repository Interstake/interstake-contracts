use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Coin, Decimal, StdResult, Storage, Uint128};
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
    pub total: Coin,
    pub partials: Vec<Stake>,
    pub earnings: Uint128,
    pub start_height: u64,
}

impl StakeDetails {
    /// Check all partial weight stakes if should be counted as full weighted stake
    pub fn consolidate_partials(&mut self, storage: &dyn Storage) -> StdResult<()> {
        let last_payment_block = LAST_PAYMENT_BLOCK.load(storage)?;
        self.partials.retain(|stake| {
            // if stake was added before last payment, it means it should be counted as full weighted stake
            if stake.join_height <= last_payment_block {
                self.total.amount += stake.amount.amount;
                return false;
            }
            true
        });
        Ok(())
    }
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const LAST_PAYMENT_BLOCK: Item<u64> = Item::new("last_payment_block");
pub const STAKE_DETAILS: Map<&Addr, StakeDetails> = Map::new("stake_details");
