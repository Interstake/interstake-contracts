use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin, Decimal, Uint128};
use cw_utils::Expiration;

use crate::state::{ClaimDetails, Config};

#[cw_serde]
pub struct InstantiateMsg {
    /// Multisig contract that is allowed to perform admin operations
    pub owner: String,
    /// account which receives commissions
    pub treasury: String,
    /// Address of validator
    pub staking_addr: String,
    /// Commission for restaking
    pub restake_commission: Decimal,
    /// Commission for transfers
    pub transfer_commission: Decimal,
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
        treasury: Option<String>,
        restake_commission: Option<Decimal>,
        transfer_commission: Option<Decimal>,
        unbonding_period: Option<u64>,
    },
    /// Updates the list of validators that will be used for staking
    UpdateValidatorList {
        new_validator_list: Vec<(String, Decimal)>,
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
    Transfer {
        recipient: String,
        amount: Uint128,
        commission_address: Option<String>,
    },
    /// Start unbonding current batch
    Reconcile {},
    /// Undelegates all tokens
    UndelegateAll {},
    /// adds (or updates) address to allowed list
    UpdateAllowedAddr {
        address: String,
        /// seconds since epoch
        expires: u64,
    },
    /// removes address from allowed list
    RemoveAllowedAddr { address: String },
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
    /// returns the expiration date if an address is found in the allowed list
    #[returns(AllowedAddrResponse)]
    AllowedAddr { address: String },
    /// returns the list of allowed addresses
    #[returns(AllowedAddrListResponse)]
    AllowedAddrList {},
}

#[cw_serde]
pub struct MigrateMsg {
    /// Multisig contract that is allowed to perform admin operations
    pub owner: String,
    /// account which receives commissions
    pub treasury: String,
    /// Address of validator
    pub staking_addr: String,
    /// Commission for restaking
    pub restake_commission: Decimal,
    /// Commission for transfers
    pub transfer_commission: Decimal,
    /// Used denom
    pub denom: String,
    /// Unbondig period in seconds. Default: 2_419_200 (28 days)
    pub unbonding_period: Option<u64>,
}

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

#[cw_serde]
pub struct AllowedAddrResponse {
    pub expires: Expiration,
}

#[cw_serde]
pub struct AllowedAddrListResponse {
    pub allowed_list: Vec<(Addr, Expiration)>,
}
