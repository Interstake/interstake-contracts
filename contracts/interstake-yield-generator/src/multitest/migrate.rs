use crate::state::Config;

use super::suite::SuiteBuilder;

#[test]
fn recently_failed_migration() {
    let mut suite = SuiteBuilder::new().build();

    let treasury = "treasury";

    let transfer_commission = "0.002";
    let migration = suite
        .migrate(
            suite.owner().as_str(),
            treasury,
            transfer_commission.to_string(),
        )
        .unwrap();

    let config: Config = suite.query_config().unwrap();
}
