#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, to_binary, Addr, BankMsg, Binary, Coin, Decimal, DelegationResponse, Deps, DepsMut,
    DistributionMsg, Env, MessageInfo, Order, QueryRequest, Response, StakingMsg, StakingQuery,
    StdResult, Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{
    DelegateResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, TotalDelegatedResponse,
};
use crate::state::{
    ClaimDetails, Config, Stake, StakeDetails, TeamCommision, CONFIG, LAST_PAYMENT_BLOCK,
    STAKE_DETAILS, TOTAL, UNBONDING_CLAIMS,
};

use std::collections::HashMap;

const CONTRACT_NAME: &str = "crates.io:interstake-yield-generator";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const TWENTY_EIGHT_DAYS_SECONDS: u64 = 3600 * 24 * 28;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = deps.api.addr_validate(&msg.owner)?;

    let team_commision = if let Some(commision) = msg.team_commision {
        TeamCommision::Some(commision)
    } else {
        TeamCommision::None
    };

    let config = Config {
        owner: owner.clone(),
        staking_addr: msg.staking_addr.clone(),
        team_commision,
        denom: msg.denom.clone(),
    };
    CONFIG.save(deps.storage, &config)?;

    // Initialize last payment block
    LAST_PAYMENT_BLOCK.save(deps.storage, &env.block.height)?;
    TOTAL.save(deps.storage, &coin(0u128, &msg.denom))?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("owner", owner.into_string())
        .add_attribute("staking_addr", &msg.staking_addr)
        .add_attribute(
            "team_commision",
            msg.team_commision.unwrap_or_default().to_string(),
        ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateConfig {
            owner,
            staking_addr,
            team_commision,
        } => execute::update_config(deps, info, owner, staking_addr, team_commision),
        ExecuteMsg::Delegate {} => execute::delegate(deps, env, info),
        ExecuteMsg::Undelegate { amount } => execute::undelegate(deps, env, info, amount),
        ExecuteMsg::Claim {} => execute::claim(deps, env, info),
        ExecuteMsg::Restake {} => execute::restake(deps, env, info),
        ExecuteMsg::UndelegateAll {} => todo!(),
    }
}

mod execute {
    use super::*;

    pub fn update_config(
        deps: DepsMut,
        info: MessageInfo,
        new_owner: Option<String>,
        new_staking_addr: Option<String>,
        new_team_commision: Option<TeamCommision>,
    ) -> Result<Response, ContractError> {
        let mut config = CONFIG.load(deps.storage)?;
        if config.owner != info.sender {
            return Err(ContractError::Unauthorized {});
        }

        if let Some(owner) = new_owner {
            let owner = deps.api.addr_validate(&owner)?;
            config.owner = owner;
        }

        if let Some(staking_addr) = new_staking_addr {
            config.staking_addr = staking_addr;
        }

        if let Some(team_commision) = new_team_commision {
            config.team_commision = team_commision;
        }

        CONFIG.save(deps.storage, &config)?;
        Ok(Response::new().add_attribute("action", "config_updated"))
    }

    pub fn delegate(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        let config = CONFIG.load(deps.storage)?;

        if info.funds.len() != 1 {
            return Err(ContractError::NoFunds {});
        }

        let amount = info.funds[0].clone();

        let msg = StakingMsg::Delegate {
            validator: config.staking_addr.to_string(),
            amount: amount.clone(),
        };

        let stake = Stake {
            amount: amount.clone(),
            join_height: env.block.height,
        };
        STAKE_DETAILS.update(
            deps.storage,
            &info.sender,
            |stake_details| -> StdResult<_> {
                let mut stake_details = stake_details.unwrap_or_default();
                if stake_details.start_height == 0 {
                    stake_details.start_height = env.block.height;
                }
                stake_details.partials.push(stake);
                Ok(stake_details)
            },
        )?;

        TOTAL.update(deps.storage, |total| -> StdResult<_> {
            Ok(coin((total.amount + amount.amount).u128(), total.denom))
        })?;

        Ok(Response::new()
            .add_attribute("action", "delegate")
            .add_attribute("validator", &config.staking_addr)
            .add_attribute("sender", info.sender.to_string())
            .add_attribute("amount", amount.to_string())
            .add_message(msg))
    }

    pub fn undelegate(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        amount: Coin,
    ) -> Result<Response, ContractError> {
        let config = CONFIG.load(deps.storage)?;

        let mut stake_details = STAKE_DETAILS
            .load(deps.storage, &info.sender)
            .map_err(|_| ContractError::DelegationNotFound {})?;

        stake_details.consolidate_partials(deps.storage)?;
        stake_details.total.amount = stake_details
            .total
            .amount
            .checked_sub(amount.amount)
            .map_err(|_| ContractError::NotEnoughToUndelegate {
                wanted: amount.amount,
                have: stake_details.total.amount,
            })?;

        let msg = StakingMsg::Undelegate {
            validator: config.staking_addr.to_string(),
            amount: amount.clone(),
        };

        TOTAL.update(deps.storage, |total| -> StdResult<_> {
            Ok(coin((total.amount - amount.amount).u128(), total.denom))
        })?;

        // Unbonding will result in coins going back to contract.
        // Create a claim to later be able to get tokens back.
        let next_month = env.block.time.plus_seconds(TWENTY_EIGHT_DAYS_SECONDS);
        UNBONDING_CLAIMS.update(deps.storage, &info.sender, |vec_claims| -> StdResult<_> {
            let mut vec_claims = vec_claims.unwrap_or_default();
            vec_claims.push(ClaimDetails {
                release_timestamp: next_month,
                amount: amount.clone(),
            });
            Ok(vec_claims)
        })?;

        STAKE_DETAILS.save(deps.storage, &info.sender, &stake_details)?;

        Ok(Response::new()
            .add_attribute("action", "undelegate")
            .add_attribute("validator", &config.staking_addr)
            .add_attribute("sender", info.sender.to_string())
            .add_attribute("amount", amount.to_string())
            .add_message(msg))
    }

    pub fn claim(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        let mut claims = query::claims(deps.as_ref(), info.sender.clone())?;

        let mut unmet_claims = vec![];

        let amounts = claims
            .clone()
            .into_iter()
            .filter(|claim| {
                // if claim release is still not met
                if claim.release_timestamp > env.block.time {
                    unmet_claims.push(claim.clone());
                    false
                } else {
                    true
                }
            })
            .enumerate()
            .map(|(index, _)| Ok(claims.remove(index)))
            .map(|claim: StdResult<ClaimDetails>| {
                let claim = claim?;
                Ok(claim.amount)
            })
            .collect::<StdResult<Vec<Coin>>>()?;

        UNBONDING_CLAIMS.save(deps.storage, &info.sender, &unmet_claims)?;

        let mut response = Response::new()
            .add_attribute("action", "claim_unbonded_tokens")
            .add_attribute("sender", info.sender.to_string());

        amounts.iter().for_each(|amount| {
            response = response.clone().add_attribute("amount", amount.amount);
            response = response
                .clone()
                .add_attribute("denom", amount.denom.clone());
        });

        if !amounts.is_empty() {
            let msg = BankMsg::Send {
                to_address: info.sender.to_string(),
                amount: amounts,
            };
            response = response.add_message(msg);
        }
        Ok(response)
    }

    pub fn restake(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        let config = CONFIG.load(deps.storage)?;
        if config.owner != info.sender {
            return Err(ContractError::Unauthorized {});
        }

        let mut stakes = STAKE_DETAILS
            .range(deps.storage, None, None, Order::Ascending)
            .map(|mapping| {
                let (addr, mut stake_detail) = mapping?;
                stake_detail.consolidate_partials(deps.storage)?;
                Ok((addr, stake_detail))
            })
            .collect::<StdResult<HashMap<Addr, StakeDetails>>>()?;

        let reward = query::reward(deps.as_ref(), &env, config.clone())?.unwrap_or_default();
        if reward.amount == Uint128::zero() {
            return Err(ContractError::RestakeNoReward {});
        }

        // Decrease reward of team_commision
        let reward = match config.team_commision {
            TeamCommision::Some(commision) => coin(
                (reward.amount - commision * reward.amount).u128(),
                reward.denom,
            ),
            TeamCommision::None => reward,
        };

        let reward_msg = DistributionMsg::WithdrawDelegatorReward {
            validator: config.staking_addr.to_string(),
        };

        let last_payment_block = LAST_PAYMENT_BLOCK.load(deps.storage)?;

        // Map of each total stake with weight 1.0 and partial stakes with appropriate weights
        let mut addr_and_weight: HashMap<Addr, Decimal> = HashMap::new();
        // Sum of all weights to calculate reward
        let mut sum_of_weights = Decimal::zero();

        // First, iterates over all stakes, calculates the weights and accumulate total sum of weights
        stakes.iter().for_each(|(addr, stake_detail)| {
            // Add total staked weight 1.0 * stake
            let weight = Decimal::from_ratio(stake_detail.total.amount, Uint128::new(1u128));
            addr_and_weight.insert(addr.clone(), weight);
            sum_of_weights += weight;

            // Iter through all partial stakes (those which doesn't count fully to reward)
            stake_detail.partials.iter().for_each(|stake| {
                // Calulate relative "height period" since last payment block
                let current_reward_range = env.block.height - last_payment_block;
                // Calculate when that stake has been added given relative height
                let join_height_compared = stake.join_height - last_payment_block;
                // Calculate ratio at which point given stake was added
                let partial_stake_weight =
                    Decimal::from_ratio(join_height_compared, current_reward_range);
                // Add partial staked weight - partial_weight * stake
                let weight = Decimal::from_ratio(
                    stake.amount.amount * partial_stake_weight,
                    Uint128::new(1u128),
                );
                addr_and_weight.insert(addr.clone(), weight);
                sum_of_weights += weight;
            });
        });

        // Second, iterate over those weights, calculate ratio weight/sum_of_weights and multiply that
        // by reward
        let mut sum_of_rewards = Uint128::zero();
        addr_and_weight
            .into_iter()
            .try_for_each::<_, StdResult<()>>(|(addr, weight)| {
                // Weight ratio of that one particular stake
                // Knowing total sum of all weights, multiply reward by ratio.
                let stakes_reward = weight / sum_of_weights * reward.amount; // TODO: Modify that by checking properly denom; later
                sum_of_rewards += stakes_reward;
                if let Some(stake_detail) = stakes.get_mut(&addr) {
                    (*stake_detail).earnings += stakes_reward;
                    (*stake_detail).total.amount += stakes_reward;
                    STAKE_DETAILS.save(deps.storage, &addr, stake_detail)?;
                }
                Ok(())
            })?;

        let delegate_msg = StakingMsg::Delegate {
            validator: config.staking_addr,
            amount: reward.clone(),
        };

        // Update last payment height with current height
        LAST_PAYMENT_BLOCK.save(deps.storage, &env.block.height)?;

        // Update total amount of staked tokens with latest reward
        TOTAL.update(deps.storage, |total| -> StdResult<_> {
            Ok(coin((total.amount + sum_of_rewards).u128(), total.denom))
        })?;

        Ok(Response::new()
            .add_attribute("action", "restake")
            .add_attribute("amount", reward.amount)
            .add_message(reward_msg)
            .add_message(delegate_msg))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query::config(deps)?),
        QueryMsg::Delegated { sender } => to_binary(&query::delegated(deps, sender)?),
        QueryMsg::TotalDelegated {} => to_binary(&query::total(deps)?),
        QueryMsg::Reward {} => to_binary(&query::reward(deps, &env, None)?),
        QueryMsg::Claims { sender } => {
            let sender = deps.api.addr_validate(&sender)?;
            to_binary(&query::claims(deps, sender)?)
        }
        QueryMsg::LastPaymentBlock {} => to_binary(&LAST_PAYMENT_BLOCK.load(deps.storage)?),
    }
}

mod query {
    use super::*;

    pub fn config(deps: Deps) -> StdResult<Config> {
        CONFIG.load(deps.storage)
    }

    pub fn delegated(deps: Deps, sender: String) -> StdResult<Option<DelegateResponse>> {
        let sender_addr = deps.api.addr_validate(&sender)?;

        let delegated = if let Some(details) = STAKE_DETAILS.may_load(deps.storage, &sender_addr)? {
            details
        } else {
            return Ok(None);
        };
        let partial_stakes: Uint128 = delegated
            .partials
            .iter()
            .map(|stake| stake.amount.amount)
            .sum();
        let total_staked = delegated.total.amount + partial_stakes;

        Ok(Some(DelegateResponse {
            start_height: delegated.start_height,
            total_staked,
            total_earnings: delegated.earnings,
        }))
    }

    pub fn total(deps: Deps) -> StdResult<TotalDelegatedResponse> {
        Ok(TotalDelegatedResponse {
            amount: TOTAL.load(deps.storage)?,
        })
    }

    pub fn reward(
        deps: Deps,
        env: &Env,
        config: impl Into<Option<Config>>,
    ) -> StdResult<Option<Coin>> {
        let config = if let Some(config) = config.into() {
            config
        } else {
            CONFIG.load(deps.storage)?
        };

        // Query reward
        let delegation_response: DelegationResponse =
            deps.querier
                .query(&QueryRequest::Staking(StakingQuery::Delegation {
                    delegator: env.contract.address.to_string(),
                    validator: config.staking_addr,
                }))?;
        let delegation = if let Some(delegation) = delegation_response.delegation {
            delegation
        } else {
            return Ok(None);
        };

        let reward = delegation.accumulated_rewards; // TODO: Check if reward is proper one and in Juno
        if reward.is_empty() {
            Ok(None)
        } else {
            Ok(Some(reward[0].clone()))
        }
    }

    pub fn claims(deps: Deps, sender: Addr) -> StdResult<Vec<ClaimDetails>> {
        UNBONDING_CLAIMS.load(deps.storage, &sender)
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::new())
}