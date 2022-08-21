use super::suite::SuiteBuilder;

use cosmwasm_std::{Addr, Decimal};

use crate::error::ContractError;
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
    let staking_addr = suite.staking();
    assert_eq!(
        suite.query_config().unwrap(),
        Config {
            owner: owner.clone(),
            staking_addr,
            team_commision: TeamCommision::None,
            denom: "juno".to_owned(),
        }
    );

    let new_staking_addr = "new_staking_addr".to_owned();
    suite
        .update_config(owner.as_str(), None, Some(new_staking_addr.clone()), None)
        .unwrap();
    assert_eq!(
        suite.query_config().unwrap(),
        Config {
            owner: owner.clone(),
            staking_addr: new_staking_addr.clone(),
            team_commision: TeamCommision::None,
            denom: "juno".to_owned(),
        }
    );

    let new_team_commision = TeamCommision::Some(Decimal::percent(5));
    suite
        .update_config(owner.as_str(), None, None, new_team_commision.clone())
        .unwrap();
    assert_eq!(
        suite.query_config().unwrap(),
        Config {
            owner: owner.clone(),
            staking_addr: new_staking_addr.clone(),
            team_commision: new_team_commision.clone(),
            denom: "juno".to_owned(),
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
            staking_addr: new_staking_addr,
            team_commision: new_team_commision,
            denom: "juno".to_owned(),
        }
    );

    // confirm that now updating with old owner results in error
    let err = suite
        .update_config(owner.as_str(), None, None, None)
        .unwrap_err();
    assert_eq!(ContractError::Unauthorized {}, err.downcast().unwrap());
}
