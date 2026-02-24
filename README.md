# MPL Agent Identity Registry

A Solana program for registering verifiable agent identities on [MPL Core](https://developers.metaplex.com/core) assets. The identity registry allows any MPL Core NFT to carry an on-chain agent identity record via the [AppData](https://developers.metaplex.com/core/external-plugins/app-data) external plugin system.

## Overview

The Agent Identity Registry attaches a PDA-based identity record to an MPL Core asset and writes an `AppData` external plugin on the asset itself. This creates a two-way link: the PDA points to the asset, and the asset carries plugin data whose authority is the PDA. Together they form a tamper-evident on-chain identity binding.

**Program ID:** `1DREGFgysWYxLnRnKQnwrxnJQeSMk2HmGaC6whw2B2p`

## How It Works

### Registration Flow

1. A caller submits a `RegisterIdentityV1` instruction with the target MPL Core asset and its collection.
2. The program derives and creates an `AgentIdentityV1` PDA from the seeds `["agent_identity", <asset_pubkey>]`.
3. The program CPIs into MPL Core to add an `AppData` external plugin on the asset, with the PDA as the `data_authority`.

After registration, anyone can verify an agent's identity by:
- Deriving the PDA from the asset's public key and checking that the account exists.
- Inspecting the asset's `AppData` plugin to confirm the data authority matches the expected PDA.

### Accounts

| Account | Type | Seeds | Size |
|---------|------|-------|------|
| `AgentIdentityV1` | PDA | `["agent_identity", asset_pubkey]` | 40 bytes |

The `AgentIdentityV1` account stores:

| Field | Type | Description |
|-------|------|-------------|
| `key` | `u8` | Account discriminator (`Key::AgentIdentityV1`) |
| `bump` | `u8` | PDA bump seed |
| `_padding` | `[u8; 6]` | Alignment padding |
| `asset` | `Pubkey` | The MPL Core asset this identity is bound to |

### Instruction: `RegisterIdentityV1`

| # | Account | Writable | Signer | Description |
|---|---------|----------|--------|-------------|
| 0 | `agentIdentity` | Yes | No | The PDA to be created (derived from asset) |
| 1 | `asset` | Yes | No | The MPL Core asset to register |
| 2 | `collection` | Yes | No | The asset's collection (optional) |
| 3 | `payer` | Yes | Yes | Pays for account rent and transaction fees |
| 4 | `authority` | No | Yes | Collection authority (optional, defaults to payer) |
| 5 | `mplCoreProgram` | No | No | MPL Core program |
| 6 | `systemProgram` | No | No | System program |

## Clients

### JavaScript (TypeScript)

The JS client is built on the [Umi](https://github.com/metaplex-foundation/umi) framework.

**Install:**

```sh
npm install @metaplex-foundation/mpl-agent-registry
```

**Setup:**

```ts
import { createUmi } from '@metaplex-foundation/umi-bundle-defaults';
import { mplAgentIdentity } from '@metaplex-foundation/mpl-agent-registry';

const umi = createUmi('https://api.mainnet-beta.solana.com')
  .use(mplAgentIdentity());
```

**Register an identity:**

```ts
import {
  registerIdentityV1,
  findAgentIdentityV1Pda,
  fetchAgentIdentityV1,
} from '@metaplex-foundation/mpl-agent-registry';

// Register an agent identity for an MPL Core asset.
await registerIdentityV1(umi, {
  asset: assetPublicKey,
  collection: collectionPublicKey,
}).sendAndConfirm(umi);

// Fetch the identity PDA.
const pda = findAgentIdentityV1Pda(umi, { asset: assetPublicKey });
const identity = await fetchAgentIdentityV1(umi, pda);
```

### Rust

**Cargo.toml:**

```toml
[dependencies]
mpl-agent-identity = "0.1.0"
```

The Rust client exposes builder-pattern instruction constructors and account deserialization generated from the on-chain IDL.

## Development

### Prerequisites

- pnpm 8.9.0
- Rust 1.83.0
- Solana CLI 2.2.1

### Build

```sh
pnpm programs:build
```

### Test

```sh
# Program tests
pnpm programs:test

# JS client tests (builds programs and client first)
pnpm clients:js:test

# Rust client tests
pnpm clients:rust:test
```

### Regenerate Clients

After any program changes, regenerate the IDL and client code:

```sh
pnpm generate
```

This runs [Shank](https://github.com/metaplex-foundation/shank) to extract the IDL from Rust annotations, then [Kinobi](https://github.com/metaplex-foundation/kinobi) to render typed JS and Rust clients from the IDL.

### Project Structure

```
programs/mpl-agent-identity/src/
  lib.rs              # Program ID declaration
  entrypoint.rs       # Entrypoint routing
  instruction.rs      # Shank-annotated instruction enum (drives IDL generation)
  error.rs            # Custom program errors
  processor/
    mod.rs            # Discriminant-based dispatch
    register.rs       # RegisterIdentityV1 logic
  state/
    mod.rs            # Account discriminator enum
    agent_identity.rs # AgentIdentityV1 PDA account

clients/js/           # TypeScript client (@metaplex-foundation/mpl-agent-registry)
clients/rust-identity/# Rust client crate (mpl-agent-identity)
configs/              # Shank and Kinobi configuration
idls/                 # Generated IDL JSON
```

## Other Programs

This repository also contains `mpl-agent-reputation` and `mpl-agent-validation` programs. These follow the same architectural patterns but are not yet finalized.

## License

Metaplex NFT Open Source License - see [LICENSE](./LICENSE) for details.
