import test from 'ava';
import {
  findTreeConfigPda,
  mplBubblegum,
} from '@metaplex-foundation/mpl-bubblegum';
import { generateSigner, publicKey, Signer } from '@metaplex-foundation/umi';

import { mintWorkReceiptV1 } from '../../src/generated/tools';
import { findReceiptsTreePda } from '../../src';
import { createUmi } from '../_setup';
import {
  bootstrapReceipts,
  MPL_CORE_CPI_SIGNER,
  setupAgentWithExecutive,
} from '../_receiptsReviews';

async function fundedClient(
  umi: Awaited<ReturnType<typeof createUmi>>
): Promise<Signer> {
  const client = generateSigner(umi);
  await umi.rpc.airdrop(client.publicKey, {
    basisPoints: 100_000_000n,
    identifier: 'SOL',
    decimals: 9,
  });
  return client;
}

test('mintWorkReceiptV1 — executive authority mismatch fails', async (t) => {
  const umi = (await createUmi()).use(mplBubblegum());
  const ctx = await bootstrapReceipts(umi);
  const setup = await setupAgentWithExecutive(umi);
  const client = await fundedClient(umi);

  // Sign with a stranger key (not the executive captured in the delegate
  // record) — should fail at the delegate-record authority check.
  const stranger = generateSigner(umi);

  await t.throwsAsync(() =>
    mintWorkReceiptV1(umi, {
      executiveAuthority: stranger,
      executionDelegateRecord: setup.executionDelegateRecord,
      agentAsset: setup.agent,
      client: client.publicKey,
      treeConfig: findTreeConfigPda(umi, { merkleTree: ctx.receiptsTree }),
      merkleTree: ctx.receiptsTree,
      coreCollection: ctx.receiptsCollection,
      mplCoreCpiSigner: MPL_CORE_CPI_SIGNER,
      receiptUri: 'https://example.com/r.json',
      treeIndex: ctx.receiptsTreeIndex,
    }).sendAndConfirm(umi)
  );
});

test('mintWorkReceiptV1 — wrong tree index fails (PDA mismatch)', async (t) => {
  const umi = (await createUmi()).use(mplBubblegum());
  const ctx = await bootstrapReceipts(umi);
  const setup = await setupAgentWithExecutive(umi);
  const client = await fundedClient(umi);

  // Use a tree-index value that doesn't match the merkle tree's seeds.
  const wrongIndex = ctx.receiptsTreeIndex + 7n;

  await t.throwsAsync(() =>
    mintWorkReceiptV1(umi, {
      executiveAuthority: setup.executive,
      executionDelegateRecord: setup.executionDelegateRecord,
      agentAsset: setup.agent,
      client: client.publicKey,
      treeConfig: findTreeConfigPda(umi, { merkleTree: ctx.receiptsTree }),
      merkleTree: ctx.receiptsTree,
      coreCollection: ctx.receiptsCollection,
      mplCoreCpiSigner: MPL_CORE_CPI_SIGNER,
      receiptUri: 'https://example.com/r.json',
      treeIndex: wrongIndex,
    }).sendAndConfirm(umi)
  );
});

test('mintWorkReceiptV1 — wrong tree account fails (matches index but not registered)', async (t) => {
  const umi = (await createUmi()).use(mplBubblegum());
  const ctx = await bootstrapReceipts(umi);
  const setup = await setupAgentWithExecutive(umi);
  const client = await fundedClient(umi);

  // PDA at index 999 is derivable but not registered (no on-chain tree
  // backing it). The PDA check passes for tree_index=999 vs the supplied
  // address, but Bubblegum then fails at tree_config-derivation /
  // uninitialised-merkle.
  const fakeTree = publicKey(findReceiptsTreePda(umi, { treeIndex: 999n }));

  await t.throwsAsync(() =>
    mintWorkReceiptV1(umi, {
      executiveAuthority: setup.executive,
      executionDelegateRecord: setup.executionDelegateRecord,
      agentAsset: setup.agent,
      client: client.publicKey,
      treeConfig: findTreeConfigPda(umi, { merkleTree: fakeTree }),
      merkleTree: fakeTree,
      coreCollection: ctx.receiptsCollection,
      mplCoreCpiSigner: MPL_CORE_CPI_SIGNER,
      receiptUri: 'https://example.com/r.json',
      treeIndex: 999n,
    }).sendAndConfirm(umi)
  );
});

test('mintWorkReceiptV1 — wrong collection fails', async (t) => {
  const umi = (await createUmi()).use(mplBubblegum());
  const ctx = await bootstrapReceipts(umi);
  const setup = await setupAgentWithExecutive(umi);
  const client = await fundedClient(umi);

  // Random pubkey isn't the canonical receipts collection.
  const otherCollection = generateSigner(umi).publicKey;

  await t.throwsAsync(() =>
    mintWorkReceiptV1(umi, {
      executiveAuthority: setup.executive,
      executionDelegateRecord: setup.executionDelegateRecord,
      agentAsset: setup.agent,
      client: client.publicKey,
      treeConfig: findTreeConfigPda(umi, { merkleTree: ctx.receiptsTree }),
      merkleTree: ctx.receiptsTree,
      coreCollection: otherCollection,
      mplCoreCpiSigner: MPL_CORE_CPI_SIGNER,
      receiptUri: 'https://example.com/r.json',
      treeIndex: ctx.receiptsTreeIndex,
    }).sendAndConfirm(umi)
  );
});

test('mintWorkReceiptV1 — empty receipt URI fails', async (t) => {
  const umi = (await createUmi()).use(mplBubblegum());
  const ctx = await bootstrapReceipts(umi);
  const setup = await setupAgentWithExecutive(umi);
  const client = await fundedClient(umi);

  await t.throwsAsync(() =>
    mintWorkReceiptV1(umi, {
      executiveAuthority: setup.executive,
      executionDelegateRecord: setup.executionDelegateRecord,
      agentAsset: setup.agent,
      client: client.publicKey,
      treeConfig: findTreeConfigPda(umi, { merkleTree: ctx.receiptsTree }),
      merkleTree: ctx.receiptsTree,
      coreCollection: ctx.receiptsCollection,
      mplCoreCpiSigner: MPL_CORE_CPI_SIGNER,
      receiptUri: '',
      treeIndex: ctx.receiptsTreeIndex,
    }).sendAndConfirm(umi)
  );
});

test('mintWorkReceiptV1 — wrong compression program fails', async (t) => {
  const umi = (await createUmi()).use(mplBubblegum());
  const ctx = await bootstrapReceipts(umi);
  const setup = await setupAgentWithExecutive(umi);
  const client = await fundedClient(umi);

  // SPL Noop has the right shape (executable, well-known) but is not the
  // canonical mpl-account-compression program. The on-chain guard pins
  // the compression program id so spoofed compression backends cannot
  // silently no-op the merkle update.
  const splNoop = publicKey('noopb9bkMVfRPU8AsbpTUg8AQkHtKwMYZiFUjNRtMmV');

  await t.throwsAsync(() =>
    mintWorkReceiptV1(umi, {
      executiveAuthority: setup.executive,
      executionDelegateRecord: setup.executionDelegateRecord,
      agentAsset: setup.agent,
      client: client.publicKey,
      treeConfig: findTreeConfigPda(umi, { merkleTree: ctx.receiptsTree }),
      merkleTree: ctx.receiptsTree,
      coreCollection: ctx.receiptsCollection,
      mplCoreCpiSigner: MPL_CORE_CPI_SIGNER,
      compressionProgram: splNoop,
      receiptUri: 'https://example.com/r.json',
      treeIndex: ctx.receiptsTreeIndex,
    }).sendAndConfirm(umi)
  );
});
