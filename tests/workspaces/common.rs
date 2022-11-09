use near_sdk::serde_json::json;
use near_units::parse_near;
use workspaces::network::Sandbox;
use workspaces::{Account, Contract, Worker};

const BASE_APPCHAIN_ID: &str = "appchain";
const BASE_VALIDATOR_ID: &str = "validator";

pub async fn initialize_contracts_and_users(
    worker: &Worker<Sandbox>,
    appchain_count: u32,
    validator_count_per_appchain: u32,
) -> anyhow::Result<(Account, Contract, Vec<Contract>, Vec<Account>)> {
    let root = worker.root_account().unwrap();
    let mut users: Vec<Account> = Vec::new();
    let mut anchors: Vec<Contract> = Vec::new();
    //
    // initialize users' accounts
    //
    for index in 1..appchain_count * validator_count_per_appchain + 1 {
        let account_id = format!("{}{}", BASE_VALIDATOR_ID, index);
        let account = root
            .create_subaccount(&account_id)
            .initial_balance(parse_near!("10 N"))
            .transact()
            .await?
            .unwrap();
        users.push(account);
    }
    //
    // appchain registry contract
    //
    let appchain_registry = root
        .create_subaccount("appchain_registry")
        .initial_balance(parse_near!("200 N"))
        .transact()
        .await?
        .unwrap();
    //
    // dao contract
    //
    let dao_contract = root
        .create_subaccount("octopus-dao")
        .initial_balance(parse_near!("10 N"))
        .transact()
        .await?
        .unwrap();
    //
    // deploy octopus council contract
    //
    let council_keeper = appchain_registry
        .create_subaccount("council-keeper")
        .initial_balance(parse_near!("10 N"))
        .transact()
        .await?
        .unwrap();
    let council_keeper = council_keeper
        .deploy(&std::fs::read(format!("res/council_keeper.wasm"))?)
        .await?
        .unwrap();
    assert!(council_keeper
        .call("new")
        .args_json(json!({
            "max_number_of_council_members": 3,
            "dao_contract_account": dao_contract.id().to_string(),
        }))
        .gas(300_000_000_000_000)
        .transact()
        .await
        .unwrap()
        .is_success());
    //
    // deploy appchain anchor contract
    //
    for index in 1..appchain_count as usize + 1 {
        let appchain_id = format!("{}{}", BASE_APPCHAIN_ID, index);
        let appchain_anchor = appchain_registry
            .create_subaccount(&appchain_id)
            .initial_balance(parse_near!("5 N"))
            .transact()
            .await?
            .unwrap();
        let appchain_anchor = appchain_anchor
            .deploy(&std::fs::read(format!("res/mock_appchain_anchor.wasm"))?)
            .await?
            .unwrap();
        let validator_accounts: Vec<String> = users[((index - 1)
            * validator_count_per_appchain as usize)
            ..(index * validator_count_per_appchain as usize)]
            .to_vec()
            .iter()
            .map(|account| account.id().to_string())
            .collect();
        assert!(root
            .call(appchain_anchor.id(), "new")
            .args_json(json!({
                "appchain_id": appchain_id,
                "validator_accounts": validator_accounts,
            }))
            .gas(300_000_000_000_000)
            .transact()
            .await
            .unwrap()
            .is_success());
        anchors.push(appchain_anchor);
    }
    //
    Ok((root, council_keeper, anchors, users))
}
