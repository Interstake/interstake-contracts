use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Coin, Decimal};

use crate::state::TeamCommision;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    /// Multisig contract that is allowed to perform admin operations
    pub owner: String,
    /// Address of validator
    pub staking_addr: String,
    /// Commission of Intrastake team
    pub team_commision: Option<Decimal>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateConfig {
        owner: Option<String>,
        staking_addr: Option<String>,
        team_commision: Option<TeamCommision>,
    },
    /// Adds amount of liquid to common staking pool
    Delegate {},
    /// Undelegates currently staked portion of token
    Undelegate {},
    /// Claims rewards and then stake them
    Restake {},
    /// Undelegates all tokens
    UndelegateAll {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Returns current configuration
    Config {},
    /// Returns total amount of delegated tokens
    TotalDelegated {},
    /// Returns information about sender's delegation
    Delegate {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct DelegateResponse {
    pub start_height: u64,
    pub total_earnings: Coin,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TotalDelegatedResponse {
    pub amount: Coin,
}
