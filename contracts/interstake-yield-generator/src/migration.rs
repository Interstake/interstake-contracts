use cw_utils::{Duration, Expiration};
use schemars::JsonSchema;
use semver::Version;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Decimal, DepsMut, Env, Timestamp};

use crate::error::ContractError;
use crate::msg::MigrateMsg;
use crate::state::{Config, CONFIG, VALIDATOR_LIST, LATEST_UNBONDING};

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ConfigV0_1_5 {
    pub owner: Addr,
    // pub staking_addr: String, // this field is removed in 0.1.5
    pub team_commission: Decimal,
    pub denom: String,
}

pub fn migrate_config(
    deps: DepsMut,
    env: Env,
    _version: &Version,
    msg: MigrateMsg,
) -> Result<(), ContractError> {
    let owner = deps.api.addr_validate(&msg.owner)?;
    let treasury = deps.api.addr_validate(&msg.treasury)?;


    let max_entries = msg.max_entries.unwrap_or(7);
    let unbonding_period = msg.unbonding_period.unwrap_or(3600*24*28);
    let min_unbonding_cooldown = Duration::Time(unbonding_period.saturating_div(max_entries));

    let new_config = Config {
        owner,
        treasury,
        restake_commission: msg.restake_commission,
        transfer_commission: msg.transfer_commission,
        denom: msg.denom.clone(),
        unbonding_period: Duration::Time(unbonding_period),
        min_unbonding_cooldown ,
    };

    // sets the latest unbonding period to 4 days from now
    LATEST_UNBONDING.save(deps.storage, &Expiration::AtTime(env.block.time.minus_seconds(3600*24*4)))?;

    VALIDATOR_LIST.save(deps.storage, msg.staking_addr, &Decimal::one())?;
    CONFIG.save(deps.storage, &new_config)?;
    Ok(())
}
