use crate::*;

#[near_bindgen]
impl OctopusCouncil {
    ///
    pub fn version(&self) -> String {
        String::from(VERSION)
    }
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
    pub fn get_ranked_validator_stakes(
        &self,
        start_index: u32,
        quantity: Option<u32>,
    ) -> Vec<ValidatorStake> {
        let all_members = match self.ranked_validators.len() > 0 {
            true => self.ranked_validators.get_slice_of(start_index, quantity),
            false => Vec::new(),
        };
        all_members
            .iter()
            .map(|account_id| {
                self.validator_stakes
                    .get(account_id)
                    .unwrap()
                    .to_json_type()
            })
            .collect()
    }
    //
    pub fn get_council_members(&self) -> Vec<AccountId> {
        self.latest_members.to_vec()
    }
    //
    pub fn get_council_change_histories(
        &self,
        start_index: U64,
        quantity: Option<U64>,
    ) -> Vec<CouncilChangeHistory> {
        self.change_histories
            .get_slice_of(&start_index.0, quantity.map(|q| q.0))
    }
}
