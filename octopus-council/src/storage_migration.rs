use crate::types::{CouncilChangeAction, CouncilChangeHistoryState};
use crate::*;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::{env, near_bindgen, AccountId};

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct OldCouncilChangeHistory {
    pub action: CouncilChangeAction,
    pub index: U64,
    pub timestamp: U64,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct OldOctopusCouncil {
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
    change_histories: LookupArray<OldCouncilChangeHistory>,
    // The index of change history which is already applied to DAO contract
    latest_applied_change_history: Option<u64>,
}

#[near_bindgen]
impl OctopusCouncil {
    #[init(ignore_state)]
    pub fn migrate_state() -> Self {
        // Deserialize the state using the old contract structure.
        let mut old_contract: OldOctopusCouncil =
            env::state_read().expect("Old state doesn't exist");
        //
        near_sdk::assert_self();
        //
        old_contract.change_histories.clear();
        // Create the new contract using the data from the old contract.
        let mut new_contract = OctopusCouncil {
            owner: old_contract.owner,
            appchain_registry_account: old_contract.appchain_registry_account,
            dao_contract_account: old_contract.dao_contract_account,
            living_appchain_ids: old_contract.living_appchain_ids,
            validator_stakes: old_contract.validator_stakes,
            ranked_validators: old_contract.ranked_validators,
            max_number_of_council_members: old_contract.max_number_of_council_members,
            latest_members: old_contract.latest_members,
            change_histories: LookupArray::new(StorageKey::CouncilChangeHistories),
        };
        //
        new_contract
            .change_histories
            .append(&mut CouncilChangeHistory {
                index: U64::from(0),
                action: types::CouncilChangeAction::MaxNumberOfMembersChanged(
                    new_contract.max_number_of_council_members,
                ),
                state: CouncilChangeHistoryState::NoNeedToApply,
                timestamp: U64::from(env::block_timestamp()),
            });
        //
        new_contract
    }
}

impl IndexedAndClearable for OldCouncilChangeHistory {
    //
    fn set_index(&mut self, index: &u64) {
        self.index = U64::from(*index);
    }
    //
    fn clear_extra_storage(&mut self) -> MultiTxsOperationProcessingResult {
        MultiTxsOperationProcessingResult::Ok
    }
}
