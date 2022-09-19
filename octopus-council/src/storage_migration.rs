use crate::*;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::{env, near_bindgen, AccountId};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct OldOctopusCouncil {
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
    #[init(ignore_state)]
    pub fn migrate_state() -> Self {
        // Deserialize the state using the old contract structure.
        let old_contract: OldOctopusCouncil = env::state_read().expect("Old state doesn't exist");
        //
        near_sdk::assert_self();
        //
        // Create the new contract using the data from the old contract.
        let mut new_contract = OctopusCouncil {
            owner: old_contract.owner,
            appchain_registry_account: old_contract.appchain_registry_account,
            living_appchain_ids: old_contract.living_appchain_ids,
            validator_stakes: old_contract.validator_stakes,
            ranked_validators: old_contract.ranked_validators,
            max_number_of_council_members: old_contract.max_number_of_council_members,
            latest_members: UnorderedSet::new(StorageKey::LatestMembers),
            change_histories: LookupArray::new(StorageKey::CouncilChangeHistories),
            latest_applied_change_history: None,
        };
        //
        new_contract
            .change_histories
            .append(&mut CouncilChangeHistory {
                action: types::CouncilChangeAction::MaxNumberOfMembersChanged(
                    new_contract.max_number_of_council_members,
                ),
                index: U64::from(0),
                timestamp: U64::from(env::block_timestamp()),
            });
        //
        new_contract
    }
}
