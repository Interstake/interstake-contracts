#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Decimal};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG, LAST_PAYMENT_BLOCK};
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

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
    let team_commision = msg.team_commision.unwrap_or_default();

    let config = Config {
        owner: owner.clone(),
        denom: msg.denom.clone(),
        staking_addr: staking_addr.clone(),
        team_commision,
    };
    CONFIG.save(deps.storage, &config)?;

    // Initialize last payment block
    LAST_PAYMENT_BLOCK.save(deps.storage, &env.block.height)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("owner", owner.into_string())
        .add_attribute("denom", &msg.denom)
        .add_attribute("staking_addr", staking_addr.into_string())
        .add_attribute("team_commision", team_commision.to_string())
    )
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
            denom,
            staking_addr,
            team_commision,
        } => execute_update_config(
            deps,
            info,
            env,
            owner,
            denom,
            staking_addr,
            team_commision,
        ),
        ExecuteMsg::Delegate {} => execute_distribute(deps, env),
        ExecuteMsg::Undelegate {} => execute_withdraw(deps, info, env),
        ExecuteMsg::Restake {} => execute_withdraw(deps, info, env),
        ExecuteMsg::UndelegateAll {} => execute_withdraw(deps, info, env),
    }
}

pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    owner: Option<String>,
    denom: Option<String>,
    staking_addr: Option<String>,
    team_commision: Option<Decimal>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    Ok(Response::new())
    // let distribution_msg = get_distribution_msg(deps.as_ref(), &env)?;
    // LAST_PAYMENT_BLOCK.save(deps.storage, &env.block.height)?;

    // let owner = deps.api.addr_validate(&owner)?;
    // let staking_addr = deps.api.addr_validate(&staking_addr)?;
    // if !validate_staking(deps.as_ref(), staking_addr.clone()) {
    //     return Err(ContractError::InvalidStakingContract {});
    // }

    // let reward_token = deps.api.addr_validate(&reward_token)?;
    // if !validate_cw20(deps.as_ref(), reward_token.clone()) {
    //     return Err(ContractError::InvalidCw20 {});
    // }

    // let config = Config {
    //     owner: owner.clone(),
    //     staking_addr: staking_addr.clone(),
    //     reward_token: reward_token.clone(),
    //     reward_rate,
    // };
    // CONFIG.save(deps.storage, &config)?;

    // Ok(Response::new()
    //     .add_messages(distribution_msg)
    //     .add_attribute("action", "update_config")
    //     .add_attribute("owner", owner.into_string())
    //     .add_attribute("staking_addr", staking_addr.into_string())
    //     .add_attribute("reward_token", reward_token.into_string())
    //     .add_attribute("reward_rate", reward_rate))
}

// fn get_distribution_msg(deps: Deps, env: &Env) -> Result<Option<CosmosMsg>, ContractError> {
//     let config = CONFIG.load(deps.storage)?;
//     let last_payment_block = LAST_PAYMENT_BLOCK.load(deps.storage)?;
//     let block_diff = env.block.height - last_payment_block;
//     let pending_rewards: Uint128 = config.reward_rate * Uint128::new(block_diff.into());
//
//     let balance_info: cw20::BalanceResponse = deps.querier.query_wasm_smart(
//         config.reward_token.clone(),
//         &cw20::Cw20QueryMsg::Balance {
//             address: env.contract.address.to_string(),
//         },
//     )?;
//
//     let amount = min(balance_info.balance, pending_rewards);
//
//     if amount == Uint128::zero() {
//         return Ok(None);
//     }
//
//     let msg = to_binary(&cw20::Cw20ExecuteMsg::Send {
//         contract: config.staking_addr.clone().into_string(),
//         amount,
//         msg: to_binary(&stake_cw20::msg::ReceiveMsg::Fund {}).unwrap(),
//     })?;
//     let send_msg: CosmosMsg = WasmMsg::Execute {
//         contract_addr: config.reward_token.into(),
//         msg,
//         funds: vec![],
//     }
//     .into();
//
//     Ok(Some(send_msg))
// }

pub fn execute_distribute(deps: DepsMut, env: Env) -> Result<Response, ContractError> {
    Ok(Response::new())
    // let msg = get_distribution_msg(deps.as_ref(), &env)?;
    // LAST_PAYMENT_BLOCK.save(deps.storage, &env.block.height)?;
    // Ok(Response::new()
    //     .add_messages(msg)
    //     .add_attribute("action", "distribute"))
}

pub fn execute_withdraw(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    Ok(Response::new())
    // let balance_info: cw20::BalanceResponse = deps.querier.query_wasm_smart(
    //     config.reward_token.clone(),
    //     &cw20::Cw20QueryMsg::Balance {
    //         address: env.contract.address.to_string(),
    //     },
    // )?;

    // let msg = to_binary(&cw20::Cw20ExecuteMsg::Transfer {
    //     recipient: config.owner.clone().into(),
    //     amount: balance_info.balance,
    // })?;
    // let send_msg: CosmosMsg = WasmMsg::Execute {
    //     contract_addr: config.reward_token.into(),
    //     msg,
    //     funds: vec![],
    // }
    // .into();

    // Ok(Response::new()
    //     .add_message(send_msg)
    //     .add_attribute("action", "withdraw")
    //     .add_attribute("amount", balance_info.balance)
    //     .add_attribute("recipient", config.owner.into_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&""),
        QueryMsg::Delegate {} => to_binary(&""),
        QueryMsg::TotalDelegated {} => to_binary(&""),
    }
}
