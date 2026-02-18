import test from 'ava';
import { fetchCollection } from '@metaplex-foundation/mpl-core';
import { publicKey } from '@metaplex-foundation/umi';
import {
  fetchAgentReputationV1,
  fetchCollectionReputationConfigV1,
  findAgentReputationV1Pda,
  findCollectionReputationConfigV1Pda,
  Key,
  registerReputationV1,
} from '../../src/generated/reputation';
import { createCollectionAndAsset, createUmi } from '../_setup';

test('it can register an asset', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  // Create the collection and asset.
  const { collection, asset } = await createCollectionAndAsset(umi);

  // When we register the asset.
  await registerReputationV1(umi, {
    asset,
    collection,
  }).sendAndConfirm(umi);

  // Then there's a Collection Reputation Config PDA.
  const collectionConfigPda = findCollectionReputationConfigV1Pda(umi, {
    collection,
  });
  const collectionConfig = await fetchCollectionReputationConfigV1(
    umi,
    collectionConfigPda
  );
  t.is(collectionConfig.key, Key.CollectionReputationConfigV1);
  t.is(collectionConfig.bump, collectionConfigPda[1]);

  // And there's an Agent Reputation PDA.
  const agentReputationPda = findAgentReputationV1Pda(umi, { asset });
  const agentReputation = await fetchAgentReputationV1(umi, agentReputationPda);
  t.is(agentReputation.key, Key.AgentReputationV1);
  t.is(agentReputation.bump, agentReputationPda[1]);

  // Then the collection has a LinkedAppData plugin.
  const collectionData = await fetchCollection(umi, collection);
  t.is(collectionData?.linkedAppDatas?.length, 1);
  t.like(collectionData?.linkedAppDatas?.[0], {
    dataAuthority: { type: 'Address', address: publicKey(collectionConfigPda) },
    authority: { type: 'UpdateAuthority' },
  });
});
