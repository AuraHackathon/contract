use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use nft::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, TokenTraitResponse, HouseInfoResponse};
use nft::state::{Config, HouseBuilding, HouseInfo, Model};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(Config), &out_dir);
    export_schema(&schema_for!(HouseBuilding), &out_dir);
    export_schema(&schema_for!(HouseInfo), &out_dir);
    export_schema(&schema_for!(Model), &out_dir);
    export_schema(&schema_for!(TokenTraitResponse), &out_dir);
    export_schema(&schema_for!(HouseInfoResponse), &out_dir);

}
