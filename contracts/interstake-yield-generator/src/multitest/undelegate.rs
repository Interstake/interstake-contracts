use super::suite::SuiteBuilder;

use cosmwasm_std::{coin, coins, Uint128};

use crate::contract::TWENTY_EIGHT_DAYS_SECONDS;
use crate::error::ContractError;
use crate::msg::DelegateResponse;
use crate::state::ClaimDetails;

#[test]
fn undelegate_without_delegation() {
    let mut suite = SuiteBuilder::new().build();
    let err = suite
        .undelegate("random_user", coin(1, "juno"))
        .unwrap_err();
    assert_eq!(
        ContractError::DelegationNotFound {},
        err.downcast().unwrap()
    );
}

#[test]
fn create_basic_claim() {
    let user = "user";
    let mut suite = SuiteBuilder::new()
        .with_funds(user, &coins(100, "juno"))
        .build();

    suite.delegate(user, coin(100, "juno")).unwrap();
    assert_eq!(
        suite.query_delegated(user).unwrap(),
        DelegateResponse {
            start_height: 12345,
            total_staked: Uint128::new(100),
            total_earnings: Uint128::zero(),
        }
    );

    suite.undelegate(user, coin(100, "juno")).unwrap();
    let current_time = suite.app.block_info().time;
    assert_eq!(
        suite.query_claims(user).unwrap(),
        vec![ClaimDetails {
            amount: coin(100, "juno"),
            release_timestamp: current_time.plus_seconds(TWENTY_EIGHT_DAYS_SECONDS)
        }]
    );

    assert_eq!(
        suite.query_delegated(user).unwrap(),
        DelegateResponse {
            start_height: 12345,
            total_staked: Uint128::zero(),
            total_earnings: Uint128::zero(),
        }
    );
}
