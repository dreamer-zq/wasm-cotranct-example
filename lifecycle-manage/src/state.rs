use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, Storage};
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton};

pub static CONFIG_KEY: &[u8] = b"lifecycle";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub frozen: bool,
    pub owner: CanonicalAddr,
}

impl State {
    /// returns true if the address is a registered admin
    pub fn is_owner(&self, addr: CanonicalAddr) -> bool {
        self.owner == addr
    }

    pub fn can_modify(&self, addr: CanonicalAddr) -> bool {
        self.is_owner(addr) && !self.frozen
    }
}

pub fn config(storage: &mut dyn Storage) -> Singleton<State> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_read(storage: &dyn Storage) -> ReadonlySingleton<State> {
    singleton_read(storage, CONFIG_KEY)
}
