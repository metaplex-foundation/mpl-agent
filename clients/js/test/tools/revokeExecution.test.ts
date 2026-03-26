import test from 'ava';
import { generateSigner, publicKey } from '@metaplex-foundation/umi';
import {
  findAgentIdentityV1Pda,
  registerIdentityV1,
} from '../../src/generated/identity';
import {
  delegateExecutionV1,
  fetchExecutionDelegateRecordV1,
  findExecutionDelegateRecordV1Pda,
  findExecutiveProfileV1Pda,
  Key,
  registerExecutiveV1,
  revokeExecutionV1,
  safeFetchExecutionDelegateRecordV1,
} from '../../src/generated/tools';
import { createCollectionAndAsset, createUmi } from '../_setup';

test('it can revoke execution delegation as the asset owner', async (t) => {
  const umi = await createUmi();

  // Create a collection, asset, and register identity.
  const { collection, asset } = await createCollectionAndAsset(umi);
  await registerIdentityV1(umi, {
    asset,
    collection,
    agentRegistrationUri: 'https://example.com/agent.json',
  }).sendAndConfirm(umi);

  // Register an executor profile and delegate.
  await registerExecutiveV1(umi, {}).sendAndConfirm(umi);
  const executiveProfile = findExecutiveProfileV1Pda(umi, {
    authority: umi.identity.publicKey,
  });

  const agentIdentityPda = findAgentIdentityV1Pda(umi, { asset });
  await delegateExecutionV1(umi, {
    executiveProfile: publicKey(executiveProfile),
    agentAsset: asset,
    agentIdentity: agentIdentityPda,
  }).sendAndConfirm(umi);

  // Verify the delegate record exists.
  const delegateRecordPda = findExecutionDelegateRecordV1Pda(umi, {
    executiveProfile: publicKey(executiveProfile),
    agentAsset: asset,
  });
  const record = await fetchExecutionDelegateRecordV1(umi, delegateRecordPda);
  t.is(record.key, Key.ExecutionDelegateRecordV1);

  // When the owner revokes.
  await revokeExecutionV1(umi, {
    executionDelegateRecord: publicKey(delegateRecordPda),
    agentAsset: asset,
    destination: umi.identity.publicKey,
  }).sendAndConfirm(umi);

  // Then the delegate record is closed.
  const closed = await safeFetchExecutionDelegateRecordV1(
    umi,
    delegateRecordPda
  );
  t.is(closed, null);
});

test('it can revoke execution delegation as the executive authority', async (t) => {
  const umi = await createUmi();

  const { collection, asset } = await createCollectionAndAsset(umi);
  await registerIdentityV1(umi, {
    asset,
    collection,
    agentRegistrationUri: 'https://example.com/agent.json',
  }).sendAndConfirm(umi);

  // Register executive with a separate authority.
  const executiveAuthority = generateSigner(umi);
  await registerExecutiveV1(umi, {
    authority: executiveAuthority,
  }).sendAndConfirm(umi);

  const executiveProfile = findExecutiveProfileV1Pda(umi, {
    authority: executiveAuthority.publicKey,
  });

  // Owner delegates.
  const agentIdentityPda = findAgentIdentityV1Pda(umi, { asset });
  await delegateExecutionV1(umi, {
    executiveProfile: publicKey(executiveProfile),
    agentAsset: asset,
    agentIdentity: agentIdentityPda,
  }).sendAndConfirm(umi);

  const delegateRecordPda = findExecutionDelegateRecordV1Pda(umi, {
    executiveProfile: publicKey(executiveProfile),
    agentAsset: asset,
  });

  // When the executive authority revokes.
  await revokeExecutionV1(umi, {
    executionDelegateRecord: publicKey(delegateRecordPda),
    agentAsset: asset,
    destination: umi.identity.publicKey,
    authority: executiveAuthority,
  }).sendAndConfirm(umi);

  // Then the delegate record is closed.
  const closed = await safeFetchExecutionDelegateRecordV1(
    umi,
    delegateRecordPda
  );
  t.is(closed, null);
});

test('it cannot revoke an uninitialized delegate record', async (t) => {
  const umi = await createUmi();

  const { collection, asset } = await createCollectionAndAsset(umi);
  await registerIdentityV1(umi, {
    asset,
    collection,
    agentRegistrationUri: 'https://example.com/agent.json',
  }).sendAndConfirm(umi);

  // Register executive but do NOT delegate.
  await registerExecutiveV1(umi, {}).sendAndConfirm(umi);
  const executiveProfile = findExecutiveProfileV1Pda(umi, {
    authority: umi.identity.publicKey,
  });

  const delegateRecordPda = findExecutionDelegateRecordV1Pda(umi, {
    executiveProfile: publicKey(executiveProfile),
    agentAsset: asset,
  });

  // Attempting to revoke a non-existent delegate record fails.
  const result = revokeExecutionV1(umi, {
    executionDelegateRecord: publicKey(delegateRecordPda),
    agentAsset: asset,
    destination: umi.identity.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'ExecutionDelegateRecordMustBeInitialized',
  });
});

test('it cannot revoke as an unauthorized party', async (t) => {
  const umi = await createUmi();

  const { collection, asset } = await createCollectionAndAsset(umi);
  await registerIdentityV1(umi, {
    asset,
    collection,
    agentRegistrationUri: 'https://example.com/agent.json',
  }).sendAndConfirm(umi);

  // Register executive with a separate authority.
  const executiveAuthority = generateSigner(umi);
  await registerExecutiveV1(umi, {
    authority: executiveAuthority,
  }).sendAndConfirm(umi);

  const executiveProfile = findExecutiveProfileV1Pda(umi, {
    authority: executiveAuthority.publicKey,
  });

  // Owner delegates.
  const agentIdentityPda = findAgentIdentityV1Pda(umi, { asset });
  await delegateExecutionV1(umi, {
    executiveProfile: publicKey(executiveProfile),
    agentAsset: asset,
    agentIdentity: agentIdentityPda,
  }).sendAndConfirm(umi);

  const delegateRecordPda = findExecutionDelegateRecordV1Pda(umi, {
    executiveProfile: publicKey(executiveProfile),
    agentAsset: asset,
  });

  // A random third party tries to revoke.
  const attacker = generateSigner(umi);
  const result = revokeExecutionV1(umi, {
    executionDelegateRecord: publicKey(delegateRecordPda),
    agentAsset: asset,
    destination: umi.identity.publicKey,
    authority: attacker,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'UnauthorizedRevoke' });
});

test('it can re-delegate after revoking', async (t) => {
  const umi = await createUmi();

  const { collection, asset } = await createCollectionAndAsset(umi);
  await registerIdentityV1(umi, {
    asset,
    collection,
    agentRegistrationUri: 'https://example.com/agent.json',
  }).sendAndConfirm(umi);

  await registerExecutiveV1(umi, {}).sendAndConfirm(umi);
  const executiveProfile = findExecutiveProfileV1Pda(umi, {
    authority: umi.identity.publicKey,
  });

  const agentIdentityPda = findAgentIdentityV1Pda(umi, { asset });

  // First delegation.
  await delegateExecutionV1(umi, {
    executiveProfile: publicKey(executiveProfile),
    agentAsset: asset,
    agentIdentity: agentIdentityPda,
  }).sendAndConfirm(umi);

  const delegateRecordPda = findExecutionDelegateRecordV1Pda(umi, {
    executiveProfile: publicKey(executiveProfile),
    agentAsset: asset,
  });

  // Revoke.
  await revokeExecutionV1(umi, {
    executionDelegateRecord: publicKey(delegateRecordPda),
    agentAsset: asset,
    destination: umi.identity.publicKey,
  }).sendAndConfirm(umi);

  // Verify closed.
  const closed = await safeFetchExecutionDelegateRecordV1(
    umi,
    delegateRecordPda
  );
  t.is(closed, null);

  // Re-delegate — should succeed.
  await delegateExecutionV1(umi, {
    executiveProfile: publicKey(executiveProfile),
    agentAsset: asset,
    agentIdentity: agentIdentityPda,
  }).sendAndConfirm(umi);

  // Verify the new delegate record exists with correct data.
  const record = await fetchExecutionDelegateRecordV1(umi, delegateRecordPda);
  t.is(record.key, Key.ExecutionDelegateRecordV1);
  t.is(record.executiveProfile, publicKey(executiveProfile));
  t.is(record.authority, umi.identity.publicKey);
  t.is(record.agentAsset, asset);
});

test('rent lamports are refunded to the destination on revoke', async (t) => {
  const umi = await createUmi();

  const { collection, asset } = await createCollectionAndAsset(umi);
  await registerIdentityV1(umi, {
    asset,
    collection,
    agentRegistrationUri: 'https://example.com/agent.json',
  }).sendAndConfirm(umi);

  await registerExecutiveV1(umi, {}).sendAndConfirm(umi);
  const executiveProfile = findExecutiveProfileV1Pda(umi, {
    authority: umi.identity.publicKey,
  });

  const agentIdentityPda = findAgentIdentityV1Pda(umi, { asset });
  await delegateExecutionV1(umi, {
    executiveProfile: publicKey(executiveProfile),
    agentAsset: asset,
    agentIdentity: agentIdentityPda,
  }).sendAndConfirm(umi);

  const delegateRecordPda = findExecutionDelegateRecordV1Pda(umi, {
    executiveProfile: publicKey(executiveProfile),
    agentAsset: asset,
  });

  // Use a separate destination to verify lamports are refunded there.
  const destination = generateSigner(umi);
  const balanceBefore = await umi.rpc.getBalance(destination.publicKey);
  t.is(balanceBefore.basisPoints, BigInt(0));

  await revokeExecutionV1(umi, {
    executionDelegateRecord: publicKey(delegateRecordPda),
    agentAsset: asset,
    destination: destination.publicKey,
  }).sendAndConfirm(umi);

  // The destination should have received the rent lamports.
  const balanceAfter = await umi.rpc.getBalance(destination.publicKey);
  t.true(
    balanceAfter.basisPoints > BigInt(0),
    'Destination should have received rent refund'
  );
});
