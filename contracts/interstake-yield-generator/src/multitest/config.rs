use super::suite::{SuiteBuilder, TWENTY_EIGHT_DAYS};

use cosmwasm_std::{coin, Addr, Decimal, StakingMsg, Timestamp};

use crate::contract::utils::compute_redelegate_msgs;
use crate::error::ContractError;
use crate::multitest::suite::{two_false_validators, validator_list};
use crate::state::{Config, TeamCommision};

#[test]
fn update_not_owner() {
    let mut suite = SuiteBuilder::new().build();

    let err = suite
        .update_config("random_user", None, None, None)
        .unwrap_err();
    assert_eq!(ContractError::Unauthorized {}, err.downcast().unwrap());
}

#[test]
fn proper_update() {
    let mut suite = SuiteBuilder::new().build();

    let owner = suite.owner();
    assert_eq!(
        suite.query_config().unwrap(),
        Config {
            owner: owner.clone(),
            team_commision: TeamCommision::None,
            denom: "ujuno".to_owned(),
            unbonding_period: Timestamp::from_seconds(TWENTY_EIGHT_DAYS),
        }
    );

    suite
        .update_config(owner.as_str(), None, None, None)
        .unwrap();
    assert_eq!(
        suite.query_config().unwrap(),
        Config {
            owner: owner.clone(),
            team_commision: TeamCommision::None,
            denom: "ujuno".to_owned(),
            unbonding_period: Timestamp::from_seconds(TWENTY_EIGHT_DAYS),
        }
    );

    let new_team_commision = TeamCommision::Some(Decimal::percent(5));
    suite
        .update_config(owner.as_str(), None, new_team_commision.clone(), None)
        .unwrap();
    assert_eq!(
        suite.query_config().unwrap(),
        Config {
            owner: owner.clone(),
            team_commision: new_team_commision.clone(),
            denom: "ujuno".to_owned(),
            unbonding_period: Timestamp::from_seconds(TWENTY_EIGHT_DAYS),
        }
    );

    let new_unbonding_period = 300_000_000u64;
    suite
        .update_config(owner.as_str(), None, None, new_unbonding_period)
        .unwrap();
    assert_eq!(
        suite.query_config().unwrap(),
        Config {
            owner: owner.clone(),
            team_commision: new_team_commision.clone(),
            denom: "ujuno".to_owned(),
            unbonding_period: Timestamp::from_seconds(new_unbonding_period),
        }
    );

    let new_owner = "new_owner".to_owned();
    suite
        .update_config(owner.as_str(), new_owner.clone(), None, None)
        .unwrap();
    assert_eq!(
        suite.query_config().unwrap(),
        Config {
            owner: Addr::unchecked(new_owner),
            team_commision: new_team_commision,
            denom: "ujuno".to_owned(),
            unbonding_period: Timestamp::from_seconds(new_unbonding_period),
        }
    );

    // confirm that now updating with old owner results in error
    let err = suite
        .update_config(owner.as_str(), None, None, None)
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
fn test_redelegate_to_new_validator_list() {
    let validators = vec![
        (Addr::unchecked("validator1"), Decimal::percent(50)),
        (Addr::unchecked("validator2"), Decimal::percent(40)),
        (Addr::unchecked("validator3"), Decimal::percent(10)),
        (Addr::unchecked("validator4"), Decimal::percent(50)),
    ];

    let msgs = compute_redelegate_msgs("ujuno", validators[..3].to_vec(), validators[1..].to_vec())
        .unwrap();

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
