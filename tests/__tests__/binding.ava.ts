import { NEAR } from "near-workspaces";
import {
  Platform,
  acceptBinding,
  assertFailure,
  getHandle,
  getProposal,
  init,
  lookupAccount,
  proposeBinding,
  cancelProposal,
} from "./helper";

const test = init();

const twitter = Platform.Twitter;
const discord = Platform.Discord;

const ERR_ACCOUNT_ALREADY_BOUND = (
  accountId: string,
  handle: string,
  platform: Platform
) =>
  `You account ${accountId} has already bound to handle ${handle} on ${platform}`;

const ERR_HANDLE_ALREADY_BOUND = (
  accountId: string,
  handle: string,
  platform: Platform
) =>
  `You handle ${handle} on ${platform} has already bound to account ${accountId}`;

const ERR_INVALID_HANDLE = "Invalid handle";
const ERR_INVALID_PROPOSAL_CREATION_TIME =
  "Proposal creation time must be in the past";
const ERR_WRONG_PROPOSAL = "Proposal is not the verified one";
const ERR_NO_PROPOSALS = "Account has no proposals";
const ERR_NO_PROPOSALS_FOR_PLATFORM = (platform: Platform) =>
  `Account has no proposals for ${platform}`;
const ERR_NO_ENOUGH_STORAGE_FEE =
  "0.01 NEAR fee is required for each binding proposal";

test("get default twitter handle", async (t) => {
  const { contract, alice } = t.context.accounts;
  t.is(await getHandle(contract, alice, twitter), "");
});

test("binding proposal requires 0.01N fee", async (t) => {
  const { contract, alice } = t.context.accounts;

  const aliceTwitterHandle = "alice001";

  // alice proposes binding
  await assertFailure(
    t,
    proposeBinding(
      contract,
      alice,
      twitter,
      aliceTwitterHandle,
      NEAR.parse("0")
    ),
    ERR_NO_ENOUGH_STORAGE_FEE
  );
});

test("submit and accept binding proposal", async (t) => {
  const { contract, manager, alice } = t.context.accounts;

  const aliceTwitterHandle = "alice001";

  // alice proposes binding
  await proposeBinding(contract, alice, twitter, aliceTwitterHandle);
  const proposal = await getProposal(contract, alice, twitter);
  t.is(proposal.handle, aliceTwitterHandle);

  // manager accepts binding
  await acceptBinding(contract, manager, alice, twitter);
  t.is(await getHandle(contract, alice, twitter), aliceTwitterHandle);
  t.is(
    await lookupAccount(contract, twitter, aliceTwitterHandle),
    alice.accountId
  );
});

test("cancel proposal", async (t) => {
  const { contract, alice } = t.context.accounts;
  const aliceTwitterHandle = "alice001";

  // alice proposes binding
  await proposeBinding(contract, alice, twitter, aliceTwitterHandle);
  const proposal = await getProposal(contract, alice, twitter);
  t.is(proposal.handle, aliceTwitterHandle);

  // alice cancels her binding proposal
  await cancelProposal(contract, alice, twitter);
  await assertFailure(
    t,
    getProposal(contract, alice, twitter),
    ERR_NO_PROPOSALS_FOR_PLATFORM(twitter)
  );
});

test("only allow 1-1 binding between account and handle on one platform", async (t) => {
  const { contract, manager, alice, bob } = t.context.accounts;

  const aliceTwitterHandle = "alice001";
  const aliceTwitterHandle2 = "alice002";
  const aliceDiscordHandle = "alice#0123";

  // alice proposes binding
  await proposeBinding(contract, alice, twitter, aliceTwitterHandle);
  // bob proposes binding to the same handle
  await proposeBinding(contract, bob, twitter, aliceTwitterHandle);

  // manager accepts binding of alice
  await acceptBinding(contract, manager, alice, twitter);
  t.is(await getHandle(contract, alice, twitter), aliceTwitterHandle);
  t.is(
    await lookupAccount(contract, twitter, aliceTwitterHandle),
    alice.accountId
  );

  // alice propose binding again, should be rejected
  await assertFailure(
    t,
    proposeBinding(contract, alice, twitter, aliceTwitterHandle2),
    ERR_ACCOUNT_ALREADY_BOUND(alice.accountId, aliceTwitterHandle, twitter)
  );

  // manager try to accept bob's proposal, should be rejected,
  // because the handle is already occupied
  await assertFailure(
    t,
    acceptBinding(contract, manager, bob, twitter),
    ERR_HANDLE_ALREADY_BOUND(alice.accountId, aliceTwitterHandle, twitter)
  );

  // alice proposes binding on Discord
  await proposeBinding(contract, alice, discord, aliceDiscordHandle);
  const proposal = await getProposal(contract, alice, discord);
  t.is(proposal.handle, aliceDiscordHandle);
});

test("can't accept nonexistent proposal", async (t) => {
  const { contract, manager, alice } = t.context.accounts;
  // manager accept a nonexistent proposal
  await assertFailure(
    t,
    acceptBinding(contract, manager, alice, twitter),
    ERR_NO_PROPOSALS
  );
});

test("can't accept proposal nonexistent on given platform", async (t) => {
  const { contract, manager, alice } = t.context.accounts;
  const aliceTwitterHandle = "alice001";
  // alice proposes binding on twitter
  await proposeBinding(contract, alice, twitter, aliceTwitterHandle);
  // manager accept alice's discord proposal, should be rejected
  await assertFailure(
    t,
    acceptBinding(contract, manager, alice, discord),
    ERR_NO_PROPOSALS_FOR_PLATFORM(discord)
  );
});

test("cannot accept proposal if provided with wrong creation time", async (t) => {
  const { contract, manager, alice } = t.context.accounts;
  const aliceTwitterHandle = "alice001";
  // alice proposes binding on twitter
  await proposeBinding(contract, alice, twitter, aliceTwitterHandle);

  // manager provides an incorrect proposal creation time, should be rejected
  await assertFailure(
    t,
    acceptBinding(contract, manager, alice, twitter, Date.now() - 10000),
    ERR_WRONG_PROPOSAL
  );

  // alice tries to make a fake binding by prosing a new binding
  const proposal = await getProposal(contract, alice, twitter);
  const elonMuskTwitterHandle = "elonmusk";
  await proposeBinding(contract, alice, twitter, elonMuskTwitterHandle);

  // manager provides an outdated proposal's creation time, should be rejected
  await assertFailure(
    t,
    acceptBinding(contract, manager, alice, twitter, proposal.created_at),
    ERR_WRONG_PROPOSAL
  );

  // manger provides a future proposal creation time, should be rejected
  await assertFailure(
    t,
    acceptBinding(contract, manager, alice, twitter, Date.now() + 10000),
    ERR_INVALID_PROPOSAL_CREATION_TIME
  );
});
