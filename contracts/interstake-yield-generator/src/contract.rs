#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, to_binary, Addr, BankMsg, Binary, Coin, Decimal, DelegationResponse, Deps, DepsMut,
    DistributionMsg, Env, MessageInfo, Order, Order::Ascending, QueryRequest, Response, StakingMsg,
    StakingQuery, StdResult, Timestamp, Uint128,
};
use cw2::set_contract_version;
use cw_utils::ensure_from_older_version;

use crate::error::ContractError;
use crate::msg::{
    ClaimsResponse, ConfigResponse, DelegateResponse, DelegatedResponse, ExecuteMsg,
    InstantiateMsg, LastPaymentBlockResponse, MigrateMsg, QueryMsg, RewardResponse,
    TotalDelegatedResponse,
};
use crate::state::{
    ClaimDetails, Config, Stake, StakeDetails, CONFIG, LAST_PAYMENT_BLOCK, STAKE_DETAILS, TOTAL,
    UNBONDING_CLAIMS, VALIDATOR_LIST,
};

use std::collections::HashMap;

const CONTRACT_NAME: &str = "crates.io:interstake-yield-generator";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = deps.api.addr_validate(&msg.owner)?;

    let unbonding_period = if let Some(unbonding_period) = msg.unbonding_period {
        Timestamp::from_seconds(unbonding_period)
    } else {
        Timestamp::from_seconds(3600 * 24 * 28) // Default: 28 days
    };

    let config = Config {
        owner: owner.clone(),
        team_commision: msg.team_commision,
        denom: msg.denom.clone(),
        unbonding_period,
    };
    CONFIG.save(deps.storage, &config)?;

    let response = Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("owner", owner.into_string())
        .add_attribute("staking_addr", &msg.staking_addr)
        .add_attribute("team_commision", msg.team_commision.to_string());

    VALIDATOR_LIST.save(deps.storage, msg.staking_addr, &Decimal::one())?;

    // Initialize last payment block
    LAST_PAYMENT_BLOCK.save(deps.storage, &env.block.height)?;
    TOTAL.save(deps.storage, &coin(0u128, &msg.denom))?;

    Ok(response)
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
            team_commission,
            unbonding_period,
        } => execute::update_config(deps, info, owner, team_commission, unbonding_period),
        ExecuteMsg::UpdateValidatorList { new_validator_list } => {
            execute::update_validator_list(deps, info, new_validator_list)
        }
        ExecuteMsg::Delegate {} => execute::delegate(deps, env, info),
        ExecuteMsg::Undelegate { amount } => execute::undelegate(deps, env, info, amount),
        ExecuteMsg::Claim {} => execute::claim(deps, env, info),
        ExecuteMsg::Restake {} => execute::restake(deps, env),
        ExecuteMsg::Transfer { recipient, amount } => {
            execute::transfer(deps, env, info.sender, recipient, amount)
        }
        ExecuteMsg::UndelegateAll {} => execute::undelegate_all(deps, env, info),
    }
}

mod execute {

    use crate::state::VALIDATOR_LIST;

    use super::{
        utils::{
            compute_redelegate_msgs, delegate_msgs_for_validators, distribute_msgs_for_validators,
            unwrap_stake_details,
        },
        *,
    };

    pub fn update_config(
        deps: DepsMut,
        info: MessageInfo,
        new_owner: Option<String>,
        new_team_commision: Option<Decimal>,
        new_unbonding_period: Option<u64>,
    ) -> Result<Response, ContractError> {
        let mut config = CONFIG.load(deps.storage)?;
        if config.owner != info.sender {
            return Err(ContractError::Unauthorized {});
        }

        if let Some(owner) = new_owner {
            let owner = deps.api.addr_validate(&owner)?;
            config.owner = owner;
        }

        if let Some(team_commission) = new_team_commission {
            config.team_commission = team_commission;
        }

        if let Some(unbonding_period) = new_unbonding_period {
            config.unbonding_period = Timestamp::from_seconds(unbonding_period);
        }

        CONFIG.save(deps.storage, &config)?;
        Ok(Response::new().add_attribute("action", "config_updated"))
    }

    pub fn update_validator_list(
        deps: DepsMut,
        info: MessageInfo,
        new_validator_list: Vec<(String, Decimal)>,
    ) -> Result<Response, ContractError> {
        let config = CONFIG.load(deps.storage)?;
        if info.sender != config.owner {
            return Err(ContractError::Unauthorized {});
        }

        let mut sum = Decimal::zero();

        let old_validator_list = VALIDATOR_LIST
            .range(deps.storage, None, None, Ascending)
            .collect::<StdResult<Vec<(String, Decimal)>>>()?;

        let total_staked = TOTAL.load(deps.storage)?;

        // redelegate funds from old validator list to new validator list
        let redelegate_msgs = compute_redelegate_msgs(
            total_staked.amount,
            &config.denom,
            old_validator_list,
            new_validator_list.clone(),
        )?;

        VALIDATOR_LIST.clear(deps.storage);
        for (validator, weight) in new_validator_list {
            sum += weight;
            VALIDATOR_LIST.save(deps.storage, validator, &weight)?;
        }

        if sum != Decimal::one() {
            return Err(ContractError::InvalidValidatorList {});
        }

        Ok(Response::new()
            .add_messages(redelegate_msgs)
            .add_attribute("action", "validator_list_updated"))
    }

    pub fn delegate(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        let denom = CONFIG.load(deps.as_ref().storage)?.denom;
        if info.funds.len() != 1 {
            return Err(ContractError::NoFunds {});
        }

        let amount = info.funds[0].clone();

        let msgs = delegate_msgs_for_validators(deps.as_ref(), amount.clone(), true)?;

        let stake = Stake {
            amount: amount.clone(),
            join_height: env.block.height,
        };
        STAKE_DETAILS.update(
            deps.storage,
            &info.sender,
            |stake_details| -> StdResult<_> {
                let mut stake_details =
                    unwrap_stake_details(stake_details, denom, env.block.height);
                stake_details.partials.push(stake);
                Ok(stake_details)
            },
        )?;

        TOTAL.update(deps.storage, |total| -> StdResult<_> {
            Ok(coin((total.amount + amount.amount).u128(), total.denom))
        })?;

        Ok(Response::new()
            .add_attribute("action", "delegate")
            .add_attribute("sender", info.sender.to_string())
            .add_attribute("amount", amount.to_string())
            .add_messages(msgs))
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

        // TODO: Check if the total amount is equal to zero -> remove entry from memory
        stake_details.consolidate_partials(deps.storage)?;
        stake_details.total.amount = stake_details
            .total
            .amount
            .checked_sub(amount.amount)
            .map_err(|_| ContractError::NotEnoughToUndelegate {
                wanted: amount.amount,
                have: stake_details.total.amount,
            })?;

        let msgs = delegate_msgs_for_validators(deps.as_ref(), amount.clone(), false)?;

        TOTAL.update(deps.storage, |total| -> StdResult<_> {
            Ok(coin((total.amount - amount.amount).u128(), total.denom))
        })?;

        // Unbonding will result in coins going back to contract.
        // Create a claim to later be able to get tokens back.
        let release_timestamp = env
            .block
            .time
            .plus_seconds(config.unbonding_period.seconds());
        UNBONDING_CLAIMS.update(deps.storage, &info.sender, |vec_claims| -> StdResult<_> {
            let mut vec_claims = vec_claims.unwrap_or_default();
            vec_claims.push(ClaimDetails {
                release_timestamp,
                amount: amount.clone(),
            });
            Ok(vec_claims)
        })?;

        STAKE_DETAILS.save(deps.storage, &info.sender, &stake_details)?;

        Ok(Response::new()
            .add_attribute("action", "undelegate")
            .add_attribute("sender", info.sender.to_string())
            .add_attribute("amount", amount.to_string())
            .add_attribute("release_timestamp", release_timestamp.to_string())
            .add_messages(msgs))
    }

    pub fn claim(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        let mut claims = query::claims(deps.as_ref(), info.sender.clone())?.claims;

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

    pub fn restake(deps: DepsMut, env: Env) -> Result<Response, ContractError> {
        let mut stakes = STAKE_DETAILS
            .range(deps.storage, None, None, Order::Ascending)
            .map(|mapping| {
                let (addr, mut stake_detail) = mapping?;
                stake_detail.consolidate_partials(deps.storage)?;
                Ok((addr, stake_detail))
            })
            .collect::<StdResult<HashMap<Addr, StakeDetails>>>()?;

        let config = CONFIG.load(deps.storage)?;
        let reward = query::reward(deps.as_ref(), &env, config.clone())?.rewards;
        if reward.len() != 1 || reward[0].amount == Uint128::zero() {
            return Ok(Response::new());
        }
        let reward = reward[0].clone();

        // Decrease reward of team_commision
        let mut commision_msgs = vec![];
        let reward = if config.team_commision == Decimal::zero() {
            reward
        } else {
            let commision_amount = config.team_commision * reward.amount;

            commision_msgs.push(BankMsg::Send {
                to_address: config.owner.to_string(),
                amount: vec![coin(commision_amount.u128(), reward.denom.clone())],
            });

            coin((reward.amount - commision_amount).u128(), reward.denom)
        };

        let reward_msgs = distribute_msgs_for_validators(deps.as_ref())?;

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
                    stake_detail.earnings += stakes_reward;
                    stake_detail.total.amount += stakes_reward;
                    STAKE_DETAILS.save(deps.storage, &addr, stake_detail)?;
                }
                Ok(())
            })?;

        let delegate_msgs = delegate_msgs_for_validators(deps.as_ref(), reward.clone(), true)?;

        // Update last payment height with current height
        LAST_PAYMENT_BLOCK.save(deps.storage, &env.block.height)?;

        // Update total amount of staked tokens with latest reward
        TOTAL.update(deps.storage, |total| -> StdResult<_> {
            Ok(coin((total.amount + sum_of_rewards).u128(), total.denom))
        })?;

        Ok(Response::new()
            .add_attribute("action", "restake")
            .add_attribute("amount", reward.amount)
            .add_messages(reward_msgs)
            .add_messages(commission_msgs)
            .add_messages(delegate_msgs))
    }

    pub fn transfer(
        deps: DepsMut,
        env: Env,
        sender: Addr,
        recipient: String,
        amount: Uint128,
    ) -> Result<Response, ContractError> {
        let recipient = deps.api.addr_validate(&recipient)?;
        let denom = CONFIG.load(deps.as_ref().storage)?.denom;

        STAKE_DETAILS.update(deps.storage, &sender, |stake_details| -> StdResult<_> {
            let mut stake_details =
                unwrap_stake_details(stake_details, denom.clone(), env.block.height);
            stake_details.total.amount = stake_details.total.amount.checked_sub(amount)?;
            Ok(stake_details)
        })?;
        STAKE_DETAILS.update(deps.storage, &recipient, |stake_details| -> StdResult<_> {
            let mut stake_details = unwrap_stake_details(stake_details, denom, env.block.height);
            stake_details.total.amount = stake_details.total.amount.checked_add(amount)?;
            Ok(stake_details)
        })?;

        Ok(Response::new()
            .add_attribute("action", "transfer")
            .add_attribute("amount", amount)
            .add_attribute("sender", &sender)
            .add_attribute("recipient", &recipient))
    }

    pub fn undelegate_all(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
    ) -> Result<Response, ContractError> {
        let config = CONFIG.load(deps.as_ref().storage)?;
        if config.owner != info.sender {
            return Err(ContractError::Unauthorized {});
        }

        let mut total_staked = Coin {
            amount: Uint128::zero(),
            denom: config.denom.clone(),
        };

        let release_timestamp = env
            .block
            .time
            .plus_seconds(config.unbonding_period.seconds());

        let mut new_claim_details: Vec<(Addr, ClaimDetails)> = vec![];
        let mut old_stake_details: Vec<(Addr, StakeDetails)> = vec![];
        // Iterate over all stakes and move copy them to old_stake_details and move them to claim_details
        for res in STAKE_DETAILS.range(deps.storage, None, None, Ascending) {
            let (addr, mut stake_details) = res?;

            // for each staker, add their stake to the total amount of undelegate
            stake_details.consolidate_partials(deps.storage)?;
            // let claim_amount = stake_details.total.clone();
            let claim_amount = Coin {
                amount: stake_details.total.amount,
                denom: config.denom.clone(),
            };

            total_staked.amount += claim_amount.amount;

            // update the stake details to reflect the undelegation
            stake_details.total = Coin {
                amount: Uint128::zero(),
                denom: config.denom.clone(),
            };

            // updates or removes stake details
            old_stake_details.push((addr.clone(), stake_details.clone()));

            // create proper claims
            new_claim_details.push((
                addr.clone(),
                ClaimDetails {
                    amount: claim_amount.clone(),
                    release_timestamp,
                },
            ));
        }

        // update STAKE_DETAILS with new stake details
        for (addr, stake_details) in old_stake_details {
            if stake_details.total.amount.is_zero() {
                STAKE_DETAILS.remove(deps.storage, &addr);
            } else {
                STAKE_DETAILS.save(deps.storage, &addr, &stake_details)?;
            }
        }

        // update CLAIM_DETAILS with new claim details
        for (addr, claim) in new_claim_details {
            UNBONDING_CLAIMS.update(deps.storage, &addr, |claims| -> StdResult<_> {
                let mut claims = claims.unwrap_or_default();
                claims.push(claim);
                Ok(claims)
            })?;
        }

        // TODO: check if total corresponds to what total_staked: WARNING: Check rounding errors
        // let total = TOTAL.load(deps.storage)?; ---

        let undelegate_msgs =
            delegate_msgs_for_validators(deps.as_ref(), total_staked.clone(), false)?;

        // Update total amount of staked tokens
        TOTAL.update(deps.storage, |total| -> StdResult<_> {
            Ok(coin(
                (total.amount.checked_sub(total_staked.amount)?).u128(),
                total.denom,
            ))
        })?;

        Ok(Response::new()
            .add_attribute("action", "undelegate_all")
            .add_attribute("amount", total_staked.amount)
            .add_attribute("release_timestamp", release_timestamp.to_string())
            .add_messages(undelegate_msgs))
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
        QueryMsg::LastPaymentBlock {} => to_binary(&query::last_payment_block(deps)?),
        QueryMsg::ValidatorWeight { validator } => to_binary(&query::validator(deps, validator)?),
        QueryMsg::ValidatorList {} => to_binary(&query::validator_list(deps)?),
    }
}

mod query {
    use crate::{
        msg::{ValidatorWeightResponse, ValidatorsResponse},
        state::VALIDATOR_LIST,
    };
    use cosmwasm_std::Order::Ascending;

    use super::*;

    pub fn config(deps: Deps) -> StdResult<ConfigResponse> {
        Ok(ConfigResponse {
            config: CONFIG.load(deps.storage)?,
        })
    }

    pub fn delegated(deps: Deps, sender: String) -> StdResult<DelegatedResponse> {
        let sender_addr = deps.api.addr_validate(&sender)?;

        let delegated = if let Some(details) = STAKE_DETAILS.may_load(deps.storage, &sender_addr)? {
            details
        } else {
            return Ok(DelegatedResponse { delegated: vec![] });
        };
        let partial_stakes: Uint128 = delegated
            .partials
            .iter()
            .map(|stake| stake.amount.amount)
            .sum();
        let total_staked = delegated.total.amount + partial_stakes;

        Ok(DelegatedResponse {
            delegated: vec![DelegateResponse {
                start_height: delegated.start_height,
                total_staked,
                total_earnings: delegated.earnings,
            }],
        })
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
    ) -> StdResult<RewardResponse> {
        let config = if let Some(config) = config.into() {
            config
        } else {
            CONFIG.load(deps.storage)?
        };

        let mut rewards: Vec<Coin> = vec![];

        for data in VALIDATOR_LIST.range(deps.storage, None, None, Ascending) {
            let (validator, _weight) = data?;
            let delegation_response: DelegationResponse =
                deps.querier
                    .query(&QueryRequest::Staking(StakingQuery::Delegation {
                        delegator: env.contract.address.to_string(),
                        validator: validator.to_string(),
                    }))?;
            if let Some(mut delegation) = delegation_response.delegation {
                // TODO: Check if reward is proper one and in Juno
                rewards.append(&mut delegation.accumulated_rewards);
            }
        }

        let mut reward = coin(0, config.denom);
        for r in rewards {
            if r.denom == reward.denom {
                reward.amount += r.amount;
            }
        }
        let reward_response = RewardResponse {
            rewards: vec![reward],
        };
        Ok(reward_response)
    }

    pub fn claims(deps: Deps, sender: Addr) -> StdResult<ClaimsResponse> {
        let claims = UNBONDING_CLAIMS
            .load(deps.storage, &sender)
            .unwrap_or_default();
        Ok(ClaimsResponse { claims })
    }

    pub fn last_payment_block(deps: Deps) -> StdResult<LastPaymentBlockResponse> {
        Ok(LastPaymentBlockResponse {
            last_payment_block: LAST_PAYMENT_BLOCK.load(deps.storage)?,
        })
    }

    pub fn validator_list(deps: Deps) -> StdResult<ValidatorsResponse> {
        let validators = VALIDATOR_LIST
            .range(deps.storage, None, None, Ascending)
            .collect::<StdResult<_>>()?;

        Ok(ValidatorsResponse { validators })
    }

    pub fn validator(deps: Deps, validator: String) -> StdResult<ValidatorWeightResponse> {
        let weight = VALIDATOR_LIST.load(deps.storage, validator)?;
        Ok(ValidatorWeightResponse { weight })
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    ensure_from_older_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::new())
}

pub mod utils {

    use cosmwasm_std::{Fraction, Order::Ascending};

    use crate::state::VALIDATOR_LIST;

    use super::*;

    pub fn delegate_msgs_for_validators(
        deps: Deps,
        amount: Coin,
        delegate: bool,
    ) -> StdResult<Vec<StakingMsg>> {
        let mut msgs = vec![];
        let coin_amount = amount.amount;
        let denom = amount.denom;

        for validator in VALIDATOR_LIST.range(deps.storage, None, None, Ascending) {
            let (val_addr, percentage) = validator.unwrap();
            let stake_amount =
                coin_amount.multiply_ratio(percentage.numerator(), percentage.denominator());
            let stake_msg: StakingMsg = if delegate {
                StakingMsg::Delegate {
                    validator: val_addr.to_string(),
                    amount: Coin {
                        denom: denom.clone(),
                        amount: stake_amount,
                    },
                }
            } else {
                StakingMsg::Undelegate {
                    validator: val_addr.to_string(),
                    amount: Coin {
                        denom: denom.clone(),
                        amount: stake_amount,
                    },
                }
            };
            msgs.push(stake_msg);
        }
        Ok(msgs)
    }

    pub fn distribute_msgs_for_validators(deps: Deps) -> StdResult<Vec<DistributionMsg>> {
        VALIDATOR_LIST
            .range(deps.storage, None, None, Ascending)
            .map(|validator| {
                let (address, _) = validator?;
                Ok(DistributionMsg::WithdrawDelegatorReward { validator: address })
            })
            .collect::<StdResult<Vec<_>>>()
    }

    pub fn compute_redelegate_msgs(
        total_delegated: Uint128,
        denom: &str,
        old_validator_list: Vec<(String, Decimal)>,
        new_validator_list: Vec<(String, Decimal)>,
    ) -> StdResult<Vec<StakingMsg>> {
        let mut msgs: Vec<StakingMsg> = vec![];

        let mut delegate_from: Vec<(String, Decimal)> = vec![];
        let mut delegate_to: Vec<(String, Decimal)> = vec![];

        for (old_validator, old_value) in old_validator_list.clone() {
            // if old validator in new validator list, compute difference
            // if let Some(new_value) = new_validator_list
            match new_validator_list
                .iter()
                .find(|(new_validator, _)| new_validator == &old_validator)
                .map(|(_, new_percentage)| new_percentage)
            {
                Some(new_value) => {
                    // if old percentage is greater than new percentage, delegate from old validator
                    if old_value.gt(new_value) {
                        delegate_from.push((old_validator, old_value.checked_sub(*new_value)?));
                        // if old percentage is less than new percentage, delegate to old validator
                    } else if old_value.lt(new_value) {
                        delegate_to.push((old_validator, *new_value - old_value));
                    }
                }
                // if old percentage is equal to new percentage, do nothing (no need to redelegate)
                None => {
                    // if old validator not in new validator list, delegate from it
                    delegate_from.push((old_validator, old_value));
                }
            }
        }

        // add new validators that are not in the old list to delegate to
        for (new_validator, new_value) in new_validator_list {
            if !old_validator_list
                .iter()
                .any(|old| old.0.eq(&new_validator))
            {
                delegate_to.push((new_validator, new_value))
            }
        }

        // now i have two lists of validators to delegate from and to
        // i need to compute the amount to delegate from each validator
        for (addr_to, mut amount_to) in delegate_to.iter_mut() {
            for (addr_from, amount_from) in delegate_from.iter_mut() {
                if amount_from.is_zero() {
                    continue;
                }

                if amount_to.gt(amount_from) {
                    // let pct_diff = amount_to.checked_sub(*amount_from)?;
                    let amount = total_delegated
                        .checked_multiply_ratio(amount_from.numerator(), amount_from.denominator())
                        .unwrap();

                    // remove value from delegate_from and update delegate_to value
                    amount_to = amount_to.checked_sub(*amount_from).unwrap();
                    *amount_from = Decimal::zero();

                    let msg = redelegate_msg(addr_from, addr_to, amount, denom.to_string());
                    msgs.push(msg);
                    continue;
                } else if amount_to.lt(amount_from) {
                    let pct_diff = amount_from.checked_sub(amount_to).unwrap();
                    let amount = total_delegated
                        .checked_multiply_ratio(pct_diff.numerator(), pct_diff.denominator())
                        .unwrap();
                    *amount_from = amount_from.checked_sub(amount_to)?;

                    let msg = redelegate_msg(addr_from, addr_to, amount, denom.to_string());
                    msgs.push(msg);
                    break;
                } else {
                    // amount_to == amount_from
                    let amount = total_delegated
                        .checked_multiply_ratio(amount_to.numerator(), amount_to.denominator())
                        .unwrap();

                    *amount_from = Decimal::zero();

                    let msg = redelegate_msg(addr_from, addr_to, amount, denom.to_string());
                    msgs.push(msg);
                    break;
                }
            }
        }

        Ok(msgs)
    }

    fn redelegate_msg(from: &str, to: &str, amount: Uint128, denom: String) -> StakingMsg {
        StakingMsg::Redelegate {
            src_validator: from.to_owned(),
            dst_validator: to.to_owned(),
            amount: coin(amount.u128(), denom),
        }
    }

    pub fn unwrap_stake_details(
        stake_details: Option<StakeDetails>,
        denom: String,
        start_height: u64,
    ) -> StakeDetails {
        stake_details.unwrap_or(StakeDetails {
            total: Coin {
                denom,
                amount: Uint128::zero(),
            },
            partials: vec![],
            earnings: Uint128::zero(),
            start_height,
        })
    }
}
