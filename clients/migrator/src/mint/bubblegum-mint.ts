import { generateSigner, percentAmount, some } from "@metaplex-foundation/umi";
import { createTree, mintToCollectionV1 } from "@metaplex-foundation/mpl-bubblegum";
import { createV1, TokenStandard } from "@metaplex-foundation/mpl-token-metadata";
import pMap from "p-map";
import type { DasUmi } from "../setup";

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

export interface MintTestCollectionOptions {
  count: number;
  concurrency: number;
  delay: number;
  collectionName: string;
  nftUri: string;
}

export async function mintTestCollection(
  umi: DasUmi,
  opts: MintTestCollectionOptions
): Promise<void> {
  console.log(`Signer: ${umi.identity.publicKey}\n`);

  // Step 1: Create merkle tree
  console.log("Creating merkle tree (depth=14, bufferSize=64)...");
  const merkleTree = generateSigner(umi);

  const createTreeTx = await createTree(umi, {
    merkleTree,
    maxDepth: 14,
    maxBufferSize: 64,
  });
  await createTreeTx.sendAndConfirm(umi);

  console.log(`  Tree: ${merkleTree.publicKey}\n`);

  // Step 2: Create collection NFT
  console.log(`Creating collection "${opts.collectionName}"...`);
  const collectionMint = generateSigner(umi);

  await createV1(umi, {
    mint: collectionMint,
    name: opts.collectionName,
    uri: opts.nftUri || "",
    sellerFeeBasisPoints: percentAmount(0),
    isCollection: true,
    tokenStandard: TokenStandard.NonFungible,
  }).sendAndConfirm(umi);

  console.log(`  Collection: ${collectionMint.publicKey}\n`);

  // Step 3: Mint compressed NFTs
  console.log(`Minting ${opts.count} compressed NFTs (concurrency: ${opts.concurrency})...\n`);
  let succeeded = 0;
  let failed = 0;

  const items = Array.from({ length: opts.count }, (_, i) => i);

  await pMap(items, async (i) => {
    const name = `Test Agent #${i + 1}`;
    try {
      await mintToCollectionV1(umi, {
        leafOwner: umi.identity.publicKey,
        merkleTree: merkleTree.publicKey,
        collectionMint: collectionMint.publicKey,
        metadata: {
          name,
          uri: opts.nftUri || "",
          sellerFeeBasisPoints: 0,
          collection: some({ key: collectionMint.publicKey, verified: false }),
          creators: [
            { address: umi.identity.publicKey, verified: false, share: 100 },
          ],
        },
      }).sendAndConfirm(umi);

      if (opts.delay > 0) await sleep(opts.delay);

      succeeded++;
      process.stdout.write(`\r  Minted ${succeeded}/${opts.count} (${failed} failed)`);
    } catch (err: any) {
      failed++;
      process.stdout.write(`\r  Minted ${succeeded}/${opts.count} (${failed} failed)`);
    }
  }, { concurrency: opts.concurrency });

  console.log(""); // newline after progress

  // Step 4: Print summary
  console.log(`\n=== Mint Complete ===`);
  console.log(`  Collection: ${collectionMint.publicKey}`);
  console.log(`  Tree:       ${merkleTree.publicKey}`);
  console.log(`  Succeeded:  ${succeeded}`);
  if (failed > 0) {
    console.log(`  Failed:     ${failed}`);
  }

  console.log(`\nTo migrate:\n  npx ts-node src/index.ts migrate -c ${collectionMint.publicKey} -s bubblegum --rpc ${umi.rpc.getEndpoint()} -k <keypair> --execute`);
}
