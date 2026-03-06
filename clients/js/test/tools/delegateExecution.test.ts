import test from 'ava';
import { generateSigner } from '@metaplex-foundation/umi';
import {
  findAgentIdentityV1Pda,
  registerIdentityV1,
} from '../../src/generated/identity';
import {
  delegateExecutionV1,
  fetchExecutionDelegateRecordV1,
  findExecutionDelegateRecordV1Pda,
  Key,
  registerExecutorV1,
} from '../../src/generated/tools';
import { createCollectionAndAsset, createUmi } from '../_setup';

test('it can delegate execution to an executor', async (t) => {
  // Given a Umi instance.
  const umi = await createUmi();

  // Create a collection and asset.
  const { collection, asset } = await createCollectionAndAsset(umi);

  // Register the asset's identity.
  await registerIdentityV1(umi, {
    asset,
    collection,
    agentRegistrationUri: 'https://example.com/agent.json',
  }).sendAndConfirm(umi);

  // Register an executor profile.
  const executorProfile = await generateSigner(umi);
  await registerExecutorV1(umi, {
    executorProfile,
  }).sendAndConfirm(umi);

  // When we delegate execution to the executor for the agent asset.
  const agentIdentityPda = findAgentIdentityV1Pda(umi, { asset });
  await delegateExecutionV1(umi, {
    executorProfile: executorProfile.publicKey,
    agentAsset: asset,
    agentIdentity: agentIdentityPda,
  }).sendAndConfirm(umi);

  // Then the execution delegate record is created.
  const delegateRecordPda = findExecutionDelegateRecordV1Pda(umi, {
    executorProfile: executorProfile.publicKey,
    agentAsset: asset,
  });
  const delegateRecord = await fetchExecutionDelegateRecordV1(
    umi,
    delegateRecordPda
  );
  t.is(delegateRecord.key, Key.ExecutionDelegateRecordV1);
  t.is(delegateRecord.bump, delegateRecordPda[1]);
  t.is(delegateRecord.executorProfile, executorProfile.publicKey);
  t.is(delegateRecord.agentAsset, asset);
});
