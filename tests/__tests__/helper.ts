import { Worker, NearAccount } from "near-workspaces";
import anyTest, { TestFn } from "ava";

export function init() {
  const test = anyTest as TestFn<{
    worker: Worker;
    accounts: Record<string, NearAccount>;
  }>;

  test.beforeEach(async (t) => {
    // Init the worker and start a Sandbox server
    const worker = await Worker.init();

    // Create accounts
    const root = worker.rootAccount;
    const owner = await root.createSubAccount("owner");
    const alice = await root.createSubAccount("alice");
    const bob = await root.createSubAccount("bob");

    // Deploy contract
    const contract = await root.createSubAccount("account-bindings");
    await contract.deploy("tests/compiled-contracts/account_bindings.wasm");
    // Initialize contract
    await owner.call(contract, "new", {
      owner_id: owner,
    });

    // Save state for test runs, it is unique for each test
    t.context.worker = worker;
    t.context.accounts = { root, owner, contract, alice, bob };
  });

  test.afterEach(async (t) => {
    // Stop Sandbox server
    await t.context.worker.tearDown().catch((error) => {
      console.log("Failed to stop the Sandbox:", error);
    });
  });

  return test;
}

type AccountId = String;
type Timestamp = number;

export interface BindingProposal {
  account_id: AccountId;
  platform: Platform;
  handle: String;
  // proposal creation time
  created_at: Timestamp;
}

export enum Platform {
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
