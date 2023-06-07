import {
  Platform,
  acceptBinding,
  getHandle,
  getProposal,
  init,
  lookupAccount,
  proposeBinding,
} from "./helper";

const test = init();

const twitter = Platform.Twitter;

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
