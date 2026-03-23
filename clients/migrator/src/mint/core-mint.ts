import { generateSigner } from "@metaplex-foundation/umi";
import { createCollection, createV2 } from "@metaplex-foundation/mpl-core";
import pMap from "p-map";
import type { DasUmi } from "../setup";

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

export interface CoreMintTestOptions {
  count: number;
  concurrency: number;
  delay: number;
  collectionName: string;
  nftUri: string;
}

export async function mintCoreTestCollection(
  umi: DasUmi,
  opts: CoreMintTestOptions
): Promise<void> {
  console.log(`Signer: ${umi.identity.publicKey}\n`);

  // Step 1: Create MPL Core collection
  console.log(`Creating MPL Core collection "${opts.collectionName}"...`);
  const collectionSigner = generateSigner(umi);

  await createCollection(umi, {
    collection: collectionSigner,
    name: opts.collectionName,
    uri: opts.nftUri || "",
  }).sendAndConfirm(umi);

  console.log(`  Collection: ${collectionSigner.publicKey}\n`);
  sleep(5000);

  // Step 2: Mint Core assets
  console.log(`Minting ${opts.count} MPL Core assets (concurrency: ${opts.concurrency})...\n`);
  let succeeded = 0;
  let failed = 0;
  const errors: string[] = [];

  const items = Array.from({ length: opts.count }, (_, i) => i);

  await pMap(items, async (i) => {
    const name = `Test Agent #${i + 1}`;
    try {
      const assetSigner = generateSigner(umi);

      await createV2(umi, {
        asset: assetSigner,
        name,
        uri: opts.nftUri || "",
        collection: collectionSigner.publicKey,
        owner: umi.identity.publicKey,
      }).sendAndConfirm(umi);

      if (opts.delay > 0) await sleep(opts.delay);

      succeeded++;
      process.stdout.write(`\r  Minted ${succeeded}/${opts.count} (${failed} failed)`);
    } catch (err: any) {
      failed++;
      errors.push(`${name}: ${err.message}`);
      process.stdout.write(`\r  Minted ${succeeded}/${opts.count} (${failed} failed)`);
    }
  }, { concurrency: opts.concurrency });

  console.log(""); // newline after progress

  // Step 3: Print summary
  console.log(`\n=== Mint Complete ===`);
  console.log(`  Collection: ${collectionSigner.publicKey}`);
  console.log(`  Succeeded:  ${succeeded}`);
  if (failed > 0) {
    console.log(`  Failed:     ${failed}`);
    for (const e of errors) {
      console.log(`    ${e}`);
    }
  }

  console.log(`\nTo register:\n  npx ts-node src/index.ts migrate -c ${collectionSigner.publicKey} -s core --rpc ${umi.rpc.getEndpoint()} -k <keypair> --execute`);
}
