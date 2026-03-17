import * as fs from "fs";
import * as path from "path";

// --- Types ---

export type MigrateStatus = "pending" | "migrated" | "failed";
export type BurnStatus = "pending" | "burned" | "failed";

export interface ManifestAsset {
  sourceAssetId: string;
  destinationAssetId: string | null;
  name: string;
  uri: string;
  owner: string;
  migrateStatus: MigrateStatus;
  burnStatus: BurnStatus;
  lastError: string | null;
  updatedAt: string;
}

export interface MigrationManifest {
  version: 1;
  sourceCollection: string;
  destinationCollection: string | null;
  createdAt: string;
  updatedAt: string;
  burnRequested: boolean;
  assets: ManifestAsset[];
}

export interface ManifestSummary {
  total: number;
  migrated: number;
  burned: number;
  failed: number;
  pending: number;
}

// --- Asset data passed in from DAS fetch ---

export interface AssetData {
  id: string;
  name: string;
  uri: string;
  owner: string;
}

// --- Write queue for concurrency safety ---

let writeQueue: Promise<void> = Promise.resolve();

function enqueueWrite(filePath: string, data: string): void {
  writeQueue = writeQueue.then(() => writeAtomic(filePath, data)).catch(() => {});
}

async function writeAtomic(filePath: string, data: string): Promise<void> {
  const tmp = filePath + ".tmp";
  await fs.promises.writeFile(tmp, data, "utf-8");
  await fs.promises.rename(tmp, filePath);
}

// --- Path helpers ---

export function manifestPath(sourceCollection: string): string {
  return path.resolve(process.cwd(), `${sourceCollection}-migration.json`);
}

// --- Load / Create / Merge ---

export function loadManifest(sourceCollection: string): MigrationManifest | null {
  const p = manifestPath(sourceCollection);
  if (!fs.existsSync(p)) return null;
  const raw = fs.readFileSync(p, "utf-8");
  return JSON.parse(raw) as MigrationManifest;
}

export function createManifest(
  sourceCollection: string,
  destCollection: string | null,
  burnRequested: boolean,
  assetData: AssetData[]
): MigrationManifest {
  const now = new Date().toISOString();
  return {
    version: 1,
    sourceCollection,
    destinationCollection: destCollection,
    createdAt: now,
    updatedAt: now,
    burnRequested,
    assets: assetData.map((a) => ({
      sourceAssetId: a.id,
      destinationAssetId: null,
      name: a.name,
      uri: a.uri,
      owner: a.owner,
      migrateStatus: "pending" as MigrateStatus,
      burnStatus: "pending" as BurnStatus,
      lastError: null,
      updatedAt: now,
    })),
  };
}

export function mergeManifest(
  existing: MigrationManifest,
  freshAssets: AssetData[]
): MigrationManifest {
  const known = new Map(existing.assets.map((a) => [a.sourceAssetId, a]));

  for (const fresh of freshAssets) {
    if (!known.has(fresh.id)) {
      known.set(fresh.id, {
        sourceAssetId: fresh.id,
        destinationAssetId: null,
        name: fresh.name,
        uri: fresh.uri,
        owner: fresh.owner,
        migrateStatus: "pending",
        burnStatus: "pending",
        lastError: null,
        updatedAt: new Date().toISOString(),
      });
    }
  }

  existing.assets = Array.from(known.values());
  existing.updatedAt = new Date().toISOString();
  return existing;
}

// --- Save ---

export function saveManifest(manifest: MigrationManifest): void {
  manifest.updatedAt = new Date().toISOString();
  enqueueWrite(manifestPath(manifest.sourceCollection), JSON.stringify(manifest, null, 2));
}

export async function saveManifestAsync(manifest: MigrationManifest): Promise<void> {
  manifest.updatedAt = new Date().toISOString();
  enqueueWrite(manifestPath(manifest.sourceCollection), JSON.stringify(manifest, null, 2));
  await writeQueue;
}

// --- Update helpers ---

export function updateAssetMigrateStatus(
  manifest: MigrationManifest,
  sourceAssetId: string,
  status: MigrateStatus,
  destAssetId?: string,
  error?: string
): void {
  const asset = manifest.assets.find((a) => a.sourceAssetId === sourceAssetId);
  if (!asset) return;
  asset.migrateStatus = status;
  if (destAssetId !== undefined) asset.destinationAssetId = destAssetId;
  if (error !== undefined) asset.lastError = error;
  asset.updatedAt = new Date().toISOString();
}

export function updateAssetBurnStatus(
  manifest: MigrationManifest,
  sourceAssetId: string,
  status: BurnStatus,
  error?: string
): void {
  const asset = manifest.assets.find((a) => a.sourceAssetId === sourceAssetId);
  if (!asset) return;
  asset.burnStatus = status;
  if (error !== undefined) asset.lastError = error;
  asset.updatedAt = new Date().toISOString();
}

// --- Summary ---

export function manifestSummary(manifest: MigrationManifest): ManifestSummary {
  let migrated = 0;
  let burned = 0;
  let failed = 0;
  let pending = 0;

  for (const a of manifest.assets) {
    if (a.migrateStatus === "failed" || a.burnStatus === "failed") {
      failed++;
    } else if (a.burnStatus === "burned") {
      burned++;
    } else if (a.migrateStatus === "migrated") {
      migrated++;
    } else {
      pending++;
    }
  }

  return { total: manifest.assets.length, migrated, burned, failed, pending };
}
