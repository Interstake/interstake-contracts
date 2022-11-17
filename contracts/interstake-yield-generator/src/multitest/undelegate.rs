use super::suite::{SuiteBuilder, TWENTY_EIGHT_DAYS};

use crate::error::ContractError;
use crate::msg::DelegateResponse;
use crate::multitest::suite::validator_list;
use crate::state::ClaimDetails;
use cosmwasm_std::{coin, coins, Decimal, Uint128};
use test_case::test_case;

#[test_case(1; "single_validator")]
#[test_case(2; "two_validators")]
fn undelegate_without_delegation(i: u32) {
    let mut suite = SuiteBuilder::new().build();
    let validators = validator_list(i);

    suite
        .update_validator_list(suite.owner().as_str(), validators)
        .unwrap();

    let err = suite
        .undelegate("random_user", coin(1, "ujuno"))
        .unwrap_err();
    assert_eq!(
        ContractError::DelegationNotFound {},
        err.downcast().unwrap()
    );
}

fn create_basic_claim(i: u32) {
    let validators = validator_list(i);
    let user = "user";
    let mut suite = SuiteBuilder::new()
        .with_funds(user, &coins(100, "ujuno"))
        .build();
    suite
        .update_validator_list(suite.owner().as_str(), validators)
        .unwrap();

    suite.delegate(user, coin(100, "ujuno")).unwrap();
    assert_eq!(
        suite.query_delegated(user).unwrap(),
        DelegateResponse {
            start_height: 12345,
            total_staked: Uint128::new(100),
            total_earnings: Uint128::zero(),
        }
    );

    suite.undelegate(user, coin(100, "ujuno")).unwrap();
    let current_time = suite.app.block_info().time;
    assert_eq!(
        suite.query_claims(user).unwrap(),
        vec![ClaimDetails {
            amount: coin(100, "ujuno"),
            release_timestamp: current_time.plus_seconds(TWENTY_EIGHT_DAYS)
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

#[test_case(1; "single_validator")]
#[test_case(2; "two_validators")]
fn undelegate_part_of_tokens(i: u32) {
    let validators = validator_list(i);
    let user = "user";
    let mut suite = SuiteBuilder::new()
        .with_funds(user, &coins(1000, "ujuno"))
        .build();

    suite
        .update_validator_list(suite.owner().as_str(), validators)
        .unwrap();

    suite.delegate(user, coin(1000, "ujuno")).unwrap();
    assert_eq!(
        suite.query_delegated(user).unwrap(),
        DelegateResponse {
            start_height: 12345,
            total_staked: Uint128::new(1000),
            total_earnings: Uint128::zero(),
        }
    );

    suite.undelegate(user, coin(700, "ujuno")).unwrap();
    let current_time = suite.app.block_info().time;
    assert_eq!(
        suite.query_claims(user).unwrap(),
        vec![ClaimDetails {
            amount: coin(700, "ujuno"),
            release_timestamp: current_time.plus_seconds(TWENTY_EIGHT_DAYS)
        }]
    );

    assert_eq!(
        suite.query_delegated(user).unwrap(),
        DelegateResponse {
            start_height: 12345,
            total_staked: Uint128::new(300),
            total_earnings: Uint128::zero(),
        }
    );
}

#[test]
fn cant_undelegate_partially_delegated_tokens() {
    let user = "user";
    let mut suite = SuiteBuilder::new()
        .with_funds(user, &coins(1100, "ujuno"))
        .build();

    suite.delegate(user, coin(500, "ujuno")).unwrap();

    // since there was no restake after that block, next delegation is considered partial
    suite.advance_height(500);
    suite.delegate(user, coin(600, "ujuno")).unwrap();
    assert_eq!(
        suite.query_delegated(user).unwrap(),
        DelegateResponse {
            start_height: 12345,
            total_staked: Uint128::new(1100),
            total_earnings: Uint128::zero(),
        }
    );

    let err = suite.undelegate(user, coin(700, "ujuno")).unwrap_err();
    assert_eq!(
        ContractError::NotEnoughToUndelegate {
            wanted: Uint128::new(700),
            have: Uint128::new(500)
        },
        err.downcast().unwrap()
    );

    suite.undelegate(user, coin(500, "ujuno")).unwrap();
    let current_time = suite.app.block_info().time;
    assert_eq!(
        suite.query_claims(user).unwrap(),
        vec![ClaimDetails {
            amount: coin(500, "ujuno"),
            release_timestamp: current_time.plus_seconds(TWENTY_EIGHT_DAYS)
        }]
    );
    assert_eq!(
        suite.query_delegated(user).unwrap(),
        DelegateResponse {
            start_height: 12345,
            total_staked: Uint128::new(600),
            total_earnings: Uint128::zero(),
        }
    );
}

#[test]
fn unexpired_claims_arent_removed() {
    let user = "user";
    let mut suite = SuiteBuilder::new()
        .with_funds(user, &coins(1200, "ujuno"))
        .build();

    suite.delegate(user, coin(500, "ujuno")).unwrap();
    suite.undelegate(user, coin(500, "ujuno")).unwrap();

    let current_time = suite.app.block_info().time;
    assert_eq!(
        suite.query_claims(user).unwrap(),
        vec![ClaimDetails {
            amount: coin(500, "ujuno"),
            release_timestamp: current_time.plus_seconds(TWENTY_EIGHT_DAYS)
        }]
    );

    // advance time to create some delegation with other timestamp
    suite.advance_time(TWENTY_EIGHT_DAYS / 2);
    suite.delegate(user, coin(700, "ujuno")).unwrap();
    suite.restake("owner").unwrap();
    suite.undelegate(user, coin(700, "ujuno")).unwrap();

    // nothing happens
    let current_time = suite.app.block_info().time;
    suite.process_staking_queue().unwrap();
    suite.claim(user).unwrap();
    assert_eq!(
        suite.query_claims(user).unwrap(),
        vec![
            ClaimDetails {
                amount: coin(500, "ujuno"),
                release_timestamp: current_time.plus_seconds(TWENTY_EIGHT_DAYS / 2)
            },
            ClaimDetails {
                amount: coin(700, "ujuno"),
                release_timestamp: current_time.plus_seconds(TWENTY_EIGHT_DAYS)
            }
        ]
    );

    // expire first claim
    suite.advance_time(TWENTY_EIGHT_DAYS / 2);
    let current_time = suite.app.block_info().time;
    suite.process_staking_queue().unwrap();
    suite.claim(user).unwrap();
    assert_eq!(
        suite.query_claims(user).unwrap(),
        vec![ClaimDetails {
            amount: coin(700, "ujuno"),
            release_timestamp: current_time.plus_seconds(TWENTY_EIGHT_DAYS / 2)
        }]
    );
}
