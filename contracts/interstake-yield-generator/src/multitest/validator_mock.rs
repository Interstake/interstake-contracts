// Validator's mock
// Created to simulate Delegate/Undelegate flow.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

use cosmwasm_std::{
    to_binary, Addr, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};
use cw_multi_test::{App, AppBuilder, AppResponse, Contract, ContractWrapper, Executor};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Delegate { validator: String, amount: Coin },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {}

fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, StdError> {
    Ok(Response::default())
}

fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, StdError> {
    dbg!(msg.clone());
    match msg {
        ExecuteMsg::Delegate { validator, amount } => Ok(Response::new()),
    }
}

fn query(deps: Deps, _env: Env, _msg: QueryMsg) -> Result<Binary, StdError> {
    Ok(to_binary(&"")?)
}

pub fn contract<C>() -> Box<dyn Contract<C>>
where
    C: Clone + fmt::Debug + PartialEq + JsonSchema + 'static,
{
    let contract = ContractWrapper::new_with_empty(execute, instantiate, query);
    Box::new(contract)
}
