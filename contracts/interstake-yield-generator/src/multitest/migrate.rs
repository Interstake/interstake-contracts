use std::str::FromStr;

use cosmwasm_std::coin;
use cosmwasm_std::coins;
use cosmwasm_std::Addr;
use cosmwasm_std::Decimal;
use cosmwasm_std::Uint128;
use cw_multi_test::Executor;
use interstake_yield_generator_v04::msg as msg_v04;

use crate::msg::MigrateMsg;
use crate::multitest::suite::TWENTY_EIGHT_DAYS;
use crate::state::ClaimDetails;

use super::suite::{SuiteBuilder, VALIDATOR_1};

#[test]
fn test_v04_to_v05() {
    // this test is intended to make sure that the contract upgrade from v0.4 to v0.5 is successful
    // v0.2 to v0.3 failed, because the config was not migrated correctly. see v0.4 changelogs
    let mut suite = SuiteBuilder::new()
        .with_funds("owner", &coins(1000, "ujuno"))
        .with_funds("user", &coins(1500, "ujuno"))
        .build();

    let user = Addr::unchecked("user");
    let owner = suite.owner();

    let treasury = "treasury";
    let transfer_commission = Decimal::from_str("0.002").unwrap();
    let restake_commission = Decimal::from_str("0.01").unwrap();

    // checks if old config is present
    let _res: msg_v04::ConfigResponse = suite
        .app
        .wrap()
        .query_wasm_smart(suite.contract_v04.clone(), &msg_v04::QueryMsg::Config {})
        .unwrap();

    // set old stake for user and owner
    let _res = suite
        .app
        .execute_contract(
            user.clone(),
            suite.contract_v04.clone(),
            &msg_v04::ExecuteMsg::Delegate {},
            &coins(1500, "ujuno"),
        )
        .unwrap();

    let _res = suite
        .app
        .execute_contract(
            owner.clone(),
            suite.contract_v04.clone(),
            &msg_v04::ExecuteMsg::Delegate {},
            &coins(1000, "ujuno"),
        )
        .unwrap();

    // suite.process_staking_queue().unwrap();

    // create claim for user and owner
    let _res = suite
        .app
        .execute_contract(
            user.clone(),
            suite.contract_v04.clone(),
            &msg_v04::ExecuteMsg::Undelegate {
                amount: coin(500, "ujuno"),
            },
            &[],
        )
        .unwrap();

    let _res = suite
        .app
        .execute_contract(
            owner.clone(),
            suite.contract_v04.clone(),
            &msg_v04::ExecuteMsg::Undelegate {
                amount: coin(100, "ujuno"),
            },
            &[],
        )
        .unwrap();

    // suite.process_staking_queue().unwrap();

    // (3) Now migrate, from latest version to latest version
    let _res = suite
        .migrate(
            suite.owner().as_str(),
            suite.contract_v04.clone(),
            suite.contract_code_id,
            &MigrateMsg {
                owner: owner.to_string(),
                treasury: treasury.to_string(),
                staking_addr: VALIDATOR_1.to_string(),
                restake_commission,
                transfer_commission,
                denom: "ujuno".to_string(),
                unbonding_period: Some(28),
                max_entries: Some(7),
            },
        )
        .unwrap();

    suite.query_contract_config(suite.contract.clone()).unwrap();

    // check if old stakes are present
    let owner_stake = suite.query_delegated(owner.clone()).unwrap();
    let user_stake = suite.query_delegated(user.clone()).unwrap();

    assert_eq!(owner_stake.total_staked, Uint128::from(900u128));
    assert_eq!(user_stake.total_staked, Uint128::from(1000u128));

    // check if old claims are present
    let owner_claim = suite.query_claims(owner.to_string()).unwrap();
    let user_claim = suite.query_claims(user.to_string()).unwrap();

    assert_eq!(
        owner_claim[0],
        ClaimDetails {
            amount: coin(100, "ujuno"),
            release_timestamp: cw_utils::Expiration::AtTime(
                suite.app.block_info().time.plus_seconds(TWENTY_EIGHT_DAYS)
            )
        }
    );

    assert_eq!(
        user_claim[0],
        ClaimDetails {
            amount: coin(500, "ujuno"),
            release_timestamp: cw_utils::Expiration::AtTime(
                suite.app.block_info().time.plus_seconds(TWENTY_EIGHT_DAYS)
            )
        }
    );
}
