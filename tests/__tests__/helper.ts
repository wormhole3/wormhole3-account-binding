import { Worker, NearAccount, TransactionResult, NEAR } from "near-workspaces";
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
    const contract = await root.createSubAccount("wormhole3-account-binding");
    await contract.deploy(
      "tests/compiled-contracts/wormhole3_account_binding.wasm"
    );
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

function parseError(e: any): string {
  try {
    let status: any =
      e && e.parse ? e.parse().result.status : JSON.parse(e.message);
    return status.Failure.ActionError.kind.FunctionCallError.ExecutionError;
  } catch (_) {
    return e.message;
  }
}

export async function assertFailure(
  test: any,
  action: Promise<unknown>,
  errorMessage?: string
) {
  let failed = false;

  try {
    const results = await action;
    if (results && results instanceof TransactionResult) {
      for (const outcome of results.receipts_outcomes) {
        if (outcome.isFailure) {
          failed = true;
          if (errorMessage) {
            const actualErr = JSON.stringify(outcome.executionFailure);
            test.truthy(
              JSON.stringify(actualErr).includes(errorMessage),
              `Bad error message. expected: "${errorMessage}", actual: "${actualErr}"`
            );
          }
        }
      }
    }
  } catch (e) {
    if (errorMessage) {
      let msg: string = parseError(e);
      test.truthy(
        msg.includes(errorMessage),
        `Bad error message. expect: "${errorMessage}", actual: "${msg}"`
      );
    }
    failed = true;
  }

  test.is(failed, true, "Function call didn't fail");
}

// Types

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
  handle: String,
  fee: NEAR = NEAR.parse("0.01N")
) {
  return user.call(
    contract,
    "propose_binding",
    {
      platform,
      handle,
    },
    {
      attachedDeposit: fee,
    }
  );
}

export async function cancelProposal(
  contract: NearAccount,
  user: NearAccount,
  platform: Platform
) {
  return user.call(contract, "cancel_binding_proposal", {
    platform,
  });
}

export async function acceptBinding(
  contract: NearAccount,
  manager: NearAccount,
  user: NearAccount,
  platform: Platform,
  verificationTimestamp: Timestamp = Date.now() - 1
) {
  return manager.call(contract, "accept_binding", {
    account_id: user,
    platform,
    verification_timestamp: verificationTimestamp,
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
