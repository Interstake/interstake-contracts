use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Coin, Decimal, Uint128};

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
    /// Only called by owner
    UpdateConfig {
        owner: Option<String>,
        staking_addr: Option<String>,
        team_commision: Option<TeamCommision>,
    },
    /// Adds amount of tokens to common staking pool
    Delegate { amount: Coin },
    /// Undelegates currently staked portion of token
    Undelegate { amount: Coin },
    /// Claims rewards and then stake them; Only called by owner
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
    Delegated { sender: String },
    /// Current available reward to claim
    Reward {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct DelegateResponse {
    pub start_height: u64,
    pub total_staked: Uint128,
    pub total_earnings: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TotalDelegatedResponse {
    pub amount: Coin,
}
