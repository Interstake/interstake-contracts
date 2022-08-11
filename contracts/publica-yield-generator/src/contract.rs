#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Binary, Coin, DelegationResponse, Deps, DepsMut, DistributionMsg,
    Env, MessageInfo, QueryRequest, Response, StakingMsg, StakingQuery, StdResult
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, Stake, TeamCommision, CONFIG, LAST_PAYMENT_BLOCK, STAKE_DETAILS};

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
    let staking_addr = deps.api.addr_validate(&msg.staking_addr)?;

    let team_commision = if let Some(commision) = msg.team_commision {
        TeamCommision::Some(commision)
    } else {
        TeamCommision::None
    };

    let config = Config {
        owner: owner.clone(),
        staking_addr: staking_addr.clone(),
        team_commision: team_commision,
    };
    CONFIG.save(deps.storage, &config)?;

    // Initialize last payment block
    LAST_PAYMENT_BLOCK.save(deps.storage, &env.block.height)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("owner", owner.into_string())
        .add_attribute("staking_addr", staking_addr.into_string())
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
        ExecuteMsg::Delegate { amount } => execute::delegate(deps, env, info, amount),
        ExecuteMsg::Undelegate { amount } => execute::undelegate(deps, info, amount),
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
            let staking_addr = deps.api.addr_validate(&staking_addr)?;
            config.staking_addr = staking_addr;
        }

        if let Some(team_commision) = new_team_commision {
            config.team_commision = team_commision;
        }

        Ok(Response::new().add_attribute("action", "config_updated"))
    }

    pub fn delegate(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        amount: Coin,
    ) -> Result<Response, ContractError> {
        let config = CONFIG.load(deps.storage)?;
        if config.owner != info.sender {
            return Err(ContractError::Unauthorized {});
        }

        let msg = StakingMsg::Delegate {
            validator: config.staking_addr.to_string(),
            amount: amount.clone(),
        };

        let stake = Stake {
            amount: amount.clone(),
            join_height: env.block.height,
        };
        STAKE_DETAILS.update(deps.storage, &info.sender, |stake_details| -> StdResult<_> {
            let mut stake_details = stake_details.unwrap_or_default();
            stake_details.partials.push(stake);
            Ok(stake_details)
        })?;

        Ok(Response::new()
            .add_attribute("action", "delegate")
            .add_attribute("validator", config.staking_addr.to_string())
            .add_attribute("sender", info.sender.to_string())
            .add_attribute("amount", amount.to_string())
            .add_message(msg))
    }

    pub fn undelegate(
        deps: DepsMut,
        info: MessageInfo,
        amount: Coin,
    ) -> Result<Response, ContractError> {
        let config = CONFIG.load(deps.storage)?;

        let msg = StakingMsg::Undelegate {
            validator: config.staking_addr.to_string(),
            amount: amount.clone(),
        };

        let mut stake_details = STAKE_DETAILS.load(deps.storage, &info.sender)?;
        stake_details.consolidate_partials(deps.storage)?;
        stake_details.total.amount = stake_details.total.amount.checked_sub(amount.amount)?;

        Ok(Response::new()
            .add_attribute("action", "undelegate")
            .add_attribute("validator", config.staking_addr.to_string())
            .add_attribute("sender", info.sender.to_string())
            .add_attribute("amount", amount.to_string())
            .add_message(msg))
    }

    pub fn restake(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        let config = CONFIG.load(deps.storage)?;
        if config.owner != info.sender {
            return Err(ContractError::Unauthorized {});
        }

        // TODO: Add consolidating_partials proper implementation
        // STAKE_DETAILS.range(deps.storage, None, None, Order::Ascending)
        //     .map(|mapping| {
        //         let (_, stake_detail) = mapping?;
        //         stake_detail.consolidate_partials(deps.storage)
        // })
        //     .collect::<StdResult<()>>()?;

        let raw_delegation_response =
            deps.querier
                .query(&QueryRequest::Staking(StakingQuery::Delegation {
                    delegator: env.contract.address.into(),
                    validator: config.staking_addr.to_string(),
                }))?;
        let delegation_response: DelegationResponse = from_binary(&raw_delegation_response)?;
        let reward = delegation_response
            .delegation
            .ok_or(ContractError::NoDelegationResponse {})?
            .accumulated_rewards; // TODO: Check if reward is proper one and in Juno
        if reward.is_empty() {
            return Err(ContractError::RestakeNoReward {});
        }

        let reward_msg = DistributionMsg::WithdrawDelegatorReward {
            validator: config.staking_addr.to_string(),
        };

        // TODO: Add accumulating rewards to users


        let delegate_msg = StakingMsg::Delegate {
            validator: config.staking_addr.into(),
            amount: reward[0].clone(),
        };

        Ok(Response::new()
            .add_attribute("action", "restake")
            .add_attribute("amount", reward[0].amount)
            .add_message(reward_msg)
            .add_message(delegate_msg))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query::config(deps)?),
        QueryMsg::Delegated { sender } => to_binary(&query::delegated(deps, sender)?),
        QueryMsg::TotalDelegated {} => to_binary(&""),
    }
}

mod query {
    use super::*;

    pub fn config(deps: Deps) -> StdResult<Config> {
        CONFIG.load(deps.storage)
    }

    pub fn delegated(deps: Deps, sender: String) -> StdResult<u128> {
        let sender_addr = deps.api.addr_validate(&sender)?;

        let delegated = STAKE_DETAILS.load(deps.storage, &sender_addr)?;
        let partial_stakes = delegated.partials.iter().map(|stake| stake.amount.amount).sum();
        let total_staked = delegated.total.amount + partial_stakes;

        Ok(DelegatedResponse {
            start_height: delegated.start_height,
            total_staked,
            current_rewards: delegated.rewards,
        }
    }
}
