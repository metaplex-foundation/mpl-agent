# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Essential Commands

```bash
# Build all Solana programs
pnpm programs:build

# Run program tests (RUST_LOG=error)
pnpm programs:test

# Run program tests with debug logs
pnpm programs:debug

# Test JavaScript client (installs, builds, then tests)
pnpm clients:js:test

# Test Rust client (cargo test-sbf)
pnpm clients:rust:test

# Generate IDLs and clients (MUST run after any program changes)
pnpm generate

# Generate IDLs only (Shank extracts from Rust annotations)
pnpm generate:idls

# Generate clients only (Kinobi renders from IDL JSON)
pnpm generate:clients

# Format and lint fix
pnpm lint:fix

# Start/stop local Amman validator
pnpm validator
pnpm validator:stop
```

## Architecture

### Two-Program Suite

This repo contains two independent Solana programs that operate on **MPL Core** assets (NFTs):

| Program | ID | Purpose |
|---------|-----|---------|
| `mpl-agent-identity` | `1DREGFgysWYxLnRnKQnwrxnJQeSMk2HmGaC6whw2B2p` | Registers agent identity for Core assets |
| `mpl-agent-reputation` | `REPREG5c1gPHuHukEyANpksLdHFaJCiTrm6zJgNhRZR` | Registers agent reputation for Core assets |

The programs are structurally parallel — each has one instruction (`RegisterIdentityV1` / `RegisterReputationV1`), two PDA account types, and the same validation flow. They do **not** CPI into each other; both only CPI into MPL Core.

### Program Structure (both programs follow this pattern)

```
programs/mpl-agent-{identity,reputation}/src/
├── lib.rs           # declare_id! macro
├── entrypoint.rs    # Routes to processor
├── instruction.rs   # ShankContext + ShankInstruction derive for IDL gen
├── processor/
│   ├── mod.rs       # Discriminant-based dispatch via bytemuck zero-copy
│   └── register.rs  # Core logic: validate PDAs, create accounts, add LinkedAppData plugin
├── state/
│   ├── mod.rs       # IdentityKey/ReputationKey enum (account discriminator)
│   ├── agent_*.rs   # Per-asset PDA (40 bytes, zero-copy Pod)
│   └── collection_*_config.rs  # Per-collection PDA (40 bytes, zero-copy Pod)
└── error.rs         # Custom errors via thiserror + num_derive
```

### PDA Derivation

| Account | Seeds | Program |
|---------|-------|---------|
| `AgentIdentityV1` | `["agent_identity", asset_pubkey]` | identity |
| `CollectionIdentityConfigV1` | `["collection_identity_config", collection_pubkey]` | identity |
| `AgentReputationV1` | `["agent_reputation", asset_pubkey]` | reputation |
| `CollectionReputationConfigV1` | `["collection_reputation_config", collection_pubkey]` | reputation |

### Key Patterns

- **Zero-copy via bytemuck**: All account structs and instruction args are `#[repr(C)]`, `Pod + Zeroable`, 8-byte aligned with compile-time size assertions. Instruction data is cast directly from the raw byte slice.
- **Shank annotations**: `ShankContext`, `ShankInstruction`, `ShankAccount`, `ShankType` drive IDL generation. The `#[skip]` attribute excludes discriminator bytes from the IDL; `#[padding]` marks alignment bytes; `#[idl_type(IdentityKey)]`/`#[idl_type(ReputationKey)]` maps raw `u8` fields to enum types in the IDL.
- **MPL Core LinkedAppData**: On registration, each program attaches a `LinkedAppData` external plugin to the MPL Core **collection** with the collection config PDA as `data_authority`. Both programs can add separate `LinkedAppData` entries to the same collection.

### Client Generation Pipeline

1. **Shank** (`configs/shank.cjs`) — Extracts IDL from Rust annotations → `idls/*.json`
2. **Kinobi** (`configs/kinobi.cjs`) — Reads IDL JSON, configures PDA seeds and account defaults, renders:
   - JS client → `clients/js/src/generated/`
   - Rust client → `clients/rust/src/generated/`

The `configs/kinobi.cjs` file is where PDA seed definitions, program name mappings, and instruction account defaults are configured. If you add/change accounts or instructions, update this file accordingly.

### JS Client

- Package: `@metaplex-foundation/mpl-agent-identity`
- Uses **UMI** framework — `src/plugin.ts` exports a `mplAgentIdentity()` UMI plugin
- Tests use **AVA** with a local Amman validator (programs must be built first)
- `clients/js/src/generated/` is entirely auto-generated — never edit directly

### Rust Client

- Crate: `mpl-agent-identity`
- `src/lib.rs` re-exports everything from `src/generated/`
- `src/generated/` is entirely auto-generated — never edit directly
- Tests gated behind `#[cfg(feature = "test-sbf")]`

## Development Workflow

1. Edit program source in `programs/mpl-agent-{identity,reputation}/`
2. `pnpm programs:build` to compile
3. `pnpm programs:test` to run program tests
4. `pnpm generate` to regenerate IDLs and clients
5. `pnpm clients:js:test` / `pnpm clients:rust:test` to test clients

## Environment

- pnpm 8.9.0, Rust 1.83.0, Solana 2.2.1
- Programs build to `programs/.bin/` — external deps (mpl_core, token_metadata, spl_noop) are fetched via `configs/scripts/program/dump.sh`
- CI config in `.github/.env` — program list, toolchain versions
