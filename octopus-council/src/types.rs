use std::collections::HashMap;

use crate::*;
use near_sdk::serde::{Deserialize, Serialize};

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
