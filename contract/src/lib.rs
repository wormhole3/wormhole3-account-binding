mod events;
mod owner;
mod view;

use events::Event;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, require, AccountId, BorshStorageKey, PanicOnDefault, Timestamp};
use std::collections::HashMap;

#[derive(BorshStorageKey, BorshSerialize)]
pub(crate) enum StorageKey {
    Managers,
    BindingProposals,
    Accounts,
    ReverseLookup,
    ReverseBindings { platform: Platform },
}

/// Platform that we will verify
/// Now only Twitter is supported
#[derive(
    Serialize, Deserialize, BorshDeserialize, BorshSerialize, Clone, PartialEq, Eq, PartialOrd, Hash,
)]
#[serde(crate = "near_sdk::serde", rename_all = "lowercase")]
pub enum Platform {
    Twitter,
    Facebook,
    Reddit,
    GitHub,
    Telegram,
    Discord,
    Instagram,
    Ethereum,
    Hive,
    Steem,
}

/// Binding Proposal
#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct BindingProposal {
    account_id: AccountId,
    platform: Platform,
    handle: String,
    // proposal creation time
    created_at: Timestamp,
}

/// Contract states
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    /// Owner ID who is usually a DAO that can upgrade the contract
    owner_id: AccountId,
    /// Manager accounts who are responsible for account binding
    managers: UnorderedSet<AccountId>,
    /// Binding proposals
    binding_proposals: UnorderedMap<AccountId, HashMap<Platform, BindingProposal>>,
    /// Mapping from NEAR account ID to social media handles.
    accounts: UnorderedMap<AccountId, HashMap<Platform, String>>,
    /// Look up NEAR account ID with twitter handle
    reverse_lookup: LookupMap<Platform, LookupMap<String, AccountId>>,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        Self {
            owner_id,
            managers: UnorderedSet::new(StorageKey::Managers),
            binding_proposals: UnorderedMap::new(StorageKey::BindingProposals),
            accounts: UnorderedMap::new(StorageKey::Accounts),
            reverse_lookup: LookupMap::new(StorageKey::ReverseLookup),
        }
    }

    /// Submit the proposal of binding my NEAR account with a social media handle
    /// Permission: can be called by any user
    /// (Optional) If the handle is already bound with an NEAR account, the proposal is invalid.
    pub fn propose_binding(&mut self, platform: Platform, handle: String) {
        require!(!handle.is_empty(), "Invalid handle");

        let account_id = env::predecessor_account_id();
        let mut proposals = self.internal_get_proposals(&account_id);

        let current_timestamp = env::block_timestamp_ms();
        let proposal = BindingProposal {
            account_id: account_id.clone(),
            platform: platform.clone(),
            handle: handle.clone(),
            created_at: current_timestamp,
        };
        proposals.insert(platform.clone(), proposal);
        self.binding_proposals.insert(&account_id, &proposals);

        Event::ProposeBinding {
            account_id: &account_id,
            platform: &platform,
            handle: &handle,
            created_at: &current_timestamp,
        }
        .emit();
    }

    /// Cancel binding proposal
    pub fn cancel_binding_proposal(&mut self, platform: Platform) {
        let account_id = env::predecessor_account_id();
        let proposal = self.internal_remove_proposal(&account_id, &platform);

        Event::CancelBindingProposal {
            account_id: &proposal.account_id,
            platform: &proposal.platform,
            handle: &proposal.handle,
            created_at: &proposal.created_at,
        }
        .emit();
    }

    /// Unbind my NEAR account from social media handles, so wormhole3 cannot post on behalf of the user.
    /// Permission: can be called by any user
    // pub fn unbind(&mut self) {}

    /// Bind NEAR accounts to proposed social media handles if authorization succeed
    /// The handles (e.g. twitter handle) provided in the proposal will be used. No need to provide via function call.
    /// To avoid fake binding attack, only proposals created before verification time will be accepted
    /// Permission: can only be called by manager account
    pub fn accept_binding(
        &mut self,
        account_id: AccountId,
        platform: Platform,
        verification_timestamp: Timestamp,
    ) {
        self.assert_manager();
        require!(
            verification_timestamp <= env::block_timestamp_ms(),
            "Verification timestamp must be in the past"
        );

        let proposal = self.internal_remove_proposal(&account_id, &platform.clone());
        require!(
            proposal.created_at < verification_timestamp,
            "Proposal is created after verification"
        );

        // insert bindings
        let mut bindings = self.internal_get_account(&account_id);
        bindings.insert(platform.clone(), proposal.handle.clone());
        self.accounts.insert(&account_id, &bindings);

        // update reverse bindings
        let mut reverse_bindings = self.internal_get_reverse_lookup(&platform);
        reverse_bindings.insert(&proposal.handle, &account_id);
        self.reverse_lookup.insert(&platform, &reverse_bindings);

        Event::BindAccount {
            account_id: &account_id,
            platform: &platform,
            handle: &proposal.handle,
        }
        .emit();
    }
}

impl Contract {
    fn internal_get_account(&self, account_id: &AccountId) -> HashMap<Platform, String> {
        self.accounts.get(account_id).unwrap_or_default()
    }

    fn internal_get_proposals(&self, account_id: &AccountId) -> HashMap<Platform, BindingProposal> {
        self.binding_proposals.get(account_id).unwrap_or_default()
    }

    fn internal_get_reverse_lookup(&self, platform: &Platform) -> LookupMap<String, AccountId> {
        self.reverse_lookup.get(platform).unwrap_or_else(|| {
            LookupMap::new(StorageKey::ReverseBindings {
                platform: platform.clone(),
            })
        })
    }

    fn internal_remove_proposal(
        &mut self,
        account_id: &AccountId,
        platform: &Platform,
    ) -> BindingProposal {
        let mut proposals = self
            .binding_proposals
            .get(account_id)
            .expect("The account has no proposals");

        proposals
            .remove(platform)
            .expect("No proposals for the platform")
    }
}

// unit tests
// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn get_default_greeting() {
//         let contract = Contract::new();
//         // this test did not call set_greeting so should return the default "Hello" greeting
//         assert_eq!(contract.get_greeting(), "Hello".to_string());
//     }

//     #[test]
//     fn set_then_get_greeting() {
//         let mut contract = Contract::new();
//         contract.set_greeting("howdy".to_string());
//         assert_eq!(contract.get_greeting(), "howdy".to_string());
//     }
// }
