import test from 'ava';
import { fetchAsset } from '@metaplex-foundation/mpl-core';
import {
  fetchAgentIdentityV2,
  findAgentIdentityV2Pda,
  Key,
  registerIdentityV1,
} from '../../src/generated/identity';
import { createCollectionAndAsset, createUmi } from '../_setup';

test('it can register an asset', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  // Create the collection and asset.
  const { collection, asset } = await createCollectionAndAsset(umi);

  const agentRegistrationUri = 'https://example.com/agent.json';

  // When we register the asset.
  await registerIdentityV1(umi, {
    asset,
    collection,
    agentRegistrationUri,
  }).sendAndConfirm(umi);

  // And there's an Agent Identity PDA.
  const agentIdentityPda = findAgentIdentityV2Pda(umi, { asset });
  const agentIdentity = await fetchAgentIdentityV2(umi, agentIdentityPda);
  t.is(agentIdentity.key, Key.AgentIdentityV2);
  t.is(agentIdentity.bump, agentIdentityPda[1]);

  const assetData = await fetchAsset(umi, asset);
  // Then the asset has an AgentIdentity plugin.
  t.is(assetData?.agentIdentities?.length, 1);
  t.like(assetData?.agentIdentities?.[0], {
    type: 'AgentIdentity',
    uri: agentRegistrationUri,
    authority: { type: 'UpdateAuthority' },
  });

  // Verify lifecycle checks for Transfer, Burn, and Execute.
  const lifecycleChecks = assetData?.agentIdentities?.[0]?.lifecycleChecks;
  t.truthy(lifecycleChecks?.transfer);
  t.truthy(lifecycleChecks?.update);
  t.truthy(lifecycleChecks?.execute);
});

test('it cannot register an asset twice', async (t) => {
  const umi = await createUmi();
  const { collection, asset } = await createCollectionAndAsset(umi);

  // First registration succeeds.
  await registerIdentityV1(umi, {
    asset,
    collection,
    agentRegistrationUri: 'https://example.com/agent.json',
  }).sendAndConfirm(umi);

  // Second registration should fail.
  const result = registerIdentityV1(umi, {
    asset,
    collection,
    agentRegistrationUri: 'https://example.com/agent2.json',
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'AgentIdentityAlreadyRegistered' });
});
