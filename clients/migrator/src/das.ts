import { publicKey } from "@metaplex-foundation/umi";
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
  return umi.rpc.getAsset(publicKey(assetId));
}
