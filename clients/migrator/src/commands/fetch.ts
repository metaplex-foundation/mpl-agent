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
