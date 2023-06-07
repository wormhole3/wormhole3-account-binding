import { addManager, getOwnerId, init, isManager, setOwner } from "./helper";

const test = init();

test("change owner and add manager", async (t) => {
  const { contract, owner, alice, bob } = t.context.accounts;

  // change owner to alice
  await setOwner(contract, owner, alice);
  t.is(await getOwnerId(contract), alice.accountId);

  // alice adds manager bob
  await addManager(contract, alice, bob);
  t.true(await isManager(contract, bob));
});
