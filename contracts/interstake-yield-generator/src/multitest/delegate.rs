use super::suite::SuiteBuilder;

use cosmwasm_std::{coin, coins, Uint128};

use crate::msg::{DelegateResponse, TotalDelegatedResponse};

const ONE_DAY: u64 = 3600 * 24;

#[test]
fn one_user() {
    let user = "user";
    let delegated = Uint128::new(100_000_000u128);
    let mut suite = SuiteBuilder::new()
        .with_funds(user, &coins(delegated.u128(), "ujuno"))
        .build();

    suite
        .delegate(user, coin(delegated.u128(), "ujuno"))
        .unwrap();
    assert_eq!(
        suite.query_delegated(user).unwrap(),
        DelegateResponse {
            start_height: 12345,
            total_staked: delegated.clone(),
            total_earnings: Uint128::zero(),
        }
    );
    assert_eq!(
        suite.query_total_delegated().unwrap(),
        TotalDelegatedResponse {
            amount: coin(delegated.u128(), "ujuno")
        }
    );

    suite.advance_time(ONE_DAY);

    let owner = suite.owner();
    let reward_amount = suite.query_reward().unwrap().amount;
    let new_delegated = delegated + reward_amount;

    assert_eq!(reward_amount.u128(), 208_219u128);
    suite.restake(owner.as_str()).unwrap();
    assert_eq!(
        suite.query_delegated(user).unwrap(),
        DelegateResponse {
            start_height: 12345,
            total_staked: new_delegated,
            total_earnings: reward_amount,
        }
    );
    assert_eq!(
        suite.query_total_delegated().unwrap(),
        TotalDelegatedResponse {
            amount: coin(new_delegated.u128(), "ujuno")
        }
    );

    // second time same operation, which accumulates previous reward
    suite.advance_time(ONE_DAY);

    let reward_amount_2 = suite.query_reward().unwrap().amount;
    let new_delegated = new_delegated + reward_amount_2;

    assert_eq!(reward_amount_2.u128(), 208_652u128);
    suite.restake(owner.as_str()).unwrap();
    assert_eq!(
        suite.query_delegated(user).unwrap(),
        DelegateResponse {
            start_height: 12345,
            total_staked: new_delegated,
            total_earnings: reward_amount + reward_amount_2
        }
    );
    assert_eq!(
        suite.query_total_delegated().unwrap(),
        TotalDelegatedResponse {
            amount: coin(new_delegated.u128(), "ujuno")
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
        .with_funds(user1, &coins(10_000, "ujuno"))
        .with_funds(user2, &coins(20_000, "ujuno"))
        .with_funds(user3, &coins(30_000, "ujuno"))
        .with_funds(user4, &coins(40_000, "ujuno"))
        .with_funds(user5, &coins(50_000, "ujuno"))
        .build();

    suite.delegate(user1, coin(10_000, "ujuno")).unwrap();
    suite.delegate(user2, coin(20_000, "ujuno")).unwrap();
    suite.delegate(user3, coin(30_000, "ujuno")).unwrap();
    suite.delegate(user4, coin(40_000, "ujuno")).unwrap();
    suite.delegate(user5, coin(50_000, "ujuno")).unwrap();
    assert_eq!(
        suite.query_total_delegated().unwrap(),
        TotalDelegatedResponse {
            amount: coin(150_000, "ujuno")
        }
    );

    let owner = suite.owner();

    // mock makes that every delegation reward is 1/10 of delegated tokens, so 15_000 tokens in this case
    assert_eq!(suite.query_reward().unwrap(), coin(15_000, "ujuno"));
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
            amount: coin(150_000 + 14_996, "ujuno")
        }
    );
}

#[test]
fn partial_user() {
    let user1 = "user1";
    let user2 = "user2";
    let user_partial = "user_partial";
    let mut suite = SuiteBuilder::new()
        .with_funds(user1, &coins(50_000, "ujuno"))
        .with_funds(user2, &coins(30_000, "ujuno"))
        .with_funds(user_partial, &coins(20_000, "ujuno"))
        .build();

    suite.delegate(user1, coin(50_000, "ujuno")).unwrap();
    suite.delegate(user2, coin(30_000, "ujuno")).unwrap();
    assert_eq!(suite.query_last_payment_block().unwrap(), 12345);

    // advance by some arbitrary height
    suite.advance_height(500);

    // now add another user in middle of autocompound period
    suite.delegate(user_partial, coin(20_000, "ujuno")).unwrap();

    // advance by same height as previously - partial user should count as 0.5
    suite.advance_height(500);

    // reward is hardcoded 10% of total staked, it doesn't matter
    assert_eq!(suite.query_reward().unwrap(), coin(10_000, "ujuno"));
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
            amount: coin(100_000 + 9_999, "ujuno")
        }
    );
}

#[test]
fn multiple_partial_users() {
    let user1 = "user1";
    let user2 = "user2";
    let user3 = "user3";
    let mut suite = SuiteBuilder::new()
        .with_funds(user1, &coins(50_000, "ujuno"))
        .with_funds(user2, &coins(30_000, "ujuno"))
        .with_funds(user3, &coins(80_000, "ujuno"))
        .build();

    // advance by some arbitrary height (0.2 weight)
    suite.advance_height(200);
    suite.delegate(user1, coin(50_000, "ujuno")).unwrap();

    // advance by some arbitrary height (0.6 weight)
    suite.advance_height(400);
    suite.delegate(user2, coin(30_000, "ujuno")).unwrap();

    // advance by some arbitrary height (0.9 weight)
    suite.advance_height(300);
    suite.delegate(user3, coin(80_000, "ujuno")).unwrap();

    // advance height so it evens out to total 1000 blocks advanced
    suite.advance_height(100);

    // reward is hardcoded 10% of total staked, it doesn't matter
    assert_eq!(suite.query_reward().unwrap(), coin(16_000, "ujuno"));
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
            amount: coin(160_000 + 16_000, "ujuno")
        }
    );
}

#[test]
fn partial_user_become_full_after_restake() {
    let user1 = "user1";
    let user2 = "user2";
    let user3 = "user3";
    let mut suite = SuiteBuilder::new()
        .with_funds(user1, &coins(40_000, "ujuno"))
        .with_funds(user2, &coins(30_000, "ujuno"))
        .with_funds(user3, &coins(35_000, "ujuno"))
        .build();

    // advance by some arbitrary height (0.2 weight)
    suite.advance_height(200);
    suite.delegate(user1, coin(40_000, "ujuno")).unwrap();
    // advance height up to 1000 blocks
    suite.advance_height(800);

    // hardcoded 10% of delegated amount
    assert_eq!(suite.query_reward().unwrap(), coin(4_000, "ujuno"));
    suite.restake(suite.owner().as_str()).unwrap();

    // user1 was lone delegator so whole reward goes to him anyway
    assert_eq!(
        suite.query_delegated(user1).unwrap(),
        DelegateResponse {
            start_height: 12345 + 200,
            total_staked: Uint128::new(40_000 + 4_000),
            total_earnings: Uint128::new(4_000),
        }
    );

    assert_eq!(
        suite.query_total_delegated().unwrap(),
        TotalDelegatedResponse {
            amount: coin(44_000, "ujuno")
        }
    );
    suite.advance_height(300);
    suite.delegate(user2, coin(30_000, "ujuno")).unwrap();
    // advance height up to 1000 blocks
    suite.advance_height(700);

    // hardcoded 10% of delegated amount
    assert_eq!(suite.query_reward().unwrap(), coin(4_400 + 3_000, "ujuno"));
    suite.restake(suite.owner().as_str()).unwrap();

    // user weights
    // user1 = 44_000 * 1.0 - this proves that he become a full delegator
    // user2 = 30_000 * 0.3 = 9_000
    //
    // sum_of_weights = 53_000

    // user1 reward ratio = 44_000 / 53_000 = 0.8301
    // user1 - 0.8301 * 7_400 reward = 6142.74
    assert_eq!(
        suite.query_delegated(user1).unwrap(),
        DelegateResponse {
            start_height: 12345 + 200,
            total_staked: Uint128::new(44_000 + 6_143),
            total_earnings: Uint128::new(4_000 + 6_143),
        }
    );

    // user2 reward ratio = 9_000 / 53_000 = 0.1698
    // user2 - 0.1698 * 7_400 reward = 1256.52
    assert_eq!(
        suite.query_delegated(user2).unwrap(),
        DelegateResponse {
            start_height: 12345 + 1300,
            total_staked: Uint128::new(30_000 + 1_256),
            total_earnings: Uint128::new(1_256),
        }
    );

    assert_eq!(
        suite.query_total_delegated().unwrap(),
        TotalDelegatedResponse {
            amount: coin(81_399, "ujuno")
        }
    );
    suite.advance_height(400);
    suite.delegate(user3, coin(35_000, "ujuno")).unwrap();
    // advance height up to 1000 blocks
    suite.advance_height(600);

    // hardcoded 10% of delegated amount
    assert_eq!(suite.query_reward().unwrap(), coin(8_140 + 3_500, "ujuno"));
    suite.restake(suite.owner().as_str()).unwrap();

    // user weights
    // user1 = 50_143 * 1.0
    // user2 = 31_256 * 1.0
    // user3 = 35_000 * 0.4 = 14_000
    //
    // sum_of_weights = 95_399

    // user1 reward ratio = 50_143 / 95_399 = 0.5256
    // user1 - 0.5256 * 11_640 reward = 6117.984
    assert_eq!(
        suite.query_delegated(user1).unwrap(),
        DelegateResponse {
            start_height: 12345 + 200,
            total_staked: Uint128::new(50_143 + 6_118),
            total_earnings: Uint128::new(4_000 + 6_143 + 6_118),
        }
    );

    // user2 reward ratio = 31_256 / 95_399 = 0.3276
    // user2 - 0.3276 * 11_640 reward = 3813.264
    assert_eq!(
        suite.query_delegated(user2).unwrap(),
        DelegateResponse {
            start_height: 12345 + 1300,
            total_staked: Uint128::new(31_256 + 3_813),
            total_earnings: Uint128::new(1256 + 3_813),
        }
    );

    // user3 reward ratio = 14_000 / 95_399 = 0.1467
    // user3 - 0.1467 * 11_640 reward = 1707.588
    assert_eq!(
        suite.query_delegated(user3).unwrap(),
        DelegateResponse {
            start_height: 12345 + 2400,
            total_staked: Uint128::new(35_000 + 1708),
            total_earnings: Uint128::new(1708),
        }
    );

    assert_eq!(
        suite.query_total_delegated().unwrap(),
        TotalDelegatedResponse {
            // old total amount + 35k delegated last round + 11_640 last reward minus rounding issue
            amount: coin(81_399 + 35_000 + 11_639, "ujuno")
        }
    );
}
