use super::suite::SuiteBuilder;

use cosmwasm_std::{coin, coins, Uint128};

use crate::msg::{DelegateResponse, TotalDelegatedResponse};

#[test]
fn one_user() {
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
    assert_eq!(
        suite.query_total_delegated().unwrap(),
        TotalDelegatedResponse {
            amount: coin(100, "juno")
        }
    );

    let owner = suite.owner();

    // mock makes that every delegation reward is 1/10 of delegated tokens, so 10 tokens in this case
    assert_eq!(suite.query_reward().unwrap(), coin(10, "juno"));
    suite.restake(owner.as_str()).unwrap();
    assert_eq!(
        suite.query_delegated(user).unwrap(),
        DelegateResponse {
            start_height: 12345,
            total_staked: Uint128::new(110),
            total_earnings: Uint128::new(10),
        }
    );
    assert_eq!(
        suite.query_total_delegated().unwrap(),
        TotalDelegatedResponse {
            amount: coin(110, "juno")
        }
    );

    // second time same operation, which accumulates previous reward
    assert_eq!(
        suite.query_reward().unwrap(),
        coin(11, "juno") // 10% * 110 staked
    );
    suite.restake(owner.as_str()).unwrap();
    assert_eq!(
        suite.query_delegated(user).unwrap(),
        DelegateResponse {
            start_height: 12345,
            total_staked: Uint128::new(121),
            total_earnings: Uint128::new(21), // 10 + 11
        }
    );
    assert_eq!(
        suite.query_total_delegated().unwrap(),
        TotalDelegatedResponse {
            amount: coin(121, "juno")
        }
    );
}
