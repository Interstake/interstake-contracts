use super::suite::SuiteBuilder;

use cosmwasm_std::{coin, coins, Uint128};

use crate::{
    msg::{DelegateResponse, TotalDelegatedResponse},
    multitest::suite::validator_list,
};
use test_case::test_case;

#[test_case(1; "single_validator")]
#[test_case(2; "two_validators")]
fn delegate_and_transfer(i: u32) {
    let validators = validator_list(i);

    let user1 = ("user1", 50_000_000u128);
    let user2 = "user2";
    let mut suite = SuiteBuilder::new()
        .with_funds(user1.0, &coins(user1.1, "ujuno"))
        .build();

    suite
        .update_validator_list(suite.owner().as_str(), validators)
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
