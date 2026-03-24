# Bubblegum Migration Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement the Bubblegum → MPL Core migration in the `clients/migrator` TypeScript CLI, including DAS pagination, collection creation, per-asset create+register transactions, and optional burn.

**Architecture:** The migrate command fetches all compressed NFTs from a Bubblegum collection via DAS, optionally creates a new Core collection, then for each asset builds a single transaction containing `createV2` (mpl-core) + `registerIdentityV1` (mpl-agent-identity). An optional `--burn` flag adds a second pass burning originals. Dry run by default; `--execute` to send.

**Tech Stack:** TypeScript, UMI framework, mpl-core, mpl-bubblegum, mpl-agent-registry (JS client), DAS API, commander CLI.

---

### Task 1: Update CLI flags and types (--execute, --agent-uri, --burn)

**Files:**
- Modify: `clients/migrator/src/types.ts`
- Modify: `clients/migrator/src/index.ts`
- Modify: `clients/migrator/src/commands/migrate.ts`

**Step 1: Update MigrateOptions type**

Replace the `MigrateOptions` interface in `clients/migrator/src/types.ts`:

```typescript
export interface MigrateOptions {
  collection: string;
  source: SourceStandard;
  destination?: string;
  keypair: string;
  batchSize: string;
  agentUri?: string;
  burn: boolean;
  execute: boolean;
  rpc: string;
  das?: string;
}
```

**Step 2: Update CLI flag definitions**

In `clients/migrator/src/index.ts`, replace the `migrate` command definition to swap `--dry-run` for `--execute`, add `--agent-uri` and `--burn`:

```typescript
program
  .command("migrate")
  .description("Migrate a collection to MPL Core with Agent Registry")
  .requiredOption("-c, --collection <address>", "Source collection address")
  .requiredOption("-s, --source <standard>", "Source standard: bubblegum | token22")
  .option("-d, --destination <address>", "Destination MPL Core collection (creates new if omitted)")
  .option("-k, --keypair <path>", "Payer keypair file", "~/.config/solana/id.json")
  .option("--batch-size <number>", "Assets per processing batch", "10")
  .option("--agent-uri <url>", "Agent registration URI (uses default if omitted)")
  .option("--burn", "Burn original compressed NFTs after migration", false)
  .option("--execute", "Actually send transactions (dry run by default)", false)
  .option("--rpc <url>", "Solana RPC URL", "https://api.mainnet-beta.solana.com")
  .option("--das <url>", "DAS API URL (defaults to RPC URL)")
  .action(migrateCommand);
```

**Step 3: Update migrate command to pass new options**

In `clients/migrator/src/commands/migrate.ts`, update the log output and the options passed to the source-specific migration functions. Replace `dryRun` references with `execute`, add `agentUri` and `burn`:

```typescript
import { createDasUmi, validateSource, validatePublicKey } from "../setup";
import { migrateBubblegum } from "../migrate/bubblegum";
import { migrateToken22 } from "../migrate/token22";
import type { MigrateOptions } from "../types";

export async function migrateCommand(opts: MigrateOptions): Promise<void> {
  const source = validateSource(opts.source);
  const collection = validatePublicKey(opts.collection, "collection");
  if (opts.destination) {
    validatePublicKey(opts.destination, "destination");
  }
  const batchSize = parseInt(opts.batchSize, 10);
  const dasUrl = opts.das ?? opts.rpc;

  console.log(`\n=== MPL Agent Migrator ===\n`);
  console.log(`Source collection:  ${collection}`);
  console.log(`Source standard:    ${source}`);
  console.log(`Destination:        ${opts.destination ?? "(new collection)"}`);
  console.log(`Batch size:         ${batchSize}`);
  console.log(`Agent URI:          ${opts.agentUri ?? "(default)"}`);
  console.log(`Burn originals:     ${opts.burn}`);
  console.log(`Execute:            ${opts.execute}`);
  console.log(`RPC:                ${opts.rpc}`);
  console.log(`DAS:                ${dasUrl}\n`);

  const umi = createDasUmi(opts.rpc, dasUrl);

  switch (source) {
    case "bubblegum":
      await migrateBubblegum(umi, {
        sourceCollection: collection,
        destinationCollection: opts.destination,
        keypairPath: opts.keypair,
        agentUri: opts.agentUri,
        burn: opts.burn,
        execute: opts.execute,
        batchSize,
      });
      break;
    case "token22":
      await migrateToken22(umi, {
        sourceCollection: collection,
        destinationCollection: opts.destination,
        keypairPath: opts.keypair,
        agentUri: opts.agentUri,
        burn: opts.burn,
        execute: opts.execute,
        batchSize,
      });
      break;
  }
}
```

**Step 4: Stub the token22 migrate to match new signature**

Update `clients/migrator/src/migrate/token22.ts` to accept the new options shape so it compiles:

```typescript
import type { DasUmi } from "../setup";

export interface Token22MigrateOptions {
  sourceCollection: string;
  destinationCollection?: string;
  keypairPath: string;
  agentUri?: string;
  burn: boolean;
  execute: boolean;
  batchSize: number;
}

export async function migrateToken22(
  umi: DasUmi,
  opts: Token22MigrateOptions
): Promise<void> {
  console.log("Token-2022 migration not yet implemented.");
}
```

**Step 5: Verify it compiles**

Run: `cd clients/migrator && npx tsc --noEmit`
Expected: no errors

**Step 6: Commit**

```bash
git add clients/migrator/src/types.ts clients/migrator/src/index.ts clients/migrator/src/commands/migrate.ts clients/migrator/src/migrate/token22.ts
git commit -m "refactor(migrator): update CLI flags — --execute, --agent-uri, --burn"
```

---

### Task 2: Add mpl-agent-registry dependency and UMI setup with keypair loading

**Files:**
- Modify: `clients/migrator/package.json`
- Modify: `clients/migrator/src/setup.ts`

**Step 1: Add the agent registry JS client as a dependency**

In `clients/migrator/package.json`, add to dependencies:

```json
"@metaplex-foundation/mpl-agent-registry": "link:../js"
```

Run: `cd clients/migrator && pnpm install`

**Step 2: Update setup.ts with keypair loading and agent identity plugin**

Replace `clients/migrator/src/setup.ts`:

```typescript
import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import { dasApi, DasApiInterface } from "@metaplex-foundation/digital-asset-standard-api";
import { mplAgentIdentity } from "@metaplex-foundation/mpl-agent-registry";
import { keypairIdentity } from "@metaplex-foundation/umi";
import type { Umi, RpcInterface, Keypair } from "@metaplex-foundation/umi";
import * as fs from "fs";
import * as path from "path";

export type DasUmi = Umi & { rpc: RpcInterface & DasApiInterface };

export function createDasUmi(rpcUrl: string, dasUrl?: string): DasUmi {
  const umi = createUmi(rpcUrl)
    .use(dasApi())
    .use(mplAgentIdentity());

  return umi as DasUmi;
}

export function loadKeypair(keypairPath: string): Keypair {
  const resolved = keypairPath.replace(/^~/, process.env.HOME ?? "~");
  const absolute = path.resolve(resolved);
  const secretKey = new Uint8Array(JSON.parse(fs.readFileSync(absolute, "utf-8")));
  // Solana keypair files are 64 bytes: first 32 are secret, full 64 is the expanded keypair
  // UMI Keypair expects { publicKey, secretKey } where secretKey is the full 64 bytes
  return {
    publicKey: toUmiPublicKey(secretKey.slice(32)),
    secretKey,
  };
}

// Helper to convert raw 32-byte public key to UMI PublicKey (base58 string)
function toUmiPublicKey(bytes: Uint8Array): import("@metaplex-foundation/umi").PublicKey {
  // UMI PublicKey is a base58 string — use the publicKey helper
  const { publicKey } = require("@metaplex-foundation/umi");
  const bs58 = require("bs58");
  return publicKey(bs58.encode(bytes));
}

export function setupSignerFromKeypair(umi: DasUmi, keypairPath: string): DasUmi {
  const kp = loadKeypair(keypairPath);
  umi.use(keypairIdentity(kp));
  return umi;
}

export function validateSource(source: string): "bubblegum" | "token22" {
  const valid = ["bubblegum", "token22"];
  if (!valid.includes(source)) {
    console.error(`Invalid source standard: "${source}". Must be one of: ${valid.join(", ")}`);
    process.exit(1);
  }
  return source as "bubblegum" | "token22";
}

export function validatePublicKey(address: string, label: string): string {
  if (!/^[1-9A-HJ-NP-Za-km-z]{32,44}$/.test(address)) {
    console.error(`Invalid ${label} address: "${address}"`);
    process.exit(1);
  }
  return address;
}
```

**Step 3: Add bs58 dependency**

Run: `cd clients/migrator && pnpm add bs58`

**Step 4: Verify it compiles**

Run: `cd clients/migrator && npx tsc --noEmit`
Expected: no errors

**Step 5: Commit**

```bash
git add clients/migrator/package.json clients/migrator/pnpm-lock.yaml clients/migrator/src/setup.ts
git commit -m "feat(migrator): add agent registry dep, keypair loading, signer setup"
```

---

### Task 3: Implement DAS pagination helper

**Files:**
- Create: `clients/migrator/src/das.ts` (replace existing stub)

**Step 1: Replace das.ts with pagination helper**

Replace `clients/migrator/src/das.ts`:

```typescript
import type { DasApiAsset } from "@metaplex-foundation/digital-asset-standard-api";
import type { DasUmi } from "./setup";

const DAS_PAGE_LIMIT = 1000;

/**
 * Fetch ALL assets in a collection by paging through DAS getAssetsByGroup.
 * Returns the complete list regardless of collection size.
 */
export async function fetchAllCollectionAssets(
  umi: DasUmi,
  collectionAddress: string
): Promise<DasApiAsset[]> {
  const allAssets: DasApiAsset[] = [];
  let page = 1;

  while (true) {
    const result = await umi.rpc.getAssetsByGroup({
      groupKey: "collection",
      groupValue: collectionAddress,
      limit: DAS_PAGE_LIMIT,
      page,
    });

    allAssets.push(...result.items);
    console.log(`  Fetched page ${page}: ${result.items.length} assets (${allAssets.length}/${result.total} total)`);

    if (allAssets.length >= result.total || result.items.length < DAS_PAGE_LIMIT) {
      break;
    }
    page++;
  }

  return allAssets;
}

/**
 * Fetch a single asset by its ID via DAS.
 */
export async function fetchAssetById(
  umi: DasUmi,
  assetId: string
): Promise<DasApiAsset> {
  const { publicKey } = require("@metaplex-foundation/umi");
  return umi.rpc.getAsset(publicKey(assetId));
}
```

**Step 2: Verify it compiles**

Run: `cd clients/migrator && npx tsc --noEmit`
Expected: no errors

**Step 3: Commit**

```bash
git add clients/migrator/src/das.ts
git commit -m "feat(migrator): DAS pagination helper for full collection fetch"
```

---

### Task 4: Implement the Bubblegum migration core

**Files:**
- Modify: `clients/migrator/src/migrate/bubblegum.ts`

This is the main implementation. Replace the entire file:

**Step 1: Write the full bubblegum migration**

```typescript
import { generateSigner, publicKey, transactionBuilder } from "@metaplex-foundation/umi";
import { createCollection, createV2 } from "@metaplex-foundation/mpl-core";
import { getAssetWithProof, burn } from "@metaplex-foundation/mpl-bubblegum";
import { registerIdentityV1 } from "@metaplex-foundation/mpl-agent-registry";
import type { DasApiAsset } from "@metaplex-foundation/digital-asset-standard-api";
import type { DasUmi } from "../setup";
import { setupSignerFromKeypair } from "../setup";
import { fetchAllCollectionAssets, fetchAssetById } from "../das";

const DEFAULT_AGENT_URI = "https://arweave.net/default-agent-identity";

export interface BubblegumMigrateOptions {
  sourceCollection: string;
  destinationCollection?: string;
  keypairPath: string;
  agentUri?: string;
  burn: boolean;
  execute: boolean;
  batchSize: number;
}

interface MigrationResult {
  succeeded: number;
  failed: number;
  skipped: number;
  errors: Array<{ assetId: string; error: string }>;
}

export async function migrateBubblegum(
  umi: DasUmi,
  opts: BubblegumMigrateOptions
): Promise<void> {
  console.log("=== Bubblegum -> MPL Core Migration ===\n");

  // Step 1: Fetch all source assets
  console.log("Step 1: Fetching all source assets...");
  const assets = await fetchAllCollectionAssets(umi, opts.sourceCollection);
  console.log(`\n  Total assets found: ${assets.length}\n`);

  if (assets.length === 0) {
    console.log("No assets found. Nothing to migrate.");
    return;
  }

  // Step 2: Validate — filter to compressed only
  const compressed = assets.filter((a) => a.compression?.compressed);
  const nonCompressed = assets.length - compressed.length;
  if (nonCompressed > 0) {
    console.log(`  Skipping ${nonCompressed} non-compressed assets`);
  }
  console.log(`  ${compressed.length} compressed assets to migrate\n`);

  if (compressed.length === 0) {
    console.log("No compressed assets found. Nothing to migrate.");
    return;
  }

  // Step 3: Print migration plan
  const agentUri = opts.agentUri ?? DEFAULT_AGENT_URI;
  console.log("Migration Plan:");
  console.log(`  Assets to create:   ${compressed.length}`);
  console.log(`  Agent URI:          ${agentUri}`);
  console.log(`  Burn originals:     ${opts.burn}`);
  console.log(`  Destination:        ${opts.destinationCollection ?? "(new collection)"}`);

  // Print first few assets as preview
  const preview = compressed.slice(0, 5);
  console.log(`\n  Preview (first ${preview.length}):`);
  for (const asset of preview) {
    const name = asset.content?.metadata?.name ?? "(unnamed)";
    const owner = asset.ownership?.owner ?? "unknown";
    console.log(`    ${asset.id} | ${name} | owner: ${owner}`);
  }
  if (compressed.length > 5) {
    console.log(`    ... and ${compressed.length - 5} more`);
  }

  if (!opts.execute) {
    console.log("\n[DRY RUN] Add --execute to send transactions.");
    return;
  }

  // Step 4: Set up signer
  console.log("\nSetting up signer...");
  setupSignerFromKeypair(umi, opts.keypairPath);
  console.log(`  Signer: ${umi.identity.publicKey}`);

  // Step 5: Create or verify destination collection
  let destinationCollection: string;
  if (opts.destinationCollection) {
    destinationCollection = opts.destinationCollection;
    console.log(`\nUsing existing destination collection: ${destinationCollection}`);
  } else {
    console.log("\nCreating new MPL Core collection...");
    const sourceCollectionData = await fetchAssetById(umi, opts.sourceCollection);
    const collectionName = sourceCollectionData.content?.metadata?.name ?? "Migrated Collection";
    const collectionUri = sourceCollectionData.content?.json_uri ?? "";

    const collectionSigner = generateSigner(umi);
    await (createCollection as any)(umi, {
      collection: collectionSigner,
      name: collectionName,
      uri: collectionUri,
    }).sendAndConfirm(umi);

    destinationCollection = collectionSigner.publicKey.toString();
    console.log(`  Created collection: ${destinationCollection}`);
  }

  // Step 6: Migrate assets in batches
  console.log(`\nMigrating ${compressed.length} assets...\n`);
  const result: MigrationResult = { succeeded: 0, failed: 0, skipped: 0, errors: [] };

  for (let i = 0; i < compressed.length; i++) {
    const asset = compressed[i];
    const name = asset.content?.metadata?.name ?? "(unnamed)";
    const uri = asset.content?.json_uri ?? "";
    const owner = asset.ownership?.owner ?? umi.identity.publicKey.toString();

    process.stdout.write(`  [${i + 1}/${compressed.length}] ${name} (${asset.id})... `);

    try {
      // Create Core asset + register identity in one transaction
      const assetSigner = generateSigner(umi);

      const createIx = (createV2 as any)(umi, {
        asset: assetSigner,
        name,
        uri,
        collection: publicKey(destinationCollection),
        owner: publicKey(owner),
      });

      const registerIx = registerIdentityV1(umi, {
        asset: assetSigner.publicKey,
        collection: publicKey(destinationCollection),
        agentRegistrationUri: agentUri,
      });

      await transactionBuilder()
        .add(createIx)
        .add(registerIx)
        .sendAndConfirm(umi);

      result.succeeded++;
      console.log("OK");
    } catch (err: any) {
      result.failed++;
      result.errors.push({ assetId: asset.id.toString(), error: err.message });
      console.log(`FAILED: ${err.message}`);
    }
  }

  // Step 7: Optional burn pass
  if (opts.burn && result.succeeded > 0) {
    console.log(`\nBurning ${compressed.length} original compressed NFTs...\n`);

    for (let i = 0; i < compressed.length; i++) {
      const asset = compressed[i];
      const name = asset.content?.metadata?.name ?? "(unnamed)";

      process.stdout.write(`  [${i + 1}/${compressed.length}] Burning ${name}... `);

      try {
        const assetProof = await getAssetWithProof(umi as any, publicKey(asset.id));
        await burn(umi as any, {
          ...assetProof,
          leafOwner: umi.identity,
        }).sendAndConfirm(umi);

        console.log("OK");
      } catch (err: any) {
        console.log(`FAILED: ${err.message}`);
      }
    }
  }

  // Step 8: Print summary
  console.log("\n=== Migration Summary ===");
  console.log(`  Destination: ${destinationCollection}`);
  console.log(`  Succeeded:   ${result.succeeded}`);
  console.log(`  Failed:      ${result.failed}`);
  console.log(`  Skipped:     ${result.skipped}`);

  if (result.errors.length > 0) {
    console.log(`\n  Errors:`);
    for (const { assetId, error } of result.errors) {
      console.log(`    ${assetId}: ${error}`);
    }
  }
}
```

**Step 2: Verify it compiles**

Run: `cd clients/migrator && npx tsc --noEmit`
Expected: no errors (may need to adjust imports based on what mpl-agent-registry exports — if `registerIdentityV1` is not a direct export, import from `@metaplex-foundation/mpl-agent-registry/generated/identity`)

**Step 3: Fix any import issues**

The `registerIdentityV1` function may need to be imported from the generated identity subpath. Check what `@metaplex-foundation/mpl-agent-registry` exports and adjust accordingly. The test file imports it as:
```typescript
import { registerIdentityV1 } from '../../src/generated/identity';
```

So in the migrator it might need:
```typescript
// If the package re-exports from generated:
import { registerIdentityV1 } from "@metaplex-foundation/mpl-agent-registry";
// Or if it needs a deeper import, check the package's index.ts exports
```

**Step 4: Commit**

```bash
git add clients/migrator/src/migrate/bubblegum.ts
git commit -m "feat(migrator): implement Bubblegum -> MPL Core migration"
```

---

### Task 5: Update fetch command to use the pagination helper

**Files:**
- Modify: `clients/migrator/src/commands/fetch.ts`

**Step 1: Rewrite fetch command to use fetchAllCollectionAssets**

```typescript
import { createDasUmi, validateSource, validatePublicKey } from "../setup";
import { fetchAllCollectionAssets } from "../das";
import type { FetchOptions } from "../types";

export async function fetchCommand(opts: FetchOptions): Promise<void> {
  const source = validateSource(opts.source);
  const collection = validatePublicKey(opts.collection, "collection");
  const limit = parseInt(opts.limit, 10);
  const dasUrl = opts.das ?? opts.rpc;

  console.log(`\nFetching assets from collection: ${collection}`);
  console.log(`Source standard: ${source}`);
  console.log(`DAS endpoint: ${dasUrl}\n`);

  const umi = createDasUmi(opts.rpc, dasUrl);

  try {
    const assets = await fetchAllCollectionAssets(umi, collection);
    const display = limit > 0 ? assets.slice(0, limit) : assets;

    console.log(`\nTotal: ${assets.length} assets (showing ${display.length})\n`);

    for (const asset of display) {
      const name = asset.content?.metadata?.name ?? "(unnamed)";
      const owner = asset.ownership?.owner ?? "unknown";
      const compressed = asset.compression?.compressed ? "compressed" : "standard";
      console.log(`  ${asset.id} | ${name} | ${owner} | ${compressed}`);
    }
  } catch (err: any) {
    console.error(`Failed to fetch assets: ${err.message}`);
    process.exit(1);
  }
}
```

**Step 2: Verify it compiles**

Run: `cd clients/migrator && npx tsc --noEmit`
Expected: no errors

**Step 3: Commit**

```bash
git add clients/migrator/src/commands/fetch.ts
git commit -m "refactor(migrator): fetch command uses pagination helper"
```

---

### Task 6: Build and verify CLI help output

**Step 1: Build the project**

Run: `cd clients/migrator && pnpm build`
Expected: clean build, dist/ directory created

**Step 2: Verify CLI help**

Run: `node clients/migrator/dist/index.js --help`
Expected: shows three commands (fetch, migrate, status)

Run: `node clients/migrator/dist/index.js migrate --help`
Expected: shows all flags including --execute, --agent-uri, --burn

**Step 3: Commit if any build fixes were needed**

---

### Task 7: Final type-check and cleanup

**Step 1: Full type-check**

Run: `cd clients/migrator && npx tsc --noEmit`

**Step 2: Fix any remaining issues**

Check for unused imports, type mismatches, etc.

**Step 3: Final commit**

```bash
git add -A clients/migrator/
git commit -m "feat(migrator): complete Bubblegum migration implementation"
```
