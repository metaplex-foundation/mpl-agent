import { publicKey, transactionBuilder } from "@metaplex-foundation/umi";
import { registerIdentityV1 } from "@metaplex-foundation/mpl-agent-registry/dist/src/generated/identity";
import pMap from "p-map";
import type { DasUmi } from "../setup";

import { setupSignerFromKeypair } from "../setup";
import { fetchAllCollectionAssets } from "../das";
import {
  loadManifest,
  createManifest,
  mergeManifest,
  saveManifest,
  saveManifestAsync,
  updateAssetMigrateStatus,
  manifestSummary,
  manifestPath,
  type AssetData,
  type MigrationManifest,
} from "../manifest";

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

export interface CoreMigrateOptions {
  sourceCollection: string;
  keypairPath: string;
  agentUri?: string;
  execute: boolean;
  batchSize: number;
  delay: number;
}

export async function migrateCore(
  umi: DasUmi,
  opts: CoreMigrateOptions
): Promise<void> {
  console.log("=== MPL Core -> Agent Registry Registration ===\n");

  // Step 1: Fetch all assets in the collection
  console.log("Step 1: Fetching all collection assets...");
  const assets = await fetchAllCollectionAssets(umi, opts.sourceCollection);
  console.log(`\n  Total assets found: ${assets.length}\n`);

  if (assets.length === 0) {
    console.log("No assets found. Nothing to register.");
    return;
  }

  // Step 2: Build manifest
  const assetData: AssetData[] = assets.map((a) => ({
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

    const summary = manifestSummary(manifest);
    console.log(`  Manifest status: ${summary.migrated} registered, ${summary.failed} failed, ${summary.pending} pending out of ${summary.total}`);
  } else {
    // Core assets are registered in-place — source and destination are the same collection.
    manifest = createManifest(opts.sourceCollection, opts.sourceCollection, false, assetData);
  }

  // Step 3: Print registration plan
  const agentUri = opts.agentUri;
  const summary = manifestSummary(manifest);
  const toProcess = summary.pending + summary.failed;

  console.log("\nRegistration Plan:");
  console.log(`  Collection:         ${opts.sourceCollection}`);
  console.log(`  Total assets:       ${summary.total}`);
  console.log(`  Already registered: ${summary.migrated}`);
  console.log(`  To process:         ${toProcess}`);
  console.log(`  Agent URI:          ${agentUri ?? "(per-asset upload to Irys)"}`);
  console.log(`  Concurrency:        ${opts.batchSize}`);
  console.log(`  TX delay:           ${opts.delay}ms`);

  const preview = assets.slice(0, 5);
  console.log(`\n  Preview (first ${preview.length}):`);
  for (const asset of preview) {
    const name = asset.content?.metadata?.name ?? "(unnamed)";
    const owner = asset.ownership?.owner ?? "unknown";
    console.log(`    ${asset.id} | ${name} | owner: ${owner}`);
  }
  if (assets.length > 5) {
    console.log(`    ... and ${assets.length - 5} more`);
  }

  if (!opts.execute) {
    console.log("\n[DRY RUN] Add --execute to send transactions.");
    return;
  }

  // Step 4: Set up signer
  console.log("\nSetting up signer...");
  setupSignerFromKeypair(umi, opts.keypairPath);
  console.log(`  Signer: ${umi.identity.publicKey}`);

  // Step 5: Register assets
  const toRegister = manifest.assets.filter(
    (a) => a.migrateStatus === "pending" || a.migrateStatus === "failed"
  );
  console.log(`\nRegistering ${toRegister.length} assets (concurrency: ${opts.batchSize})...\n`);

  let succeeded = 0;
  let failed = 0;

  await pMap(toRegister, async (manifestAsset) => {
    const { sourceAssetId, name, uri } = manifestAsset;

    try {
      // Determine agentRegistrationUri
      let assetAgentUri = agentUri; // from --agent-uri (static override)

      if (!agentUri) {
        // Fetch off-chain metadata for image/description
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
              agentId: sourceAssetId,
              agentRegistry: "solana:101:metaplex",
            },
          ],
          supportedTrust: [],
        };

        assetAgentUri = await umi.uploader.uploadJson(registrationData);
      }

      const registerIx = registerIdentityV1(umi, {
        asset: publicKey(sourceAssetId),
        collection: publicKey(opts.sourceCollection),
        agentRegistrationUri: assetAgentUri!,
      });

      await transactionBuilder()
        .add(registerIx)
        .sendAndConfirm(umi);

      if (opts.delay > 0) await sleep(opts.delay);

      succeeded++;
      updateAssetMigrateStatus(manifest, sourceAssetId, "migrated", sourceAssetId);
      saveManifest(manifest);
      process.stdout.write(`\r  Registered ${succeeded}/${toRegister.length} (${failed} failed)`);
    } catch (err: any) {
      failed++;
      updateAssetMigrateStatus(manifest, sourceAssetId, "failed", undefined, err.message);
      saveManifest(manifest);
      process.stdout.write(`\r  Registered ${succeeded}/${toRegister.length} (${failed} failed)`);
    }
  }, { concurrency: opts.batchSize });

  console.log(""); // newline after progress
  await saveManifestAsync(manifest);

  // Step 6: Print summary
  const finalSummary = manifestSummary(manifest);
  console.log("\n=== Registration Summary ===");
  console.log(`  Manifest:    ${manifestPath(opts.sourceCollection)}`);
  console.log(`  Collection:  ${opts.sourceCollection}`);
  console.log(`  Total:       ${finalSummary.total}`);
  console.log(`  Registered:  ${finalSummary.migrated}`);
  console.log(`  Failed:      ${finalSummary.failed}`);
  console.log(`  Pending:     ${finalSummary.pending}`);

  const failedAssets = manifest.assets.filter(
    (a) => a.migrateStatus === "failed"
  );
  if (failedAssets.length > 0) {
    console.log(`\n  Errors:`);
    for (const a of failedAssets) {
      console.log(`    ${a.sourceAssetId}: ${a.lastError}`);
    }
  }
}
