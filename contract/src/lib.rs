mod events;
mod owner;
mod types;
mod upgrade;
mod view;

use events::Event;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::serde::Serialize;
use near_sdk::{
    env, near_bindgen, require, AccountId, Balance, PanicOnDefault, Timestamp, ONE_NEAR,
};
use std::collections::HashMap;
use types::*;

const PROPOSAL_STORAGE_COST: Balance = ONE_NEAR / 100; // 0.01 NEAR

const ERR_INVALID_HANDLE: &str = "Invalid handle";
const ERR_INVALID_PROPOSAL_CREATION_TIME: &str = "Proposal creation time must be in the past";
const ERR_WRONG_PROPOSAL: &str = "Proposal is not the verified one";
const ERR_NO_PROPOSALS: &str = "Account has no proposals";
const ERR_NO_ENOUGH_STORAGE_FEE: &str = "0.01 NEAR fee is required for each binding proposal";

/// Contract states
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    /// Owner ID who is usually a DAO that can upgrade the contract
    owner_id: AccountId,
    /// Manager accounts who are responsible for account binding
    managers: UnorderedSet<AccountId>,
    /// Binding proposals
    proposals: UnorderedMap<AccountId, HashMap<Platform, BindingProposal>>,
    /// Mapping from NEAR account ID to social media handles.
    bindings: UnorderedMap<AccountId, HashMap<Platform, String>>,
    /// Look up NEAR account ID with social media handle
    reverse_lookup: LookupMap<Platform, LookupMap<String, AccountId>>,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        Self {
            owner_id,
            managers: UnorderedSet::new(StorageKey::Managers),
            proposals: UnorderedMap::new(StorageKey::Proposals),
            bindings: UnorderedMap::new(StorageKey::Bindings),
            reverse_lookup: LookupMap::new(StorageKey::ReverseLookup),
        }
    }

    /// Submit the proposal of binding my NEAR account with a social media handle
    /// Permission: can be called by any user
    /// (Optional) If the handle is already bound with an NEAR account, the proposal is invalid.
    #[payable]
    pub fn propose_binding(&mut self, platform: Platform, handle: String) {
        require!(
            env::attached_deposit() >= PROPOSAL_STORAGE_COST,
            ERR_NO_ENOUGH_STORAGE_FEE
        );
        require!(!handle.is_empty(), ERR_INVALID_HANDLE);

        let account_id = env::predecessor_account_id();

        // check account and handle are not bound yet
        let bindings = self.internal_get_bindings(&account_id);
        require!(
            !bindings.contains_key(&platform),
            format!(
                "You account {} has already bound to handle {} on {}",
                account_id,
                bindings.get(&platform).unwrap(),
                platform
            )
        );
        let reverse_bindings = self.internal_get_reverse_bindings(&platform);
        require!(
            !reverse_bindings.contains_key(&handle),
            format!(
                "You handle {} on {} has already bound to account {}",
                handle,
                platform,
                reverse_bindings.get(&handle).unwrap(),
            )
        );

        // create proposal
        let mut proposals = self.internal_get_proposals(&account_id);
        let current_timestamp = env::block_timestamp_ms();
        let proposal = BindingProposal {
            account_id: account_id.clone(),
            platform: platform.clone(),
            handle: handle.clone(),
            created_at: current_timestamp,
        };
        proposals.insert(platform.clone(), proposal);
        self.proposals.insert(&account_id, &proposals);

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

    /// Bind NEAR accounts to proposed social media handles if authorization succeed
    /// The handle (e.g. twitter handle) provided in the proposal will be used. No need to provide in the function.
    /// To avoid fake binding attack, the creation timestamp of the proposal needs to be provided.
    /// Permission: can only be called by manager account
    pub fn accept_binding(
        &mut self,
        account_id: AccountId,
        platform: Platform,
        proposal_created_at: Timestamp,
    ) {
        self.assert_manager();
        require!(
            proposal_created_at < env::block_timestamp_ms(),
            ERR_INVALID_PROPOSAL_CREATION_TIME
        );

        let proposal = self.internal_remove_proposal(&account_id, &platform.clone());
        require!(
            proposal.created_at == proposal_created_at,
            ERR_WRONG_PROPOSAL
        );

        // insert bindings
        let mut bindings = self.internal_get_bindings(&account_id);
        require!(
            !bindings.contains_key(&platform),
            format!(
                "You account {} has already bound to handle {} on {}",
                account_id,
                bindings.get(&platform).unwrap(),
                platform
            )
        );
        bindings.insert(platform.clone(), proposal.handle.clone());
        self.bindings.insert(&account_id, &bindings);

        // update reverse bindings
        let mut reverse_bindings = self.internal_get_reverse_bindings(&platform);
        require!(
            !reverse_bindings.contains_key(&proposal.handle),
            format!(
                "You handle {} on {} has already bound to account {}",
                proposal.handle,
                platform,
                reverse_bindings.get(&proposal.handle).unwrap(),
            )
        );
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
    fn internal_get_bindings(&self, account_id: &AccountId) -> HashMap<Platform, String> {
        self.bindings.get(account_id).unwrap_or_default()
    }

    fn internal_get_proposals(&self, account_id: &AccountId) -> HashMap<Platform, BindingProposal> {
        self.proposals.get(account_id).unwrap_or_default()
    }

    fn internal_get_reverse_bindings(&self, platform: &Platform) -> LookupMap<String, AccountId> {
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
        let mut proposals = self.proposals.get(account_id).expect(ERR_NO_PROPOSALS);
        let proposal = proposals
            .remove(platform)
            .unwrap_or_else(|| panic!("Account has no proposals for {}", platform));

        self.proposals.insert(account_id, &proposals);
        proposal
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
