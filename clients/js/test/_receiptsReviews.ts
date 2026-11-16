/* eslint-disable import/no-extraneous-dependencies */
import {
  createCollection,
  create as createAsset,
} from '@metaplex-foundation/mpl-core';
import {
  findTreeConfigPda,
  hashLeafV2,
  TokenStandard,
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  hashMetadataDataV2 as _hashMetadataDataV2,
} from '@metaplex-foundation/mpl-bubblegum';
import {
  generateSigner,
  publicKey,
  PublicKey,
  Signer,
  some,
  Umi,
} from '@metaplex-foundation/umi';

import {
  createReviewsCollectionV1,
  findReviewsCollectionPda,
  findReviewsTreePda,
  registerReviewsTreeV1,
} from '../src/generated/reputation';
import {
  createReceiptsCollectionV1,
  delegateExecutionV1,
  findExecutionDelegateRecordV1Pda,
  findExecutiveProfileV1Pda,
  findReceiptsCollectionPda,
  findReceiptsTreePda,
  registerExecutiveV1,
  registerReceiptsTreeV1,
} from '../src/generated/tools';
import {
  findAgentIdentityV1Pda,
  registerIdentityV1,
} from '../src/generated/identity';

export const MPL_CORE_CPI_SIGNER = publicKey(
  'CbNY3JiXdXNE9tPNEk1aRZVEkWdj2v7kfJLNQwZZgpXk'
);

/** keccak256("") — Bubblegum's `DEFAULT_ASSET_DATA_HASH`. */
export const DEFAULT_ASSET_DATA_HASH = new Uint8Array([
  197, 210, 70, 1, 134, 247, 35, 60, 146, 126, 125, 178, 220, 199, 3, 192, 229,
  0, 182, 83, 202, 130, 39, 59, 123, 250, 216, 4, 93, 133, 164, 112,
]);

export const TREE_MAX_DEPTH = 5;
export const TREE_MAX_BUFFER = 8;

/**
 * Run an idempotent bootstrap call, swallowing ONLY the
 * "already-initialized" custom-program error for the given code. Any
 * other failure rethrows so real regressions don't silently pass.
 */
async function maybe<T>(p: Promise<T>, expectedHex: string): Promise<void> {
  try {
    await p;
  } catch (e: any) {
    const needle = `custom program error: ${expectedHex}`;
    const msg = String(e?.message ?? '');
    const causeMsg = String(e?.cause?.message ?? '');
    const logs: string[] = (e?.logs ?? e?.cause?.logs ?? []) as string[];
    if (
      msg.includes(needle) ||
      causeMsg.includes(needle) ||
      logs.some((l) => l.includes(needle))
    ) {
      return;
    }
    throw e;
  }
}

/** Custom program error code for MplAgentToolsError::ReceiptsCollectionAlreadyInitialized. */
const RECEIPTS_COLLECTION_ALREADY_INITIALIZED_HEX = '0x14';
/** Custom program error code for MplAgentReputationError::ReviewsCollectionAlreadyInitialized. */
const REVIEWS_COLLECTION_ALREADY_INITIALIZED_HEX = '0xd';

export interface ReceiptsBootstrap {
  receiptsCollection: PublicKey;
  receiptsTree: PublicKey;
  receiptsTreeIndex: bigint;
}

export interface ReceiptsReviewsBootstrap extends ReceiptsBootstrap {
  reviewsCollection: PublicKey;
  reviewsTree: PublicKey;
  reviewsTreeIndex: bigint;
}

/**
 * Permissionless bootstrap: idempotently create the canonical receipts
 * collection, then allocate a fresh receipts tree at a random index for
 * this test run. Random indices keep tests isolated (each gets a dedicated
 * empty tree) and avoid races on a shared `next_tree_index` counter.
 */
export async function bootstrapReceipts(umi: Umi): Promise<ReceiptsBootstrap> {
  const receiptsCollection = publicKey(findReceiptsCollectionPda(umi));

  await maybe(
    createReceiptsCollectionV1(umi, {}).sendAndConfirm(umi),
    RECEIPTS_COLLECTION_ALREADY_INITIALIZED_HEX
  );

  const receiptsTreeIndex = randomU64();
  const receiptsTree = publicKey(
    findReceiptsTreePda(umi, { treeIndex: receiptsTreeIndex })
  );
  await registerReceiptsTreeV1(umi, {
    merkleTree: receiptsTree,
    treeConfig: findTreeConfigPda(umi, { merkleTree: receiptsTree }),
    treeIndex: receiptsTreeIndex,
    maxDepth: TREE_MAX_DEPTH,
    maxBufferSize: TREE_MAX_BUFFER,
    canopyDepth: 0,
  }).sendAndConfirm(umi);

  return {
    receiptsCollection,
    receiptsTree,
    receiptsTreeIndex,
  };
}

/**
 * Extends `bootstrapReceipts` with the reviews-side counterpart:
 * idempotently creates the reviews collection and allocates a fresh
 * reviews tree at a random index. Returns the combined context every
 * `LeaveReviewV1` test needs.
 */
export async function bootstrapReceiptsAndReviews(
  umi: Umi
): Promise<ReceiptsReviewsBootstrap> {
  const receipts = await bootstrapReceipts(umi);
  const reviewsCollection = publicKey(findReviewsCollectionPda(umi));

  await maybe(
    createReviewsCollectionV1(umi, {}).sendAndConfirm(umi),
    REVIEWS_COLLECTION_ALREADY_INITIALIZED_HEX
  );

  const reviewsTreeIndex = randomU64();
  const reviewsTree = publicKey(
    findReviewsTreePda(umi, { treeIndex: reviewsTreeIndex })
  );
  await registerReviewsTreeV1(umi, {
    merkleTree: reviewsTree,
    treeConfig: findTreeConfigPda(umi, { merkleTree: reviewsTree }),
    treeIndex: reviewsTreeIndex,
    maxDepth: TREE_MAX_DEPTH,
    maxBufferSize: TREE_MAX_BUFFER,
    canopyDepth: 0,
  }).sendAndConfirm(umi);

  return {
    ...receipts,
    reviewsCollection,
    reviewsTree,
    reviewsTreeIndex,
  };
}

function randomU64(): bigint {
  // 56 random bits — comfortably within u64, no chance of collision in a
  // test run.
  const buf = new Uint8Array(7);
  crypto.getRandomValues(buf);
  let v = 0n;
  for (const b of buf) v = (v << 8n) | BigInt(b);
  return v;
}

export interface AgentSetup {
  agent: PublicKey;
  executive: Signer;
  executiveProfile: PublicKey;
  executionDelegateRecord: PublicKey;
}

/** Stand up an agent core asset + register identity + delegate to a fresh executive. */
export async function setupAgentWithExecutive(umi: Umi): Promise<AgentSetup> {
  const agentCollection = generateSigner(umi);
  await createCollection(umi, {
    collection: agentCollection,
    name: 'Agents',
    uri: 'https://example.com/agents.json',
  }).sendAndConfirm(umi);

  const asset = generateSigner(umi);
  await createAsset(umi, {
    asset,
    name: 'Test Agent',
    uri: 'https://example.com/agent.json',
    collection: { publicKey: agentCollection.publicKey } as any,
  }).sendAndConfirm(umi);

  await registerIdentityV1(umi, {
    asset: asset.publicKey,
    collection: agentCollection.publicKey,
    agentRegistrationUri: 'https://example.com/agent.json',
  }).sendAndConfirm(umi);

  const executive = generateSigner(umi);
  await registerExecutiveV1(umi, { authority: executive }).sendAndConfirm(umi);

  const executiveProfile = publicKey(
    findExecutiveProfileV1Pda(umi, { authority: executive.publicKey })
  );
  const agentIdentity = findAgentIdentityV1Pda(umi, { asset: asset.publicKey });
  await delegateExecutionV1(umi, {
    executiveProfile,
    agentAsset: asset.publicKey,
    agentIdentity,
  }).sendAndConfirm(umi);

  const executionDelegateRecord = publicKey(
    findExecutionDelegateRecordV1Pda(umi, {
      executiveProfile,
      agentAsset: asset.publicKey,
    })
  );

  return {
    agent: asset.publicKey,
    executive,
    executiveProfile,
    executionDelegateRecord,
  };
}

/** Compute the leaf hash of a receipt minted by `MintWorkReceiptV1`. */
export function hashReceiptLeaf(
  umi: Umi,
  input: {
    merkleTree: PublicKey;
    leafIndex: number | bigint;
    owner: PublicKey;
    agent: PublicKey;
    client: PublicKey;
    receiptsCollection: PublicKey;
    receiptUri: string;
  }
): Uint8Array {
  return hashLeafV2(umi, {
    merkleTree: input.merkleTree,
    owner: input.owner,
    leafIndex: input.leafIndex,
    metadata: {
      name: 'Agent Work Receipt',
      symbol: 'AGENTRCPT',
      uri: input.receiptUri,
      sellerFeeBasisPoints: 0,
      primarySaleHappened: false,
      isMutable: false,
      tokenStandard: some(TokenStandard.NonFungible),
      creators: [
        { address: input.agent, verified: false, share: 100 },
        { address: input.client, verified: false, share: 0 },
      ],
      collection: some(input.receiptsCollection),
    },
  });
}

/** Compute the data_hash a receipt minted by `MintWorkReceiptV1` carries. */
export function receiptDataHash(input: {
  receiptUri: string;
  agent: PublicKey;
  client: PublicKey;
  receiptsCollection: PublicKey;
}): Uint8Array {
  // eslint-disable-next-line global-require, @typescript-eslint/no-require-imports
  const { hashMetadataDataV2 } =
    require('@metaplex-foundation/mpl-bubblegum') as typeof import('@metaplex-foundation/mpl-bubblegum');
  return hashMetadataDataV2({
    name: 'Agent Work Receipt',
    symbol: 'AGENTRCPT',
    uri: input.receiptUri,
    sellerFeeBasisPoints: 0,
    primarySaleHappened: false,
    isMutable: false,
    tokenStandard: some(TokenStandard.NonFungible),
    creators: [
      { address: input.agent, verified: false, share: 100 },
      { address: input.client, verified: false, share: 0 },
    ],
    collection: some(input.receiptsCollection),
  });
}

/** Compute the creator_hash a receipt minted by `MintWorkReceiptV1` carries. */
export function receiptCreatorHash(
  agent: PublicKey,
  client: PublicKey
): Uint8Array {
  // eslint-disable-next-line global-require, @typescript-eslint/no-require-imports
  const { hashMetadataCreators } =
    require('@metaplex-foundation/mpl-bubblegum') as typeof import('@metaplex-foundation/mpl-bubblegum');
  return hashMetadataCreators([
    { address: agent, verified: false, share: 100 },
    { address: client, verified: false, share: 0 },
  ]);
}

/** Read the most recent root from the on-chain merkle tree account. */
export async function getCurrentTreeRoot(
  umi: Umi,
  merkleTree: PublicKey
): Promise<Uint8Array> {
  const account = await umi.rpc.getAccount(merkleTree);
  if (!account.exists) throw new Error('tree account missing');
  // ConcurrentMerkleTreeHeader (v1) is 88 bytes; the first changelog
  // entry's root immediately follows.
  return account.data.slice(88, 88 + 32);
}
