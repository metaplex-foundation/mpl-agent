# `mint-test` Command Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a `mint-test` CLI command that creates a Bubblegum compressed NFT collection on devnet for testing migration.

**Architecture:** Single new command that creates a merkle tree, a Token Metadata collection NFT, and mints N compressed NFTs into it. All using the existing UMI setup with mpl-bubblegum SDK.

**Tech Stack:** mpl-bubblegum (createTree, mintToCollectionV1), mpl-token-metadata (createV1 for collection NFT), UMI, commander

---

### Task 1: Add mpl-token-metadata dependency

**Files:**
- Modify: `clients/migrator/package.json`

**Step 1: Add dependency**

Add `@metaplex-foundation/mpl-token-metadata` to dependencies:

```json
"@metaplex-foundation/mpl-token-metadata": "^3.2.1"
```

**Step 2: Install**

Run: `cd clients/migrator && pnpm install`

---

### Task 2: Add MintTestOptions type and CLI command

**Files:**
- Modify: `clients/migrator/src/types.ts`
- Modify: `clients/migrator/src/index.ts`
- Create: `clients/migrator/src/commands/mint-test.ts`

**Step 1: Add type to types.ts**

```typescript
export interface MintTestOptions {
  keypair: string;
  rpc: string;
  count: string;
  name: string;
  uri: string;
}
```

**Step 2: Create commands/mint-test.ts**

```typescript
import { createDasUmi, setupSignerFromKeypair } from "../setup";
import { mintTestCollection } from "../mint/bubblegum-mint";
import type { MintTestOptions } from "../types";

export async function mintTestCommand(opts: MintTestOptions): Promise<void> {
  const count = parseInt(opts.count, 10);
  if (isNaN(count) || count < 1) {
    console.error("--count must be a positive integer");
    process.exit(1);
  }

  console.log(`\nMint Test Configuration:`);
  console.log(`  RPC:        ${opts.rpc}`);
  console.log(`  Keypair:    ${opts.keypair}`);
  console.log(`  Count:      ${count}`);
  console.log(`  Name:       ${opts.name}`);
  console.log(`  URI:        ${opts.uri || "(empty)"}\n`);

  const umi = createDasUmi(opts.rpc);
  setupSignerFromKeypair(umi, opts.keypair);

  try {
    await mintTestCollection(umi, {
      count,
      collectionName: opts.name,
      nftUri: opts.uri,
    });
  } catch (err: any) {
    console.error(`\nFailed: ${err.message}`);
    process.exit(1);
  }
}
```

**Step 3: Add command to index.ts**

Import `mintTestCommand` and add:

```typescript
program
  .command("mint-test")
  .description("Mint a test Bubblegum compressed NFT collection on devnet")
  .option("-k, --keypair <path>", "Payer keypair file", "~/.config/solana/id.json")
  .option("--rpc <url>", "Solana RPC URL", "https://api.devnet.solana.com")
  .option("--count <number>", "Number of NFTs to mint", "10")
  .option("--name <string>", "Collection name", "Test Agent Collection")
  .option("--uri <url>", "Metadata URI for NFTs", "")
  .action(mintTestCommand);
```

Note: RPC defaults to devnet (not mainnet like other commands).

---

### Task 3: Implement bubblegum-mint.ts core logic

**Files:**
- Create: `clients/migrator/src/mint/bubblegum-mint.ts`

**Step 1: Create the file with full implementation**

The file should export `mintTestCollection(umi, opts)` which:

1. **Creates a merkle tree** using `createTree` from mpl-bubblegum
   - `generateSigner(umi)` for the tree keypair
   - `maxDepth: 14`, `maxBufferSize: 64` (supports up to ~16k assets)
   - This is an async function that returns a TransactionBuilder — must `await` it, then `sendAndConfirm`

2. **Creates a collection NFT** using `createV1` from mpl-token-metadata
   - `generateSigner(umi)` for the mint keypair
   - `name: opts.collectionName`, `uri: ""`, `sellerFeeBasisPoints: 0`
   - `isCollection: true`
   - `tokenStandard: TokenStandard.NonFungible`

3. **Mints N compressed NFTs** using `mintToCollectionV1` from mpl-bubblegum
   - `leafOwner: umi.identity.publicKey`
   - `merkleTree: treeKeypair.publicKey`
   - `collectionMint: collectionMint.publicKey`
   - `metadata`: `{ name: "Test Agent #N", uri: opts.nftUri, sellerFeeBasisPoints: 0, collection: { key: collectionMint.publicKey, verified: false }, creators: [{ address: umi.identity.publicKey, verified: false, share: 100 }] }`

4. **Prints summary** with collection address, tree address, mint count, and copy-paste migrate command

Key imports:
```typescript
import { generateSigner, publicKey } from "@metaplex-foundation/umi";
import { createTree, mintToCollectionV1 } from "@metaplex-foundation/mpl-bubblegum";
import { createV1, TokenStandard } from "@metaplex-foundation/mpl-token-metadata";
```

---

### Task 4: Build and verify

**Step 1: Build**

Run: `cd clients/migrator && pnpm build`

Fix any TypeScript errors.

**Step 2: Verify CLI help**

Run: `cd clients/migrator && npx ts-node src/index.ts mint-test --help`

Expected output should show all flags with defaults.

**Step 3: Commit**

```bash
git add clients/migrator/
git commit -m "feat(migrator): add mint-test command for Bubblegum test collections"
```
