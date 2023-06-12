use crate::*;
use near_sdk::{near_bindgen, AccountId};
use std::collections::HashMap;

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Account {
    account_id: AccountId,
    proposals: HashMap<Platform, BindingProposal>,
    bindings: HashMap<Platform, String>,
}

#[near_bindgen]
impl Contract {
    /// Get account info including bound handles
    pub fn get_account(&self, account_id: AccountId) -> Account {
        let proposals = self.internal_get_proposals(&account_id);
        let bindings = self.internal_get_bindings(&account_id);
        Account {
            account_id,
            proposals,
            bindings,
        }
    }

    /// Returns the number of accounts that have positive balance on this staking pool.
    pub fn get_number_of_accounts(&self) -> u64 {
        self.bindings.len()
    }

    /// Returns the list of accounts
    pub fn get_accounts(&self, from_index: u64, limit: u64) -> Vec<Account> {
        let keys = self.bindings.keys_as_vector();

        (from_index..std::cmp::min(from_index + limit, keys.len()))
            .map(|index| self.get_account(keys.get(index).unwrap()))
            .collect()
    }

    /// Get binding proposal of account
    pub fn get_proposal(
        &self,
        account_id: AccountId,
        platform: Platform,
    ) -> Option<BindingProposal> {
        let proposals = self.internal_get_proposals(&account_id);
        proposals.get(&platform).cloned()
    }

    /// Get social media handle from account ID
    pub fn get_handle(&self, account_id: AccountId, platform: Platform) -> Option<String> {
        let bindings = self.internal_get_bindings(&account_id);
        bindings.get(&platform).cloned()
    }

    // Look up NEAR account with handle
    pub fn lookup_account(&self, platform: Platform, handle: String) -> Option<AccountId> {
        self.internal_get_reverse_bindings(&platform).get(&handle)
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
