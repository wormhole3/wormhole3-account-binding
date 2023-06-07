use crate::*;
use events::Event;
use near_sdk::{near_bindgen, AccountId};

const ERR_NOT_OWNER: &str = "Only owner can perform this action";
const ERR_NOT_MANAGER: &str = "Only manager can perform this action";

#[near_bindgen]
impl Contract {
    pub fn set_owner(&mut self, new_owner_id: AccountId) {
        self.assert_owner();
        let old_owner_id = self.owner_id.clone();
        self.owner_id = new_owner_id.clone();
        Event::ChangeOwner {
            old_owner_id: &old_owner_id,
            new_owner_id: &new_owner_id,
        }
        .emit();
    }

    pub fn add_manager(&mut self, manager_id: AccountId) {
        self.assert_owner();
        self.managers.insert(&manager_id);
        Event::AddManager {
            manager_id: &manager_id,
        }
        .emit();
    }

    pub fn remove_manager(&mut self, manager_id: AccountId) -> bool {
        self.assert_owner();
        let removed = self.managers.remove(&manager_id);
        Event::RemoveManager {
            manager_id: &manager_id,
        }
        .emit();
        removed
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
