use crate::*;
use near_sdk::{near_bindgen, AccountId};
use std::collections::HashMap;

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Account {
    account_id: AccountId,
    bindings: HashMap<Platform, String>,
}

#[near_bindgen]
impl Contract {
    /// Get account info including bound handles
    pub fn get_account(&self, account_id: AccountId) -> Account {
        let bindings = self.internal_get_account(&account_id);
        Account {
            account_id,
            bindings,
        }
    }

    /// Returns the number of accounts that have positive balance on this staking pool.
    pub fn get_number_of_accounts(&self) -> u64 {
        self.accounts.len()
    }

    /// Returns the list of accounts
    pub fn get_accounts(&self, from_index: u64, limit: u64) -> Vec<Account> {
        let keys = self.accounts.keys_as_vector();

        (from_index..std::cmp::min(from_index + limit, keys.len()))
            .map(|index| self.get_account(keys.get(index).unwrap()))
            .collect()
    }

    /// Get binding proposal of account
    pub fn get_proposal(&self, account_id: AccountId, platform: Platform) -> BindingProposal {
        let proposals = self.internal_get_proposals(&account_id);
        proposals
            .get(&platform)
            .expect("No proposals found")
            .clone()
    }

    /// Get social media handle from account ID
    /// `platform` can only be `twitter` for now
    pub fn get_handle(&self, account_id: AccountId, platform: Platform) -> String {
        let bindings = self.internal_get_account(&account_id);
        bindings.get(&platform).unwrap_or(&"".to_string()).clone()
    }

    // Look up NEAR account with handle
    // `platform` can only be `twitter` for now
    pub fn lookup_account(&self, platform: Platform, handle: String) -> AccountId {
        self.internal_get_reverse_lookup(&platform)
            .get(&handle)
            .unwrap_or_else(|| AccountId::new_unchecked("".to_string()))
    }

    // Get contract owner ID
    pub fn get_owner_id(&self) -> AccountId {
        self.owner_id.clone()
    }

    // Check whether account is a manager
    pub fn is_manager(&self, account_id: AccountId) -> bool {
        self.managers.contains(&account_id)
    }
}
