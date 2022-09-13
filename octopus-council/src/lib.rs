mod ranked_lookup_array;
mod types;
mod views;

use near_contract_standards::upgrade::Ownable;
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::{LookupMap, UnorderedMap},
    env,
    json_types::U128,
    near_bindgen, AccountId, BorshStorageKey, Gas, PanicOnDefault, Promise,
};
use ranked_lookup_array::{MultiTxsOperationProcessingResult, RankValueHolder, RankedLookupArray};
use std::{collections::HashMap, ops::Mul, str::FromStr};
use types::{ValidatorStake, ValidatorStakeRecord};

/// Constants for gas.
const T_GAS_CAP_FOR_MULTI_TXS_PROCESSING: u64 = 150;
const T_GAS_FOR_BURN_WRAPPED_APPCHAIN_TOKEN: u64 = 35;

/// Storage keys for collections of sub-struct in main contract
#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKey {
    ValidatorStakes,
    OrderedValidators,
    ValidatorStakeInAppchains(AccountId),
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct InternalValidatorStake {
    //
    validator_id: AccountId,
    // key: appchain id, value: total stake in the appchain anchor
    stake_in_appchains: UnorderedMap<String, U128>,
    // total stake in all appchain anchors
    total_stake: U128,
    // the rank of the validator in all validators
    overall_rank: u32,
}

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct OctopusCouncil {
    //
    owner: AccountId,
    //
    appchain_registry_account: AccountId,
    //
    living_appchain_ids: Vec<String>,
    //
    validator_stakes: LookupMap<AccountId, InternalValidatorStake>,
    //
    ranked_validators: RankedLookupArray<AccountId>,
    //
    max_number_of_council_members: u32,
}

#[near_bindgen]
impl OctopusCouncil {
    #[init]
    pub fn new(max_number_of_council_members: u32) -> Self {
        assert!(!env::state_exists(), "The contract is already initialized.");
        let account_id = String::from(env::current_account_id().as_str());
        let (_first, second) = account_id.split_once(".").expect(
            "This contract must be deployed as a sub-account of octopus appchain registry.",
        );
        Self {
            owner: env::current_account_id(),
            appchain_registry_account: AccountId::from_str(second).unwrap(),
            living_appchain_ids: Vec::new(),
            validator_stakes: LookupMap::new(StorageKey::ValidatorStakes),
            ranked_validators: RankedLookupArray::new(StorageKey::OrderedValidators),
            max_number_of_council_members,
        }
    }
    // Assert that the contract is called by appchain registry contract.
    fn assert_registry(&self) {
        assert_eq!(
            env::predecessor_account_id(),
            self.appchain_registry_account,
            "This function can only be called by appchain registry contract."
        );
    }
    // Assert that the contract is called by an appchain anchor contract and
    // return the appchain id corresponding to the predecessor account
    fn assert_living_anchor(&self) -> String {
        let account_id = String::from(env::predecessor_account_id().as_str());
        let (first, second) = account_id.split_once(".").expect(
            "This contract must be deployed as a sub-account of octopus appchain registry.",
        );
        let appchain_id = first.to_string();
        assert!(
            AccountId::from_str(second)
                .unwrap()
                .eq(&self.appchain_registry_account)
                && self.living_appchain_ids.contains(&appchain_id),
            "This function can only be called by a living appchain anchor contract."
        );
        appchain_id
    }
    ///
    pub fn sync_living_appchain_ids(&mut self, appchain_ids: Vec<String>) {
        self.assert_registry();
        self.living_appchain_ids = appchain_ids;
    }
    ///
    pub fn sync_validator_stakes_of_anchor(&mut self, stake_records: Vec<ValidatorStakeRecord>) {
        let appchain_id = self.assert_living_anchor();
        let mut changed = false;
        for stake_record in stake_records {
            let mut validator_stake = self
                .validator_stakes
                .get(&stake_record.validator_id)
                .unwrap_or(InternalValidatorStake::new(&stake_record.validator_id));
            validator_stake.update_stake_record(&appchain_id, &stake_record);
            changed = self.update_validator_rank_of(&mut validator_stake);
            self.validator_stakes
                .insert(&stake_record.validator_id, &validator_stake);
        }
        if changed {
            todo!()
        }
    }
    // the function will return true if the rank of validator stake has been changed and updated,
    // otherwise return false.
    fn update_validator_rank_of(&mut self, validator_stake: &mut InternalValidatorStake) -> bool {
        let current_index = validator_stake.overall_rank;
        let new_index = match self.ranked_validators.get(current_index) {
            Some(account_id) => {
                assert!(
                    account_id.eq(&validator_stake.validator_id),
                    "Invalid internal state of ordered validators. Need to reset."
                );
                self.ranked_validators.insert(
                    current_index,
                    &validator_stake.validator_id,
                    &self.validator_stakes,
                )
            }
            None => self
                .ranked_validators
                .append(&validator_stake.validator_id, &self.validator_stakes),
        };
        validator_stake.overall_rank = new_index;
        new_index != current_index
    }
}

impl Ownable for OctopusCouncil {
    //
    fn get_owner(&self) -> AccountId {
        self.owner.clone()
    }
    //
    fn set_owner(&mut self, owner: AccountId) {
        self.owner = owner;
    }
}

impl InternalValidatorStake {
    //
    pub fn new(validator_id: &AccountId) -> Self {
        Self {
            validator_id: validator_id.clone(),
            stake_in_appchains: UnorderedMap::new(StorageKey::ValidatorStakeInAppchains(
                validator_id.clone(),
            )),
            total_stake: U128(0),
            overall_rank: 0,
        }
    }
    //
    pub fn update_stake_record(
        &mut self,
        appchain_id: &String,
        stake_record: &ValidatorStakeRecord,
    ) {
        assert_eq!(
            self.validator_id, stake_record.validator_id,
            "Mismatch validator id in `ValidatorStakeRecord`."
        );
        let old_value = self.stake_in_appchains.get(&appchain_id).unwrap_or(U128(0));
        self.stake_in_appchains
            .insert(appchain_id, &stake_record.total_stake);
        self.total_stake.0 = self.total_stake.0 - old_value.0 + stake_record.total_stake.0;
    }
    //
    pub fn to_json_type(&self) -> ValidatorStake {
        let mut stake_in_appchains = HashMap::<String, U128>::new();
        for appchain_id in self.stake_in_appchains.keys() {
            stake_in_appchains.insert(
                appchain_id.clone(),
                self.stake_in_appchains.get(&appchain_id).unwrap(),
            );
        }
        ValidatorStake {
            validator_id: self.validator_id.clone(),
            stake_in_appchains,
            total_stake: self.total_stake.clone(),
            overall_rank: self.overall_rank,
        }
    }
}

impl RankValueHolder<AccountId> for LookupMap<AccountId, InternalValidatorStake> {
    //
    fn get_rank_value_of(&self, member: &AccountId) -> u128 {
        self.get(&member).unwrap().total_stake.0
    }
}
