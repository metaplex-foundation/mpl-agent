import test from 'ava';
import {
  createNoopSigner,
  generateSigner,
  publicKey,
  some,
} from '@metaplex-foundation/umi';
import { execute, findAssetSignerPda } from '@metaplex-foundation/mpl-core';
import {
  createAssociatedToken,
  createMint,
  findAssociatedTokenPda,
  mintTokensTo,
  setComputeUnitLimit,
} from '@metaplex-foundation/mpl-toolbox';
import {
  initializeV2,
  findGenesisAccountV2Pda,
} from '@metaplex-foundation/genesis';
import {
  fetchAgentIdentityV2,
  findAgentIdentityV2Pda,
  Key,
  registerIdentityV1,
  setAgentTokenV1,
} from '../../src/generated/identity';
import { createCollectionAndAsset, createUmi } from '../_setup';

/** Create a Genesis account via initializeV2 with the given funding mode.
 *  Authority defaults to the umi payer. */
async function createGenesisAccount(
  umi: Awaited<ReturnType<typeof createUmi>>,
  fundingMode: number
) {
  const baseMint = generateSigner(umi);
  const genesisAccountPda = findGenesisAccountV2Pda(umi, {
    baseMint: baseMint.publicKey,
    genesisIndex: 0,
  });

  await initializeV2(umi, {
    baseMint,
    fundingMode,
    totalSupplyBaseToken: 1_000_000_000n,
    name: 'Test Token',
    uri: 'https://example.com/metadata.json',
    symbol: 'TST',
  })
    .prepend(setComputeUnitLimit(umi, { units: 400_000 }))
    .sendAndConfirm(umi);

  return { baseMint: baseMint.publicKey, genesisAccount: genesisAccountPda };
}

/** Create a Genesis account via Execute CPI so the asset signer PDA is the authority. */
async function createGenesisAccountViaExecute(
  umi: Awaited<ReturnType<typeof createUmi>>,
  asset: ReturnType<typeof publicKey>,
  collection: ReturnType<typeof publicKey>,
  fundingMode: number
) {
  const baseMint = generateSigner(umi);
  const genesisAccountPda = findGenesisAccountV2Pda(umi, {
    baseMint: baseMint.publicKey,
    genesisIndex: 0,
  });
  const assetSignerPda = findAssetSignerPda(umi, { asset });

  // Build the inner initializeV2 instruction.
  const innerTx = initializeV2(umi, {
    baseMint,
    authority: createNoopSigner(publicKey(assetSignerPda)),
    fundingMode,
    totalSupplyBaseToken: 1_000_000_000n,
    name: 'Test Token',
    uri: 'https://example.com/metadata.json',
    symbol: 'TST',
  });

  // Wrap in execute CPI.
  const executeTx = execute(umi, {
    asset: { publicKey: asset },
    collection: { publicKey: collection },
    instructions: innerTx,
  });

  // The mpl-core execute() helper doesn't propagate inner TransactionBuilder
  // signers. Manually add them (excluding the asset signer which signs via CPI).
  const assetSignerKey = publicKey(assetSignerPda);
  const innerSigners = innerTx.items.flatMap((item) => item.signers);
  const items = executeTx.items;
  for (const item of items) {
    item.signers.push(
      ...innerSigners.filter((s) => s.publicKey !== assetSignerKey)
    );
  }

  await executeTx
    .setItems(items)
    .prepend(setComputeUnitLimit(umi, { units: 600_000 }))
    .sendAndConfirm(umi);

  return { baseMint: baseMint.publicKey, genesisAccount: genesisAccountPda };
}

test('it can set an agent token', async (t) => {
  const umi = await createUmi();
  const { collection, asset } = await createCollectionAndAsset(umi);

  // Register identity.
  await registerIdentityV1(umi, {
    asset,
    collection,
    agentRegistrationUri: 'https://example.com/agent.json',
  }).sendAndConfirm(umi);

  // Create a Genesis account via Execute CPI so asset signer is the authority.
  const { baseMint, genesisAccount } = await createGenesisAccountViaExecute(
    umi,
    asset,
    collection,
    0
  );

  // Set agent token via Execute CPI.
  const assetSignerPda = findAssetSignerPda(umi, { asset });

  await execute(umi, {
    asset: { publicKey: asset },
    collection: { publicKey: collection },
    instructions: setAgentTokenV1(umi, {
      asset,
      genesisAccount,
      authority: createNoopSigner(publicKey(assetSignerPda)),
    }),
  }).sendAndConfirm(umi);

  // Verify the agent token was set to the base_mint (NOT the genesis account address).
  const agentIdentityPda = findAgentIdentityV2Pda(umi, { asset });
  const agentIdentity = await fetchAgentIdentityV2(umi, agentIdentityPda);
  t.is(agentIdentity.key, Key.AgentIdentityV2);
  t.deepEqual(agentIdentity.agentToken, some(baseMint));
});

test('it cannot set agent token without asset signer authority', async (t) => {
  const umi = await createUmi();
  const { collection, asset } = await createCollectionAndAsset(umi);

  await registerIdentityV1(umi, {
    asset,
    collection,
    agentRegistrationUri: 'https://example.com/agent.json',
  }).sendAndConfirm(umi);

  const { genesisAccount } = await createGenesisAccount(umi, 0);

  // Call SetAgentTokenV1 directly (not via Execute) - payer is authority, not asset signer.
  const result = setAgentTokenV1(umi, {
    asset,
    genesisAccount,
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

  const { genesisAccount: genesisAccount1 } =
    await createGenesisAccountViaExecute(umi, asset, collection, 0);
  const { genesisAccount: genesisAccount2 } =
    await createGenesisAccountViaExecute(umi, asset, collection, 0);

  const assetSignerPda = findAssetSignerPda(umi, { asset });

  // First set succeeds.
  await execute(umi, {
    asset: { publicKey: asset },
    collection: { publicKey: collection },
    instructions: setAgentTokenV1(umi, {
      asset,
      genesisAccount: genesisAccount1,
      authority: createNoopSigner(publicKey(assetSignerPda)),
    }),
  }).sendAndConfirm(umi);

  // Second set should fail.
  const result = execute(umi, {
    asset: { publicKey: asset },
    collection: { publicKey: collection },
    instructions: setAgentTokenV1(umi, {
      asset,
      genesisAccount: genesisAccount2,
      authority: createNoopSigner(publicKey(assetSignerPda)),
    }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { message: /0x7/ });
});

test('it cannot set agent token with invalid genesis account', async (t) => {
  const umi = await createUmi();
  const { collection, asset } = await createCollectionAndAsset(umi);

  await registerIdentityV1(umi, {
    asset,
    collection,
    agentRegistrationUri: 'https://example.com/agent.json',
  }).sendAndConfirm(umi);

  // Use a random account (not owned by the Genesis program).
  const fakeAccount = generateSigner(umi);

  const assetSignerPda = findAssetSignerPda(umi, { asset });

  const result = execute(umi, {
    asset: { publicKey: asset },
    collection: { publicKey: collection },
    instructions: setAgentTokenV1(umi, {
      asset,
      genesisAccount: fakeAccount.publicKey,
      authority: createNoopSigner(publicKey(assetSignerPda)),
    }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { message: /0xa/ });
});

test('it cannot set agent token with transfer-funded genesis', async (t) => {
  const umi = await createUmi();
  const { collection, asset } = await createCollectionAndAsset(umi);

  await registerIdentityV1(umi, {
    asset,
    collection,
    agentRegistrationUri: 'https://example.com/agent.json',
  }).sendAndConfirm(umi);

  // Create a Genesis account with funding_mode = Transfer (1).
  // Transfer mode requires pre-creating the mint and funding an ATA.
  const authority = generateSigner(umi);
  const baseMint = generateSigner(umi);
  const totalSupply = 1_000_000_000n;

  // Create mint with authority as mint/freeze authority.
  await createMint(umi, {
    mint: baseMint,
    mintAuthority: authority.publicKey,
    freezeAuthority: authority.publicKey,
    decimals: 9,
  }).sendAndConfirm(umi);

  // Create authority's ATA and mint tokens to it.
  const authorityAta = findAssociatedTokenPda(umi, {
    mint: baseMint.publicKey,
    owner: authority.publicKey,
  });
  await createAssociatedToken(umi, {
    mint: baseMint.publicKey,
    owner: authority.publicKey,
  }).sendAndConfirm(umi);

  await mintTokensTo(umi, {
    mint: baseMint.publicKey,
    token: authorityAta,
    mintAuthority: authority,
    amount: totalSupply,
  }).sendAndConfirm(umi);

  // Initialize Genesis with Transfer mode.
  const genesisAccountPda = findGenesisAccountV2Pda(umi, {
    baseMint: baseMint.publicKey,
    genesisIndex: 0,
  });

  await initializeV2(umi, {
    baseMint,
    authority,
    fundingMode: 1,
    totalSupplyBaseToken: totalSupply,
    name: 'Transfer Token',
    uri: 'https://example.com/metadata.json',
    symbol: 'TFR',
  })
    .prepend(setComputeUnitLimit(umi, { units: 400_000 }))
    .sendAndConfirm(umi);

  const assetSignerPda = findAssetSignerPda(umi, { asset });

  const result = execute(umi, {
    asset: { publicKey: asset },
    collection: { publicKey: collection },
    instructions: setAgentTokenV1(umi, {
      asset,
      genesisAccount: genesisAccountPda,
      authority: createNoopSigner(publicKey(assetSignerPda)),
    }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { message: /0xb/ });
});

test('it cannot set agent token on unregistered identity', async (t) => {
  const umi = await createUmi();
  const { collection, asset } = await createCollectionAndAsset(umi);

  // Do NOT register identity.

  const { genesisAccount } = await createGenesisAccount(umi, 0);
  const assetSignerPda = findAssetSignerPda(umi, { asset });

  const result = execute(umi, {
    asset: { publicKey: asset },
    collection: { publicKey: collection },
    instructions: setAgentTokenV1(umi, {
      asset,
      genesisAccount,
      authority: createNoopSigner(publicKey(assetSignerPda)),
    }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { message: /0x8/ });
});
