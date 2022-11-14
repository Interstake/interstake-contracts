use super::suite::{single_validator, two_validators, SuiteBuilder};

use cosmwasm_std::{coin, coins, Decimal, Uint128};

use crate::msg::{DelegateResponse, TotalDelegatedResponse};

#[test]
fn delegate_and_transfer_single_validator() {
    delegate_and_transfer(single_validator());
}

#[test]
fn delegate_and_transfer_two_validators() {
    delegate_and_transfer(two_validators());
}

fn delegate_and_transfer(validator_list: Vec<(String, Decimal)>) {
    let user1 = ("user1", 50_000_000u128);
    let user2 = "user2";
    let mut suite = SuiteBuilder::new()
        .with_funds(user1.0, &coins(user1.1, "ujuno"))
        .build();

    suite
        .update_validator_list(suite.owner().as_str(), validator_list)
        .unwrap();

    suite.delegate(user1.0, coin(user1.1, "ujuno")).unwrap();

    assert_eq!(
        suite.query_delegated(user1.0).unwrap(),
        DelegateResponse {
            start_height: 12345,
            total_staked: user1.1.into(),
            total_earnings: Uint128::zero(),
        }
    );
    assert_eq!(
        suite.query_total_delegated().unwrap(),
        TotalDelegatedResponse {
            amount: coin(user1.1, "ujuno")
        }
    );

    suite.advance_height(500);
    suite.restake(suite.owner().as_str()).unwrap();

    suite
        .transfer(user1.0, user2, Uint128::new(30_000_000u128))
        .unwrap();
    assert_eq!(
        suite.query_delegated(user1.0).unwrap(),
        DelegateResponse {
            start_height: 12345,
            total_staked: Uint128::new(20_003_012u128),
            total_earnings: Uint128::new(3_012u128),
        }
    );
    assert_eq!(
        suite.query_delegated(user2).unwrap(),
        DelegateResponse {
            start_height: 12345 + 500,
            total_staked: Uint128::new(30_000_000u128),
            total_earnings: Uint128::zero(),
        }
    );
    assert_eq!(
        suite.query_total_delegated().unwrap(),
        TotalDelegatedResponse {
            amount: coin(50_003_012u128, "ujuno")
        }
    );
}
