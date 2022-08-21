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

#[test]
fn multiple_users() {
    let user1 = "user1";
    let user2 = "user2";
    let user3 = "user3";
    let user4 = "user4";
    let user5 = "user5";
    let mut suite = SuiteBuilder::new()
        .with_funds(user1, &coins(10_000, "juno"))
        .with_funds(user2, &coins(20_000, "juno"))
        .with_funds(user3, &coins(30_000, "juno"))
        .with_funds(user4, &coins(40_000, "juno"))
        .with_funds(user5, &coins(50_000, "juno"))
        .build();

    suite.delegate(user1, coin(10_000, "juno")).unwrap();
    suite.delegate(user2, coin(20_000, "juno")).unwrap();
    suite.delegate(user3, coin(30_000, "juno")).unwrap();
    suite.delegate(user4, coin(40_000, "juno")).unwrap();
    suite.delegate(user5, coin(50_000, "juno")).unwrap();
    assert_eq!(
        suite.query_total_delegated().unwrap(),
        TotalDelegatedResponse {
            amount: coin(150_000, "juno")
        }
    );

    let owner = suite.owner();

    // mock makes that every delegation reward is 1/10 of delegated tokens, so 15_000 tokens in this case
    assert_eq!(suite.query_reward().unwrap(), coin(15_000, "juno"));
    suite.restake(owner.as_str()).unwrap();
    // user1 - 10_000 delegated / 150_000 total * 15_000 reward = 0.0666 * 15_000 = 999
    assert_eq!(
        suite.query_delegated(user1).unwrap(),
        DelegateResponse {
            start_height: 12345,
            total_staked: Uint128::new(10_999),
            total_earnings: Uint128::new(999),
        }
    );

    // user2 - 20_000 delegated / 150_000 total * 15_000 reward = 0.1333 * 15_000 = 1_999.5
    assert_eq!(
        suite.query_delegated(user2).unwrap(),
        DelegateResponse {
            start_height: 12345,
            total_staked: Uint128::new(21_999),
            total_earnings: Uint128::new(1_999),
        }
    );

    // user3 - 30_000 delegated / 150_000 total * 15_000 reward = 0.2 * 15_000 = 3_000
    assert_eq!(
        suite.query_delegated(user3).unwrap(),
        DelegateResponse {
            start_height: 12345,
            total_staked: Uint128::new(33_000),
            total_earnings: Uint128::new(3_000),
        }
    );

    // user4 - 40_000 delegated / 150_000 total * 15_000 reward = 0.2666 * 15_000 = 3_999
    assert_eq!(
        suite.query_delegated(user4).unwrap(),
        DelegateResponse {
            start_height: 12345,
            total_staked: Uint128::new(43_999),
            total_earnings: Uint128::new(3_999),
        }
    );

    // user5 - 50_000 delegated / 150_000 total * 15_000 reward = 0.3333 * 15_000 = 4_999.5
    assert_eq!(
        suite.query_delegated(user5).unwrap(),
        DelegateResponse {
            start_height: 12345,
            total_staked: Uint128::new(54_999),
            total_earnings: Uint128::new(4_999),
        }
    );
    // due to rounding issues, we are losing some parts of rewards (currently sums up to 14_996)
    assert_eq!(
        suite.query_total_delegated().unwrap(),
        TotalDelegatedResponse {
            amount: coin(150_000 + 14_996, "juno")
        }
    );
}

#[test]
fn partial_user() {
    let user1 = "user1";
    let user2 = "user2";
    let user_partial = "user_partial";
    let mut suite = SuiteBuilder::new()
        .with_funds(user1, &coins(50_000, "juno"))
        .with_funds(user2, &coins(30_000, "juno"))
        .with_funds(user_partial, &coins(20_000, "juno"))
        .build();

    suite.delegate(user1, coin(50_000, "juno")).unwrap();
    suite.delegate(user2, coin(30_000, "juno")).unwrap();
    assert_eq!(suite.query_last_payment_block().unwrap(), 12345);

    // advance by some arbitrary height
    suite.advance_height(500);

    // now add another user in middle of autocompound period
    suite.delegate(user_partial, coin(20_000, "juno")).unwrap();

    // advance by same height as previously - partial user should count as 0.5
    suite.advance_height(500);

    // reward is hardcoded 10% of total staked, it doesn't matter
    assert_eq!(suite.query_reward().unwrap(), coin(10_000, "juno"));
    suite.restake(suite.owner().as_str()).unwrap();

    // user weights
    // user1 = 50_000 * 1.0
    // user2 = 30_000 * 1.0
    // user_partial = 20_000 * 0.5 = 10_000
    //
    // sum_of_weights = 90_000

    // user1 reward ratio = 50_000 / 90_000 = 0.5555
    // user1 - 0.5555 * 10_000 reward = 5_555
    assert_eq!(
        suite.query_delegated(user1).unwrap(),
        DelegateResponse {
            start_height: 12345,
            total_staked: Uint128::new(50_000 + 5_555),
            total_earnings: Uint128::new(5_555),
        }
    );

    // user2 reward ratio = 30_000 / 90_000 = 0.3333
    // user2 - 0.3333 * 10_000 reward = 3_333
    assert_eq!(
        suite.query_delegated(user2).unwrap(),
        DelegateResponse {
            start_height: 12345,
            total_staked: Uint128::new(30_000 + 3_333),
            total_earnings: Uint128::new(3_333),
        }
    );

    // user_partial reward ratio = 10_000 / 90_000 = 0.1111
    // user_partial = 0.1111 * 10_000 reward = 1_111
    assert_eq!(
        suite.query_delegated(user_partial).unwrap(),
        DelegateResponse {
            start_height: 12345 + 500,
            total_staked: Uint128::new(20_000 + 1_111),
            total_earnings: Uint128::new(1_111),
        }
    );
    assert_eq!(
        suite.query_total_delegated().unwrap(),
        TotalDelegatedResponse {
            // again, lost one token due to rounding issues
            amount: coin(100_000 + 9_999, "juno")
        }
    );
}

#[test]
fn multiple_partial_users() {
    let user1 = "user1";
    let user2 = "user2";
    let user3 = "user3";
    let mut suite = SuiteBuilder::new()
        .with_funds(user1, &coins(50_000, "juno"))
        .with_funds(user2, &coins(30_000, "juno"))
        .with_funds(user3, &coins(80_000, "juno"))
        .build();

    // advance by some arbitrary height (0.2 weight)
    suite.advance_height(200);
    suite.delegate(user1, coin(50_000, "juno")).unwrap();

    // advance by some arbitrary height (0.6 weight)
    suite.advance_height(400);
    suite.delegate(user2, coin(30_000, "juno")).unwrap();

    // advance by some arbitrary height (0.9 weight)
    suite.advance_height(300);
    suite.delegate(user3, coin(80_000, "juno")).unwrap();

    // advance height so it evens out to total 1000 blocks advanced
    suite.advance_height(100);

    // reward is hardcoded 10% of total staked, it doesn't matter
    assert_eq!(suite.query_reward().unwrap(), coin(16_000, "juno"));
    suite.restake(suite.owner().as_str()).unwrap();

    // user weights
    // user1 = 50_000 * 0.2 = 10_000
    // user2 = 30_000 * 0.6 = 18_000
    // user3 = 80_000 * 0.9 = 72_000
    //
    // sum_of_weights = 100_000

    // user1 reward ratio = 10_000 / 100_000 = 0.1
    // user1 - 0.1 * 16_000 reward = 1600
    assert_eq!(
        suite.query_delegated(user1).unwrap(),
        DelegateResponse {
            start_height: 12345 + 200,
            total_staked: Uint128::new(50_000 + 1_600),
            total_earnings: Uint128::new(1_600),
        }
    );

    // user2 reward ratio = 18_000 / 100_000 = 0.18
    // user2 - 0.18 * 16_000 reward = 2_880
    assert_eq!(
        suite.query_delegated(user2).unwrap(),
        DelegateResponse {
            start_height: 12345 + 600,
            total_staked: Uint128::new(30_000 + 2_880),
            total_earnings: Uint128::new(2_880),
        }
    );

    // user3 reward ratio = 72_000 / 100_000 = 0.72
    // user3 = 0.72 * 16_000 reward = 11_520
    assert_eq!(
        suite.query_delegated(user3).unwrap(),
        DelegateResponse {
            start_height: 12345 + 900,
            total_staked: Uint128::new(80_000 + 11_520),
            total_earnings: Uint128::new(11_520),
        }
    );
    assert_eq!(
        suite.query_total_delegated().unwrap(),
        TotalDelegatedResponse {
            amount: coin(160_000 + 16_000, "juno")
        }
    );
}
