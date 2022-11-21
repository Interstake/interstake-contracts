use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin, Decimal, Uint128};

use crate::state::{ClaimDetails, Config, TeamCommision};

#[cw_serde]
pub struct InstantiateMsg {
    /// Multisig contract that is allowed to perform admin operations
    pub owner: String,
    /// Address of validator
    pub staking_addr: String,
    /// Commission of Intrastake team
    pub team_commision: Option<Decimal>,
    /// Used denom
    pub denom: String,
    /// Unbondig period in seconds. Default: 2_419_200 (28 days)
    pub unbonding_period: Option<u64>,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Only called by owner
    UpdateConfig {
        owner: Option<String>,
        team_commision: Option<TeamCommision>,
        unbonding_period: Option<u64>,
    },
    /// Updates the list of validators that will be used for staking
    UpdateValidatorList {
        new_validator_list: Vec<(Addr, Decimal)>,
    },
    /// Adds amount of tokens to common staking pool
    Delegate {},
    /// Undelegates currently staked portion of token
    Undelegate { amount: Coin },
    /// Transfers to sender any unbonding claims that met their deadline
    Claim {},
    /// Claims rewards and then stake them; Only called by owner
    Restake {},
    /// Transfer amount of staked tokens to other address
    Transfer { recipient: String, amount: Uint128 },
    /// Undelegates all tokens
    UndelegateAll {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Returns current configuration
    #[returns(ConfigResponse)]
    Config {},
    /// Returns total amount of delegated tokens
    #[returns(TotalDelegatedResponse)]
    TotalDelegated {},
    /// Returns information about sender's delegation
    #[returns(DelegatedResponse)]
    Delegated { sender: String },
    /// Current available reward to claim
    #[returns(RewardResponse)]
    Reward {},
    /// Returns all current unbonding claims for sender
    #[returns(ClaimsResponse)]
    Claims { sender: String },
    /// Last payment block height
    #[returns(LastPaymentBlockResponse)]
    LastPaymentBlock {},
    #[returns(ValidatorsResponse)]
    ValidatorList {},
    #[returns(ValidatorWeightResponse)]
    ValidatorWeight { validator: String },
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub struct ConfigResponse {
    pub config: Config,
}

#[cw_serde]
pub struct RewardResponse {
    pub rewards: Vec<Coin>,
}

#[cw_serde]
pub struct ClaimsResponse {
    pub claims: Vec<ClaimDetails>,
}

#[cw_serde]
pub struct DelegatedResponse {
    pub delegated: Vec<DelegateResponse>,
}

#[cw_serde]
pub struct DelegateResponse {
    pub start_height: u64,
    pub total_staked: Uint128,
    pub total_earnings: Uint128,
}

#[cw_serde]
pub struct TotalDelegatedResponse {
    pub amount: Coin,
}

#[cw_serde]
pub struct LastPaymentBlockResponse {
    pub last_payment_block: u64,
}

#[cw_serde]
pub struct ValidatorsResponse {
    pub validators: Vec<(String, Decimal)>,
}

#[cw_serde]
pub struct ValidatorWeightResponse {
    pub weight: Decimal,
}
