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
    const manager = await root.createSubAccount("manager");
    const alice = await root.createSubAccount("alice");
    const bob = await root.createSubAccount("bob");

    // Deploy contract
    const contract = await root.createSubAccount("account-bindings");
    await contract.deploy("tests/compiled-contracts/account_bindings.wasm");
    // Initialize contract
    await owner.call(contract, "new", {
      owner_id: owner,
    });

    // Add manager
    await addManager(contract, owner, manager);

    // Save state for test runs, it is unique for each test
    t.context.worker = worker;
    t.context.accounts = { root, contract, owner, manager, alice, bob };
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
  Twitter = "twitter",
  Facebook = "facebook",
  Reddit = "reddit",
  GitHub = "github",
  Telegram = "telegram",
  Discord = "discord",
  Instagram = "instagram",
  Ethereum = "ethereum",
  Hive = "hive",
  Steem = "steem",
}

// Binding methods

export async function proposeBinding(
  contract: NearAccount,
  user: NearAccount,
  platform: Platform,
  handle: String
) {
  return user.call(contract, "propose_binding", {
    platform,
    handle,
  });
}

export async function acceptBinding(
  contract: NearAccount,
  manager: NearAccount,
  user: NearAccount,
  platform: Platform
) {
  await manager.call(contract, "accept_binding", {
    account_id: user,
    platform,
    verification_timestamp: Date.now() - 1,
  });
}

export async function getProposal(
  contract: NearAccount,
  user: NearAccount,
  platform: Platform
): Promise<BindingProposal> {
  return contract.view("get_proposal", {
    account_id: user,
    platform,
  });
}

export async function getHandle(
  contract: NearAccount,
  user: NearAccount,
  platform: Platform
): Promise<String> {
  return contract.view("get_handle", {
    account_id: user,
    platform,
  });
}

export async function lookupAccount(
  contract: NearAccount,
  platform: Platform,
  handle: String
): Promise<String> {
  return contract.view("lookup_account", {
    platform,
    handle,
  });
}

// Admin methods

export async function setOwner(
  contract: NearAccount,
  owner: NearAccount,
  new_owner: NearAccount
) {
  return owner.call(contract, "set_owner", {
    new_owner_id: new_owner,
  });
}

export async function addManager(
  contract: NearAccount,
  owner: NearAccount,
  manager: NearAccount
) {
  return owner.call(contract, "add_manager", {
    manager_id: manager,
  });
}

export async function removeManager(
  contract: NearAccount,
  owner: NearAccount,
  manager: NearAccount
) {
  return owner.call(contract, "remove_manager", {
    manager_id: manager,
  });
}

export async function getOwnerId(contract: NearAccount) {
  return contract.view("get_owner_id");
}

export async function isManager(contract: NearAccount, accountId: NearAccount) {
  return contract.view("is_manager", {
    account_id: accountId,
  });
}
