/**
 * Hand-written PDA helpers for standalone PDAs (collections, authority,
 * trees). Emitted by the kinobi-tools config because kinobi 1.0-alpha
 * doesn't render find*Pda helpers for PDAs added via addPdasVisitor.
 */

import { Context, Pda } from '@metaplex-foundation/umi';
import { string, u64 } from '@metaplex-foundation/umi/serializers';

const PROGRAM_ID = 'TLREGni9ZEyGC3vnPZtqUh95xQ8oPqJSvNjvB7FGK8S';

function pda(
  context: Pick<Context, 'eddsa' | 'programs'>,
  seeds: Uint8Array[]
): Pda {
  const programId = context.programs.getPublicKey('mplAgentTools', PROGRAM_ID);
  return context.eddsa.findPda(programId, seeds);
}

export function findReceiptsCollectionPda(
  context: Pick<Context, 'eddsa' | 'programs'>
): Pda {
  return pda(context, [
    string({ size: 'variable' }).serialize('receipts_collection'),
  ]);
}

export function findReceiptsAuthorityPda(
  context: Pick<Context, 'eddsa' | 'programs'>
): Pda {
  return pda(context, [
    string({ size: 'variable' }).serialize('receipts_authority'),
  ]);
}

export function findReceiptsTreePda(
  context: Pick<Context, 'eddsa' | 'programs'>,
  seeds: { treeIndex: number | bigint }
): Pda {
  return pda(context, [
    string({ size: 'variable' }).serialize('receipts_tree'),
    u64().serialize(seeds.treeIndex),
  ]);
}
