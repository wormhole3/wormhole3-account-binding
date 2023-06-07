import { BindingProposal, init } from "./helper";

const test = init();

test("get default twitter handle", async (t) => {
  const { contract, alice } = t.context.accounts;
  const handle = await contract.view("get_handle", {
    account_id: alice,
    platform: "twitter",
  });
  t.is(handle, "");
});

test("submit binding proposal", async (t) => {
  const { alice, contract } = t.context.accounts;
  await alice.call(contract, "propose_binding", {
    platform: "twitter",
    handle: "alice001",
  });
  const proposal: BindingProposal = await contract.view("get_proposal", {
    account_id: alice,
    platform: "twitter",
  });
  t.is(proposal.handle, "alice001");
});
