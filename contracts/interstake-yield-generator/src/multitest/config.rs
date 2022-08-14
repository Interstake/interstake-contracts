use super::suite::SuiteBuilder;

use crate::error::ContractError;

#[test]
fn update_not_owner() {
    let mut suite = SuiteBuilder::new().build();

    let err = suite
        .update_config("random_user", None, None, None)
        .unwrap_err();
    assert_eq!(ContractError::Unauthorized {}, err.downcast().unwrap());
}
