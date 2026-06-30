/**
 * Hand-written PDA helpers for standalone PDAs (collections, authority,
 * trees). Emitted by the kinobi-reputation config because kinobi 1.0-alpha
 * doesn't render find*Pda helpers for PDAs added via addPdasVisitor.
 */

import { Context, Pda } from '@metaplex-foundation/umi';
import { string, u64 } from '@metaplex-foundation/umi/serializers';

const PROGRAM_ID = 'REPREG5c1gPHuHukEyANpksLdHFaJCiTrm6zJgNhRZR';

function pda(
  context: Pick<Context, 'eddsa' | 'programs'>,
  seeds: Uint8Array[]
): Pda {
  const programId = context.programs.getPublicKey(
    'mplAgentReputation',
    PROGRAM_ID
  );
  return context.eddsa.findPda(programId, seeds);
}

export function findReviewsCollectionPda(
  context: Pick<Context, 'eddsa' | 'programs'>
): Pda {
  return pda(context, [
    string({ size: 'variable' }).serialize('reviews_collection'),
  ]);
}

export function findReviewsAuthorityPda(
  context: Pick<Context, 'eddsa' | 'programs'>
): Pda {
  return pda(context, [
    string({ size: 'variable' }).serialize('reviews_authority'),
  ]);
}

export function findReviewsTreePda(
  context: Pick<Context, 'eddsa' | 'programs'>,
  seeds: { treeIndex: number | bigint }
): Pda {
  return pda(context, [
    string({ size: 'variable' }).serialize('reviews_tree'),
    u64().serialize(seeds.treeIndex),
  ]);
}
