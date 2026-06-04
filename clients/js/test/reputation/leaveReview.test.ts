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
} from '@metaplex-foundation/umi';

import {
  fetchReviewRecordV1,
  findReviewRecordV1Pda,
  Key as ReputationKey,
  leaveReviewV1,
} from '../../src/generated/reputation';
import { createUmi } from '../_setup';
import {
  bootstrapReceiptsAndReviews,
  DEFAULT_ASSET_DATA_HASH,
  getCurrentTreeRoot,
  MPL_CORE_CPI_SIGNER,
  receiptDataHash,
  setupAgentWithExecutive,
} from '../_receiptsReviews';
import { mintWorkReceiptV1 } from '../../src/generated/tools';

test('program-managed trees: full receipt → review flow', async (t) => {
  const umi = (await createUmi()).use(mplBubblegum());

  const {
    receiptsCollection,
    reviewsCollection,
    receiptsTree,
    receiptsTreeIndex,
    reviewsTree,
    reviewsTreeIndex,
  } = await bootstrapReceiptsAndReviews(umi);

  const { agent, executive, executionDelegateRecord } =
    await setupAgentWithExecutive(umi);

  const client = generateSigner(umi);
  await umi.rpc.airdrop(client.publicKey, {
    basisPoints: 100_000_000n,
    identifier: 'SOL',
    decimals: 9,
  });

  // Mint the work receipt.
  const receiptUri = 'https://example.com/job-1-receipt.json';
  await mintWorkReceiptV1(umi, {
    executiveAuthority: executive,
    executionDelegateRecord,
    agentAsset: agent,
    client,
    treeConfig: findTreeConfigPda(umi, { merkleTree: receiptsTree }),
    merkleTree: receiptsTree,
    coreCollection: receiptsCollection,
    mplCoreCpiSigner: MPL_CORE_CPI_SIGNER,
    receiptUri,
    treeIndex: receiptsTreeIndex,
  }).sendAndConfirm(umi);

  const [receiptAssetId] = findLeafAssetIdPda(umi, {
    merkleTree: receiptsTree,
    leafIndex: 0,
  });
  const reviewRecord = findReviewRecordV1Pda(umi, {
    receiptAssetId: publicKey(receiptAssetId),
  });

  const sharedArgs = {
    payer: client,
    reviewer: client,
    asset: agent,
    leafOwner: umi.payer.publicKey,
    treeConfig: findTreeConfigPda(umi, { merkleTree: reviewsTree }),
    merkleTree: reviewsTree,
    coreCollection: reviewsCollection,
    mplCoreCpiSigner: MPL_CORE_CPI_SIGNER,
    receiptsMerkleTree: receiptsTree,
    receiptsCollection,
    reviewRecord,
    reviewsTreeIndex,
    receiptsTreeIndex,
    receiptNonce: 0n,
    receiptIndex: 0,
    receiptRoot: publicKeyBytes(
      publicKey(await getCurrentTreeRoot(umi, receiptsTree))
    ),
    receiptDataHash: receiptDataHash({
      receiptUri,
      agent,
      receiptsCollection,
    }),
    receiptAssetDataHash: DEFAULT_ASSET_DATA_HASH,
    receiptFlags: 0,
  } as const;

  await leaveReviewV1(umi, {
    ...sharedArgs,
    rating: 5,
    feedbackUri: 'https://example.com/job-1-review.json',
  }).sendAndConfirm(umi);

  const record = await fetchReviewRecordV1(umi, reviewRecord);
  t.is(record.key, ReputationKey.ReviewRecordV1);
  t.is(record.reviewer, client.publicKey);
  t.is(record.receiptAssetId, publicKey(receiptAssetId));

  // Second review on the same receipt fails (PDA already initialized).
  await t.throwsAsync(() =>
    leaveReviewV1(umi, {
      ...sharedArgs,
      rating: 1,
      feedbackUri: 'https://example.com/job-1-review-2.json',
    }).sendAndConfirm(umi)
  );
});
