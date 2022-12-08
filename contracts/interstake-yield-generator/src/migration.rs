use schemars::JsonSchema;
use semver::Version;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, DepsMut, Timestamp};
use cw_storage_plus::Item;

use crate::error::ContractError;
use crate::msg::MigrateMsg;
use crate::state::{Config, TeamCommission, CONFIG};

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ConfigV0_1_5 {
    pub owner: Addr,
    // pub staking_addr: String, // this field is removed in 0.1.5
    pub team_commission: TeamCommission,
    pub denom: String,
}

pub fn migrate_config(
    deps: DepsMut,
    version: &Version,
    msg: MigrateMsg,
) -> Result<(), ContractError> {
    if *version < "0.1.4".parse::<Version>().unwrap() {
        let old_storage: Item<ConfigV0_1_5> = Item::new("config");
        let config = old_storage.load(deps.storage)?;

        let treasury = deps.api.addr_validate(msg.treasury.as_str())?;
        let new_config = Config {
            owner: config.owner,
            treasury,
            team_commission: config.team_commission,
            denom: config.denom,
            unbonding_period: Timestamp::from_seconds(3600 * 24 * 28), // default 28 days
        };
        CONFIG.save(deps.storage, &new_config)?
    }
    Ok(())
}
