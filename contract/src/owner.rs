use crate::*;
use near_sdk::{near_bindgen, AccountId};

const ERR_NOT_OWNER: &str = "Only owner can perform this action";
const ERR_NOT_MANAGER: &str = "Only manager can perform this action";

/// public view functions
#[near_bindgen]
impl Contract {
    pub fn set_owner(&mut self, new_owner_id: AccountId) {
        self.assert_owner();
        self.owner_id = new_owner_id;
    }

    pub fn add_manager(&mut self, manager_id: AccountId) {
        self.assert_owner();
        self.managers.insert(&manager_id);
    }

    pub fn remove_manager(&mut self, manager_id: AccountId) -> bool {
        self.assert_owner();
        self.managers.remove(&manager_id)
    }
}

impl Contract {
    /// Asserts that the method was called by the owner.
    pub(crate) fn assert_owner(&self) {
        require!(
            env::predecessor_account_id() == self.owner_id,
            ERR_NOT_OWNER
        );
    }

    pub(crate) fn assert_manager(&self) {
        require!(
            self.managers.contains(&env::predecessor_account_id()),
            ERR_NOT_MANAGER
        );
    }
}
