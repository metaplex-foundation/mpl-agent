import { generateSigner, publicKey, transactionBuilder } from "@metaplex-foundation/umi";
import { createCollection, createV2 } from "@metaplex-foundation/mpl-core";
import { getAssetWithProof, burn } from "@metaplex-foundation/mpl-bubblegum";
import { registerIdentityV1 } from "@metaplex-foundation/mpl-agent-registry/dist/src/generated/identity";
import pMap from "p-map";
import type { DasUmi } from "../setup";

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));
import { setupSignerFromKeypair } from "../setup";
import { fetchAllCollectionAssets, fetchAssetById } from "../das";
import {
  loadManifest,
  createManifest,
  mergeManifest,
  saveManifest,
  saveManifestAsync,
  updateAssetMigrateStatus,
  updateAssetBurnStatus,
  manifestSummary,
  manifestPath,
  type AssetData,
  type MigrationManifest,
} from "../manifest";

export interface BubblegumMigrateOptions {
  sourceCollection: string;
  destinationCollection?: string;
  keypairPath: string;
  agentUri?: string;
  burn: boolean;
  execute: boolean;
  batchSize: number;
  delay: number;
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

  // Step 2b: Build manifest
  const assetData: AssetData[] = compressed.map((a) => ({
    id: a.id.toString(),
    name: a.content?.metadata?.name ?? "(unnamed)",
    uri: a.content?.json_uri ?? "",
    owner: a.ownership?.owner ?? "unknown",
  }));

  let manifest: MigrationManifest;
  const existing = loadManifest(opts.sourceCollection);

  if (existing) {
    console.log(`  Resuming from existing manifest: ${manifestPath(opts.sourceCollection)}`);
    manifest = mergeManifest(existing, assetData);

    // Restore destination collection from manifest if available
    if (existing.destinationCollection) {
      if (opts.destinationCollection && opts.destinationCollection !== existing.destinationCollection) {
        console.log(`  WARNING: --destination ${opts.destinationCollection} differs from manifest's ${existing.destinationCollection}`);
        console.log(`  Using manifest's destination collection.`);
      }
      opts.destinationCollection = existing.destinationCollection;
    }

    const summary = manifestSummary(manifest);
    console.log(`  Manifest status: ${summary.migrated} migrated, ${summary.burned} burned, ${summary.failed} failed, ${summary.pending} pending out of ${summary.total}`);
  } else {
    manifest = createManifest(opts.sourceCollection, opts.destinationCollection ?? null, opts.burn, assetData);
  }

  // Step 3: Print migration plan
  const agentUri = opts.agentUri;
  const summary = manifestSummary(manifest);
  const toProcess = summary.pending + summary.failed;

  console.log("\nMigration Plan:");
  console.log(`  Total assets:       ${summary.total}`);
  console.log(`  Already migrated:   ${summary.migrated}`);
  console.log(`  Already burned:     ${summary.burned}`);
  console.log(`  To process:         ${toProcess}`);
  console.log(`  Agent URI:          ${agentUri ?? "(per-asset upload to Irys)"}`);
  console.log(`  Burn originals:     ${opts.burn}`);
  console.log(`  Concurrency:        ${opts.batchSize}`);
  console.log(`  TX delay:           ${opts.delay}ms`);
  console.log(`  Destination:        ${opts.destinationCollection ?? "(new collection)"}`);

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

  // Persist destination into manifest
  manifest.destinationCollection = destinationCollection;
  await saveManifestAsync(manifest);

  // Step 6: Migrate assets
  const toMigrate = manifest.assets.filter(
    (a) => a.migrateStatus === "pending" || a.migrateStatus === "failed"
  );
  console.log(`\nMigrating ${toMigrate.length} assets (concurrency: ${opts.batchSize})...\n`);

  let succeeded = 0;
  let failed = 0;

  await pMap(toMigrate, async (manifestAsset) => {
    const { sourceAssetId, name, uri, owner } = manifestAsset;

    try {
      const assetSigner = generateSigner(umi);

      // Determine agentRegistrationUri
      let assetAgentUri = agentUri; // from --agent-uri (static override)

      if (!agentUri) {
        // Fetch old off-chain metadata for image/description
        let image = "";
        let description = "";
        try {
          const resp = await fetch(uri);
          const offchain = await resp.json() as { image?: string; description?: string };
          image = offchain.image ?? "";
          description = offchain.description ?? "";
        } catch { /* use defaults */ }

        const registrationData = {
          name,
          type: "https://eips.ethereum.org/EIPS/eip-8004#registration-v1",
          image,
          active: true,
          services: [],
          description,
          x402Support: false,
          registrations: [
            {
              agentId: assetSigner.publicKey.toString(),
              agentRegistry: "solana:101:metaplex",
            },
          ],
          supportedTrust: [],
        };

        assetAgentUri = await umi.uploader.uploadJson(registrationData);
      }

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
        agentRegistrationUri: assetAgentUri!,
      });

      await transactionBuilder()
        .add(createIx)
        .add(registerIx)
        .sendAndConfirm(umi);

      if (opts.delay > 0) await sleep(opts.delay);

      succeeded++;
      updateAssetMigrateStatus(manifest, sourceAssetId, "migrated", assetSigner.publicKey.toString());
      saveManifest(manifest);
      process.stdout.write(`\r  Migrated ${succeeded}/${toMigrate.length} (${failed} failed)`);
    } catch (err: any) {
      failed++;
      updateAssetMigrateStatus(manifest, sourceAssetId, "failed", undefined, err.message);
      saveManifest(manifest);
      process.stdout.write(`\r  Migrated ${succeeded}/${toMigrate.length} (${failed} failed)`);
    }
  }, { concurrency: opts.batchSize });

  console.log(""); // newline after progress
  await saveManifestAsync(manifest);

  // Step 7: Optional burn pass
  if (opts.burn) {
    const toBurn = manifest.assets.filter(
      (a) => a.migrateStatus === "migrated" && a.burnStatus !== "burned"
    );

    if (toBurn.length > 0) {
      console.log(`\nBurning ${toBurn.length} original compressed NFTs (concurrency: ${opts.batchSize})...\n`);

      let burned = 0;
      let burnFailed = 0;

      await pMap(toBurn, async (manifestAsset) => {
        try {
          const assetProof = await getAssetWithProof(umi as any, publicKey(manifestAsset.sourceAssetId));
          await burn(umi as any, {
            ...assetProof,
            leafOwner: umi.identity,
          }).sendAndConfirm(umi);

          if (opts.delay > 0) await sleep(opts.delay);

          burned++;
          updateAssetBurnStatus(manifest, manifestAsset.sourceAssetId, "burned");
          saveManifest(manifest);
          process.stdout.write(`\r  Burned ${burned}/${toBurn.length} (${burnFailed} failed)`);
        } catch (err: any) {
          burnFailed++;
          updateAssetBurnStatus(manifest, manifestAsset.sourceAssetId, "failed", err.message);
          saveManifest(manifest);
          process.stdout.write(`\r  Burned ${burned}/${toBurn.length} (${burnFailed} failed)`);
        }
      }, { concurrency: opts.batchSize });

      console.log(""); // newline after progress
      await saveManifestAsync(manifest);
    }
  }

  // Step 8: Print summary
  const finalSummary = manifestSummary(manifest);
  console.log("\n=== Migration Summary ===");
  console.log(`  Manifest:    ${manifestPath(opts.sourceCollection)}`);
  console.log(`  Destination: ${destinationCollection}`);
  console.log(`  Total:       ${finalSummary.total}`);
  console.log(`  Migrated:    ${finalSummary.migrated}`);
  console.log(`  Burned:      ${finalSummary.burned}`);
  console.log(`  Failed:      ${finalSummary.failed}`);
  console.log(`  Pending:     ${finalSummary.pending}`);

  const failedAssets = manifest.assets.filter(
    (a) => a.migrateStatus === "failed" || a.burnStatus === "failed"
  );
  if (failedAssets.length > 0) {
    console.log(`\n  Errors:`);
    for (const a of failedAssets) {
      const phase = a.migrateStatus === "failed" ? "migrate" : "burn";
      console.log(`    ${a.sourceAssetId} [${phase}]: ${a.lastError}`);
    }
  }
}
