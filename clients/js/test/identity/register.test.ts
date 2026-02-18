import test from 'ava';
import { fetchCollection } from '@metaplex-foundation/mpl-core';
import { publicKey } from '@metaplex-foundation/umi';
import {
  fetchAgentIdentityV1,
  fetchCollectionIdentityConfigV1,
  findAgentIdentityV1Pda,
  findCollectionIdentityConfigV1Pda,
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

  // Then there's a Collection Config PDA.
  const collectionConfigPda = findCollectionIdentityConfigV1Pda(umi, {
    collection,
  });
  const collectionConfig = await fetchCollectionIdentityConfigV1(
    umi,
    collectionConfigPda
  );
  t.is(collectionConfig.key, Key.CollectionIdentityConfigV1);
  t.is(collectionConfig.bump, collectionConfigPda[1]);

  // And there's an Agent Identity PDA.
  const agentIdentityPda = findAgentIdentityV1Pda(umi, { asset });
  const agentIdentity = await fetchAgentIdentityV1(umi, agentIdentityPda);
  t.is(agentIdentity.key, Key.AgentIdentityV1);
  t.is(agentIdentity.bump, agentIdentityPda[1]);

  // Then the collection has a LinkedAppData plugin.
  const collectionData = await fetchCollection(umi, collection);
  t.is(collectionData?.linkedAppDatas?.length, 1);
  t.like(collectionData?.linkedAppDatas?.[0], {
    dataAuthority: { type: 'Address', address: publicKey(collectionConfigPda) },
    authority: { type: 'UpdateAuthority' },
  });
});
