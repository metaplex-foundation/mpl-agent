import test from 'ava';
import {
  createNoopSigner,
  generateSigner,
  publicKey,
  some,
} from '@metaplex-foundation/umi';
import { execute, findAssetSignerPda } from '@metaplex-foundation/mpl-core';
import {
  createAccountWithRent,
  createMint,
  SPL_TOKEN_PROGRAM_ID,
} from '@metaplex-foundation/mpl-toolbox';
import {
  fetchAgentIdentityV2,
  findAgentIdentityV2Pda,
  Key,
  registerIdentityV1,
  setAgentTokenV1,
} from '../../src/generated/identity';
import { createCollectionAndAsset, createUmi } from '../_setup';

test('it can set an agent token', async (t) => {
  const umi = await createUmi();
  const { collection, asset } = await createCollectionAndAsset(umi);

  // Register identity.
  await registerIdentityV1(umi, {
    asset,
    collection,
    agentRegistrationUri: 'https://example.com/agent.json',
  }).sendAndConfirm(umi);

  // Create a token account (owned by SPL Token program).
  const agentTokenSigner = generateSigner(umi);
  await createMint(umi, {
    mint: agentTokenSigner,
  }).sendAndConfirm(umi);

  // Set agent token via Execute CPI.
  const agentIdentityPda = findAgentIdentityV2Pda(umi, { asset });
  const assetSignerPda = findAssetSignerPda(umi, { asset });

  await execute(umi, {
    asset: { publicKey: asset },
    collection: { publicKey: collection },
    instructions: setAgentTokenV1(umi, {
      asset,
      agentToken: agentTokenSigner.publicKey,
      authority: createNoopSigner(publicKey(assetSignerPda)),
    }),
  }).sendAndConfirm(umi);

  // Verify the agent token was set.
  const agentIdentity = await fetchAgentIdentityV2(umi, agentIdentityPda);
  t.is(agentIdentity.key, Key.AgentIdentityV2);
  t.deepEqual(agentIdentity.agentToken, some(agentTokenSigner.publicKey));
});

test('it cannot set agent token without asset signer authority', async (t) => {
  const umi = await createUmi();
  const { collection, asset } = await createCollectionAndAsset(umi);

  await registerIdentityV1(umi, {
    asset,
    collection,
    agentRegistrationUri: 'https://example.com/agent.json',
  }).sendAndConfirm(umi);

  const agentTokenSigner = generateSigner(umi);
  await createMint(umi, {
    mint: agentTokenSigner,
  }).sendAndConfirm(umi);

  // Call SetAgentTokenV1 directly (not via Execute) - payer is authority, not asset signer.
  const result = setAgentTokenV1(umi, {
    asset,
    agentToken: agentTokenSigner.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'OnlyAssetSignerCanSetAgentToken' });
});

test('it cannot set agent token twice', async (t) => {
  const umi = await createUmi();
  const { collection, asset } = await createCollectionAndAsset(umi);

  await registerIdentityV1(umi, {
    asset,
    collection,
    agentRegistrationUri: 'https://example.com/agent.json',
  }).sendAndConfirm(umi);

  const agentTokenSigner1 = generateSigner(umi);
  await createMint(umi, {
    mint: agentTokenSigner1,
  }).sendAndConfirm(umi);

  const agentTokenSigner2 = generateSigner(umi);
  await createMint(umi, {
    mint: agentTokenSigner2,
  }).sendAndConfirm(umi);

  const assetSignerPda = findAssetSignerPda(umi, { asset });

  // First set succeeds.
  await execute(umi, {
    asset: { publicKey: asset },
    collection: { publicKey: collection },
    instructions: setAgentTokenV1(umi, {
      asset,
      agentToken: agentTokenSigner1.publicKey,
      authority: createNoopSigner(publicKey(assetSignerPda)),
    }),
  }).sendAndConfirm(umi);

  // Second set should fail.
  const result = execute(umi, {
    asset: { publicKey: asset },
    collection: { publicKey: collection },
    instructions: setAgentTokenV1(umi, {
      asset,
      agentToken: agentTokenSigner2.publicKey,
      authority: createNoopSigner(publicKey(assetSignerPda)),
    }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { message: /0x7/ });
});

test('it cannot set agent token with invalid token account', async (t) => {
  const umi = await createUmi();
  const { collection, asset } = await createCollectionAndAsset(umi);

  await registerIdentityV1(umi, {
    asset,
    collection,
    agentRegistrationUri: 'https://example.com/agent.json',
  }).sendAndConfirm(umi);

  // Use a random account (not owned by SPL Token program) as the token.
  const fakeToken = generateSigner(umi);

  const assetSignerPda = findAssetSignerPda(umi, { asset });

  const result = execute(umi, {
    asset: { publicKey: asset },
    collection: { publicKey: collection },
    instructions: setAgentTokenV1(umi, {
      asset,
      agentToken: fakeToken.publicKey,
      authority: createNoopSigner(publicKey(assetSignerPda)),
    }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { message: /0x5/ });
});

test('it cannot set agent token on unregistered identity', async (t) => {
  const umi = await createUmi();
  const { collection, asset } = await createCollectionAndAsset(umi);

  // Do NOT register identity.

  const agentTokenSigner = generateSigner(umi);
  await createAccountWithRent(umi, {
    newAccount: agentTokenSigner,
    space: 165,
    programId: SPL_TOKEN_PROGRAM_ID,
  }).sendAndConfirm(umi);
  const agentToken = agentTokenSigner.publicKey;
  const assetSignerPda = findAssetSignerPda(umi, { asset });

  const result = execute(umi, {
    asset: { publicKey: asset },
    collection: { publicKey: collection },
    instructions: setAgentTokenV1(umi, {
      asset,
      agentToken,
      authority: createNoopSigner(publicKey(assetSignerPda)),
    }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { message: /0x8/ });
});
