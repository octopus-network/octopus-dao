use crate::*;

#[near_bindgen]
impl OctopusCouncil {
    ///
    pub fn get_living_appchain_ids(&self) -> Vec<String> {
        self.living_appchain_ids.clone()
    }
    //
    pub fn get_validator_stake_of(&self, account_id: AccountId) -> ValidatorStake {
        self.validator_stakes
            .get(&account_id)
            .expect("Invalid validator id.")
            .to_json_type()
    }
    //
    pub fn get_council_members(&self) -> Vec<AccountId> {
        let all_members = self.ranked_validators.get_slice_of(0, None);
        if all_members.len() > self.max_number_of_council_members as usize {
            return all_members
                .split_at(self.max_number_of_council_members as usize)
                .0
                .to_vec();
        }
        all_members
    }
}
