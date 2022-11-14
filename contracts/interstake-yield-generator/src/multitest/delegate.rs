use super::suite::{single_validator, two_validators, SuiteBuilder};

use cosmwasm_std::{assert_approx_eq, coin, coins, Decimal, Uint128};

use crate::msg::{DelegateResponse, TotalDelegatedResponse};

const ONE_DAY: u64 = 3600 * 24;

#[test]
fn one_user_one_validator() {
    one_user(single_validator())
}

#[test]
fn one_user_two_validators() {
    one_user(two_validators())
}

fn one_user(validators: Vec<(String, Decimal)>) {
    let user = "user";
    let delegated = Uint128::new(100_000_000u128);
    let mut suite = SuiteBuilder::new()
        .with_funds(user, &coins(delegated.u128(), "ujuno"))
        .build();
    suite
        .update_validator_list(suite.owner().as_str(), validators.clone())
        .unwrap();

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

    suite.advance_time(ONE_DAY);

    let owner = suite.owner();
    let reward_amount = suite.query_reward().unwrap().amount;
    let new_delegated = delegated + reward_amount;

    // Default validator commision is 5%, APR is 80%
    // 100_000_000 * 0.95 * 0.8 * (1/365) = 208_218.72
    assert_approx_eq!(reward_amount.u128(), 208_219u128, "0.00001");
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
}

impl User {
    pub fn new(name: &str, delegated: u128) -> User {
        User {
            name: name.into(),
            delegated: delegated.into(),
        }
    }
}

#[test]
fn multiple_users_single_validator() {
    multiple_users(single_validator())
}

#[test]
fn multiple_users_two_validators() {
    multiple_users(two_validators())
}

fn multiple_users(validators: Vec<(String, Decimal)>) {
    let user1 = User::new("user1", 100_000_000);
    let user2 = User::new("user2", 200_000_000);
    let user3 = User::new("user3", 300_000_000);
    let user4 = User::new("user4", 400_000_000);
    let user5 = User::new("user5", 500_000_000);
    let mut suite = SuiteBuilder::new()
        .with_funds(&user1.name, &coins(user1.delegated.u128(), "ujuno"))
        .with_funds(&user2.name, &coins(user2.delegated.u128(), "ujuno"))
        .with_funds(&user3.name, &coins(user3.delegated.u128(), "ujuno"))
        .with_funds(&user4.name, &coins(user4.delegated.u128(), "ujuno"))
        .with_funds(&user5.name, &coins(user5.delegated.u128(), "ujuno"))
        .build();

    suite
        .update_validator_list(suite.owner().as_str(), validators)
        .unwrap();

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

    assert_approx_eq!(reward_amount.u128(), 3_123_287u128, "0.00001");
    suite.restake(owner.as_str()).unwrap();

    // weight = 100_000_000 / 1_500_000_000
    let user1_reward = reward_amount * Decimal::from_ratio(1u128, 15u128);
    assert_eq!(
        suite.query_delegated(user1.name).unwrap(),
        DelegateResponse {
            start_height: 12345,
            total_staked: user1.delegated + user1_reward,
            total_earnings: user1_reward,
        }
    );

    // weight = 200_000_000 / 1_500_000_000
    let user2_reward = reward_amount * Decimal::from_ratio(2u128, 15u128);
    assert_eq!(
        suite.query_delegated(user2.name).unwrap(),
        DelegateResponse {
            start_height: 12345,
            total_staked: user2.delegated + user2_reward,
            total_earnings: user2_reward,
        }
    );

    // weight = 300_000_000 / 1_500_000_000
    let user3_reward = reward_amount * Decimal::from_ratio(3u128, 15u128);
    assert_eq!(
        suite.query_delegated(user3.name).unwrap(),
        DelegateResponse {
            start_height: 12345,
            total_staked: user3.delegated + user3_reward,
            total_earnings: user3_reward,
        }
    );

    // weight = 400_000_000 / 1_500_000_000
    let user4_reward = reward_amount * Decimal::from_ratio(4u128, 15u128);
    assert_eq!(
        suite.query_delegated(user4.name).unwrap(),
        DelegateResponse {
            start_height: 12345,
            total_staked: user4.delegated + user4_reward,
            total_earnings: user4_reward,
        }
    );

    // weight = 500_000_000 / 1_500_000_000
    let user5_reward = reward_amount * Decimal::from_ratio(5u128, 15u128);
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
fn partial_user_single_validator() {
    partial_user(single_validator())
}

#[test]
fn partial_user_two_validators() {
    partial_user(two_validators())
}

fn partial_user(validators: Vec<(String, Decimal)>) {
    let user1 = User::new("user1", 50_000_000_000);
    let user2 = User::new("user2", 30_000_000_000);
    let user_partial = User::new("user_partial", 20_000_000_000);
    let mut suite = SuiteBuilder::new()
        .with_funds(&user1.name, &coins(user1.delegated.u128(), "ujuno"))
        .with_funds(&user2.name, &coins(user2.delegated.u128(), "ujuno"))
        .with_funds(
            &user_partial.name,
            &coins(user_partial.delegated.u128(), "ujuno"),
        )
        .build();

    suite
        .update_validator_list(suite.owner().as_str(), validators)
        .unwrap();

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
    let user1_reward = reward_amount * Decimal::from_ratio(5u128, 9u128);
    assert_eq!(
        suite.query_delegated(&user1.name).unwrap(),
        DelegateResponse {
            start_height: 12345,
            total_staked: user1.delegated + user1_reward,
            total_earnings: user1_reward,
        }
    );

    // user2 reward ratio = 30_000 / 90_000 = 0.3333
    let user2_reward = reward_amount * Decimal::from_ratio(3u128, 9u128);
    assert_eq!(
        suite.query_delegated(&user2.name).unwrap(),
        DelegateResponse {
            start_height: 12345,
            total_staked: user2.delegated + user2_reward,
            total_earnings: user2_reward,
        }
    );

    // user_partial reward ratio = 10_000 / 90_000 = 0.1111
    let user_partial_reward = reward_amount * Decimal::from_ratio(1u128, 9u128);
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
fn multiple_partial_users_single_validator() {
    multiple_partial_users(single_validator())
}

#[test]
fn multiple_partial_users_two_validators() {
    multiple_partial_users(two_validators())
}

fn multiple_partial_users(validators: Vec<(String, Decimal)>) {
    let user1 = User::new("user1", 50_000_000_000);
    let user2 = User::new("user2", 30_000_000_000);
    let user3 = User::new("user3", 80_000_000_000);
    let mut suite = SuiteBuilder::new()
        .with_funds(&user1.name, &coins(user1.delegated.u128(), "ujuno"))
        .with_funds(&user2.name, &coins(user2.delegated.u128(), "ujuno"))
        .with_funds(&user3.name, &coins(user3.delegated.u128(), "ujuno"))
        .build();

    suite
        .update_validator_list(suite.owner().as_str(), validators)
        .unwrap();

    // advance by some arbitrary height (0.2 weight)
    suite.advance_height(200);
    suite
        .delegate(&user1.name, coin(user1.delegated.u128(), "ujuno"))
        .unwrap();

    // advance by some arbitrary height (0.6 weight)
    suite.advance_height(400);
    suite
        .delegate(&user2.name, coin(user2.delegated.u128(), "ujuno"))
        .unwrap();

    // advance by some arbitrary height (0.9 weight)
    suite.advance_height(300);
    suite
        .delegate(&user3.name, coin(user3.delegated.u128(), "ujuno"))
        .unwrap();

    // advance height so it evens out to total 1000 blocks advanced
    suite.advance_height(100);

    let reward_amount = suite.query_reward().unwrap().amount;
    assert_eq!(reward_amount.u128(), 8_434_804u128);

    suite.restake(suite.owner().as_str()).unwrap();

    // user weights
    // user1 = 50_000 * 0.2 = 10_000
    // user2 = 30_000 * 0.6 = 18_000
    // user3 = 80_000 * 0.9 = 72_000
    //
    // sum_of_weights = 100_000

    // user1 reward ratio = 10_000 / 100_000 = 0.1
    let user1_reward = reward_amount * Decimal::percent(10);
    assert_eq!(
        suite.query_delegated(&user1.name).unwrap(),
        DelegateResponse {
            start_height: 12345 + 200,
            total_staked: user1.delegated + user1_reward,
            total_earnings: user1_reward,
        }
    );

    // user2 reward ratio = 18_000 / 100_000 = 0.18
    let user2_reward = reward_amount * Decimal::percent(18);
    assert_eq!(
        suite.query_delegated(&user2.name).unwrap(),
        DelegateResponse {
            start_height: 12345 + 600,
            total_staked: user2.delegated + user2_reward,
            total_earnings: user2_reward,
        }
    );

    // user3 reward ratio = 72_000 / 100_000 = 0.72
    let user3_reward = reward_amount * Decimal::percent(72);
    assert_eq!(
        suite.query_delegated(&user3.name).unwrap(),
        DelegateResponse {
            start_height: 12345 + 900,
            total_staked: user3.delegated + user3_reward,
            total_earnings: user3_reward,
        }
    );

    assert_approx_eq!(
        suite.query_total_delegated().unwrap().amount.amount.u128(),
        160_000_000_000u128 + reward_amount.u128(),
        "0.00000001"
    );
}

#[test]
fn partial_user_become_full_after_restake_single_validator() {
    partial_user_become_full_after_restake(single_validator())
}

#[test]
fn partial_user_become_full_after_restake_two_validators() {
    partial_user_become_full_after_restake(two_validators())
}

fn partial_user_become_full_after_restake(validators: Vec<(String, Decimal)>) {
    let user1 = User::new("user1", 40_000_000_000);
    let user2 = User::new("user2", 30_000_000_000);
    let user3 = User::new("user3", 35_000_000_000);
    let mut suite = SuiteBuilder::new()
        .with_funds(&user1.name, &coins(user1.delegated.u128(), "ujuno"))
        .with_funds(&user2.name, &coins(user2.delegated.u128(), "ujuno"))
        .with_funds(&user3.name, &coins(user3.delegated.u128(), "ujuno"))
        .build();

    suite
        .update_validator_list(suite.owner().as_str(), validators)
        .unwrap();

    // advance by some arbitrary height (0.2 weight)
    suite.advance_height(200);
    suite
        .delegate(&user1.name, coin(user1.delegated.u128(), "ujuno"))
        .unwrap();
    // advance height up to 1000 blocks
    suite.advance_height(800);

    let reward1_amount = suite.query_reward().unwrap().amount;
    assert_eq!(reward1_amount.u128(), 4_819_888u128);

    suite.restake(suite.owner().as_str()).unwrap();

    // user1 was lone delegator so whole reward goes to him anyway
    let user1_reward1 = reward1_amount;
    let user1_restaked = user1.delegated + user1_reward1;
    assert_eq!(
        suite.query_delegated(&user1.name).unwrap(),
        DelegateResponse {
            start_height: 12345 + 200,
            total_staked: user1_restaked,
            total_earnings: user1_reward1,
        }
    );

    assert_approx_eq!(
        suite.query_total_delegated().unwrap().amount.amount.u128(),
        40_000_000_000u128 + reward1_amount.u128(),
        "0.00000001"
    );

    suite.advance_height(300);

    suite
        .delegate(&user2.name, coin(user2.delegated.u128(), "ujuno"))
        .unwrap();

    // advance height up to 1000 blocks
    suite.advance_height(700);

    let reward2_amount = suite.query_reward().unwrap().amount;
    assert_eq!(reward2_amount.u128(), 7_350_910u128);
    suite.restake(suite.owner().as_str()).unwrap();

    // user weights
    // user1 = 40_000 + reward * 1.0 - this proves that he become a full delegator
    // user2 = 30_000 * 0.3 = 9_000

    let sum_of_weights = user1_restaked + user2.delegated * Decimal::percent(30);

    let user1_reward2 = reward2_amount * Decimal::from_ratio(user1_restaked, sum_of_weights);
    let user1_restaked = user1_restaked + user1_reward2;
    assert_eq!(
        suite.query_delegated(&user1.name).unwrap(),
        DelegateResponse {
            start_height: 12345 + 200,
            total_staked: user1_restaked,
            total_earnings: user1_reward1 + user1_reward2,
        }
    );

    let user2_reward2 = reward2_amount
        * Decimal::from_ratio(user2.delegated * Decimal::percent(30), sum_of_weights);
    let user2_restaked = user2.delegated + user2_reward2;
    assert_eq!(
        suite.query_delegated(&user2.name).unwrap(),
        DelegateResponse {
            start_height: 12345 + 1300,
            total_staked: user2_restaked,
            total_earnings: user2_reward2,
        }
    );

    assert_approx_eq!(
        dbg!(suite.query_total_delegated().unwrap().amount.amount.u128()),
        70_000_000_000u128 + reward1_amount.u128() + reward2_amount.u128(),
        "0.00000001"
    );

    suite.advance_height(400);
    suite
        .delegate(&user3.name, coin(user3.delegated.u128(), "ujuno"))
        .unwrap();
    // advance height up to 1000 blocks
    suite.advance_height(600);

    let reward3_amount = suite.query_reward().unwrap().amount;
    assert_eq!(reward3_amount.u128(), 10_966_712u128);
    suite.restake(suite.owner().as_str()).unwrap();

    // user weights
    // user1 = 50_143 * 1.0
    // user2 = 31_256 * 1.0
    // user3 = 35_000 * 0.4 = 14_000

    let sum_of_weights = user1_restaked + user2_restaked + user3.delegated * Decimal::percent(40);

    // user1 reward ratio = 50_143 / 95_399 = 0.5256
    let user1_reward3 = reward3_amount * Decimal::from_ratio(user1_restaked, sum_of_weights);
    assert_eq!(
        suite.query_delegated(&user1.name).unwrap(),
        DelegateResponse {
            start_height: 12345 + 200,
            total_staked: user1_restaked + user1_reward3,
            total_earnings: user1_reward1 + user1_reward2 + user1_reward3,
        }
    );

    // user2 reward ratio = 31_256 / 95_399 = 0.3276
    let user2_reward3 = reward3_amount * Decimal::from_ratio(user2_restaked, sum_of_weights);
    assert_eq!(
        suite.query_delegated(&user2.name).unwrap(),
        DelegateResponse {
            start_height: 12345 + 1300,
            total_staked: user2_restaked + user2_reward3,
            total_earnings: user2_reward2 + user2_reward3,
        }
    );

    // user3 reward ratio = 14_000 / 95_399 = 0.1467
    let user3_reward3 = reward3_amount
        * Decimal::from_ratio(user3.delegated * Decimal::percent(40), sum_of_weights);
    assert_eq!(
        suite.query_delegated(&user3.name).unwrap(),
        DelegateResponse {
            start_height: 12345 + 2400,
            total_staked: user3.delegated + user3_reward3,
            total_earnings: user3_reward3,
        }
    );

    assert_approx_eq!(
        dbg!(suite.query_total_delegated().unwrap().amount.amount.u128()),
        105_000_000_000u128 + reward1_amount.u128() + reward2_amount.u128() + reward3_amount.u128(),
        "0.00000001"
    );
}
