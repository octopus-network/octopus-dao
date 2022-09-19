use crate::*;
use near_sdk::{
    json_types::U64,
    serde::{Deserialize, Serialize},
};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct IndexRange {
    pub start_index: U64,
    pub end_index: U64,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum MultiTxsOperationProcessingResult {
    NeedMoreGas,
    Ok,
    Error(String),
}

impl MultiTxsOperationProcessingResult {
    ///
    pub fn is_ok(&self) -> bool {
        match self {
            MultiTxsOperationProcessingResult::Ok => true,
            _ => false,
        }
    }
    ///
    pub fn is_need_more_gas(&self) -> bool {
        match self {
            MultiTxsOperationProcessingResult::NeedMoreGas => true,
            _ => false,
        }
    }
    ///
    pub fn is_error(&self) -> bool {
        match self {
            MultiTxsOperationProcessingResult::Error(_) => true,
            _ => false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct ValidatorStakeRecord {
    pub validator_id: AccountId,
    pub total_stake: U128,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct ValidatorStake {
    //
    pub validator_id: AccountId,
    // key: appchain id, value: total stake in the appchain anchor
    pub stake_in_appchains: HashMap<String, U128>,
    // total stake in all appchain anchors
    pub total_stake: U128,
    // the rank of the validator in all validators
    pub overall_rank: u32,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum CouncilChangeAction {
    MaxNumberOfMembersChanged(u32),
    MemberAdded(AccountId),
    MemberRemoved(AccountId),
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct CouncilChangeHistory {
    pub action: CouncilChangeAction,
    pub index: U64,
    pub timestamp: U64,
}

impl IndexedAndClearable for CouncilChangeHistory {
    //
    fn set_index(&mut self, index: &u64) {
        self.index = U64::from(*index);
    }
    //
    fn clear_extra_storage(&mut self) -> MultiTxsOperationProcessingResult {
        MultiTxsOperationProcessingResult::Ok
    }
}
