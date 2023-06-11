#![allow(unused)]

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{near_bindgen, AccountId, PanicOnDefault};

type Key = String;

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct ConsumerChainPosContract {
    pub should_bond_success: bool,
    pub should_change_key_success: bool,
}

#[near_bindgen]
impl ConsumerChainPosContract {
    #[init]
    pub fn new() -> Self {
        ConsumerChainPosContract {
            should_bond_success: true,
            should_change_key_success: true,
        }
    }

    pub fn set_should_bond_success(&mut self, should_bond_success: bool) {
        self.should_bond_success = should_bond_success
    }

    pub fn set_should_change_key_success(&mut self, should_change_key_success: bool) {
        self.should_change_key_success = should_change_key_success
    }

    pub fn bond(&self, staker_id: AccountId, key: Key) -> bool {
        self.should_bond_success
    }

    pub fn change_key(&self, staker_id: AccountId, key: Key) -> bool {
        self.should_change_key_success
    }
}
