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
        let new_contract = OctopusCouncil {
            owner: old_contract.owner,
            appchain_registry_account: old_contract.appchain_registry_account,
            dao_contract_account: old_contract.dao_contract_account,
            living_appchain_ids: old_contract.living_appchain_ids,
            validator_stakes: old_contract.validator_stakes,
            ranked_validators: old_contract.ranked_validators,
            max_number_of_council_members: old_contract.max_number_of_council_members,
            latest_members: old_contract.latest_members,
            excluding_validator_accounts: old_contract.excluding_validator_accounts,
            change_histories: old_contract.change_histories,
            validators_waiting_to_update_rank: UnorderedSet::new(
                StorageKey::ValidatorsWaitingToUpdateRank,
            ),
        };
        //
        //
        new_contract
    }
}
