use super::suite::SuiteBuilder;

use cosmwasm_std::{coin, coins, Addr, Decimal};

use crate::error::ContractError;
use crate::state::{Config, TeamCommision};

#[test]
fn one_user() {
    let user = "user";
    let mut suite = SuiteBuilder::new()
        .with_funds(user, &coins(100, "juno"))
        .build();

    suite.delegate(user, coin(100, "juno")).unwrap();
}
