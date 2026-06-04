import test from 'ava';
import {
  findTreeConfigPda,
  findLeafAssetIdPda,
  mplBubblegum,
} from '@metaplex-foundation/mpl-bubblegum';
import {
  generateSigner,
  publicKey,
  publicKeyBytes,
  PublicKey,
  Signer,
} from '@metaplex-foundation/umi';

import {
  findReviewRecordV1Pda,
  leaveReviewV1,
} from '../../src/generated/reputation';
import { mintWorkReceiptV1 } from '../../src/generated/tools';
import { findReceiptsTreePda, findReviewsTreePda } from '../../src';
import { createUmi } from '../_setup';
import {
  bootstrapReceiptsAndReviews,
  DEFAULT_ASSET_DATA_HASH,
  getCurrentTreeRoot,
  MPL_CORE_CPI_SIGNER,
  receiptDataHash,
  setupAgentWithExecutive,
} from '../_receiptsReviews';

/**
 * Common setup: bootstrap + agent + client + a real minted receipt, plus
 * a `sharedArgs` object pre-populated with everything LeaveReviewV1 needs
 * for the happy path. Each test mutates one field to flip into a negative.
 */
async function setupContext(umi: Awaited<ReturnType<typeof createUmi>>) {
  const ctx = await bootstrapReceiptsAndReviews(umi);
  const agentSetup = await setupAgentWithExecutive(umi);

  const client: Signer = generateSigner(umi);
  await umi.rpc.airdrop(client.publicKey, {
    basisPoints: 100_000_000n,
    identifier: 'SOL',
    decimals: 9,
  });

  const receiptUri = 'https://example.com/job/receipt.json';
  await mintWorkReceiptV1(umi, {
    executiveAuthority: agentSetup.executive,
    executionDelegateRecord: agentSetup.executionDelegateRecord,
    agentAsset: agentSetup.agent,
    client,
    treeConfig: findTreeConfigPda(umi, { merkleTree: ctx.receiptsTree }),
    merkleTree: ctx.receiptsTree,
    coreCollection: ctx.receiptsCollection,
    mplCoreCpiSigner: MPL_CORE_CPI_SIGNER,
    receiptUri,
    treeIndex: ctx.receiptsTreeIndex,
  }).sendAndConfirm(umi);

  const [receiptAssetId] = findLeafAssetIdPda(umi, {
    merkleTree: ctx.receiptsTree,
    leafIndex: 0,
  });
  const reviewRecord = findReviewRecordV1Pda(umi, {
    receiptAssetId: publicKey(receiptAssetId),
  });

  const sharedArgs = {
    payer: client,
    reviewer: client,
    asset: agentSetup.agent,
    leafOwner: umi.payer.publicKey,
    treeConfig: findTreeConfigPda(umi, { merkleTree: ctx.reviewsTree }),
    merkleTree: ctx.reviewsTree,
    coreCollection: ctx.reviewsCollection,
    mplCoreCpiSigner: MPL_CORE_CPI_SIGNER,
    receiptsMerkleTree: ctx.receiptsTree,
    receiptsCollection: ctx.receiptsCollection,
    reviewRecord,
    reviewsTreeIndex: ctx.reviewsTreeIndex,
    receiptsTreeIndex: ctx.receiptsTreeIndex,
    receiptNonce: 0n,
    receiptIndex: 0,
    receiptRoot: publicKeyBytes(
      publicKey(await getCurrentTreeRoot(umi, ctx.receiptsTree))
    ),
    receiptDataHash: receiptDataHash({
      receiptUri,
      agent: agentSetup.agent,
      receiptsCollection: ctx.receiptsCollection,
    }),
    receiptAssetDataHash: DEFAULT_ASSET_DATA_HASH,
    receiptFlags: 0,
    rating: 5,
    feedbackUri: 'https://example.com/review.json',
  } as const;

  return { ctx, agentSetup, client, sharedArgs };
}

test('leaveReviewV1 — rejects rating 0', async (t) => {
  const umi = (await createUmi()).use(mplBubblegum());
  const { sharedArgs } = await setupContext(umi);
  await t.throwsAsync(() =>
    leaveReviewV1(umi, { ...sharedArgs, rating: 0 }).sendAndConfirm(umi)
  );
});

test('leaveReviewV1 — rejects rating 6', async (t) => {
  const umi = (await createUmi()).use(mplBubblegum());
  const { sharedArgs } = await setupContext(umi);
  await t.throwsAsync(() =>
    leaveReviewV1(umi, { ...sharedArgs, rating: 6 }).sendAndConfirm(umi)
  );
});

test('leaveReviewV1 — rejects empty feedback URI', async (t) => {
  const umi = (await createUmi()).use(mplBubblegum());
  const { sharedArgs } = await setupContext(umi);
  await t.throwsAsync(() =>
    leaveReviewV1(umi, { ...sharedArgs, feedbackUri: '' }).sendAndConfirm(umi)
  );
});

test('leaveReviewV1 — rejects leafOwner that does not match asset owner', async (t) => {
  const umi = (await createUmi()).use(mplBubblegum());
  const { sharedArgs } = await setupContext(umi);
  const wrong = generateSigner(umi).publicKey;
  await t.throwsAsync(() =>
    leaveReviewV1(umi, { ...sharedArgs, leafOwner: wrong }).sendAndConfirm(umi)
  );
});

test('leaveReviewV1 — rejects mismatched reviews collection', async (t) => {
  const umi = (await createUmi()).use(mplBubblegum());
  const { sharedArgs } = await setupContext(umi);
  const wrongCollection: PublicKey = generateSigner(umi).publicKey;
  await t.throwsAsync(() =>
    leaveReviewV1(umi, {
      ...sharedArgs,
      coreCollection: wrongCollection,
    }).sendAndConfirm(umi)
  );
});

test('leaveReviewV1 — rejects mismatched receipts collection', async (t) => {
  const umi = (await createUmi()).use(mplBubblegum());
  const { sharedArgs } = await setupContext(umi);
  const wrongCollection: PublicKey = generateSigner(umi).publicKey;
  await t.throwsAsync(() =>
    leaveReviewV1(umi, {
      ...sharedArgs,
      receiptsCollection: wrongCollection,
    }).sendAndConfirm(umi)
  );
});

test('leaveReviewV1 — rejects reviews tree at wrong PDA index', async (t) => {
  const umi = (await createUmi()).use(mplBubblegum());
  const { sharedArgs } = await setupContext(umi);

  // Bump the tree_index arg so the merkle_tree no longer matches the PDA
  // derivation. Bubblegum's tree_config also won't match, but our check
  // for the tree-PDA derivation trips first.
  await t.throwsAsync(() =>
    leaveReviewV1(umi, {
      ...sharedArgs,
      reviewsTreeIndex: sharedArgs.reviewsTreeIndex + 5n,
    }).sendAndConfirm(umi)
  );
});

test('leaveReviewV1 — rejects merkle_tree that does not match reviewsTreeIndex', async (t) => {
  const umi = (await createUmi()).use(mplBubblegum());
  const { sharedArgs } = await setupContext(umi);

  // Swap to a tree at a different PDA index — the on-chain check
  // (`check_reviews_tree_pda`) compares the supplied tree to
  // `["reviews_tree", reviewsTreeIndex_le]`.
  const fakeTree = publicKey(findReviewsTreePda(umi, { treeIndex: 9999n }));
  await t.throwsAsync(() =>
    leaveReviewV1(umi, {
      ...sharedArgs,
      merkleTree: fakeTree,
      treeConfig: findTreeConfigPda(umi, { merkleTree: fakeTree }),
    }).sendAndConfirm(umi)
  );
});

test('leaveReviewV1 — rejects receipts_merkle_tree that is not the canonical PDA', async (t) => {
  // Without this PDA check, an attacker could stand up their own
  // Bubblegum-compatible compression tree, append a forged work-receipt
  // leaf to it (the attacker is the tree authority), then pass it as
  // `receipts_merkle_tree`. The on-chain `verify_leaf_cpi` would happily
  // confirm the forged leaf is in the attacker's tree and a fake review
  // would be minted against any target agent.
  //
  // The fix derives the canonical receipts-tree PDA from the supplied
  // `receipts_tree_index` arg and rejects any tree that isn't at that
  // exact PDA. Substituting a different (still-canonical) receipts tree
  // here trips the check before verify_leaf is even reached.
  const umi = (await createUmi()).use(mplBubblegum());
  const { sharedArgs } = await setupContext(umi);

  const otherTree = publicKey(findReceiptsTreePda(umi, { treeIndex: 9999n }));
  await t.throwsAsync(() =>
    leaveReviewV1(umi, {
      ...sharedArgs,
      receiptsMerkleTree: otherTree,
    }).sendAndConfirm(umi)
  );
});

test('leaveReviewV1 — rejects bogus receipt data_hash (verify_leaf fails)', async (t) => {
  const umi = (await createUmi()).use(mplBubblegum());
  const { sharedArgs } = await setupContext(umi);

  // Flip a byte of the data_hash — the reconstructed leaf hash won't
  // match what's actually in the receipts tree, so mpl-account-
  // compression's verify_leaf CPI returns an error.
  const bogus = new Uint8Array(sharedArgs.receiptDataHash);
  bogus[0] ^= 0xff;

  await t.throwsAsync(() =>
    leaveReviewV1(umi, {
      ...sharedArgs,
      receiptDataHash: bogus,
    }).sendAndConfirm(umi)
  );
});

test('leaveReviewV1 — rejects receipt replay against a different agent', async (t) => {
  // A receipt minted for AgentA must not be usable to fake a review for
  // AgentB. The on-chain code computes the expected creator_hash from
  // `ctx.accounts.asset.key`, so the reconstructed leaf hash differs
  // from the real leaf in the receipts tree → verify_leaf rejects.
  const umi = (await createUmi()).use(mplBubblegum());
  const { sharedArgs, agentSetup: agentASetup } = await setupContext(umi);

  // Stand up a second, unrelated agent (different asset, different
  // executive, different delegate record). Nothing was minted for them.
  const agentBSetup = await setupAgentWithExecutive(umi);

  // Reuse AgentA's receipt proof params verbatim, but flip the reviewed
  // asset to AgentB. The failure must come from mpl-account-compression's
  // verify_leaf (error 0x1771 = ConcurrentMerkleTreeError) — that's the
  // exact signal of leaf-hash mismatch. Asserting it specifically
  // prevents this test from silently passing if some earlier check ever
  // starts rejecting the call first.
  await t.throwsAsync(
    leaveReviewV1(umi, {
      ...sharedArgs,
      asset: agentBSetup.agent,
    }).sendAndConfirm(umi),
    { message: /0x1771/ }
  );

  // Quiet unused-var lint:
  void agentASetup;
});
