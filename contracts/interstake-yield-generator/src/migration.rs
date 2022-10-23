use schemars::JsonSchema;
use semver::Version;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, DepsMut, Timestamp};
use cw_storage_plus::Item;

use crate::error::ContractError;
use crate::state::{Config, TeamCommision, CONFIG};

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ConfigV0_1_4 {
    pub owner: Addr,
    pub staking_addr: String,
    pub team_commision: TeamCommision,
    pub denom: String,
}

pub fn migrate_config(deps: DepsMut, version: &Version) -> Result<(), ContractError> {
    if *version < "0.1.4".parse::<Version>().unwrap() {
        let old_storage: Item<ConfigV0_1_4> = Item::new("config");
        let config = old_storage.load(deps.storage)?;

        let new_config = Config {
            owner: config.owner,
            staking_addr: config.staking_addr,
            team_commision: config.team_commision,
            denom: config.denom,
            unbonding_period: Timestamp::from_seconds(3600 * 24 * 28), // default 28 days
        };
        CONFIG.save(deps.storage, &new_config)?
    }
    Ok(())
}