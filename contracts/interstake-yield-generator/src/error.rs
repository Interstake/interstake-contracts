use cosmwasm_std::StdError;
use thiserror::Error;

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
}
