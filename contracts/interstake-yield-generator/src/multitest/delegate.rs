use super::suite::SuiteBuilder;

use cosmwasm_std::{coin, coins, Addr, Decimal, Uint128};

use crate::error::ContractError;
use crate::msg::DelegateResponse;
use crate::state::{Config, TeamCommision};

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

    let owner = suite.owner();

    //suite.restake(owner.as_str()).unwrap();
}
