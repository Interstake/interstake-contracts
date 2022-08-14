use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};
use interstake_yield_generator::msg::{
    DelegateResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, TotalDelegatedResponse,
};
use interstake_yield_generator::state::{ClaimDetails, Config, Stake, StakeDetails, TeamCommision};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(DelegateResponse), &out_dir);
    export_schema(&schema_for!(TotalDelegatedResponse), &out_dir);
    export_schema(&schema_for!(Config), &out_dir);
    export_schema(&schema_for!(Stake), &out_dir);
    export_schema(&schema_for!(StakeDetails), &out_dir);
    export_schema(&schema_for!(MigrateMsg), &out_dir);
    export_schema(&schema_for!(TeamCommision), &out_dir);
    export_schema(&schema_for!(ClaimDetails), &out_dir);
}
