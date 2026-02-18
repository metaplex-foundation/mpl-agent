import test from 'ava';
import { fetchCollection } from '@metaplex-foundation/mpl-core';
import { publicKey } from '@metaplex-foundation/umi';
import {
  fetchAgentValidationV1,
  fetchCollectionValidationConfigV1,
  findAgentValidationV1Pda,
  findCollectionValidationConfigV1Pda,
  Key,
  registerValidationV1,
} from '../../src/generated/validation';
import { createCollectionAndAsset, createUmi } from '../_setup';

test('it can register an asset', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  // Create the collection and asset.
  const { collection, asset } = await createCollectionAndAsset(umi);

  // When we register the asset.
  await registerValidationV1(umi, {
    asset,
    collection,
  }).sendAndConfirm(umi);

  // Then there's a Collection Validation Config PDA.
  const collectionConfigPda = findCollectionValidationConfigV1Pda(umi, {
    collection,
  });
  const collectionConfig = await fetchCollectionValidationConfigV1(
    umi,
    collectionConfigPda
  );
  t.is(collectionConfig.key, Key.CollectionValidationConfigV1);
  t.is(collectionConfig.bump, collectionConfigPda[1]);

  // And there's an Agent Validation PDA.
  const agentValidationPda = findAgentValidationV1Pda(umi, { asset });
  const agentValidation = await fetchAgentValidationV1(umi, agentValidationPda);
  t.is(agentValidation.key, Key.AgentValidationV1);
  t.is(agentValidation.bump, agentValidationPda[1]);

  // Then the collection has a LinkedAppData plugin.
  const collectionData = await fetchCollection(umi, collection);
  t.is(collectionData?.linkedAppDatas?.length, 1);
  t.like(collectionData?.linkedAppDatas?.[0], {
    dataAuthority: { type: 'Address', address: publicKey(collectionConfigPda) },
    authority: { type: 'UpdateAuthority' },
  });
});
