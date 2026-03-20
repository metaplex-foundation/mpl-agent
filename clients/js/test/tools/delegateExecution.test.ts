import test from 'ava';
import { generateSigner, publicKey, sol } from '@metaplex-foundation/umi';
import { execute, findAssetSignerPda } from '@metaplex-foundation/mpl-core';
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
  await registerExecutiveV1(umi, {}).sendAndConfirm(umi);
  const executiveProfile = findExecutiveProfileV1Pda(umi, {
    authority: umi.identity.publicKey,
  });

  // When we delegate execution to the executor for the agent asset.
  const agentIdentityPda = findAgentIdentityV1Pda(umi, { asset });
  await delegateExecutionV1(umi, {
    executiveProfile: publicKey(executiveProfile),
    agentAsset: asset,
    agentIdentity: agentIdentityPda,
  }).sendAndConfirm(umi);

  // Then the execution delegate record is created.
  const delegateRecordPda = findExecutionDelegateRecordV1Pda(umi, {
    executiveProfile: publicKey(executiveProfile),
    agentAsset: asset,
  });
  const delegateRecord = await fetchExecutionDelegateRecordV1(
    umi,
    delegateRecordPda
  );
  t.is(delegateRecord.key, Key.ExecutionDelegateRecordV1);
  t.is(delegateRecord.bump, delegateRecordPda[1]);
  t.is(delegateRecord.executiveProfile, publicKey(executiveProfile));
  t.is(delegateRecord.authority, umi.identity.publicKey);
  t.is(delegateRecord.agentAsset, asset);
});

test('it can execute as the execution delegate without the owner signing', async (t) => {
  const umi = await createUmi();

  // Create a collection and asset (owner = umi.identity).
  const { collection, asset } = await createCollectionAndAsset(umi);

  // Register identity on the asset (adds AgentIdentity plugin with execute lifecycle check).
  await registerIdentityV1(umi, {
    asset,
    collection,
    agentRegistrationUri: 'https://example.com/agent.json',
  }).sendAndConfirm(umi);

  // Create a separate authority for the executive (different from asset owner).
  const executiveAuthority = generateSigner(umi);

  // Register executive profile with the custom authority.
  await registerExecutiveV1(umi, {
    authority: executiveAuthority,
  }).sendAndConfirm(umi);

  const executiveProfile = findExecutiveProfileV1Pda(umi, {
    authority: executiveAuthority.publicKey,
  });

  // Owner delegates execution to the executive for the agent asset.
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

  // Execute as the delegate (NOT the owner).
  // The delegate record must be at account index 7 (first remaining account)
  // for the AgentIdentity plugin's validate_execute to verify it.
  // We wrap a SPL Noop instruction so the delegate record is the only remaining account key.
  const noopProgramId = publicKey(
    'noopb9bkMVfRPU8AsbpTUg8AQkHtKwMYZiFUjNRtMmV'
  );
  await execute(umi, {
    asset: { publicKey: asset },
    collection: { publicKey: collection },
    authority: executiveAuthority,
    instructions: [
      {
        programId: noopProgramId,
        keys: [
          {
            pubkey: publicKey(delegateRecordPda),
            isSigner: false,
            isWritable: false,
          },
        ],
        data: new Uint8Array([]),
      },
    ],
  }).sendAndConfirm(umi);

  t.pass();
});

test('it can transfer SOL via delegate execution', async (t) => {
  const umi = await createUmi();

  const { collection, asset } = await createCollectionAndAsset(umi);

  await registerIdentityV1(umi, {
    asset,
    collection,
    agentRegistrationUri: 'https://example.com/agent.json',
  }).sendAndConfirm(umi);

  const executiveAuthority = generateSigner(umi);
  await registerExecutiveV1(umi, {
    authority: executiveAuthority,
  }).sendAndConfirm(umi);

  const executiveProfile = findExecutiveProfileV1Pda(umi, {
    authority: executiveAuthority.publicKey,
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

  // Fund the asset signer PDA so it has SOL to transfer.
  const assetSignerPda = findAssetSignerPda(umi, { asset });
  await umi.rpc.airdrop(publicKey(assetSignerPda), sol(1));

  // Execute a system transfer as the delegate.
  // The delegate record is the first remaining account (index 7) for the AgentIdentity
  // plugin check. Core strips it before the inner CPI so the system transfer
  // correctly receives [assetSigner, recipient].
  const recipient = generateSigner(umi);
  const systemProgramId = publicKey('11111111111111111111111111111111');

  // System Transfer instruction data: u32 type (2) + u64 lamports
  const transferData = new Uint8Array(12);
  const view = new DataView(transferData.buffer);
  view.setUint32(0, 2, true); // Transfer = 2
  view.setBigUint64(4, BigInt(500_000_000), true); // 0.5 SOL

  await execute(umi, {
    asset: { publicKey: asset },
    collection: { publicKey: collection },
    authority: executiveAuthority,
    instructions: [
      {
        programId: systemProgramId,
        keys: [
          {
            pubkey: publicKey(delegateRecordPda),
            isSigner: false,
            isWritable: false,
          },
          {
            pubkey: publicKey(assetSignerPda),
            isSigner: false,
            isWritable: true,
          },
          { pubkey: recipient.publicKey, isSigner: false, isWritable: true },
        ],
        data: transferData,
      },
    ],
  }).sendAndConfirm(umi);

  // Verify the recipient received the SOL.
  const recipientBalance = await umi.rpc.getBalance(recipient.publicKey);
  t.is(recipientBalance.basisPoints, BigInt(500_000_000));
});

test('the owner can still execute even with the AgentIdentity plugin', async (t) => {
  const umi = await createUmi();

  // Create a collection and asset (owner = umi.identity).
  const { collection, asset } = await createCollectionAndAsset(umi);

  // Register identity on the asset (adds AgentIdentity plugin with execute lifecycle check).
  await registerIdentityV1(umi, {
    asset,
    collection,
    agentRegistrationUri: 'https://example.com/agent.json',
  }).sendAndConfirm(umi);

  // The owner can still execute without any delegate record.
  // The AgentIdentity plugin abstains when no valid delegate record is at index 7,
  // and the owner is approved by Core's default ownership check.
  const noopProgramId = publicKey(
    'noopb9bkMVfRPU8AsbpTUg8AQkHtKwMYZiFUjNRtMmV'
  );
  await execute(umi, {
    asset: { publicKey: asset },
    collection: { publicKey: collection },
    instructions: [
      {
        programId: noopProgramId,
        keys: [],
        data: new Uint8Array([]),
      },
    ],
  }).sendAndConfirm(umi);

  t.pass();
});

test('it cannot execute without a valid execution delegate', async (t) => {
  const umi = await createUmi();

  // Create a collection and asset (owner = umi.identity).
  const { collection, asset } = await createCollectionAndAsset(umi);

  // Register identity on the asset (adds AgentIdentity plugin with execute lifecycle check).
  await registerIdentityV1(umi, {
    asset,
    collection,
    agentRegistrationUri: 'https://example.com/agent.json',
  }).sendAndConfirm(umi);

  // Create a non-owner signer who has NOT been delegated execution.
  const attacker = generateSigner(umi);

  // Attempt to execute as the attacker without a delegate record.
  const noopProgramId = publicKey(
    'noopb9bkMVfRPU8AsbpTUg8AQkHtKwMYZiFUjNRtMmV'
  );
  const result = execute(umi, {
    asset: { publicKey: asset },
    collection: { publicKey: collection },
    authority: attacker,
    instructions: [
      {
        programId: noopProgramId,
        keys: [],
        data: new Uint8Array([]),
      },
    ],
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { message: /0x1a/ });
});

test('it cannot delegate execution if not the asset owner', async (t) => {
  const umi = await createUmi();

  const { collection, asset } = await createCollectionAndAsset(umi);

  await registerIdentityV1(umi, {
    asset,
    collection,
    agentRegistrationUri: 'https://example.com/agent.json',
  }).sendAndConfirm(umi);

  // Register an executive profile for a separate authority.
  const executiveAuthority = generateSigner(umi);
  await registerExecutiveV1(umi, {
    authority: executiveAuthority,
  }).sendAndConfirm(umi);

  const executiveProfile = findExecutiveProfileV1Pda(umi, {
    authority: executiveAuthority.publicKey,
  });

  // A non-owner tries to delegate execution.
  const nonOwner = generateSigner(umi);
  const agentIdentityPda = findAgentIdentityV1Pda(umi, { asset });
  const result = delegateExecutionV1(umi, {
    executiveProfile: publicKey(executiveProfile),
    agentAsset: asset,
    agentIdentity: agentIdentityPda,
    authority: nonOwner,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'AssetOwnerMustBeTheOneToDelegateExecution',
  });
});

test('it cannot delegate execution without a registered identity', async (t) => {
  const umi = await createUmi();

  // Create a collection and asset but do NOT register identity.
  const { asset } = await createCollectionAndAsset(umi);

  // Register an executive profile.
  await registerExecutiveV1(umi, {}).sendAndConfirm(umi);
  const executiveProfile = findExecutiveProfileV1Pda(umi, {
    authority: umi.identity.publicKey,
  });

  // Use a fake agent identity PDA (it won't be initialized).
  const agentIdentityPda = findAgentIdentityV1Pda(umi, { asset });
  const result = delegateExecutionV1(umi, {
    executiveProfile: publicKey(executiveProfile),
    agentAsset: asset,
    agentIdentity: agentIdentityPda,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'AgentIdentityNotRegistered' });
});

test('it cannot delegate execution twice to the same executive', async (t) => {
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

  // First delegation succeeds.
  await delegateExecutionV1(umi, {
    executiveProfile: publicKey(executiveProfile),
    agentAsset: asset,
    agentIdentity: agentIdentityPda,
  }).sendAndConfirm(umi);

  // Second delegation to the same executive for the same asset fails.
  const result = delegateExecutionV1(umi, {
    executiveProfile: publicKey(executiveProfile),
    agentAsset: asset,
    agentIdentity: agentIdentityPda,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'ExecutionDelegateRecordMustBeUninitialized',
  });
});

test('it cannot delegate execution with an uninitialized executive profile', async (t) => {
  const umi = await createUmi();

  const { collection, asset } = await createCollectionAndAsset(umi);

  await registerIdentityV1(umi, {
    asset,
    collection,
    agentRegistrationUri: 'https://example.com/agent.json',
  }).sendAndConfirm(umi);

  // Use a PDA for an executive profile that was never registered.
  const fakeAuthority = generateSigner(umi);
  const uninitializedProfile = findExecutiveProfileV1Pda(umi, {
    authority: fakeAuthority.publicKey,
  });

  const agentIdentityPda = findAgentIdentityV1Pda(umi, { asset });
  const result = delegateExecutionV1(umi, {
    executiveProfile: publicKey(uninitializedProfile),
    agentAsset: asset,
    agentIdentity: agentIdentityPda,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'ExecutiveProfileMustBeUninitialized' });
});
