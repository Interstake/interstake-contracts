use super::suite::{SuiteBuilder, TWENTY_EIGHT_DAYS};

use crate::msg::DelegateResponse;
use crate::multitest::suite::validator_list;
use crate::state::ClaimDetails;
use crate::{error::ContractError, multitest::suite::FOUR_DAYS};
use cosmwasm_std::{coin, coins, Addr, Uint128};
use cw_utils::{Duration, Expiration};
use test_case::test_case;

#[test_case(1; "single_validator")]
#[test_case(2; "two_validators")]
#[test_case(8; "eight_validators")]
fn undelegate_without_delegation(i: u32) {
    let mut suite = SuiteBuilder::new().with_multiple_validators(i).build();
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
#[test_case(1; "single_validator")]
#[test_case(2; "two_validators")]
#[test_case(8; "eight validators")]
fn create_basic_claim(i: u32) {
    let validators = validator_list(i);
    let user = "user";
    let mut suite = SuiteBuilder::new()
        .with_multiple_validators(i)
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
    suite.batch_unbond(user).unwrap();
    let current_time = suite.app.block_info().time;
    assert_eq!(
        suite.query_claims(user).unwrap(),
        vec![ClaimDetails {
            amount: coin(100, "ujuno"),
            release_timestamp: Expiration::AtTime(current_time.plus_seconds(TWENTY_EIGHT_DAYS))
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

    let err = suite.batch_unbond(user).unwrap_err();
    assert_eq!(
        ContractError::UnbondingCooldownNotExpired {
            min_cooldown: Duration::Time(TWENTY_EIGHT_DAYS.saturating_div(7u64)),
            latest_unbonding: Expiration::AtTime(suite.app.block_info().time)
        },
        err.downcast().unwrap()
    );

    suite.advance_time(FOUR_DAYS);
    suite.batch_unbond(user).unwrap();
}

#[test_case(1; "single_validator")]
#[test_case(2; "two_validators")]
#[test_case(8; "eight_validators")]
fn undelegate_part_of_tokens(i: u32) {
    let validators = validator_list(i);
    let user = "user";
    let mut suite = SuiteBuilder::new()
        .with_funds(user, &coins(1000, "ujuno"))
        .with_multiple_validators(i)
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
    suite.batch_unbond(user).unwrap();
    let current_time = suite.app.block_info().time;
    let first_unbonding = Expiration::AtTime(current_time.plus_seconds(TWENTY_EIGHT_DAYS));
    assert_eq!(
        suite.query_claims(user).unwrap(),
        vec![ClaimDetails {
            amount: coin(700, "ujuno"),
            release_timestamp: first_unbonding
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

    // undelegate the half in multiple steps
    suite.advance_time(TWENTY_EIGHT_DAYS / 2);
    let current_time = suite.app.block_info().time;
    let second_unbonding = Expiration::AtTime(current_time.plus_seconds(TWENTY_EIGHT_DAYS));
    suite.undelegate(user, coin(60, "ujuno")).unwrap();
    suite.undelegate(user, coin(40, "ujuno")).unwrap();
    suite.undelegate(user, coin(20, "ujuno")).unwrap();
    suite.undelegate(user, coin(40, "ujuno")).unwrap();
    // this will create one claim for 150 ujuno
    suite.batch_unbond(user).unwrap();
    assert_eq!(
        suite.query_claims(user).unwrap(),
        vec![
            ClaimDetails {
                amount: coin(700, "ujuno"),
                release_timestamp: first_unbonding
            },
            ClaimDetails {
                amount: coin(160, "ujuno"),
                release_timestamp: second_unbonding
            }
        ]
    );

    suite.advance_time(TWENTY_EIGHT_DAYS / 4);
    let current_time = suite.app.block_info().time;
    let third_unbonding = Expiration::AtTime(current_time.plus_seconds(TWENTY_EIGHT_DAYS));
    suite.undelegate(user, coin(80, "ujuno")).unwrap();
    suite.batch_unbond(user).unwrap();
    // suite.undelegate(user, coin(60, "ujuno")).unwrap();
    // suite.batch_unbond(user).unwrap_err(); // cooldown not expired

    suite.advance_time(TWENTY_EIGHT_DAYS / 4);
    let _current_time = suite.app.block_info().time;
    // let fourth_unbonding = Expiration::AtTime(current_time.plus_seconds(TWENTY_EIGHT_DAYS));
    suite.batch_unbond(user).unwrap();
    assert_eq!(
        suite.query_claims(user).unwrap(),
        vec![
            ClaimDetails {
                amount: coin(700, "ujuno"),
                release_timestamp: first_unbonding
            },
            ClaimDetails {
                amount: coin(160, "ujuno"),
                release_timestamp: second_unbonding
            },
            ClaimDetails {
                amount: coin(80, "ujuno"),
                release_timestamp: third_unbonding
            },
            // ClaimDetails {
            //     amount: coin(60, "ujuno"),
            //     release_timestamp: fourth_unbonding
            // }
        ]
    );

    // mature all four claims
    suite.advance_time(TWENTY_EIGHT_DAYS);
    suite.process_staking_queue().unwrap();

    // TODO: This fails due to rounding errors at delegate and undelegate msgs.
    // suite.claim(user).unwrap();
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
    suite.batch_unbond(user).unwrap();
    let current_time = suite.app.block_info().time;
    assert_eq!(
        suite.query_claims(user).unwrap(),
        vec![ClaimDetails {
            amount: coin(500, "ujuno"),
            release_timestamp: Expiration::AtTime(current_time.plus_seconds(TWENTY_EIGHT_DAYS))
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
    suite.batch_unbond(user).unwrap();

    let current_time = suite.app.block_info().time;
    assert_eq!(
        suite.query_claims(user).unwrap(),
        vec![ClaimDetails {
            amount: coin(500, "ujuno"),
            release_timestamp: Expiration::AtTime(current_time.plus_seconds(TWENTY_EIGHT_DAYS))
        }]
    );

    // advance time to create some delegation with other timestamp
    suite.advance_time(TWENTY_EIGHT_DAYS / 2);
    suite.delegate(user, coin(700, "ujuno")).unwrap();
    suite.restake("owner").unwrap();
    suite.undelegate(user, coin(700, "ujuno")).unwrap();
    suite.batch_unbond(user).unwrap();

    // nothing happens
    let current_time = suite.app.block_info().time;
    suite.process_staking_queue().unwrap();
    suite.claim(user).unwrap();
    assert_eq!(
        suite.query_claims(user).unwrap(),
        vec![
            ClaimDetails {
                amount: coin(500, "ujuno"),
                release_timestamp: Expiration::AtTime(
                    current_time.plus_seconds(TWENTY_EIGHT_DAYS / 2)
                )
            },
            ClaimDetails {
                amount: coin(700, "ujuno"),
                release_timestamp: Expiration::AtTime(current_time.plus_seconds(TWENTY_EIGHT_DAYS))
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
            release_timestamp: Expiration::AtTime(current_time.plus_seconds(TWENTY_EIGHT_DAYS / 2))
        }]
    );
}

#[test_case(1, 10; "single_validator ten users")]
#[test_case(5, 10; "5 validator ten users")]
#[test_case(1, 13; "1 validator thirdteen users")]
#[test_case(5, 13; "5 validator thirdteen users")]
fn undelegate_multiple_users_reconcile(i: u32, n_users: u32) {
    let validators = validator_list(i);

    let users = (0..n_users).map(|i| format!("user{i}")).collect::<Vec<_>>();

    let all_funds = users
        .iter()
        .map(|user| (Addr::unchecked(user), coins(1000, "ujuno")))
        .collect::<Vec<_>>();

    let mut suite = SuiteBuilder::new()
        .with_multiple_validators(i)
        .with_multiple_funds(&all_funds)
        .build();
    suite.update_validator_list("owner", validators).unwrap();

    for user in users.iter() {
        suite.delegate(user, coin(100, "ujuno")).unwrap();
    }

    for user in users.iter() {
        suite.undelegate(user, coin(100, "ujuno")).unwrap();
    }

    // if n_users is more then 7, this should still allow people to claim anad not trigger maxEntries error
    for user in users.iter() {
        assert_eq!(suite.query_pending_claims(user).unwrap(), Uint128::new(100),);
    }
    suite.batch_unbond("owner").unwrap();

    let current_time = suite.app.block_info().time;
    for user in users.iter() {
        assert_eq!(
            suite.query_claims(user).unwrap(),
            vec![ClaimDetails {
                amount: coin(100, "ujuno"),
                release_timestamp: Expiration::AtTime(current_time.plus_seconds(TWENTY_EIGHT_DAYS))
            }]
        );
    }
}

#[test_case(1, 1; "single_validator one user")]
#[test_case(2, 1; "two_validators one user")]
#[test_case(5, 1; "five_validators one user")]
#[test_case(1, 5; "single_validator five users")]
#[test_case(2, 5; "two_validators five users")]
fn undelegate_all(i: u32, n_users: u32) {
    let validators = validator_list(i);

    let users = (0..n_users).map(|i| format!("user{i}")).collect::<Vec<_>>();

    let all_funds = users
        .iter()
        .map(|user| (Addr::unchecked(user), coins(1000, "ujuno")))
        .collect::<Vec<_>>();

    let mut suite = SuiteBuilder::new()
        .with_multiple_validators(i)
        .with_multiple_funds(&all_funds)
        .build();

    suite
        .update_validator_list(suite.owner().as_str(), validators)
        .unwrap();

    for user in &users {
        suite.delegate(user, coin(700, "ujuno")).unwrap();

        let res: DelegateResponse = suite.query_delegated(user).unwrap();
        assert_eq!(res.total_staked, Uint128::new(700));
    }

    // all funds should now be fully delegated
    suite.advance_time(TWENTY_EIGHT_DAYS);

    let res = suite.undelegate_all(users[0].as_str()).unwrap_err();
    assert_eq!(ContractError::Unauthorized {}, res.downcast().unwrap());

    let res = suite.undelegate_all(suite.owner().as_str());
    assert!(res.is_ok(), "undelegate_all by owner failed: {res:?}");

    // see if all the delegations are actually gone
    let stake = suite.query_all_delegations().unwrap();
    let total_delegation = stake.iter().map(|d| d.amount.amount.u128()).sum::<u128>();
    assert_eq!(total_delegation, 0u128);

    // all previously delegated funds should be in the claim_details
    for user in &users {
        let current_time = suite.app.block_info().time;
        assert_eq!(
            suite.query_claims(user.as_str()).unwrap(),
            vec![ClaimDetails {
                amount: coin(700, "ujuno"),
                release_timestamp: Expiration::AtTime(current_time.plus_seconds(TWENTY_EIGHT_DAYS))
            }]
        );
    }

    suite.advance_time(TWENTY_EIGHT_DAYS);
    suite.process_staking_queue().unwrap();

    for user in &users {
        if let Err(err) = suite.claim(user.as_str()) {
            panic!("claim failed: {err:?}");
        }
    }

    // all funds should now be undelegated
    for user in &users {
        assert_eq!(
            suite
                .app
                .wrap()
                .query_balance(user.as_str(), "ujuno")
                .unwrap()
                .amount,
            Uint128::new(1000)
        );
    }
}
