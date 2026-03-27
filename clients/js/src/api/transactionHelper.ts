import { Umi } from '@metaplex-foundation/umi';
import { MintAgentResponse, SignAndSendOptions } from './types';

/**
 * Signs and sends the transaction returned by {@link mintAgent}.
 *
 * @param umi - Umi instance configured with the signer identity and RPC
 * @param mintResponse - The response from {@link mintAgent}
 * @param options - Optional confirmation settings
 * @returns The transaction signature
 *
 * @example
 * ```ts
 * const mintResult = await mintAgent(umi, config, input);
 * const signature = await signAndSendAgentTransaction(umi, mintResult);
 * ```
 */
export async function signAndSendAgentTransaction(
  umi: Umi,
  mintResponse: MintAgentResponse,
  options?: SignAndSendOptions
): Promise<Uint8Array> {
  const commitment = options?.commitment ?? 'confirmed';
  const preflightCommitment = options?.preflightCommitment ?? 'confirmed';
  const skipPreflight = options?.skipPreflight ?? false;

  const signedTransaction = await umi.identity.signTransaction(
    mintResponse.transaction
  );

  const signature = await umi.rpc.sendTransaction(signedTransaction, {
    skipPreflight,
    commitment,
    preflightCommitment,
  });

  await umi.rpc.confirmTransaction(signature, {
    strategy: {
      type: 'blockhash',
      ...mintResponse.blockhash,
    },
    commitment,
  });

  return signature;
}
