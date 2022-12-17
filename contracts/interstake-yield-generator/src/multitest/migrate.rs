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
    let transfer_commission = Decimal::from_str("0.002").unwrap();
    let restake_commission = Decimal::from_str("0.01").unwrap();

    // First we migrate to v03.
    let _res = suite
        .migrate(
            suite.owner().as_str(),
            suite.contract_v03_code_id,
            treasury,
            transfer_commission.to_string(),
            &MigrateMsgV03 {
                treasury: owner.to_string(),
                transfer_commission,
            },
        )
        .unwrap();

    // Migration is succesful, but querying the config fails (on mainnet).
    let err = suite.query_config().unwrap_err();

    // Now er try to re-migrate, from V0.03 to the new version( probably v0.3.1).
    let _res = suite.migrate(
        suite.owner().as_str(),
        suite.contract_code_id,
        treasury,
        "0.002".to_string(),
        &MigrateMsg {
            owner: owner.to_string(),
            treasury: treasury.to_string(),
            staking_addr: VALIDATOR_1.to_string(),
            restake_commission,
            transfer_commission,
            denom: "ujuno".to_string(),
            unbonding_period: None,
        },
    );

    // the config should now be correct.
    // but the config somehow is still broken.
    let config: Config = suite.query_config().unwrap();
}
