use crate::*;
use near_sdk::{log, AccountId, Timestamp};
use serde::Serialize;
use serde_json::json;

const EVENT_STANDARD: &str = "account-bindings";
const EVENT_STANDARD_VERSION: &str = "1.0.0";

#[derive(Serialize, Clone)]
#[serde(tag = "event", content = "data")]
#[serde(rename_all = "snake_case")]
#[must_use = "Don't forget to `.emit()` this event"]
pub enum Event<'a> {
    ProposeBinding {
        account_id: &'a AccountId,
        platform: &'a Platform,
        handle: &'a String,
        created_at: &'a Timestamp,
    },
    CancelBindingProposal {
        account_id: &'a AccountId,
        platform: &'a Platform,
        handle: &'a String,
        created_at: &'a Timestamp,
    },
    BindAccount {
        account_id: &'a AccountId,
        platform: &'a Platform,
        handle: &'a String,
    },
}

impl Event<'_> {
    pub fn emit(&self) {
        let data = json!(self);
        let event_json = json!({
            "standard": EVENT_STANDARD,
            "version": EVENT_STANDARD_VERSION,
            "event": data["event"],
            "data": [data["data"]]
        })
        .to_string();
        log!("EVENT_JSON:{}", event_json);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::{self, test_env::alice};

    #[test]
    fn prposal_binding() {
        Event::ProposeBinding {
            account_id: &alice(),
            platform: &Platform::Twitter,
            handle: &"alice001".to_string(),
            created_at: &1600000000000,
        }
        .emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"account-bindings","version":"1.0.0","event":"propose_binding","data":[{"account_id":"alice.near","platform":"twitter","handle":"alice001","created_at":1600000000000}]}"#
        );
    }
}
