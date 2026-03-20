import test from 'ava';
import { fetchAsset } from '@metaplex-foundation/mpl-core';
import { publicKey } from '@metaplex-foundation/umi';
import {
  fetchAgentReputationV1,
  findAgentReputationV1Pda,
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

  // And there's an Agent Reputation PDA.
  const agentReputationPda = findAgentReputationV1Pda(umi, { asset });
  const agentReputation = await fetchAgentReputationV1(umi, agentReputationPda);
  t.is(agentReputation.key, Key.AgentReputationV1);
  t.is(agentReputation.bump, agentReputationPda[1]);

  // Then the asset has a AppData plugin.
  const assetData = await fetchAsset(umi, asset);
  t.is(assetData?.appDatas?.length, 1);
  t.like(assetData?.appDatas?.[0], {
    dataAuthority: { type: 'Address', address: publicKey(agentReputationPda) },
    authority: { type: 'UpdateAuthority' },
  });
});
