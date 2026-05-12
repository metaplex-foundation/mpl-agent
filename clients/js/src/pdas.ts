/**
 * Hand-written PDA helpers for the program-managed collections and trees.
 * Kinobi 1.0-alpha.2 doesn't emit `findPda` helpers for standalone PDAs
 * (PDAs that aren't also ShankAccounts), so we provide them here.
 */

import { Context, Pda, PublicKey } from '@metaplex-foundation/umi';
import { string, u64 } from '@metaplex-foundation/umi/serializers';

const TOOLS_PROGRAM_ID =
  'TLREGni9ZEyGC3vnPZtqUh95xQ8oPqJSvNjvB7FGK8S' as PublicKey;
const REPUTATION_PROGRAM_ID =
  'REPREG5c1gPHuHukEyANpksLdHFaJCiTrm6zJgNhRZR' as PublicKey;

/** Canonical receipts collection (agent-tools). */
export function findReceiptsCollectionPda(
  context: Pick<Context, 'eddsa' | 'programs'>
): Pda {
  const programId = context.programs.getPublicKey(
    'mplAgentTools',
    TOOLS_PROGRAM_ID
  );
  return context.eddsa.findPda(programId, [
    string({ size: 'variable' }).serialize('receipts_collection'),
  ]);
}

/** Receipts merkle tree at `["receipts_tree", index_le]`. */
export function findReceiptsTreePda(
  context: Pick<Context, 'eddsa' | 'programs'>,
  seeds: { treeIndex: number | bigint }
): Pda {
  const programId = context.programs.getPublicKey(
    'mplAgentTools',
    TOOLS_PROGRAM_ID
  );
  return context.eddsa.findPda(programId, [
    string({ size: 'variable' }).serialize('receipts_tree'),
    u64().serialize(seeds.treeIndex),
  ]);
}

/** Canonical reviews collection (agent-reputation). */
export function findReviewsCollectionPda(
  context: Pick<Context, 'eddsa' | 'programs'>
): Pda {
  const programId = context.programs.getPublicKey(
    'mplAgentReputation',
    REPUTATION_PROGRAM_ID
  );
  return context.eddsa.findPda(programId, [
    string({ size: 'variable' }).serialize('reviews_collection'),
  ]);
}

/** Reviews merkle tree at `["reviews_tree", index_le]`. */
export function findReviewsTreePda(
  context: Pick<Context, 'eddsa' | 'programs'>,
  seeds: { treeIndex: number | bigint }
): Pda {
  const programId = context.programs.getPublicKey(
    'mplAgentReputation',
    REPUTATION_PROGRAM_ID
  );
  return context.eddsa.findPda(programId, [
    string({ size: 'variable' }).serialize('reviews_tree'),
    u64().serialize(seeds.treeIndex),
  ]);
}
