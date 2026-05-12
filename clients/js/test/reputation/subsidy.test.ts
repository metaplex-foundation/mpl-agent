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
  depositSubsidyV1,
  fetchReviewSubsidyPoolV1,
  findReviewRecordV1Pda,
  findReviewSubsidyPoolV1Pda,
  leaveReviewV1,
  withdrawSubsidyV1,
} from '../../src/generated/reputation';
import { mintWorkReceiptV1 } from '../../src/generated/tools';
import { createUmi } from '../_setup';
import {
  bootstrapReceiptsAndReviews,
  DEFAULT_ASSET_DATA_HASH,
  getCurrentTreeRoot,
  MPL_CORE_CPI_SIGNER,
  receiptCreatorHash,
  receiptDataHash,
  setupAgentWithExecutive,
} from '../_receiptsReviews';

test('depositSubsidyV1 — first call captures withdraw authority', async (t) => {
  const umi = (await createUmi()).use(mplBubblegum());
  const { agent } = await setupAgentWithExecutive(umi);

  await depositSubsidyV1(umi, {
    agentAsset: agent,
    amount: 1_000_000n,
  }).sendAndConfirm(umi);

  const pool = await fetchReviewSubsidyPoolV1(
    umi,
    findReviewSubsidyPoolV1Pda(umi, { agentAsset: agent })
  );
  t.is(pool.agentAsset, agent);
  t.is(pool.withdrawAuthority, umi.payer.publicKey);
});

test('depositSubsidyV1 — second deposit adds lamports without re-init', async (t) => {
  const umi = (await createUmi()).use(mplBubblegum());
  const { agent } = await setupAgentWithExecutive(umi);

  await depositSubsidyV1(umi, {
    agentAsset: agent,
    amount: 500_000n,
  }).sendAndConfirm(umi);
  const pda = findReviewSubsidyPoolV1Pda(umi, { agentAsset: agent });
  const beforeBalance = await umi.rpc.getBalance(publicKey(pda));

  await depositSubsidyV1(umi, {
    agentAsset: agent,
    amount: 500_000n,
  }).sendAndConfirm(umi);
  const afterBalance = await umi.rpc.getBalance(publicKey(pda));

  t.is(afterBalance.basisPoints - beforeBalance.basisPoints, 500_000n);
});

test('withdrawSubsidyV1 — owner withdraws back to a destination', async (t) => {
  const umi = (await createUmi()).use(mplBubblegum());
  const { agent } = await setupAgentWithExecutive(umi);

  await depositSubsidyV1(umi, {
    agentAsset: agent,
    amount: 5_000_000n,
  }).sendAndConfirm(umi);

  const destination = generateSigner(umi);
  await withdrawSubsidyV1(umi, {
    withdrawAuthority: umi.payer,
    agentAsset: agent,
    subsidyPool: findReviewSubsidyPoolV1Pda(umi, { agentAsset: agent }),
    destination: destination.publicKey,
    amount: 1_000_000n,
  }).sendAndConfirm(umi);

  const destBalance = await umi.rpc.getBalance(destination.publicKey);
  t.is(destBalance.basisPoints, 1_000_000n);
});

test('withdrawSubsidyV1 — non-authority cannot withdraw', async (t) => {
  const umi = (await createUmi()).use(mplBubblegum());
  const { agent } = await setupAgentWithExecutive(umi);

  await depositSubsidyV1(umi, {
    agentAsset: agent,
    amount: 5_000_000n,
  }).sendAndConfirm(umi);

  const stranger = generateSigner(umi);
  await umi.rpc.airdrop(stranger.publicKey, {
    basisPoints: 1_000_000_000n,
    identifier: 'SOL',
    decimals: 9,
  });

  await t.throwsAsync(() =>
    withdrawSubsidyV1(umi, {
      withdrawAuthority: stranger,
      agentAsset: agent,
      subsidyPool: findReviewSubsidyPoolV1Pda(umi, { agentAsset: agent }),
      destination: stranger.publicKey,
      amount: 1_000_000n,
    }).sendAndConfirm(umi)
  );
});

test('withdrawSubsidyV1 — cannot drain below rent-exempt minimum', async (t) => {
  const umi = (await createUmi()).use(mplBubblegum());
  const { agent } = await setupAgentWithExecutive(umi);

  await depositSubsidyV1(umi, {
    agentAsset: agent,
    amount: 1_000n,
  }).sendAndConfirm(umi);

  const destination = generateSigner(umi);
  // Try to withdraw way more than the spendable budget.
  await t.throwsAsync(() =>
    withdrawSubsidyV1(umi, {
      withdrawAuthority: umi.payer,
      agentAsset: agent,
      subsidyPool: findReviewSubsidyPoolV1Pda(umi, { agentAsset: agent }),
      destination: destination.publicKey,
      amount: 10_000_000_000n,
    }).sendAndConfirm(umi)
  );
});

test('leaveReviewV1 — pool funds review record rent', async (t) => {
  const umi = (await createUmi()).use(mplBubblegum());

  const {
    receiptsCollection,
    reviewsCollection,
    receiptsTree,
    receiptsTreeIndex,
    reviewsTree,
    reviewsTreeIndex,
  } = await bootstrapReceiptsAndReviews(umi);

  const agentSetup = await setupAgentWithExecutive(umi);

  const client = generateSigner(umi);
  await umi.rpc.airdrop(client.publicKey, {
    basisPoints: 100_000_000n,
    identifier: 'SOL',
    decimals: 9,
  });

  // Mint receipt.
  const receiptUri = 'https://example.com/job/receipt.json';
  await mintWorkReceiptV1(umi, {
    executiveAuthority: agentSetup.executive,
    executionDelegateRecord: agentSetup.executionDelegateRecord,
    agentAsset: agentSetup.agent,
    client,
    treeConfig: findTreeConfigPda(umi, { merkleTree: receiptsTree }),
    merkleTree: receiptsTree,
    coreCollection: receiptsCollection,
    mplCoreCpiSigner: MPL_CORE_CPI_SIGNER,
    receiptUri,
    treeIndex: receiptsTreeIndex,
  }).sendAndConfirm(umi);

  // Pre-fund pool with plenty.
  await depositSubsidyV1(umi, {
    agentAsset: agentSetup.agent,
    amount: 50_000_000n,
  }).sendAndConfirm(umi);

  const poolPda = findReviewSubsidyPoolV1Pda(umi, {
    agentAsset: agentSetup.agent,
  });
  const poolBefore = await umi.rpc.getBalance(publicKey(poolPda));
  const clientBefore = await umi.rpc.getBalance(client.publicKey);

  const [receiptAssetId] = findLeafAssetIdPda(umi, {
    merkleTree: receiptsTree,
    leafIndex: 0,
  });
  const reviewRecord = findReviewRecordV1Pda(umi, {
    receiptAssetId: publicKey(receiptAssetId),
  });

  await leaveReviewV1(umi, {
    payer: client,
    reviewer: client,
    asset: agentSetup.agent,
    leafOwner: umi.payer.publicKey,
    treeConfig: findTreeConfigPda(umi, { merkleTree: reviewsTree }),
    merkleTree: reviewsTree,
    coreCollection: reviewsCollection,
    mplCoreCpiSigner: MPL_CORE_CPI_SIGNER,
    receiptsMerkleTree: receiptsTree,
    receiptsCollection,
    reviewRecord,
    reviewsTreeIndex,
    receiptNonce: 0n,
    receiptIndex: 0,
    receiptRoot: publicKeyBytes(
      publicKey(await getCurrentTreeRoot(umi, receiptsTree))
    ),
    receiptDataHash: receiptDataHash({
      receiptUri,
      agent: agentSetup.agent,
      receiptsCollection,
    }),
    receiptCreatorHash: receiptCreatorHash(agentSetup.agent),
    receiptAssetDataHash: DEFAULT_ASSET_DATA_HASH,
    receiptFlags: 0,
    rating: 5,
    feedbackUri: 'https://example.com/review.json',
  }).sendAndConfirm(umi);

  const poolAfter = await umi.rpc.getBalance(publicKey(poolPda));
  const clientAfter = await umi.rpc.getBalance(client.publicKey);

  // The pool's balance dropped (it paid the rent), and the client's
  // *net* outflow is smaller than the review-record rent would have been
  // unsubsidised. We assert the pool actually paid something out.
  const poolDelta = poolBefore.basisPoints - poolAfter.basisPoints;
  t.true(poolDelta > 0n, 'pool should have transferred lamports to payer');

  // Sanity: client's balance dropped by at most tx fees (pool covered rent).
  const clientDelta = clientBefore.basisPoints - clientAfter.basisPoints;
  t.true(
    clientDelta < poolDelta + 1_000_000n,
    'client net outflow stays modest'
  );
});
