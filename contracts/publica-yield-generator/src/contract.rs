#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StakingMsg, StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, TeamCommision, CONFIG, LAST_PAYMENT_BLOCK, STAKE_DETAILS};

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
        ExecuteMsg::Delegate { sender, amount } => {
            let sender = deps.api.addr_validate(&sender)?;
            execute::delegate(deps, info, sender, amount)
        }
        ExecuteMsg::Undelegate { sender, amount } => {
            let sender = deps.api.addr_validate(&sender)?;
            execute::undelegate(deps, info, sender, amount)
        }
        ExecuteMsg::Restake {} => todo!(),
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
        info: MessageInfo,
        sender: Addr,
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

        STAKE_DETAILS.update(deps.storage, &sender, |stake_details| -> StdResult<_> {
            let mut stake_details = stake_details.unwrap_or_default();
            stake_details.amount += amount.amount;
            Ok(stake_details)
        })?;

        Ok(Response::new()
            .add_attribute("action", "delegate")
            .add_attribute("validator", config.staking_addr.to_string())
            .add_attribute("sender", sender.to_string())
            .add_attribute("amount", amount.to_string())
            .add_message(msg))
    }

    pub fn undelegate(
        deps: DepsMut,
        info: MessageInfo,
        sender: Addr,
        amount: Coin,
    ) -> Result<Response, ContractError> {
        let config = CONFIG.load(deps.storage)?;
        if config.owner != info.sender {
            return Err(ContractError::Unauthorized {});
        }

        let msg = StakingMsg::Undelegate {
            validator: config.staking_addr.to_string(),
            amount: amount.clone(),
        };

        STAKE_DETAILS.update(deps.storage, &sender, |stake_details| -> StdResult<_> {
            let mut stake_details = stake_details.unwrap_or_default();
            stake_details.amount -= amount.amount;
            Ok(stake_details)
        })?;

        Ok(Response::new()
            .add_attribute("action", "undelegate")
            .add_attribute("validator", config.staking_addr.to_string())
            .add_attribute("sender", sender.to_string())
            .add_attribute("amount", amount.to_string())
            .add_message(msg))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&""),
        QueryMsg::Delegate {} => to_binary(&""),
        QueryMsg::TotalDelegated {} => to_binary(&""),
    }
}
