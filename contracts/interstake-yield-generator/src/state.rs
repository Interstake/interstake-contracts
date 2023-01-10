use cw_utils::{Expiration, Duration};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Coin, Decimal, StdResult, Storage, Timestamp, Uint128};
use cw_storage_plus::{Item, Map};

use crate::ContractError;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub owner: Addr,
    pub treasury: Addr,
    pub restake_commission: Decimal,
    pub transfer_commission: Decimal,
    pub denom: String,
    pub unbonding_period: Duration,
    pub min_unbonding_cooldown: Duration,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Stake {
    pub amount: Coin,
    pub join_height: u64,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, Eq, PartialEq, JsonSchema)]
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
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ClaimDetails {
    pub release_timestamp: Expiration,
    pub amount: Coin,
}

pub const CONFIG: Item<Config> = Item::new("config");
// Total amount of staked tokens
// TODO: Replace with Vec<Coin>
pub const TOTAL: Item<Coin> = Item::new("total");
pub const LAST_PAYMENT_BLOCK: Item<u64> = Item::new("last_payment_block");

pub const STAKE_DETAILS: Map<&Addr, StakeDetails> = Map::new("stake_details");

pub const LATEST_UNBONDING: Item<Expiration> = Item::new("latest_unbonding");
pub const PENDING_CLAIMS: Map<&Addr, Uint128> = Map::new("pending_claims");
pub const UNBONDING_CLAIMS: Map<&Addr, Vec<ClaimDetails>> = Map::new("unbonding_claims");

pub const VALIDATOR_LIST: Map<String, Decimal> = Map::new("validator_list");
pub const ALLOWED_ADDRESSES: Map<&Addr, Expiration> = Map::new("allowed_addresses");
