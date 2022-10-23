use super::suite::SuiteBuilder;

use cosmwasm_std::{assert_approx_eq, coin, coins, Decimal, Uint128};

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
            total_staked: delegated,
            total_earnings: Uint128::zero(),
        }
    );
    assert_eq!(
        suite.query_total_delegated().unwrap(),
        TotalDelegatedResponse {
            amount: coin(delegated.u128(), "ujuno")
        }
    );

    suite.advance_time(ONE_DAY * 365);

    let owner = suite.owner();
    let reward_amount = suite.query_reward().unwrap().amount;
    let new_delegated = delegated + reward_amount;

    // Default validator commision is 5%, APR is 80%
    // 100_000_000 * 0.95 * 0.8 * (1/365) = 208_218.72
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

struct User {
    pub name: String,
    pub delegated: Uint128,
    pub weight: Decimal,
}

impl User {
    pub fn new(name: &str, delegated: u128, weight: Decimal) -> User {
        User {
            name: name.into(),
            delegated: delegated.into(),
            weight,
        }
    }
}

#[test]
fn multiple_users() {
    let user1 = User::new("user1", 100_000_000, Decimal::from_ratio(1u128, 15u128));
    let user2 = User::new("user2", 200_000_000, Decimal::from_ratio(2u128, 15u128));
    let user3 = User::new("user3", 300_000_000, Decimal::from_ratio(3u128, 15u128));
    let user4 = User::new("user4", 400_000_000, Decimal::from_ratio(4u128, 15u128));
    let user5 = User::new("user5", 500_000_000, Decimal::from_ratio(5u128, 15u128));
    let mut suite = SuiteBuilder::new()
        .with_funds(&user1.name, &coins(user1.delegated.u128(), "ujuno"))
        .with_funds(&user2.name, &coins(user2.delegated.u128(), "ujuno"))
        .with_funds(&user3.name, &coins(user3.delegated.u128(), "ujuno"))
        .with_funds(&user4.name, &coins(user4.delegated.u128(), "ujuno"))
        .with_funds(&user5.name, &coins(user5.delegated.u128(), "ujuno"))
        .build();

    suite
        .delegate(&user1.name, coin(user1.delegated.u128(), "ujuno"))
        .unwrap();
    suite
        .delegate(&user2.name, coin(user2.delegated.u128(), "ujuno"))
        .unwrap();
    suite
        .delegate(&user3.name, coin(user3.delegated.u128(), "ujuno"))
        .unwrap();
    suite
        .delegate(&user4.name, coin(user4.delegated.u128(), "ujuno"))
        .unwrap();
    suite
        .delegate(&user5.name, coin(user5.delegated.u128(), "ujuno"))
        .unwrap();
    assert_eq!(
        suite.query_total_delegated().unwrap(),
        TotalDelegatedResponse {
            amount: coin(1_500_000_000, "ujuno")
        }
    );

    suite.advance_time(ONE_DAY);

    let owner = suite.owner();
    let reward_amount = suite.query_reward().unwrap().amount;

    assert_eq!(reward_amount.u128(), 3_123_287u128);
    suite.restake(owner.as_str()).unwrap();

    let user1_reward = reward_amount * user1.weight;
    assert_eq!(
        suite.query_delegated(user1.name).unwrap(),
        DelegateResponse {
            start_height: 12345,
            total_staked: user1.delegated + user1_reward,
            total_earnings: user1_reward,
        }
    );

    let user2_reward = reward_amount * user2.weight;
    assert_eq!(
        suite.query_delegated(user2.name).unwrap(),
        DelegateResponse {
            start_height: 12345,
            total_staked: user2.delegated + user2_reward,
            total_earnings: user2_reward,
        }
    );

    let user3_reward = reward_amount * user3.weight;
    assert_eq!(
        suite.query_delegated(user3.name).unwrap(),
        DelegateResponse {
            start_height: 12345,
            total_staked: user3.delegated + user3_reward,
            total_earnings: user3_reward,
        }
    );

    let user4_reward = reward_amount * user4.weight;
    assert_eq!(
        suite.query_delegated(user4.name).unwrap(),
        DelegateResponse {
            start_height: 12345,
            total_staked: user4.delegated + user4_reward,
            total_earnings: user4_reward,
        }
    );

    let user5_reward = reward_amount * user5.weight;
    assert_eq!(
        suite.query_delegated(user5.name).unwrap(),
        DelegateResponse {
            start_height: 12345,
            total_staked: user5.delegated + user5_reward,
            total_earnings: user5_reward,
        }
    );

    // due to rounding issues, we are losing some parts of rewards
    assert_approx_eq!(
        suite.query_total_delegated().unwrap().amount.amount.u128(),
        1_500_000_000u128 + reward_amount.u128(),
        "0.00000001"
    );
}

#[test]
fn partial_user() {
    let user1 = User::new("user1", 50_000_000_000, Decimal::from_ratio(5u128, 9u128));
    let user2 = User::new("user2", 30_000_000_000, Decimal::from_ratio(3u128, 9u128));
    let user_partial = User::new(
        "user_partial",
        20_000_000_000,
        Decimal::from_ratio(1u128, 9u128),
    );
    let mut suite = SuiteBuilder::new()
        .with_funds(&user1.name, &coins(user1.delegated.u128(), "ujuno"))
        .with_funds(&user2.name, &coins(user2.delegated.u128(), "ujuno"))
        .with_funds(
            &user_partial.name,
            &coins(user_partial.delegated.u128(), "ujuno"),
        )
        .build();

    suite
        .delegate(&user1.name, coin(user1.delegated.u128(), "ujuno"))
        .unwrap();
    suite
        .delegate(&user2.name, coin(user2.delegated.u128(), "ujuno"))
        .unwrap();
    assert_eq!(suite.query_last_payment_block().unwrap(), 12345);

    // advance by some arbitrary time
    suite.advance_time(ONE_DAY);

    // now add another user in middle of autocompound period
    suite
        .delegate(
            &user_partial.name,
            coin(user_partial.delegated.u128(), "ujuno"),
        )
        .unwrap();

    // advance by same time as previously - partial user should count as 0.5
    suite.advance_time(ONE_DAY);

    let reward_amount = suite.query_reward().unwrap().amount;

    assert_eq!(reward_amount.u128(), 374_794_520u128);
    suite.restake(suite.owner().as_str()).unwrap();

    // user weights
    // user1 = 50_000 * 1.0
    // user2 = 30_000 * 1.0
    // user_partial = 20_000 * 0.5 = 10_000
    //
    // sum_of_weights = 90_000

    // user1 reward ratio = 50_000 / 90_000 = 0.5555
    let user1_reward = reward_amount * user1.weight;
    assert_eq!(
        suite.query_delegated(&user1.name).unwrap(),
        DelegateResponse {
            start_height: 12345,
            total_staked: user1.delegated + user1_reward,
            total_earnings: user1_reward,
        }
    );

    // user2 reward ratio = 30_000 / 90_000 = 0.3333
    let user2_reward = reward_amount * user2.weight;
    assert_eq!(
        suite.query_delegated(&user2.name).unwrap(),
        DelegateResponse {
            start_height: 12345,
            total_staked: user2.delegated + user2_reward,
            total_earnings: user2_reward,
        }
    );

    // user_partial reward ratio = 10_000 / 90_000 = 0.1111
    let user_partial_reward = reward_amount * user_partial.weight;
    let user_partial_height = 12345 + ONE_DAY / 5; // height = time / 5;
    assert_eq!(
        suite.query_delegated(&user_partial.name).unwrap(),
        DelegateResponse {
            start_height: user_partial_height,
            total_staked: user_partial.delegated + user_partial_reward,
            total_earnings: user_partial_reward,
        }
    );

    // again, lost one token due to rounding issues
    assert_approx_eq!(
        suite.query_total_delegated().unwrap().amount.amount.u128(),
        100_000_000_000u128 + reward_amount.u128(),
        "0.00000001"
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
