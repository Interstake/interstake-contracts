use thiserror::Error;

use cosmwasm_std::{StdError, Uint128};

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    OverflowError(#[from] cosmwasm_std::OverflowError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid Cw20")]
    InvalidCw20 {},

    #[error("Invalid Staking Contract")]
    InvalidStakingContract {},

    #[error("Restake error - no reward available to restake")]
    RestakeNoReward {},

    #[error("No funds sent to delegate")]
    NoFunds {},

    #[error(
        "Not enough fully delegated tokens to undelegate; you wanted: {wanted}, you have: {have}"
    )]
    NotEnoughToUndelegate { wanted: Uint128, have: Uint128 },

    #[error("Validators weights do not sum to 1.0")]
    InvalidValidatorList {},

    #[error("Delegation not found")]
    DelegationNotFound {},

    #[error("Expiration must be at least a month from now")]
    ExpirationTooSoon {},

    #[error("Address {address} not found in allowed list of commissions")]
    CommissionAddressNotFound { address: String },

    #[error("Allowance of Commission Address {address} is expired")]
    CommissionAddressExpired { address: String },

    #[error("Commission address may not be the same as the recipient")]
    CommissionAddressSameAsRecipient {},

    #[error("Unbonding called too soon after last unbonding")]
    UnbondingTooSoon {},

    #[error("Semver parsing error: {0}")]
    SemVer(String),
}

impl From<semver::Error> for ContractError {
    fn from(err: semver::Error) -> Self {
        Self::SemVer(err.to_string())
    }
}
