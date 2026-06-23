import test from 'ava';
import {
  findTreeConfigPda,
  findLeafAssetIdPda,
  mplBubblegum,
} from '@metaplex-foundation/mpl-bubblegum';
import { generateSigner, publicKey } from '@metaplex-foundation/umi';

import {
  closeWorkReceiptV1,
  mintWorkReceiptV1,
} from '../../src/generated/tools';
import { createUmi } from '../_setup';
import {
  bootstrapReceipts,
  DEFAULT_ASSET_DATA_HASH,
  MPL_CORE_CPI_SIGNER,
  receiptCreatorHash,
  receiptDataHash,
  setupAgentWithExecutive,
} from '../_receiptsReviews';

async function mintReceiptForClient(
  umi: Awaited<ReturnType<typeof createUmi>>,
  ctx: Awaited<ReturnType<typeof bootstrapReceipts>>
) {
  const setup = await setupAgentWithExecutive(umi);
  const client = generateSigner(umi);
  await umi.rpc.airdrop(client.publicKey, {
    basisPoints: 100_000_000n,
    identifier: 'SOL',
    decimals: 9,
  });

  const receiptUri = 'https://example.com/job-spam-receipt.json';
  await mintWorkReceiptV1(umi, {
    executiveAuthority: setup.executive,
    executionDelegateRecord: setup.executionDelegateRecord,
    agentAsset: setup.agent,
    client: client.publicKey,
    treeConfig: findTreeConfigPda(umi, { merkleTree: ctx.receiptsTree }),
    merkleTree: ctx.receiptsTree,
    coreCollection: ctx.receiptsCollection,
    mplCoreCpiSigner: MPL_CORE_CPI_SIGNER,
    receiptUri,
    treeIndex: ctx.receiptsTreeIndex,
  }).sendAndConfirm(umi);

  return { setup, client, receiptUri };
}

test('closeWorkReceiptV1 — leaf owner can close their own receipt', async (t) => {
  const umi = (await createUmi()).use(mplBubblegum());
  const ctx = await bootstrapReceipts(umi);
  const { setup, client, receiptUri } = await mintReceiptForClient(umi, ctx);

  const [receiptAssetId] = findLeafAssetIdPda(umi, {
    merkleTree: ctx.receiptsTree,
    leafIndex: 0,
  });

  // Confirm the asset exists pre-close.
  // (Smoke check — we'll verify it's gone after by re-trying close.)

  await closeWorkReceiptV1(umi, {
    leafOwner: client,
    treeConfig: findTreeConfigPda(umi, { merkleTree: ctx.receiptsTree }),
    merkleTree: ctx.receiptsTree,
    coreCollection: ctx.receiptsCollection,
    mplCoreCpiSigner: MPL_CORE_CPI_SIGNER,
    treeIndex: ctx.receiptsTreeIndex,
    root: await currentRoot(umi, ctx.receiptsTree),
    dataHash: receiptDataHash({
      receiptUri,
      agent: setup.agent,
      client: client.publicKey,
      receiptsCollection: ctx.receiptsCollection,
    }),
    creatorHash: receiptCreatorHash(setup.agent, client.publicKey),
    assetDataHash: DEFAULT_ASSET_DATA_HASH,
    flags: 0,
    nonce: 0n,
    index: 0,
  }).sendAndConfirm(umi);

  // Retrying the same close on the (now-burned) leaf must fail —
  // compression rejects the now-stale leaf proof.
  const newRoot = await currentRoot(umi, ctx.receiptsTree);
  await t.throwsAsync(() =>
    closeWorkReceiptV1(umi, {
      leafOwner: client,
      treeConfig: findTreeConfigPda(umi, { merkleTree: ctx.receiptsTree }),
      merkleTree: ctx.receiptsTree,
      coreCollection: ctx.receiptsCollection,
      mplCoreCpiSigner: MPL_CORE_CPI_SIGNER,
      treeIndex: ctx.receiptsTreeIndex,
      root: newRoot,
      dataHash: receiptDataHash({
        receiptUri,
        agent: setup.agent,
        client: client.publicKey,
        receiptsCollection: ctx.receiptsCollection,
      }),
      creatorHash: receiptCreatorHash(setup.agent, client.publicKey),
      assetDataHash: DEFAULT_ASSET_DATA_HASH,
      flags: 0,
      nonce: 0n,
      index: 0,
    }).sendAndConfirm(umi)
  );

  // Quiet unused-var lint:
  void receiptAssetId;
});

test("closeWorkReceiptV1 — stranger cannot close someone else's receipt", async (t) => {
  const umi = (await createUmi()).use(mplBubblegum());
  const ctx = await bootstrapReceipts(umi);
  const { setup, client, receiptUri } = await mintReceiptForClient(umi, ctx);

  // Stranger signs as leaf_owner with a wallet that's not the real owner.
  // Bubblegum's BurnV2 reconstructs the leaf using `leaf_owner.key`, which
  // won't match the real leaf in the tree.
  const stranger = generateSigner(umi);
  await umi.rpc.airdrop(stranger.publicKey, {
    basisPoints: 100_000_000n,
    identifier: 'SOL',
    decimals: 9,
  });

  const root = await currentRoot(umi, ctx.receiptsTree);
  await t.throwsAsync(() =>
    closeWorkReceiptV1(umi, {
      leafOwner: stranger,
      treeConfig: findTreeConfigPda(umi, { merkleTree: ctx.receiptsTree }),
      merkleTree: ctx.receiptsTree,
      coreCollection: ctx.receiptsCollection,
      mplCoreCpiSigner: MPL_CORE_CPI_SIGNER,
      treeIndex: ctx.receiptsTreeIndex,
      root,
      dataHash: receiptDataHash({
        receiptUri,
        agent: setup.agent,
        client: client.publicKey,
        receiptsCollection: ctx.receiptsCollection,
      }),
      creatorHash: receiptCreatorHash(setup.agent, client.publicKey),
      assetDataHash: DEFAULT_ASSET_DATA_HASH,
      flags: 0,
      nonce: 0n,
      index: 0,
    }).sendAndConfirm(umi)
  );
});

async function currentRoot(
  umi: Awaited<ReturnType<typeof createUmi>>,
  merkleTree: ReturnType<typeof publicKey>
): Promise<Uint8Array> {
  const account = await umi.rpc.getAccount(merkleTree);
  if (!account.exists) throw new Error('tree account missing');
  return account.data.slice(88, 88 + 32);
}
