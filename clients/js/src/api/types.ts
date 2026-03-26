import {
  BlockhashWithExpiryBlockHeight,
  PublicKey,
  RpcSendTransactionOptions,
  Transaction,
} from '@metaplex-foundation/umi';

// ─── Network ────────────────────────────────────────────────────────────────

export type SvmNetwork =
  | 'solana-mainnet'
  | 'solana-devnet'
  | 'localnet'
  | 'eclipse-mainnet'
  | 'sonic-mainnet'
  | 'sonic-devnet'
  | 'fogo-mainnet'
  | 'fogo-testnet';

// ─── Agent Metadata (on-chain registration payload) ─────────────────────────

export interface AgentService {
  /** Service name (e.g. 'chat', 'trading', 'data-analysis') */
  name: string;
  /** Service endpoint URL */
  endpoint: string;
}

export interface AgentRegistration {
  /** Agent identifier within the registry */
  agentId: string;
  /** Name or identifier of the agent registry */
  agentRegistry: string;
}

export interface AgentMetadata {
  /** Agent type (e.g. 'agent') */
  type: string;
  /** Agent name */
  name: string;
  /** Agent description */
  description: string;
  /** Services the agent provides */
  services: AgentService[];
  /** External registry registrations */
  registrations: AgentRegistration[];
  /** Supported trust mechanisms (e.g. 'tee', 'zkp') */
  supportedTrust: string[];
}

// ─── API Configuration ──────────────────────────────────────────────────────

export interface AgentApiConfig {
  /** Base URL of the Metaplex API. Defaults to 'https://api.metaplex.com'. */
  baseUrl?: string;
  /** Custom fetch implementation. Defaults to globalThis.fetch. */
  fetch?: typeof globalThis.fetch;
}

// ─── Mint Agent Input ───────────────────────────────────────────────────────

export interface MintAgentInput {
  /** The agent owner's wallet public key (will sign the transaction) */
  wallet: PublicKey | string;
  /** Network to mint on. Defaults to 'solana-mainnet'. */
  network?: SvmNetwork;
  /** Core asset name for the agent NFT */
  name: string;
  /** Metadata URI for the Core asset */
  uri: string;
  /** Agent metadata stored on-chain */
  agentMetadata: AgentMetadata;
}

// ─── API Response Types ─────────────────────────────────────────────────────

export interface MintAgentResponse {
  /** Deserialized Umi transaction ready to be signed and sent */
  transaction: Transaction;
  /** Blockhash for transaction validity */
  blockhash: BlockhashWithExpiryBlockHeight;
  /** The Core asset address of the minted agent */
  assetAddress: string;
}

export interface MintAndSubmitAgentResult {
  /** Transaction signature */
  signature: Uint8Array;
  /** The Core asset address of the minted agent */
  assetAddress: string;
}

// ─── Sign and Send Options ──────────────────────────────────────────────────

export interface SignAndSendOptions extends RpcSendTransactionOptions {
  /** Custom tx sender which returns the signature of the transaction */
  txSender?: (tx: Transaction) => Promise<Uint8Array>;
}
