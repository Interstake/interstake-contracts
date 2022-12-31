use schemars::JsonSchema;
use semver::Version;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Decimal, DepsMut, Env, Timestamp};

use crate::error::ContractError;
use crate::msg::MigrateMsg;
use crate::state::{Config, UnbondInfo, CONFIG, UNBOND_INFO, VALIDATOR_LIST};

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

    let unbonding_period = if let Some(unbonding_period) = msg.unbonding_period {
        Timestamp::from_seconds(unbonding_period)
    } else {
        Timestamp::from_seconds(3600 * 24 * 28) // Default: 28 days
    };

    let new_config = Config {
        owner,
        treasury,
        restake_commission: msg.restake_commission,
        transfer_commission: msg.transfer_commission,
        denom: msg.denom.clone(),
        unbonding_period,
    };

    // sets the latest unbonding period to 4 days from now
    UNBOND_INFO.save(deps.storage, &UnbondInfo::now(env.block.time))?;

    VALIDATOR_LIST.save(deps.storage, msg.staking_addr, &Decimal::one())?;
    CONFIG.save(deps.storage, &new_config)?;
    Ok(())
}
