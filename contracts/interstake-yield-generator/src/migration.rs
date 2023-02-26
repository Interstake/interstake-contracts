use cw_utils::{Duration, Expiration};
use schemars::JsonSchema;
use semver::Version;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Decimal, DepsMut, Env, StdResult};

use crate::error::ContractError;
use crate::msg::MigrateMsg;
use crate::state::{
    ClaimDetails, Config, CONFIG, LATEST_UNBONDING, UNBONDING_CLAIMS, VALIDATOR_LIST,
};

use interstake_yield_generator_v04::state::UNBONDING_CLAIMS as UNBONDING_CLAIMS_V0_4;

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
    let unbonding_period = msg.unbonding_period.unwrap_or(3600 * 24 * 28);
    let min_unbonding_cooldown = Duration::Time(unbonding_period.saturating_div(max_entries));

    let new_config = Config {
        owner,
        treasury,
        restake_commission: msg.restake_commission,
        transfer_commission: msg.transfer_commission,
        denom: msg.denom.clone(),
        unbonding_period: Duration::Time(unbonding_period),
        min_unbonding_cooldown,
    };

    // migrate unbonding claimsv0.4 to v0.5
    let mut new_claims: Vec<(Addr, ClaimDetails)> = vec![];

    UNBONDING_CLAIMS_V0_4
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .for_each(|item| {
            let (k, v) = item.unwrap();

            v.into_iter().for_each(|claim| {
                let new_claim = ClaimDetails {
                    amount: claim.amount,
                    release_timestamp: Expiration::AtTime(claim.release_timestamp),
                };
                new_claims.push((k.clone(), new_claim));
            });
        });

    UNBONDING_CLAIMS_V0_4.clear(deps.storage);

    // save new claims
    for (k, v) in new_claims {
        UNBONDING_CLAIMS.update(deps.storage, &k, |old| -> StdResult<_> {
            let mut new_claims = old.unwrap_or_default(); // default is empty vec
            new_claims.push(v);
            Ok(new_claims)
        })?;
    }

    // sets the latest unbonding period to 4 days from now
    LATEST_UNBONDING.save(
        deps.storage,
        &Expiration::AtTime(env.block.time.minus_seconds(3600 * 24 * 4)),
    )?;

    VALIDATOR_LIST.save(deps.storage, msg.staking_addr, &Decimal::one())?;

    CONFIG.save(deps.storage, &new_config)?;
    Ok(())
}
