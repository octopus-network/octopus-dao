mod common;

use near_sdk::{
    serde_json::{self, json},
    AccountId,
};
use octopus_council::types::{CouncilChangeHistory, ValidatorStake};

#[tokio::test]
async fn test_sync_staking_amount() -> anyhow::Result<()> {
    //
    let worker = workspaces::sandbox().await?;
    let (_root, council, anchors, _users) =
        common::initialize_contracts_and_users(&worker, 10, 20).await?;
    //
    //
    //
    for anchor in anchors {
        let result = anchor
            .call(&worker, "sync_validator_stakes_of_anchor")
            .gas(200_000_000_000_000)
            .transact()
            .await;
        println!("{:?}", result);
        println!();
    }
    //
    //
    //
    let result = council
        .call(&worker, "get_living_appchain_ids")
        .view()
        .await?
        .json::<Vec<String>>()
        .unwrap();
    println!(
        "Living appchain ids: {}",
        serde_json::to_string::<Vec<String>>(&result).unwrap()
    );
    //
    let result = council
        .call(&worker, "get_council_members")
        .view()
        .await?
        .json::<Vec<AccountId>>()
        .unwrap();
    println!(
        "Result of 'get_council_members': {:?}",
        serde_json::to_string::<Vec<AccountId>>(&result).unwrap()
    );
    //
    let result = council
        .call(&worker, "get_ranked_validator_stakes")
        .args_json(json!( {
            "start_index": 0,
            "quantity": null,
        }))?
        .view()
        .await?
        .json::<Vec<ValidatorStake>>()
        .unwrap();
    println!(
        "Result of 'get_ranked_validator_stakes': {:?}",
        serde_json::to_string::<Vec<ValidatorStake>>(&result).unwrap()
    );
    //
    let result = council
        .call(&worker, "get_council_change_histories")
        .args_json(json!( {
            "start_index": "0",
            "quantity": null,
        }))?
        .view()
        .await?
        .json::<Vec<CouncilChangeHistory>>()
        .unwrap();
    println!(
        "Result of 'get_council_change_histories': {:?}",
        serde_json::to_string::<Vec<CouncilChangeHistory>>(&result).unwrap()
    );
    //
    Ok(())
}
