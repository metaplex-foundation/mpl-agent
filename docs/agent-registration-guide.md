# Agent Registration via Metaplex API

This guide covers registering an AI agent on-chain using the Metaplex Agent Registry SDK and the Metaplex API.

## Overview

The simplest way to register an agent is through the hosted **Metaplex API** at `https://api.metaplex.com`. The flow is:

1. Call `POST /v1/agents/mint` with the agent metadata and your wallet address
2. The API stores agent metadata on metaplex.com and returns an unsigned Solana transaction
3. You sign and submit the transaction to the Solana network
4. A **Core asset** (NFT) is created representing your agent, and an **Agent Identity** PDA is registered on-chain

The SDK wraps all of this into two simple functions:
- **`mintAgent`** — calls the API and returns a deserialized Umi transaction for manual signing
- **`mintAndSubmitAgent`** — convenience wrapper that also signs and submits in one call

## Installation

```bash
npm install @metaplex-foundation/mpl-agent-registry @metaplex-foundation/umi @metaplex-foundation/umi-bundle-defaults
```

## Setup

```typescript
import { createUmi } from '@metaplex-foundation/umi-bundle-defaults';
import { keypairIdentity } from '@metaplex-foundation/umi';
import { mplAgentIdentity } from '@metaplex-foundation/mpl-agent-registry';

// Create a Umi instance pointed at your preferred RPC
const umi = createUmi('https://api.mainnet-beta.solana.com')
  .use(mplAgentIdentity());

// Load your keypair (the agent owner's wallet)
// In production, use your preferred key management solution
const keypair = umi.eddsa.createKeypairFromSecretKey(mySecretKeyBytes);
umi.use(keypairIdentity(keypair));
```

## Quick Start: Mint an Agent (one-liner)

The simplest path — mint and submit in a single call:

```typescript
import { mintAndSubmitAgent } from '@metaplex-foundation/mpl-agent-registry';

const result = await mintAndSubmitAgent(umi, {}, {
  wallet: umi.identity.publicKey,
  name: 'My AI Agent',
  uri: 'https://example.com/agent-metadata.json',
  agentMetadata: {
    type: 'agent',
    name: 'My AI Agent',
    description: 'An autonomous trading agent',
    services: [
      { name: 'trading', endpoint: 'https://myagent.ai/trade' },
    ],
    registrations: [],
    supportedTrust: [],
  },
});

console.log('Agent minted!');
console.log('Asset address:', result.assetAddress);
console.log('Tx signature:', result.signature);
```

## Step-by-Step: Mint an Agent (manual signing)

For more control over transaction signing and submission:

```typescript
import {
  mintAgent,
  signAndSendAgentTransaction,
} from '@metaplex-foundation/mpl-agent-registry';

// Step 1: Call the API to get an unsigned transaction
const mintResult = await mintAgent(umi, {}, {
  wallet: umi.identity.publicKey,
  name: 'My AI Agent',
  uri: 'https://example.com/agent-metadata.json',
  agentMetadata: {
    type: 'agent',
    name: 'My AI Agent',
    description: 'An autonomous trading agent',
    services: [
      { name: 'trading', endpoint: 'https://myagent.ai/trade' },
      { name: 'analysis', endpoint: 'https://myagent.ai/analyze' },
    ],
    registrations: [
      { agentId: 'agent-123', agentRegistry: 'my-registry' },
    ],
    supportedTrust: ['tee'],
  },
});

console.log('Asset address:', mintResult.assetAddress);

// Step 2: Sign and send using the helper
const signature = await signAndSendAgentTransaction(umi, mintResult);
console.log('Confirmed signature:', signature);
```

## Using Devnet

Pass `network: 'solana-devnet'` and point your Umi instance at devnet RPC:

```typescript
const umi = createUmi('https://api.devnet.solana.com')
  .use(mplAgentIdentity());

const result = await mintAndSubmitAgent(umi, {}, {
  wallet: umi.identity.publicKey,
  network: 'solana-devnet',
  name: 'Test Agent',
  uri: 'https://example.com/test-metadata.json',
  agentMetadata: {
    type: 'agent',
    name: 'Test Agent',
    description: 'A test agent on devnet',
    services: [],
    registrations: [],
    supportedTrust: [],
  },
});
```

## Supported Networks

The API supports the following networks:

| Network | Value |
|---------|-------|
| Solana Mainnet | `solana-mainnet` (default) |
| Solana Devnet | `solana-devnet` |
| Localnet | `localnet` |
| Eclipse Mainnet | `eclipse-mainnet` |
| Sonic Mainnet | `sonic-mainnet` |
| Sonic Devnet | `sonic-devnet` |
| Fogo Mainnet | `fogo-mainnet` |
| Fogo Testnet | `fogo-testnet` |

## Custom API Base URL

If you need to target a staging or self-hosted API:

```typescript
const result = await mintAgent(
  umi,
  { baseUrl: 'https://staging-api.metaplex.com' },
  {
    wallet: umi.identity.publicKey,
    name: 'My Agent',
    uri: 'https://example.com/metadata.json',
    agentMetadata: {
      type: 'agent',
      name: 'My Agent',
      description: 'Test agent',
      services: [],
      registrations: [],
      supportedTrust: [],
    },
  }
);
```

## Custom Transaction Sender

If you have your own transaction-sending infrastructure (e.g. Jito bundles,
priority fees, or retry logic):

```typescript
const result = await mintAndSubmitAgent(
  umi,
  {},
  {
    wallet: umi.identity.publicKey,
    name: 'My Agent',
    uri: 'https://example.com/metadata.json',
    agentMetadata: {
      type: 'agent',
      name: 'My Agent',
      description: 'Agent with custom tx sender',
      services: [],
      registrations: [],
      supportedTrust: [],
    },
  },
  {
    txSender: async (tx) => {
      // Your custom signing and sending logic
      const signed = await umi.identity.signTransaction(tx);
      const sig = await myCustomSend(signed);
      return sig;
    },
  }
);
```

## Error Handling

The SDK provides typed errors for different failure modes:

```typescript
import {
  mintAgent,
  isAgentApiError,
  isAgentApiNetworkError,
  isAgentValidationError,
} from '@metaplex-foundation/mpl-agent-registry';

try {
  const result = await mintAgent(umi, {}, input);
} catch (err) {
  if (isAgentValidationError(err)) {
    // Client-side validation failed
    console.error(`Validation error on field "${err.field}": ${err.message}`);
  } else if (isAgentApiNetworkError(err)) {
    // Network issue reaching the API
    console.error('Network error:', err.message);
    console.error('Cause:', err.cause);
  } else if (isAgentApiError(err)) {
    // API returned an error response
    console.error(`API error (${err.statusCode}): ${err.message}`);
    console.error('Response body:', err.responseBody);
  } else {
    throw err;
  }
}
```

## API Reference

### `mintAgent(umi, config, input)`

Calls `POST /v1/agents/mint` and returns a deserialized transaction.

| Parameter | Type | Description |
|-----------|------|-------------|
| `umi` | `Umi` | Umi instance with identity and RPC configured |
| `config` | `AgentApiConfig \| null` | Optional API config (base URL, custom fetch) |
| `input` | `MintAgentInput` | Agent details and wallet |

Returns `Promise<MintAgentResponse>`:
- `transaction` — `Transaction` to sign and send
- `blockhash` — blockhash for confirmation
- `assetAddress` — the Core asset address (agent NFT)

### `mintAndSubmitAgent(umi, config, input, options?)`

Convenience wrapper: calls `mintAgent`, then signs and sends the transaction.

Returns `Promise<MintAndSubmitAgentResult>`:
- `signature` — confirmed transaction signature
- `assetAddress` — the Core asset address

### `signAndSendAgentTransaction(umi, mintResponse, options?)`

Signs and sends the transaction returned by `mintAgent`, then confirms it.

### Types

```typescript
interface MintAgentInput {
  wallet: PublicKey | string;
  network?: SvmNetwork;               // default: 'solana-mainnet'
  name: string;                        // Core asset name
  uri: string;                         // Metadata URI
  agentMetadata: AgentMetadata;
}

interface AgentMetadata {
  type: string;                        // e.g. 'agent'
  name: string;                        // Agent display name
  description: string;                 // Agent description
  services: AgentService[];            // Services the agent provides
  registrations: AgentRegistration[];  // External registry entries
  supportedTrust: string[];            // Trust mechanisms (e.g. 'tee')
}

interface AgentService {
  name: string;      // e.g. 'trading', 'chat'
  endpoint: string;  // Service URL
}

interface AgentRegistration {
  agentId: string;       // ID within the registry
  agentRegistry: string; // Registry identifier
}

interface AgentApiConfig {
  baseUrl?: string;     // default: 'https://api.metaplex.com'
  fetch?: typeof fetch; // custom fetch implementation
}
```
