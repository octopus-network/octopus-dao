use crate::*;

pub trait RankValueHolder<T: BorshDeserialize + BorshSerialize> {
    ///
    fn get_rank_value_of(&self, member: &T) -> u128;
    ///
    fn update_rank_of(&mut self, member: &T, new_rank: u32);
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct RankedLookupArray<T: BorshDeserialize + BorshSerialize> {
    /// The anchor event data map.
    lookup_map: LookupMap<u32, T>,
    /// The length of the array.
    length: u32,
}

impl<T> RankedLookupArray<T>
where
    T: BorshDeserialize + BorshSerialize,
{
    ///
    pub fn new(storage_key: StorageKey) -> Self {
        Self {
            lookup_map: LookupMap::new(storage_key),
            length: 0,
        }
    }
    ///
    pub fn get(&self, index: u32) -> Option<T> {
        self.lookup_map.get(&index)
    }
    ///
    pub fn get_slice_of(&self, start_index: u32, quantity: Option<u32>) -> Vec<T> {
        let mut results = Vec::<T>::new();
        assert!(
            start_index < self.length,
            "Start index is out of bound of the array."
        );
        let end_index = match quantity {
            Some(quantity) => match quantity > self.length - start_index - 1 {
                true => self.length - 1,
                false => start_index + quantity - 1,
            },
            None => self.length - 1,
        };
        for index in start_index..end_index + 1 {
            if let Some(record) = self.get(index) {
                results.push(record);
            }
        }
        results
    }
    ///
    pub fn append<S: RankValueHolder<T>>(&mut self, record: &T, rank_value_holder: &mut S) -> u32 {
        self.lookup_map.insert(&self.length, &record);
        rank_value_holder.update_rank_of(record, self.length);
        self.length += 1;
        //
        self.move_up_by_rank_value(record, self.length - 1, rank_value_holder)
    }
    //
    fn move_up_by_rank_value<S: RankValueHolder<T>>(
        &mut self,
        record: &T,
        index: u32,
        rank_value_holder: &mut S,
    ) -> u32 {
        let mut current_index = index;
        while current_index > 0 {
            let previous_index = current_index - 1;
            let previous_rank_value =
                rank_value_holder.get_rank_value_of(&self.get(previous_index).unwrap());
            let current_rank_value = rank_value_holder.get_rank_value_of(record);
            if current_rank_value <= previous_rank_value {
                break;
            } else {
                self.swap((previous_index, current_index), rank_value_holder);
                current_index = previous_index;
            }
        }
        current_index
    }
    ///
    pub fn insert<S: RankValueHolder<T>>(
        &mut self,
        index: u32,
        record: &T,
        rank_value_holder: &mut S,
    ) -> u32 {
        assert!(index < self.length, "Index is out of bound of the array.");
        self.lookup_map.insert(&index, record);
        //
        let mut new_index = self.move_up_by_rank_value(record, index, rank_value_holder);
        if new_index == index {
            new_index = self.move_down_by_rank_value(record, index, rank_value_holder);
        }
        new_index
    }
    //
    fn move_down_by_rank_value<S: RankValueHolder<T>>(
        &mut self,
        record: &T,
        index: u32,
        rank_value_holder: &mut S,
    ) -> u32 {
        let mut current_index = index;
        if self.length > 1 {
            while current_index < self.length - 1 {
                let next_index = current_index + 1;
                let next_rank_value =
                    rank_value_holder.get_rank_value_of(&self.get(next_index).unwrap());
                let current_rank_value = rank_value_holder.get_rank_value_of(record);
                if current_rank_value >= next_rank_value {
                    break;
                } else {
                    self.swap((next_index, current_index), rank_value_holder);
                    current_index = next_index;
                }
            }
        }
        current_index
    }
    ///
    pub fn len(&self) -> u32 {
        self.length
    }
    ///
    pub fn clear(&mut self) -> MultiTxsOperationProcessingResult {
        while self.length > 0
            && env::used_gas() <= Gas::ONE_TERA.mul(T_GAS_CAP_FOR_MULTI_TXS_PROCESSING)
        {
            self.lookup_map.remove(&(self.length - 1));
            self.length -= 1
        }
        if self.length > 0
            && env::used_gas() > Gas::ONE_TERA.mul(T_GAS_CAP_FOR_MULTI_TXS_PROCESSING)
        {
            MultiTxsOperationProcessingResult::NeedMoreGas
        } else {
            MultiTxsOperationProcessingResult::Ok
        }
    }
    ///
    fn swap<S: RankValueHolder<T>>(&mut self, index_pair: (u32, u32), rank_value_holder: &mut S) {
        assert!(
            index_pair.0 < self.length
                && index_pair.1 < self.length
                && index_pair.0 != index_pair.1,
            "Invalid index pair to swap."
        );
        let t0 = self.lookup_map.get(&index_pair.0).unwrap();
        let t1 = self.lookup_map.get(&index_pair.1).unwrap();
        self.lookup_map.insert(&index_pair.0, &t1);
        rank_value_holder.update_rank_of(&t1, index_pair.0);
        self.lookup_map.insert(&index_pair.1, &t0);
        rank_value_holder.update_rank_of(&t0, index_pair.1);
    }
}
