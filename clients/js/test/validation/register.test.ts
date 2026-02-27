import test from 'ava';
import { fetchAsset } from '@metaplex-foundation/mpl-core';
import { publicKey } from '@metaplex-foundation/umi';
import {
  fetchAgentValidationV1,
  findAgentValidationV1Pda,
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

  // And there's an Agent Validation PDA.
  const agentValidationPda = findAgentValidationV1Pda(umi, { asset });
  const agentValidation = await fetchAgentValidationV1(umi, agentValidationPda);
  t.is(agentValidation.key, Key.AgentValidationV1);
  t.is(agentValidation.bump, agentValidationPda[1]);

  // Then the asset has a AppData plugin.
  const assetData = await fetchAsset(umi as any, asset);
  t.is(assetData?.appDatas?.length, 1);
  t.like(assetData?.appDatas?.[0], {
    dataAuthority: { type: 'Address', address: publicKey(agentValidationPda) },
    authority: { type: 'UpdateAuthority' },
  });
});
