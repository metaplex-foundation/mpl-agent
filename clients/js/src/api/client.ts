import { Umi } from '@metaplex-foundation/umi';
import { base64 } from '@metaplex-foundation/umi/serializers';
import {
  AgentApiConfig,
  MintAgentInput,
  MintAgentResponse,
  MintAndSubmitAgentResult,
  SignAndSendOptions,
} from './types';
import { signAndSendAgentTransaction } from './transactionHelper';
import { agentApiError, agentApiNetworkError } from './errors';

const DEFAULT_BASE_URL = 'https://api.metaplex.com';

function resolveBaseUrl(config: AgentApiConfig): string {
  return config.baseUrl ?? DEFAULT_BASE_URL;
}

function toKeyString(key: string | { toString(): string }): string {
  return typeof key === 'string' ? key : key.toString();
}

// ─── Payload Builder ────────────────────────────────────────────────────────

function buildMintAgentPayload(input: MintAgentInput) {
  return {
    wallet: toKeyString(input.wallet),
    network: input.network ?? 'solana-mainnet',
    name: input.name,
    uri: input.uri,
    agentMetadata: input.agentMetadata,
  };
}

// ─── Client Functions ───────────────────────────────────────────────────────

/**
 * Mints an agent via the Metaplex Agent API.
 *
 * Returns a deserialized Umi transaction that must be signed and sent
 * to the Solana network. The API creates a Core asset and registers the
 * agent identity on-chain, with metadata stored on metaplex.com.
 *
 * @example
 * ```ts
 * const result = await mintAgent(umi, {}, {
 *   wallet: umi.identity.publicKey,
 *   name: 'My AI Agent',
 *   uri: 'https://example.com/agent-metadata.json',
 *   agentMetadata: {
 *     type: 'agent',
 *     name: 'My AI Agent',
 *     description: 'An autonomous trading agent',
 *     services: [{ name: 'trading', endpoint: 'https://myagent.ai/trade' }],
 *     registrations: [],
 *     supportedTrust: [],
 *   },
 * });
 *
 * // Sign and send using the helper
 * const signature = await signAndSendAgentTransaction(umi, result);
 * ```
 */
export async function mintAgent(
  umi: Umi,
  configInput: AgentApiConfig | undefined | null,
  input: MintAgentInput
): Promise<MintAgentResponse> {
  const config = configInput ?? {};
  const payload = buildMintAgentPayload(input);
  const fetchFn = config.fetch ?? globalThis.fetch;
  const baseUrl = resolveBaseUrl(config);

  let response: Response;
  try {
    response = await fetchFn(`${baseUrl}/v1/agents/mint`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
    });
  } catch (err) {
    throw agentApiNetworkError(
      `Network error calling mintAgent: ${(err as Error).message}`,
      err as Error
    );
  }

  let body: any;
  try {
    body = await response.json();
  } catch (parseError) {
    const rawText = await response.text().catch(() => '<unable to read body>');
    throw agentApiError(
      `Failed to parse mintAgent response as JSON: ${(parseError as Error).message} — raw body: ${rawText}`,
      response.status,
      { rawText }
    );
  }

  if (!response.ok || !body.success) {
    throw agentApiError(
      body.error ?? `API returned status ${response.status}`,
      response.status,
      body
    );
  }

  const txBytes = base64.serialize(body.tx as string);
  const transaction = umi.transactions.deserialize(txBytes);

  return {
    transaction,
    blockhash: body.blockhash,
    assetAddress:
      typeof body.assetAddress === 'string'
        ? body.assetAddress
        : body.assetAddress.toString(),
  };
}

/**
 * High-level convenience method that mints an agent and submits the
 * transaction to the network in a single call.
 *
 * 1. Calls the Metaplex Agent API to get an unsigned transaction
 * 2. Signs and sends the transaction to the Solana RPC via Umi
 *
 * @example
 * ```ts
 * const result = await mintAndSubmitAgent(umi, {}, {
 *   wallet: umi.identity.publicKey,
 *   name: 'My AI Agent',
 *   uri: 'https://example.com/agent-metadata.json',
 *   agentMetadata: {
 *     type: 'agent',
 *     name: 'My AI Agent',
 *     description: 'An autonomous trading agent',
 *     services: [],
 *     registrations: [],
 *     supportedTrust: [],
 *   },
 * });
 * console.log(`Agent minted! Asset: ${result.assetAddress}`);
 * ```
 */
export async function mintAndSubmitAgent(
  umi: Umi,
  config: AgentApiConfig | undefined | null,
  input: MintAgentInput,
  signAndSendOptions?: SignAndSendOptions
): Promise<MintAndSubmitAgentResult> {
  const mintResult = await mintAgent(umi, config, input);

  let signature: Uint8Array;
  if (signAndSendOptions?.txSender) {
    signature = await signAndSendOptions.txSender(mintResult.transaction);
  } else {
    signature = await signAndSendAgentTransaction(umi, mintResult, {
      commitment: signAndSendOptions?.commitment ?? 'confirmed',
      preflightCommitment:
        signAndSendOptions?.preflightCommitment ?? 'confirmed',
      skipPreflight: signAndSendOptions?.skipPreflight ?? false,
    });
  }

  return {
    signature,
    assetAddress: mintResult.assetAddress,
  };
}
