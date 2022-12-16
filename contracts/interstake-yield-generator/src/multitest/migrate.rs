use std::str::FromStr;

use cosmwasm_std::Decimal;
use interstake_yield_generator_v03::msg::MigrateMsg as MigrateMsgV03;

use crate::{msg::MigrateMsg, state::Config};

use super::suite::{SuiteBuilder, VALIDATOR_1};

#[test]
fn recently_failed_migration() {
    let mut suite = SuiteBuilder::new().build();
    let owner = suite.owner();

    let treasury = "treasury";

    let transfer_commission = "0.002";
    let _res = suite
        .migrate(
            suite.owner().as_str(),
            treasury,
            transfer_commission.to_string(),
            MigrateMsgV03 {
                treasury: owner.to_string(),
                transfer_commission: Decimal::from_str("0.002"),
            },
        )
        .unwrap();

    // we dont know why this errors, but it does.
    let err = suite.query_config().unwrap_err();

    let _res = suite.migrate(
        suite.owner().as_str(),
        treasury,
        "0.002".to_string(),
        MigrateMsg {
            owner: owner.to_string(),
            treasury: treasury.to_string(),
            staking_addr: VALIDATOR_1.to_string(),
            restake_commission: Decimal::from_str("0.01"),
            transfer_commission: Decimal::from_str("0.002"),
            denom: "ujuno".to_string(),
            unbonding_period: None,
        },
    );

    let config: Config = suite.query_config().unwrap();
}
