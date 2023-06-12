use crate::*;
use near_sdk::{env, Gas, GasWeight, Promise, PromiseOrValue};

const GAS_FOR_GET_OWNER_ID: Gas = Gas(10_000_000_000_000);

#[near_bindgen]
impl Contract {
    /// Should only be called by this contract on migration.
    /// This is NOOP implementation. KEEP IT if you haven't changed contract state.
    /// If you have changed state, you need to implement migration from old state (keep the old
    /// struct with different name to deserialize it first).
    /// After migration goes live, revert back to this implementation for next updates.
    #[init(ignore_state)]
    #[private]
    pub fn migrate() -> Self {
        let contract: Contract = env::state_read().expect("ERR_NOT_INITIALIZED");
        contract
    }

    pub fn upgrade(&mut self) -> PromiseOrValue<AccountId> {
        self.assert_owner();
        let code = env::input().expect("ERR_CONTRACT_IS_NOT_INITIALIZED");
        // Create batch action promise for the current contract ID, with 3 actions:
        // 1. deploy contract with code taken from register 0
        // 2. call `migrate()` for migrating contract states
        // 3. call `get_owner_id()` to ensure the migration works and owner ID can be deserialized
        Promise::new(env::current_account_id())
            .deploy_contract(code)
            .function_call_weight("migrate".into(), vec![], 0, Gas(0), GasWeight(1))
            .function_call_weight(
                "get_owner_id".into(),
                vec![],
                0,
                GAS_FOR_GET_OWNER_ID,
                GasWeight(0),
            )
            .into()
    }
}
