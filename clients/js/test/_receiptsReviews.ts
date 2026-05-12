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
  createSignerFromKeypair,
  generateSigner,
  publicKey,
  PublicKey,
  Signer,
  some,
  Umi,
} from '@metaplex-foundation/umi';

import {
  fetchReviewsConfigV1,
  findReviewsConfigV1Pda,
  initializeReviewsConfigV1,
  registerReviewsTreeV1,
} from '../src/generated/reputation';
import {
  delegateExecutionV1,
  fetchToolsConfigV1,
  findExecutionDelegateRecordV1Pda,
  findExecutiveProfileV1Pda,
  findToolsConfigV1Pda,
  initializeToolsConfigV1,
  registerExecutiveV1,
  registerReceiptsTreeV1,
} from '../src/generated/tools';
import {
  findReceiptsCollectionPda,
  findReceiptsTreePda,
  findReviewsCollectionPda,
  findReviewsTreePda,
} from '../src';
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
 * Idempotent: best-effort initialize each program's config. Subsequent
 * runs against the same validator skip the already-initialized step.
 */
async function maybe<T>(p: Promise<T>): Promise<void> {
  try {
    await p;
  } catch {
    // Already initialized.
  }
}

export interface ReceiptsReviewsBootstrap {
  receiptsCollection: PublicKey;
  reviewsCollection: PublicKey;
  receiptsTree: PublicKey;
  receiptsTreeIndex: bigint;
  reviewsTree: PublicKey;
  reviewsTreeIndex: bigint;
}

/**
 * Initialize the singletons if needed and allocate one fresh receipts
 * tree + reviews tree pair (using the current `next_tree_index` on each
 * program). Returns everything callers need to drive `MintWorkReceiptV1`
 * and `LeaveReviewV1`.
 */
/**
 * Deterministic admin keypair shared by all test files. AVA spawns each
 * test file in its own worker, so without a stable seed each file's
 * `umi.payer` would differ and only the first file to run would be able
 * to register trees against the config. We derive a fixed keypair and
 * airdrop to it on demand.
 */
const SHARED_ADMIN_SEED = new Uint8Array([
  // 32 stable bytes — "agent-receipts-reviews-tests-1234".
  0x61, 0x67, 0x65, 0x6e, 0x74, 0x2d, 0x72, 0x65, 0x63, 0x65, 0x69, 0x70, 0x74,
  0x73, 0x2d, 0x72, 0x65, 0x76, 0x69, 0x65, 0x77, 0x73, 0x2d, 0x74, 0x65, 0x73,
  0x74, 0x73, 0x2d, 0x31, 0x32, 0x33,
]);

export function getSharedAdmin(umi: Umi): Signer {
  const kp = umi.eddsa.createKeypairFromSeed(SHARED_ADMIN_SEED);
  return createSignerFromKeypair(umi, kp);
}

/** Make sure the shared admin has enough SOL to bootstrap and register trees. */
async function ensureAdminFunded(umi: Umi, admin: Signer): Promise<void> {
  const balance = await umi.rpc.getBalance(admin.publicKey);
  // Trees + collection rent + tx fees easily fit in 0.5 SOL for the
  // small dimensions we use.
  if (balance.basisPoints < 500_000_000n) {
    await umi.rpc.airdrop(admin.publicKey, {
      basisPoints: 1_000_000_000n,
      identifier: 'SOL',
      decimals: 9,
    });
  }
}

export async function bootstrapReceiptsAndReviews(
  umi: Umi
): Promise<ReceiptsReviewsBootstrap> {
  const admin = getSharedAdmin(umi);
  await ensureAdminFunded(umi, admin);

  const receiptsCollection = publicKey(findReceiptsCollectionPda(umi));
  const reviewsCollection = publicKey(findReviewsCollectionPda(umi));

  await maybe(
    initializeToolsConfigV1(umi, {
      admin,
      collection: receiptsCollection,
    }).sendAndConfirm(umi)
  );

  await maybe(
    initializeReviewsConfigV1(umi, {
      admin,
      reviewsCollection,
      receiptsCollection,
    }).sendAndConfirm(umi)
  );

  const toolsConfig = await fetchToolsConfigV1(umi, findToolsConfigV1Pda(umi));
  const receiptsTreeIndex = toolsConfig.nextTreeIndex;
  const receiptsTree = publicKey(
    findReceiptsTreePda(umi, { treeIndex: receiptsTreeIndex })
  );
  await registerReceiptsTreeV1(umi, {
    admin,
    merkleTree: receiptsTree,
    treeConfig: findTreeConfigPda(umi, { merkleTree: receiptsTree }),
    maxDepth: TREE_MAX_DEPTH,
    maxBufferSize: TREE_MAX_BUFFER,
    canopyDepth: 0,
  }).sendAndConfirm(umi);

  const reviewsConfig = await fetchReviewsConfigV1(
    umi,
    findReviewsConfigV1Pda(umi)
  );
  const reviewsTreeIndex = reviewsConfig.nextTreeIndex;
  const reviewsTree = publicKey(
    findReviewsTreePda(umi, { treeIndex: reviewsTreeIndex })
  );
  await registerReviewsTreeV1(umi, {
    admin,
    merkleTree: reviewsTree,
    treeConfig: findTreeConfigPda(umi, { merkleTree: reviewsTree }),
    maxDepth: TREE_MAX_DEPTH,
    maxBufferSize: TREE_MAX_BUFFER,
    canopyDepth: 0,
  }).sendAndConfirm(umi);

  return {
    receiptsCollection,
    reviewsCollection,
    receiptsTree,
    receiptsTreeIndex,
    reviewsTree,
    reviewsTreeIndex,
  };
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
      creators: [{ address: input.agent, verified: false, share: 100 }],
      collection: some(input.receiptsCollection),
    },
  });
}

/** Compute the data_hash a receipt minted by `MintWorkReceiptV1` carries. */
export function receiptDataHash(input: {
  receiptUri: string;
  agent: PublicKey;
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
    creators: [{ address: input.agent, verified: false, share: 100 }],
    collection: some(input.receiptsCollection),
  });
}

/** Compute the creator_hash a receipt minted by `MintWorkReceiptV1` carries. */
export function receiptCreatorHash(agent: PublicKey): Uint8Array {
  // eslint-disable-next-line global-require, @typescript-eslint/no-require-imports
  const { hashMetadataCreators } =
    require('@metaplex-foundation/mpl-bubblegum') as typeof import('@metaplex-foundation/mpl-bubblegum');
  return hashMetadataCreators([
    { address: agent, verified: false, share: 100 },
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
