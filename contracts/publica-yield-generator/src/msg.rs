use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

use cosmwasm_std::Decimal;

use crate::state::Config;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// Multisig contract that is allowed to perform admin operations
    pub owner: String,
    /// Denom in which contract stakes
    pub denom: String,
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
        denom: Option<String>,
        staking_addr: Option<String>,
        team_commision: Option<Decimal>,
    },
    Delegate {},
    Undelegate {},
    ClaimRewards {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Info {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InfoResponse {
    pub config: Config,
    pub last_payment_block: u64,
}
