import test from 'ava';
import {
  findTreeConfigPda,
  mplBubblegum,
} from '@metaplex-foundation/mpl-bubblegum';
import { generateSigner, publicKey } from '@metaplex-foundation/umi';

import {
  fetchToolsConfigV1,
  findToolsConfigV1Pda,
  initializeToolsConfigV1,
  Key as ToolsKey,
  registerReceiptsTreeV1,
} from '../../src/generated/tools';
import { findReceiptsCollectionPda, findReceiptsTreePda } from '../../src';
import { createUmi } from '../_setup';
import {
  bootstrapReceiptsAndReviews,
  getSharedAdmin,
  TREE_MAX_BUFFER,
  TREE_MAX_DEPTH,
} from '../_receiptsReviews';

test('initializeToolsConfigV1 — cannot be called twice', async (t) => {
  const umi = (await createUmi()).use(mplBubblegum());
  await bootstrapReceiptsAndReviews(umi);

  // A second call must fail — the config PDA is now owned by us with data.
  await t.throwsAsync(() =>
    initializeToolsConfigV1(umi, {
      admin: getSharedAdmin(umi),
      collection: publicKey(findReceiptsCollectionPda(umi)),
    }).sendAndConfirm(umi)
  );
});

test('initializeToolsConfigV1 — captures admin and collection', async (t) => {
  const umi = (await createUmi()).use(mplBubblegum());
  await bootstrapReceiptsAndReviews(umi);

  const cfg = await fetchToolsConfigV1(umi, findToolsConfigV1Pda(umi));
  t.is(cfg.key, ToolsKey.ToolsConfigV1);
  t.is(cfg.admin, getSharedAdmin(umi).publicKey);
  t.is(cfg.collection, publicKey(findReceiptsCollectionPda(umi)));
});

test('registerReceiptsTreeV1 — admin only', async (t) => {
  const umi = (await createUmi()).use(mplBubblegum());
  const ctx = await bootstrapReceiptsAndReviews(umi);

  // Bootstrap already used index `ctx.receiptsTreeIndex`. Next is +1.
  const cfg = await fetchToolsConfigV1(umi, findToolsConfigV1Pda(umi));
  const nextIndex = cfg.nextTreeIndex;
  const nextTree = publicKey(
    findReceiptsTreePda(umi, { treeIndex: nextIndex })
  );

  const stranger = generateSigner(umi);
  await umi.rpc.airdrop(stranger.publicKey, {
    basisPoints: 1_000_000_000n,
    identifier: 'SOL',
    decimals: 9,
  });

  await t.throwsAsync(() =>
    registerReceiptsTreeV1(umi, {
      admin: stranger,
      merkleTree: nextTree,
      treeConfig: findTreeConfigPda(umi, { merkleTree: nextTree }),
      maxDepth: TREE_MAX_DEPTH,
      maxBufferSize: TREE_MAX_BUFFER,
      canopyDepth: 0,
    }).sendAndConfirm(umi)
  );
  // Quiet unused-var lint:
  void ctx;
});

test('registerReceiptsTreeV1 — increments next_tree_index', async (t) => {
  const umi = (await createUmi()).use(mplBubblegum());
  await bootstrapReceiptsAndReviews(umi);

  const admin = getSharedAdmin(umi);
  const before = (await fetchToolsConfigV1(umi, findToolsConfigV1Pda(umi)))
    .nextTreeIndex;

  const tree = publicKey(findReceiptsTreePda(umi, { treeIndex: before }));
  await registerReceiptsTreeV1(umi, {
    admin,
    merkleTree: tree,
    treeConfig: findTreeConfigPda(umi, { merkleTree: tree }),
    maxDepth: TREE_MAX_DEPTH,
    maxBufferSize: TREE_MAX_BUFFER,
    canopyDepth: 0,
  }).sendAndConfirm(umi);

  const after = (await fetchToolsConfigV1(umi, findToolsConfigV1Pda(umi)))
    .nextTreeIndex;
  t.is(after, before + 1n);
});

test('registerReceiptsTreeV1 — wrong tree PDA fails', async (t) => {
  const umi = (await createUmi()).use(mplBubblegum());
  await bootstrapReceiptsAndReviews(umi);

  const wrongTree = publicKey(findReceiptsTreePda(umi, { treeIndex: 9999n }));

  await t.throwsAsync(() =>
    registerReceiptsTreeV1(umi, {
      admin: getSharedAdmin(umi),
      merkleTree: wrongTree,
      treeConfig: findTreeConfigPda(umi, { merkleTree: wrongTree }),
      maxDepth: TREE_MAX_DEPTH,
      maxBufferSize: TREE_MAX_BUFFER,
      canopyDepth: 0,
    }).sendAndConfirm(umi)
  );
});
