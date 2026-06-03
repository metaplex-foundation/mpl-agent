import test from 'ava';
import {
  findTreeConfigPda,
  mplBubblegum,
} from '@metaplex-foundation/mpl-bubblegum';
import { publicKey } from '@metaplex-foundation/umi';

import {
  createReceiptsCollectionV1,
  findReceiptsCollectionPda,
  findReceiptsTreePda,
  registerReceiptsTreeV1,
} from '../../src/generated/tools';
import { createUmi } from '../_setup';
import {
  bootstrapReceiptsAndReviews,
  TREE_MAX_BUFFER,
  TREE_MAX_DEPTH,
} from '../_receiptsReviews';

test('createReceiptsCollectionV1 — second call fails (collection already initialized)', async (t) => {
  const umi = (await createUmi()).use(mplBubblegum());
  await bootstrapReceiptsAndReviews(umi);

  await t.throwsAsync(() =>
    createReceiptsCollectionV1(umi, {}).sendAndConfirm(umi)
  );
});

test('createReceiptsCollectionV1 — collection lives at the canonical PDA', async (t) => {
  const umi = (await createUmi()).use(mplBubblegum());
  await bootstrapReceiptsAndReviews(umi);

  const account = await umi.rpc.getAccount(
    publicKey(findReceiptsCollectionPda(umi))
  );
  t.true(account.exists);
});

test('registerReceiptsTreeV1 — racing the same index fails the loser', async (t) => {
  const umi = (await createUmi()).use(mplBubblegum());
  await bootstrapReceiptsAndReviews(umi);

  // Random index so re-running against the same validator session doesn't
  // collide with a previously-registered tree at a hard-coded index.
  const buf = new Uint8Array(7);
  crypto.getRandomValues(buf);
  let index = 0n;
  for (const b of buf) index = (index << 8n) | BigInt(b);
  const tree = publicKey(findReceiptsTreePda(umi, { treeIndex: index }));
  await registerReceiptsTreeV1(umi, {
    merkleTree: tree,
    treeConfig: findTreeConfigPda(umi, { merkleTree: tree }),
    treeIndex: index,
    maxDepth: TREE_MAX_DEPTH,
    maxBufferSize: TREE_MAX_BUFFER,
    canopyDepth: 0,
  }).sendAndConfirm(umi);

  await t.throwsAsync(() =>
    registerReceiptsTreeV1(umi, {
      merkleTree: tree,
      treeConfig: findTreeConfigPda(umi, { merkleTree: tree }),
      treeIndex: index,
      maxDepth: TREE_MAX_DEPTH,
      maxBufferSize: TREE_MAX_BUFFER,
      canopyDepth: 0,
    }).sendAndConfirm(umi)
  );
});

test('registerReceiptsTreeV1 — wrong tree PDA for the supplied index fails', async (t) => {
  const umi = (await createUmi()).use(mplBubblegum());
  await bootstrapReceiptsAndReviews(umi);

  // Pass an unrelated tree pubkey while claiming an arbitrary index — the
  // PDA derivation check rejects.
  const wrongTree = publicKey(findReceiptsTreePda(umi, { treeIndex: 1n }));
  const claimedIndex = 2n;

  await t.throwsAsync(() =>
    registerReceiptsTreeV1(umi, {
      merkleTree: wrongTree,
      treeConfig: findTreeConfigPda(umi, { merkleTree: wrongTree }),
      treeIndex: claimedIndex,
      maxDepth: TREE_MAX_DEPTH,
      maxBufferSize: TREE_MAX_BUFFER,
      canopyDepth: 0,
    }).sendAndConfirm(umi)
  );
});
