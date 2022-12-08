use super::suite::{SuiteBuilder, TWENTY_EIGHT_DAYS};

use cosmwasm_std::{coin, Addr, Decimal, StakingMsg, Timestamp, Uint128};

use crate::contract::utils::compute_redelegate_msgs;
use crate::error::ContractError;
use crate::multitest::suite::{two_false_validators, validator_list};
use crate::state::Config;

#[test]
fn update_not_owner() {
    let mut suite = SuiteBuilder::new().build();

    let err = suite
        .update_config("random_user", None, None, None, None)
        .unwrap_err();
    assert_eq!(ContractError::Unauthorized {}, err.downcast().unwrap());
}

#[test]
fn proper_update() {
    let mut suite = SuiteBuilder::new().build();

    let owner = suite.owner();
    let treasury = suite.treasury();
    assert_eq!(
        suite.query_config().unwrap(),
        Config {
            owner: owner.clone(),
            treasury: treasury.clone(),
            team_commission: Decimal::zero(),
            denom: "ujuno".to_owned(),
            unbonding_period: Timestamp::from_seconds(TWENTY_EIGHT_DAYS),
        }
    );

    suite
        .update_config(owner.as_str(), None, None, None, None)
        .unwrap();
    assert_eq!(
        suite.query_config().unwrap(),
        Config {
            owner: owner.clone(),
            treasury: treasury.clone(),
            team_commission: Decimal::zero(),
            denom: "ujuno".to_owned(),
            unbonding_period: Timestamp::from_seconds(TWENTY_EIGHT_DAYS),
        }
    );

    let new_team_commission = Decimal::percent(5);
    suite
        .update_config(
            owner.as_str(),
            None,
            None,
            new_team_commission.clone(),
            None,
        )
        .unwrap();
    assert_eq!(
        suite.query_config().unwrap(),
        Config {
            owner: owner.clone(),
            treasury: treasury.clone(),
            team_commission: new_team_commission.clone(),
            denom: "ujuno".to_owned(),
            unbonding_period: Timestamp::from_seconds(TWENTY_EIGHT_DAYS),
        }
    );

    let new_unbonding_period = 300_000_000u64;
    suite
        .update_config(owner.as_str(), None, None, None, new_unbonding_period)
        .unwrap();
    assert_eq!(
        suite.query_config().unwrap(),
        Config {
            owner: owner.clone(),
            treasury: treasury.clone(),
            team_commission: new_team_commission.clone(),
            denom: "ujuno".to_owned(),
            unbonding_period: Timestamp::from_seconds(new_unbonding_period),
        }
    );

    let new_treasury = "new_treasury".to_owned();
    suite
        .update_config(owner.as_str(), None, new_treasury.clone(), None, None)
        .unwrap();
    assert_eq!(
        suite.query_config().unwrap(),
        Config {
            owner: owner.clone(),
            treasury: Addr::unchecked(new_treasury.clone()),
            team_commission: new_team_commission.clone(),
            denom: "ujuno".to_owned(),
            unbonding_period: Timestamp::from_seconds(new_unbonding_period),
        }
    );

    let new_owner = "new_owner".to_owned();
    suite
        .update_config(owner.as_str(), new_owner.clone(), None, None, None)
        .unwrap();
    assert_eq!(
        suite.query_config().unwrap(),
        Config {
            owner: Addr::unchecked(new_owner),
            treasury: Addr::unchecked(new_treasury),
            team_commission: new_team_commission,
            denom: "ujuno".to_owned(),
            unbonding_period: Timestamp::from_seconds(new_unbonding_period),
        }
    );

    // confirm that now updating with old owner results in error
    let err = suite
        .update_config(owner.as_str(), None, None, None, None)
        .unwrap_err();
    assert_eq!(ContractError::Unauthorized {}, err.downcast().unwrap());
}

#[test]
fn update_validator_list() {
    let mut suite = SuiteBuilder::new().build();

    let owner = suite.owner();

    suite
        .update_validator_list(owner.as_str(), validator_list(1))
        .unwrap();
    assert_eq!(suite.query_validator_list().unwrap(), validator_list(1));

    suite
        .update_validator_list(owner.as_str(), validator_list(2))
        .unwrap();
    assert_eq!(suite.query_validator_list().unwrap(), validator_list(2));

    let err = suite
        .update_validator_list(owner.as_str(), two_false_validators())
        .unwrap_err();
    assert_eq!(
        ContractError::InvalidValidatorList {},
        err.downcast().unwrap()
    );
}

#[test]
fn test_redelegate_replace_single_validator() {
    let validators1 = vec![
        ("validator1".to_owned(), Decimal::percent(50)),
        ("validator2".to_owned(), Decimal::percent(40)),
        ("validator3".to_owned(), Decimal::percent(10)),
    ];
    let validators2 = vec![
        ("validator2".to_owned(), Decimal::percent(40)),
        ("validator3".to_owned(), Decimal::percent(10)),
        ("validator4".to_owned(), Decimal::percent(50)),
    ];

    let msgs =
        compute_redelegate_msgs(Uint128::new(100u128), "ujuno", validators1, validators2).unwrap();

    assert_eq!(msgs.len(), 1);

    assert_eq!(
        msgs[0],
        StakingMsg::Redelegate {
            src_validator: "validator1".to_string(),
            dst_validator: "validator4".to_string(),
            amount: coin(50u128, "ujuno")
        }
    );
}

#[test]
fn test_redelegate_replace_all_validators() {
    let validators1 = vec![
        ("validator1".to_owned(), Decimal::percent(50)),
        ("validator2".to_owned(), Decimal::percent(20)),
        ("validator3".to_owned(), Decimal::percent(30)),
    ];
    let validators2 = vec![
        ("validator4".to_owned(), Decimal::percent(25)),
        ("validator5".to_owned(), Decimal::percent(25)),
        ("validator6".to_owned(), Decimal::percent(50)),
    ];

    let msgs =
        compute_redelegate_msgs(Uint128::new(100u128), "ujuno", validators1, validators2).unwrap();

    assert_eq!(msgs.len(), 4);

    assert_eq!(
        msgs,
        vec![
            StakingMsg::Redelegate {
                src_validator: "validator1".to_string(),
                dst_validator: "validator4".to_string(),
                amount: coin(25u128, "ujuno")
            },
            StakingMsg::Redelegate {
                src_validator: "validator1".to_string(),
                dst_validator: "validator5".to_string(),
                amount: coin(25u128, "ujuno")
            },
            StakingMsg::Redelegate {
                src_validator: "validator2".to_string(),
                dst_validator: "validator6".to_string(),
                amount: coin(20u128, "ujuno")
            },
            StakingMsg::Redelegate {
                src_validator: "validator3".to_string(),
                dst_validator: "validator6".to_string(),
                amount: coin(30u128, "ujuno")
            },
        ]
    );
}

#[test]
fn test_redelegate_update_and_replace_some() {
    let validators1 = vec![
        ("validator1".to_owned(), Decimal::percent(50)), // -10 (reduce)
        ("validator2".to_owned(), Decimal::percent(20)), //  +5 (increase)
        ("validator3".to_owned(), Decimal::percent(15)), // -15 (remove)
        ("validator4".to_owned(), Decimal::percent(10)), //   0 (unchanged)
        ("validator5".to_owned(), Decimal::percent(5)),  //  -5 (reduce)
    ];
    let validators2 = vec![
        ("validator1".to_owned(), Decimal::percent(40)),
        ("validator2".to_owned(), Decimal::percent(25)),
        ("validator4".to_owned(), Decimal::percent(10)),
        ("validator6".to_owned(), Decimal::percent(25)), // +25 (added)
    ];

    let msgs =
        compute_redelegate_msgs(Uint128::new(100u128), "ujuno", validators1, validators2).unwrap();

    //
    assert_eq!(msgs.len(), 4);

    assert_eq!(
        msgs,
        vec![
            StakingMsg::Redelegate {
                src_validator: "validator1".to_string(),
                dst_validator: "validator2".to_string(),
                amount: coin(5u128, "ujuno")
            },
            StakingMsg::Redelegate {
                src_validator: "validator1".to_string(),
                dst_validator: "validator6".to_string(),
                amount: coin(5u128, "ujuno")
            },
            StakingMsg::Redelegate {
                src_validator: "validator3".to_string(),
                dst_validator: "validator6".to_string(),
                amount: coin(15u128, "ujuno")
            },
            StakingMsg::Redelegate {
                src_validator: "validator5".to_string(),
                dst_validator: "validator6".to_string(),
                amount: coin(5u128, "ujuno")
            },
        ]
    );
}

#[test]
fn test_redelegate_remove_some_validators() {
    let validators1 = vec![
        ("validator1".to_owned(), Decimal::percent(50)),
        ("validator2".to_owned(), Decimal::percent(20)),
        ("validator3".to_owned(), Decimal::percent(30)),
    ];
    let validators2 = vec![("validator2".to_owned(), Decimal::percent(100))];

    let msgs =
        compute_redelegate_msgs(Uint128::new(100u128), "ujuno", validators1, validators2).unwrap();

    //
    assert_eq!(msgs.len(), 2);

    assert_eq!(
        msgs,
        vec![
            StakingMsg::Redelegate {
                src_validator: "validator1".to_string(),
                dst_validator: "validator2".to_string(),
                amount: coin(50u128, "ujuno")
            },
            StakingMsg::Redelegate {
                src_validator: "validator3".to_string(),
                dst_validator: "validator2".to_string(),
                amount: coin(30u128, "ujuno")
            },
        ]
    );
}
