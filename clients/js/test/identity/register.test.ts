import test from 'ava';
import { fetchAsset } from '@metaplex-foundation/mpl-core';
import { publicKey } from '@metaplex-foundation/umi';
import {
  fetchAgentIdentityV1,
  findAgentIdentityV1Pda,
  Key,
  registerIdentityV1,
} from '../../src/generated/identity';
import { createCollectionAndAsset, createUmi } from '../_setup';

test('it can register an asset', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  // Create the collection and asset.
  const { collection, asset } = await createCollectionAndAsset(umi);

  // When we register the asset.
  await registerIdentityV1(umi, {
    asset,
    collection,
  }).sendAndConfirm(umi);

  // And there's an Agent Identity PDA.
  const agentIdentityPda = findAgentIdentityV1Pda(umi, { asset });
  const agentIdentity = await fetchAgentIdentityV1(umi, agentIdentityPda);
  t.is(agentIdentity.key, Key.AgentIdentityV1);
  t.is(agentIdentity.bump, agentIdentityPda[1]);

  // Then the asset has a AppData plugin.
  const assetData = await fetchAsset(umi, asset);
  t.is(assetData?.appDatas?.length, 1);
  t.like(assetData?.appDatas?.[0], {
    dataAuthority: { type: 'Address', address: publicKey(agentIdentityPda) },
    authority: { type: 'UpdateAuthority' },
  });
});
