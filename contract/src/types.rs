use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{AccountId, BorshStorageKey, Timestamp};
use std::fmt;

#[derive(BorshStorageKey, BorshSerialize)]
pub(crate) enum StorageKey {
    Managers,
    Proposals,
    Bindings,
    ReverseLookup,
    ReverseBindings { platform: Platform },
}

/// Platform that we will verify
#[derive(
    Serialize,
    Deserialize,
    BorshDeserialize,
    BorshSerialize,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Hash,
    Debug,
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

/// display platform in lower case
impl fmt::Display for Platform {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", format!("{:?}", self).to_lowercase())
    }
}

/// Binding Proposal
#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct BindingProposal {
    pub account_id: AccountId,
    pub platform: Platform,
    pub handle: String,
    // proposal creation time
    pub created_at: Timestamp,
}
