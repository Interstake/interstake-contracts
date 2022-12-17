use std::str::FromStr;

use cosmwasm_std::Decimal;
use interstake_yield_generator_v03::msg::MigrateMsg as MigrateMsgV03;

use crate::msg::MigrateMsg;

use super::suite::{SuiteBuilder, VALIDATOR_1};

#[test]
fn recently_failed_migration() {
    // This test is a reproduction of a bug that happened on mainnet and
    // it includes a potential fix, which includes overwriting the config completely.
    let mut suite = SuiteBuilder::new().build();
    let owner = suite.owner();

    let treasury = "treasury";
    let transfer_commission = Decimal::from_str("0.002").unwrap();
    let restake_commission = Decimal::from_str("0.01").unwrap();

    // (1) First we migrate to v03. this is what happened on mainnet.
    let _res = suite
        .migrate(
            suite.owner().as_str(),
            suite.contract_v02.clone(),
            suite.contract_v03_code_id,
            &MigrateMsgV03 {
                treasury: owner.to_string(),
                transfer_commission,
            },
        )
        .unwrap();

    // Migration is succesful, but querying the config fails (on mainnet).
    suite
        .query_contract_config(suite.contract_v02.clone())
        .unwrap_err();

    // (2) Now er try to re-migrate, from V0.3.0 to the new version( probably v0.3.1).
    let _res = suite
        .migrate(
            suite.owner().as_str(),
            suite.contract_v03.clone(),
            suite.contract_code_id,
            &MigrateMsg {
                owner: owner.to_string(),
                treasury: treasury.to_string(),
                staking_addr: VALIDATOR_1.to_string(),
                restake_commission,
                transfer_commission,
                denom: "ujuno".to_string(),
                unbonding_period: None,
            },
        )
        .unwrap();

    // the config should now be correct.
    suite
        .query_contract_config(suite.contract_v03.clone())
        .unwrap();

    // (3) Now er try to re-migrate, from latest version to latest version
    let _res = suite
        .migrate(
            suite.owner().as_str(),
            suite.contract.clone(),
            suite.contract_code_id,
            &MigrateMsg {
                owner: owner.to_string(),
                treasury: treasury.to_string(),
                staking_addr: VALIDATOR_1.to_string(),
                restake_commission,
                transfer_commission,
                denom: "ujuno".to_string(),
                unbonding_period: None,
            },
        )
        .unwrap();

    suite.query_contract_config(suite.contract.clone()).unwrap();
}
