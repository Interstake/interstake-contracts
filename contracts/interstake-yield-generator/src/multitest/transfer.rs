use super::suite::SuiteBuilder;

use cosmwasm_std::{coin, coins, Decimal, Uint128};
use cw_utils::Expiration;

use crate::{
    msg::{DelegateResponse, TotalDelegatedResponse},
    multitest::suite::{validator_list, TWENTY_EIGHT_DAYS},
    state::Config,
    ContractError,
};
use test_case::test_case;

#[test_case(1; "single_validator")]
#[test_case(2; "two_validators")]
fn delegate_and_transfer(i: u32) {
    let validators = validator_list(i);

    let user1 = ("user1", 50_000_000u128);
    let user2 = "user2";
    let mut suite = SuiteBuilder::new()
        .with_funds(user1.0, &coins(user1.1, "ujuno"))
        .build();

    suite
        .update_validator_list(suite.owner().as_str(), validators)
        .unwrap();

    suite.delegate(user1.0, coin(user1.1, "ujuno")).unwrap();

    assert_eq!(
        suite.query_delegated(user1.0).unwrap(),
        DelegateResponse {
            start_height: 12345,
            total_staked: user1.1.into(),
            total_earnings: Uint128::zero(),
        }
    );
    assert_eq!(
        suite.query_total_delegated().unwrap(),
        TotalDelegatedResponse {
            amount: coin(user1.1, "ujuno")
        }
    );

    suite.advance_height(500);
    suite.restake(suite.owner().as_str()).unwrap();

    suite
        .transfer(user1.0, user2, Uint128::new(30_000_000u128), None)
        .unwrap();
    assert_eq!(
        suite.query_delegated(user1.0).unwrap(),
        DelegateResponse {
            start_height: 12345,
            total_staked: Uint128::new(20_003_012u128),
            total_earnings: Uint128::new(3_012u128),
        }
    );
    assert_eq!(
        suite.query_delegated(user2).unwrap(),
        DelegateResponse {
            start_height: 12345 + 500,
            total_staked: Uint128::new(30_000_000u128),
            total_earnings: Uint128::zero(),
        }
    );
    assert_eq!(
        suite.query_total_delegated().unwrap(),
        TotalDelegatedResponse {
            amount: coin(50_003_012u128, "ujuno")
        }
    );
}

#[test]
fn transfer_with_commission() {
    let user1 = ("user1", 50_000_000u128);
    let user2 = "user2";
    let mut suite = SuiteBuilder::new()
        .with_funds(user1.0, &coins(user1.1, "ujuno"))
        .with_restake_commission(Decimal::percent(10))
        .build();

    let config: Config = suite.query_config().unwrap();
    assert_eq!(config.restake_commission, Decimal::percent(10));

    suite.delegate(user1.0, coin(user1.1, "ujuno")).unwrap();

    assert_eq!(
        suite.query_delegated(user1.0).unwrap(),
        DelegateResponse {
            start_height: 12345,
            total_staked: user1.1.into(),
            total_earnings: Uint128::zero(),
        }
    );
    assert_eq!(
        suite.query_total_delegated().unwrap(),
        TotalDelegatedResponse {
            amount: coin(user1.1, "ujuno")
        }
    );
    suite.advance_height(500);
    suite.restake(suite.owner().as_str()).unwrap();

    suite
        .transfer(user1.0, user2, Uint128::new(30_000_000u128), None)
        .unwrap();
    assert_eq!(
        suite.query_delegated(user1.0).unwrap(),
        DelegateResponse {
            start_height: 12345,
            total_staked: Uint128::new(20_002_711u128),
            total_earnings: Uint128::new(2_711u128),
        }
    );
    assert_eq!(
        suite.query_delegated(user2).unwrap(),
        DelegateResponse {
            start_height: 12345 + 500,
            total_staked: Uint128::new(30_000_000u128 - 3_000_000u128),
            total_earnings: Uint128::zero(),
        }
    );
    assert_eq!(
        suite.query_delegated(suite.treasury()).unwrap(),
        DelegateResponse {
            start_height: 12345 + 500,
            total_staked: Uint128::new(3_000_000u128),
            total_earnings: Uint128::zero(),
        }
    );
    assert_eq!(
        suite.query_total_delegated().unwrap(),
        TotalDelegatedResponse {
            amount: coin(50_002_711u128, "ujuno")
        }
    );
}

#[test]
fn transfer_with_allowed_address() {
    let user1 = ("user1", 50_000_000u128);
    let user2 = "user2";
    let allowed1 = "allowed_address1";
    let allowed2 = "allowed_address2";
    let mut suite = SuiteBuilder::new()
        .with_funds(user1.0, &coins(user1.1, "ujuno"))
        .with_restake_commission(Decimal::percent(10))
        .build();

    let config: Config = suite.query_config().unwrap();
    assert_eq!(config.restake_commission, Decimal::percent(10));

    suite.delegate(user1.0, coin(user1.1, "ujuno")).unwrap();
    suite.advance_height(500);
    suite.restake(suite.owner().as_str()).unwrap();

    let err: ContractError = suite
        .transfer(
            user1.0,
            allowed1,
            Uint128::new(30_000_000u128),
            Some(allowed1.to_string()),
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(
        err,
        ContractError::CommissionAddressNotFound {
            address: allowed1.to_string()
        }
    );

    suite
        .update_allowed_addr(suite.owner().as_str(), allowed1, None)
        .unwrap();

    let expiration = suite.query_allowed_addr(allowed1).unwrap();
    assert_eq!(
        expiration,
        Expiration::AtTime(suite.app.block_info().time.plus_seconds(TWENTY_EIGHT_DAYS))
    );
    suite
        .update_allowed_addr(
            suite.owner().as_str(),
            allowed2,
            Some(
                suite
                    .app
                    .block_info()
                    .time
                    .plus_seconds(TWENTY_EIGHT_DAYS - 1)
                    .seconds(),
            ),
        )
        .unwrap_err();

    suite
        .update_allowed_addr(suite.owner().as_str(), allowed2, None)
        .unwrap();

    suite
        .transfer(
            user1.0,
            user2,
            30_000_000u128.into(),
            Some(allowed1.to_string()),
        )
        .unwrap();

    let user_1_delegated = suite.query_delegated(user1.0).unwrap();
    let user_2_delegated = suite.query_delegated(user2).unwrap();
    let treasury_delegated = suite.query_delegated(suite.treasury()).unwrap();
    let allowed1_delegated = suite.query_delegated(allowed1).unwrap();

    assert_eq!(user_1_delegated.total_staked.u128(), 20_002_711u128);
    assert_eq!(user_2_delegated.total_staked.u128(), 27_000_000u128);
    assert_eq!(treasury_delegated.total_staked.u128(), 1_500_000u128);
    assert_eq!(allowed1_delegated.total_staked.u128(), 1_500_000u128);

    suite
        .remove_allowed_addr(suite.owner().as_str(), allowed2)
        .unwrap();

    let err: ContractError = suite
        .transfer(
            user2,
            user1.0,
            20_000_000u128.into(),
            Some(allowed2.to_string()),
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(
        err,
        ContractError::CommissionAddressNotFound {
            address: allowed2.to_string()
        }
    );
}
