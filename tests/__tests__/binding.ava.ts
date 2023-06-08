import {
  Platform,
  acceptBinding,
  assertFailure,
  getHandle,
  getProposal,
  init,
  lookupAccount,
  proposeBinding,
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

test("get default twitter handle", async (t) => {
  const { contract, alice } = t.context.accounts;
  t.is(await getHandle(contract, alice, twitter), "");
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
