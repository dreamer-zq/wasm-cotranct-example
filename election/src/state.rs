use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub start: u64,
    pub end: u64,
    pub candidates: Vec<String>,
    pub votes: Vec<VoteInfo>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VoteInfo {
    pub voter: String,
    pub candidate: String,
}

pub const STATE: Item<State> = Item::new("state");
