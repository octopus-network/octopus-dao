mod lookup_array;
mod ranked_lookup_array;
mod storage_migration;
mod sudo_functions;
pub mod types;
mod upgrade;
mod views;

use lookup_array::{IndexedAndClearable, LookupArray};
use near_contract_standards::upgrade::Ownable;
use near_sdk::{
    assert_self,
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::{LookupMap, UnorderedMap, UnorderedSet},
    env, ext_contract,
    json_types::{U128, U64},
    log, near_bindgen,
    serde::{Deserialize, Serialize},
    AccountId, BorshStorageKey, Gas, PanicOnDefault, Promise, PromiseResult,
};
use ranked_lookup_array::{RankValueHolder, RankedLookupArray};
use std::{collections::HashMap, ops::Mul, str::FromStr};
use types::{
    CouncilChangeHistory, CouncilChangeHistoryState, IndexRange, MultiTxsOperationProcessingResult,
    ValidatorStake, ValidatorStakeRecord,
};

const VERSION: &str = "v0.4.0";
/// Constants for gas.
const T_GAS_CAP_FOR_MULTI_TXS_PROCESSING: u64 = 150;
const T_GAS_FOR_ADD_PROPOSAL: u64 = 5;
const T_GAS_FOR_RESOLVE_ADD_PROPOSAL: u64 = 25;
const T_GAS_FOR_ACT_PROPOSAL: u64 = 7;
const T_GAS_FOR_RESOLVE_ACT_PROPOSAL: u64 = 5;

#[ext_contract(ext_self)]
trait ResolverForSelfCallback {
    /// Resolver for adding proposal to DAO contract
    fn resolve_add_proposal(&mut self, change_history: &mut CouncilChangeHistory);
    /// Resolver for acting proposal to DAO contract
    fn resolve_act_proposal(&mut self, change_history: &mut CouncilChangeHistory);
}

/// Storage keys for collections of sub-struct in main contract
#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKey {
    ValidatorStakes,
    OrderedValidators,
    ValidatorStakeInAppchains(AccountId),
    OctopusCouncilWasm,
    LatestMembers,
    CouncilChangeHistories,
    ValidatorsWaitingToUpdateRank,
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
    dao_contract_account: AccountId,
    //
    living_appchain_ids: Vec<String>,
    //
    validator_stakes: LookupMap<AccountId, InternalValidatorStake>,
    //
    ranked_validators: RankedLookupArray<AccountId>,
    //
    max_number_of_council_members: u32,
    //
    latest_members: UnorderedSet<AccountId>,
    //
    excluding_validator_accounts: Vec<AccountId>,
    //
    change_histories: LookupArray<CouncilChangeHistory>,
    //
    validators_waiting_to_update_rank: UnorderedSet<AccountId>,
}

#[near_bindgen]
impl OctopusCouncil {
    #[init]
    pub fn new(max_number_of_council_members: u32, dao_contract_account: AccountId) -> Self {
        assert!(!env::state_exists(), "The contract is already initialized.");
        let account_id = String::from(env::current_account_id().as_str());
        let parts = account_id.split(".").collect::<Vec<&str>>();
        assert!(
            parts.len() > 2,
            "This contract must be deployed as a sub-account of octopus appchain registry.",
        );
        let (_first, second) = account_id.split_once(".").unwrap();
        let result = Self {
            owner: env::current_account_id(),
            appchain_registry_account: AccountId::from_str(second).unwrap(),
            dao_contract_account,
            living_appchain_ids: Vec::new(),
            validator_stakes: LookupMap::new(StorageKey::ValidatorStakes),
            ranked_validators: RankedLookupArray::new(StorageKey::OrderedValidators),
            max_number_of_council_members,
            latest_members: UnorderedSet::new(StorageKey::LatestMembers),
            excluding_validator_accounts: Vec::new(),
            change_histories: LookupArray::new(StorageKey::CouncilChangeHistories),
            validators_waiting_to_update_rank: UnorderedSet::new(
                StorageKey::ValidatorsWaitingToUpdateRank,
            ),
        };
        result
    }
    // Assert that the contract is called by an appchain anchor contract and
    // return the appchain id corresponding to the predecessor account
    fn assert_and_update_living_appchain_ids(&mut self) -> String {
        let account_id = String::from(env::predecessor_account_id().as_str());
        let (first, second) = account_id.split_once(".").expect(
            "This contract can only be called by a sub-account of octopus appchain registry.",
        );
        let appchain_id = first.to_string();
        assert!(
            AccountId::from_str(second)
                .unwrap()
                .eq(&self.appchain_registry_account),
            "This function can only be called by an appchain anchor contract."
        );
        if !self.living_appchain_ids.contains(&appchain_id) {
            self.living_appchain_ids.push(appchain_id.clone());
        }
        appchain_id
    }
    ///
    pub fn sync_validator_stakes_of_anchor(&mut self, stake_records: Vec<ValidatorStakeRecord>) {
        let appchain_id = self.assert_and_update_living_appchain_ids();
        for stake_record in stake_records {
            let mut validator_stake = self
                .validator_stakes
                .get(&stake_record.validator_id)
                .unwrap_or(InternalValidatorStake::new(&stake_record.validator_id));
            if validator_stake.update_stake_record(&appchain_id, &stake_record) {
                self.validator_stakes
                    .insert(&stake_record.validator_id, &validator_stake);
                log!(
                    "Total stake of validator '{}' has changed, need to update rank.",
                    stake_record.validator_id
                );
                self.validators_waiting_to_update_rank
                    .insert(&stake_record.validator_id);
            }
        }
    }
    ///
    pub fn update_council_change_histories(&mut self) -> MultiTxsOperationProcessingResult {
        let validator_ids = self.validators_waiting_to_update_rank.to_vec();
        if validator_ids.len() > 0 {
            for validator_id in validator_ids {
                let mut validator_stake = self.validator_stakes.get(&validator_id).unwrap();
                self.update_validator_rank_of(&mut validator_stake);
                self.validators_waiting_to_update_rank.remove(&validator_id);
                if env::used_gas() > Gas::ONE_TERA.mul(T_GAS_CAP_FOR_MULTI_TXS_PROCESSING) {
                    break;
                }
            }
            return MultiTxsOperationProcessingResult::NeedMoreGas;
        } else {
            self.check_and_generate_change_histories();
            MultiTxsOperationProcessingResult::Ok
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
                    "Invalid internal state of ordered validators. Account at index '{}' is '{}', but the updating validator is '{}'.",
                    current_index, account_id, validator_stake.validator_id,
                );
                self.ranked_validators.insert(
                    current_index,
                    &validator_stake.validator_id,
                    &mut self.validator_stakes,
                )
            }
            None => self
                .ranked_validators
                .append(&validator_stake.validator_id, &mut self.validator_stakes),
        };
        new_index != current_index
    }
    //
    fn check_and_generate_change_histories(&mut self) {
        let validator_accounts = self.ranked_validators.get_slice_of(0, None);
        // generate a new array of council members
        let mut council_members = Vec::new();
        for account_id in validator_accounts {
            if !self.excluding_validator_accounts.contains(&account_id) {
                council_members.push(account_id);
                if council_members.len() >= self.max_number_of_council_members as usize {
                    break;
                }
            }
        }
        // update `latest_members` and generate change histories
        for account_id in &council_members {
            if !self.latest_members.contains(account_id) {
                self.latest_members.insert(account_id);
                let history = self.change_histories.append(&mut CouncilChangeHistory {
                    index: U64::from(0),
                    action: types::CouncilChangeAction::MemberAdded(account_id.clone()),
                    state: CouncilChangeHistoryState::WaitingForApplying,
                    timestamp: U64::from(env::block_timestamp()),
                });
                log!(
                    "Council change history generated: '{}'",
                    near_sdk::serde_json::to_string(&history).unwrap()
                );
            }
        }
        for account_id in &self.latest_members.to_vec() {
            if !council_members.contains(account_id) {
                self.latest_members.remove(account_id);
                let history = self.change_histories.append(&mut CouncilChangeHistory {
                    index: U64::from(0),
                    action: types::CouncilChangeAction::MemberRemoved(account_id.clone()),
                    state: CouncilChangeHistoryState::WaitingForApplying,
                    timestamp: U64::from(env::block_timestamp()),
                });
                log!(
                    "Council change history generated: '{}'",
                    near_sdk::serde_json::to_string(&history).unwrap()
                );
            }
        }
    }
    ///
    pub fn set_dao_contract_account(&mut self, account_id: AccountId) {
        self.assert_owner();
        self.dao_contract_account = account_id;
    }
    ///
    pub fn apply_change_histories_to_dao_contract(
        &mut self,
        start_index: U64,
    ) -> MultiTxsOperationProcessingResult {
        assert!(
            self.dao_contract_account.to_string().len() > 0,
            "Invalid account id of DAO contract."
        );
        let index_range = self.change_histories.index_range();
        let mut index = start_index.0;
        while index <= index_range.end_index.0
            && env::used_gas() < Gas::ONE_TERA.mul(T_GAS_CAP_FOR_MULTI_TXS_PROCESSING)
        {
            let mut change_history = self.change_histories.get(&index).unwrap();
            match change_history.state {
                CouncilChangeHistoryState::WaitingForApplying => {
                    self.add_proposal_to_dao_contract(&mut change_history)
                }
                CouncilChangeHistoryState::ProposalAdded(_) => {
                    self.act_proposal_on_dao_contract(&mut change_history)
                }
                _ => (),
            }
            index += 1;
        }
        if index > index_range.end_index.0 {
            MultiTxsOperationProcessingResult::Ok
        } else {
            MultiTxsOperationProcessingResult::NeedMoreGas
        }
    }
    //
    fn add_proposal_to_dao_contract(&mut self, change_history: &mut CouncilChangeHistory) {
        #[derive(Serialize, Deserialize, Clone)]
        #[serde(crate = "near_sdk::serde")]
        enum ProposalKind {
            /// Add member to given role in the policy. This is short cut to updating the whole policy.
            AddMemberToRole { member_id: AccountId, role: String },
            /// Remove member to given role in the policy. This is short cut to updating the whole policy.
            RemoveMemberFromRole { member_id: AccountId, role: String },
        }
        #[derive(Serialize, Deserialize, Clone)]
        #[serde(crate = "near_sdk::serde")]
        struct ProposalInput {
            /// Description of this proposal.
            pub description: String,
            /// Kind of proposal with relevant information.
            pub kind: ProposalKind,
        }
        #[derive(Serialize, Deserialize, Clone)]
        #[serde(crate = "near_sdk::serde")]
        struct Input {
            pub proposal: ProposalInput,
        }
        let args = match &change_history.action {
            types::CouncilChangeAction::MemberAdded(account_id) => Input {
                proposal: ProposalInput {
                    description: format!(
                        "Add '{}' to council based on the rule in contract '{}'.",
                        account_id,
                        env::current_account_id()
                    ),
                    kind: ProposalKind::AddMemberToRole {
                        member_id: account_id.clone(),
                        role: "council".to_string(),
                    },
                },
            },
            types::CouncilChangeAction::MemberRemoved(account_id) => Input {
                proposal: ProposalInput {
                    description: format!(
                        "Remove '{}' from council based on the rule in contract '{}'.",
                        account_id,
                        env::current_account_id()
                    ),
                    kind: ProposalKind::RemoveMemberFromRole {
                        member_id: account_id.clone(),
                        role: "council".to_string(),
                    },
                },
            },
        };
        let args = near_sdk::serde_json::to_vec(&args)
            .expect("Failed to serialize the cross contract args using JSON.");
        Promise::new(self.dao_contract_account.clone())
            .function_call(
                "add_proposal".to_string(),
                args,
                0,
                Gas::ONE_TERA.mul(T_GAS_FOR_ADD_PROPOSAL),
            )
            .then(
                ext_self::ext(env::current_account_id())
                    .with_attached_deposit(0)
                    .with_static_gas(Gas::ONE_TERA.mul(T_GAS_FOR_RESOLVE_ADD_PROPOSAL))
                    .with_unused_gas_weight(0)
                    .resolve_add_proposal(change_history),
            );
    }
    //
    fn act_proposal_on_dao_contract(&mut self, change_history: &mut CouncilChangeHistory) {
        #[derive(Serialize, Deserialize, Clone)]
        #[serde(crate = "near_sdk::serde")]
        enum Action {
            /// Vote to approve given proposal or bounty.
            VoteApprove,
        }
        #[derive(Serialize, Deserialize, Clone)]
        #[serde(crate = "near_sdk::serde")]
        struct Input {
            pub id: u64,
            pub action: Action,
            pub memo: Option<String>,
        }
        let args = Input {
            id: match change_history.state {
                CouncilChangeHistoryState::ProposalAdded(proposal_id) => proposal_id,
                _ => panic!(
                    "Invalid state of change history: '{}'",
                    near_sdk::serde_json::to_string(change_history).unwrap()
                ),
            },
            action: Action::VoteApprove,
            memo: Some(format!(
                "Automatically vote approve by '{}'.",
                env::current_account_id()
            )),
        };
        let args = near_sdk::serde_json::to_vec(&args)
            .expect("Failed to serialize the cross contract args using JSON.");
        Promise::new(self.dao_contract_account.clone())
            .function_call(
                "act_proposal".to_string(),
                args,
                0,
                Gas::ONE_TERA.mul(T_GAS_FOR_ACT_PROPOSAL),
            )
            .then(
                ext_self::ext(env::current_account_id())
                    .with_attached_deposit(0)
                    .with_static_gas(Gas::ONE_TERA.mul(T_GAS_FOR_RESOLVE_ACT_PROPOSAL))
                    .with_unused_gas_weight(0)
                    .resolve_act_proposal(change_history),
            );
    }
    ///
    pub fn set_max_number_of_council_members(&mut self, max_number_of_council_members: u32) {
        self.assert_owner();
        assert!(
            self.max_number_of_council_members != max_number_of_council_members,
            "The value is not changed."
        );
        self.max_number_of_council_members = max_number_of_council_members;
        //
        self.check_and_generate_change_histories();
    }
    ///
    pub fn set_excluding_validator_accounts(&mut self, accounts: Vec<AccountId>) {
        self.assert_owner();
        self.excluding_validator_accounts = accounts;
        //
        self.check_and_generate_change_histories();
    }
    /// Called by valid validator accounts,
    /// to exclude self from council members
    pub fn exclude_validator_from_council(&mut self) {
        let validator_id = env::predecessor_account_id();
        assert!(
            self.validator_stakes.contains_key(&validator_id),
            "Only valid validator can call this function."
        );
        assert!(
            !self.excluding_validator_accounts.contains(&validator_id),
            "Validator '{}' is already excluded.",
            validator_id
        );
        //
        self.excluding_validator_accounts.push(validator_id);
        self.check_and_generate_change_histories();
    }
    /// Called by excluding validator account,
    /// to recover self from excluding validator accounts
    pub fn recover_excluding_validator(&mut self) {
        let validator_id = env::predecessor_account_id();
        assert!(
            self.excluding_validator_accounts.contains(&validator_id),
            "Only excluding validator can call this function."
        );
        //
        self.excluding_validator_accounts = self
            .excluding_validator_accounts
            .iter()
            .filter(|account| !(*account).eq(&validator_id))
            .map(|account| account.clone())
            .collect::<Vec<AccountId>>();
        self.check_and_generate_change_histories();
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
            overall_rank: u32::MAX,
        }
    }
    //
    pub fn update_stake_record(
        &mut self,
        appchain_id: &String,
        stake_record: &ValidatorStakeRecord,
    ) -> bool {
        assert_eq!(
            self.validator_id, stake_record.validator_id,
            "Mismatch validator id in `ValidatorStakeRecord`."
        );
        let old_value = self.stake_in_appchains.get(&appchain_id).unwrap_or(U128(0));
        if stake_record.total_stake != old_value {
            self.stake_in_appchains
                .insert(appchain_id, &stake_record.total_stake);
            self.total_stake.0 = self.total_stake.0 - old_value.0 + stake_record.total_stake.0;
            true
        } else {
            false
        }
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
    //
    fn update_rank_of(&mut self, member: &AccountId, new_rank: u32) {
        let mut validator_stake = self.get(&member).unwrap();
        validator_stake.overall_rank = new_rank;
        self.insert(member, &validator_stake);
    }
}

#[near_bindgen]
impl ResolverForSelfCallback for OctopusCouncil {
    //
    fn resolve_add_proposal(&mut self, change_history: &mut CouncilChangeHistory) {
        assert_self();
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(bytes) => {
                change_history.state = CouncilChangeHistoryState::ProposalAdded(
                    near_sdk::serde_json::from_slice::<u64>(&bytes).unwrap(),
                );
                self.change_histories
                    .insert(&change_history.index.0, change_history);
                //
                self.act_proposal_on_dao_contract(change_history);
            }
            PromiseResult::Failed => {
                log!(
                    "Failed to add proposal for change history: '{}'",
                    near_sdk::serde_json::to_string(&change_history).unwrap()
                );
            }
        }
    }
    //
    fn resolve_act_proposal(&mut self, change_history: &mut CouncilChangeHistory) {
        assert_self();
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                let proposal_id = match change_history.state {
                    CouncilChangeHistoryState::ProposalAdded(id) => id,
                    _ => panic!(
                        "Invalid state of council change history: '{}'",
                        near_sdk::serde_json::to_string(change_history).unwrap()
                    ),
                };
                change_history.state = CouncilChangeHistoryState::ProposalApproved(proposal_id);
                self.change_histories
                    .insert(&change_history.index.0, change_history);
            }
            PromiseResult::Failed => {
                log!(
                    "Failed to act proposal for change history: '{}'",
                    near_sdk::serde_json::to_string(&change_history).unwrap()
                );
            }
        }
    }
}
