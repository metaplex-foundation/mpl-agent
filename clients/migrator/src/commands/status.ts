import { validatePublicKey } from "../setup";
import { loadManifest, manifestSummary, manifestPath } from "../manifest";
import type { StatusOptions } from "../types";

export async function statusCommand(opts: StatusOptions): Promise<void> {
  const collection = validatePublicKey(opts.collection, "collection");

  const manifest = loadManifest(collection);
  if (!manifest) {
    console.log(`\nNo migration manifest found at ${manifestPath(collection)}`);
    console.log("Run a migration first to create a manifest.");
    return;
  }

  const summary = manifestSummary(manifest);

  console.log(`\n=== Migration Status ===`);
  console.log(`  Source:       ${manifest.sourceCollection}`);
  console.log(`  Destination:  ${manifest.destinationCollection ?? "(not yet created)"}`);
  console.log(`  Created:      ${manifest.createdAt}`);
  console.log(`  Updated:      ${manifest.updatedAt}`);
  console.log(`  Burn:         ${manifest.burnRequested}`);
  console.log();
  console.log(`  Total:        ${summary.total}`);
  console.log(`  Migrated:     ${summary.migrated}`);
  console.log(`  Burned:       ${summary.burned}`);
  console.log(`  Failed:       ${summary.failed}`);
  console.log(`  Pending:      ${summary.pending}`);

  const failedAssets = manifest.assets.filter(
    (a) => a.migrateStatus === "failed" || a.burnStatus === "failed"
  );
  if (failedAssets.length > 0) {
    console.log(`\n  Failed assets:`);
    for (const a of failedAssets) {
      const phase = a.migrateStatus === "failed" ? "migrate" : "burn";
      console.log(`    ${a.sourceAssetId} [${phase}]: ${a.lastError ?? "unknown error"}`);
    }
  }
}
