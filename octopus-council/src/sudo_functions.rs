use crate::*;

#[near_bindgen]
impl OctopusCouncil {
    ///
    pub fn clear_council_members_and_regenerate_change_histories(&mut self) {
        self.assert_owner();
        //
        self.latest_members.clear();
        //
        // self.change_histories.clear();
        let change_history_index_range = self.change_histories.index_range();
        for index in
            change_history_index_range.start_index.0..change_history_index_range.end_index.0 + 1
        {
            env::storage_remove(&get_storage_key_in_lookup_array(
                &StorageKey::CouncilChangeHistories,
                &index,
            ));
        }
        //
        self.check_and_generate_change_histories();
    }
}

fn get_storage_key_in_lookup_array<T: BorshSerialize>(prefix: &StorageKey, index: &T) -> Vec<u8> {
    [prefix.try_to_vec().unwrap(), index.try_to_vec().unwrap()].concat()
}
