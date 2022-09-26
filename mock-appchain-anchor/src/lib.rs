use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env,
    json_types::U128,
    log, near_bindgen,
    serde::{Deserialize, Serialize},
    AccountId, Gas, PanicOnDefault, Promise,
};
use std::{ops::Mul, str::FromStr};

const T_GAS_FOR_SYNC_STAKING_AMOUNT_TO_COUNCIL: u64 = 150;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    appchain_id: String,
    appchain_registry: AccountId,
    validator_accounts: Vec<AccountId>,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(appchain_id: String, validator_accounts: Vec<AccountId>) -> Self {
        let account_id = String::from(env::current_account_id().as_str());
        let parts = account_id.split(".").collect::<Vec<&str>>();
        assert!(
            parts.len() > 2,
            "This contract must be deployed as a sub-account of octopus appchain registry.",
        );
        let (_first, second) = account_id.split_once(".").unwrap();
        Self {
            appchain_id,
            appchain_registry: AccountId::from_str(second).unwrap(),
            validator_accounts,
        }
    }
    ///
    pub fn sync_validator_stakes_of_anchor(&mut self) {
        #[derive(Serialize, Deserialize, Clone)]
        #[serde(crate = "near_sdk::serde")]
        struct StakingRecord {
            pub validator_id: AccountId,
            pub total_stake: U128,
        }
        #[derive(Serialize, Deserialize, Clone)]
        #[serde(crate = "near_sdk::serde")]
        struct Input {
            pub stake_records: Vec<StakingRecord>,
        }
        let mut stake_amount = get_random_u32();
        let args = Input {
            stake_records: self
                .validator_accounts
                .iter()
                .map(|v| {
                    stake_amount += 1;
                    StakingRecord {
                        validator_id: v.clone(),
                        total_stake: U128::from(stake_amount as u128),
                    }
                })
                .collect::<Vec<StakingRecord>>(),
        };
        log!(
            "Calling param of 'sync_validator_stakes_of_anchor': '{}'",
            near_sdk::serde_json::to_string(&args).unwrap()
        );
        let args = near_sdk::serde_json::to_vec(&args)
            .expect("Failed to serialize the cross contract args using JSON.");
        let contract_account =
            AccountId::from_str(format!("octopus-council.{}", self.appchain_registry).as_str())
                .unwrap();
        Promise::new(contract_account).function_call(
            "sync_validator_stakes_of_anchor".to_string(),
            args,
            0,
            Gas::ONE_TERA.mul(T_GAS_FOR_SYNC_STAKING_AMOUNT_TO_COUNCIL),
        );
    }
}

fn get_random_u32() -> u32 {
    let seed = env::random_seed();
    let mut result: u32 = 0;
    for i in 0..4 as u32 {
        result += *seed.get(i as usize).unwrap() as u32 * 16_u32.pow(3 - i);
    }
    result
}
