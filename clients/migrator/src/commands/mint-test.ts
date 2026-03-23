import { createDasUmi, setupSignerFromKeypair, validateSource } from "../setup";
import { mintTestCollection } from "../mint/bubblegum-mint";
import { mintCoreTestCollection } from "../mint/core-mint";
import type { MintTestOptions } from "../types";

export async function mintTestCommand(opts: MintTestOptions): Promise<void> {
  const source = validateSource(opts.source);
  const count = parseInt(opts.count, 10);
  if (isNaN(count) || count < 1) {
    console.error("--count must be a positive integer");
    process.exit(1);
  }

  const concurrency = parseInt(opts.concurrency, 10);
  if (isNaN(concurrency) || concurrency < 1) {
    console.error("--concurrency must be a positive integer");
    process.exit(1);
  }

  const delay = parseInt(opts.delay, 10);

  console.log(`\nMint Test Configuration:`);
  console.log(`  Source:      ${source}`);
  console.log(`  RPC:         ${opts.rpc}`);
  console.log(`  Keypair:     ${opts.keypair}`);
  console.log(`  Count:       ${count}`);
  console.log(`  Concurrency: ${concurrency}`);
  console.log(`  TX delay:    ${delay}ms`);
  console.log(`  Name:        ${opts.name}`);
  console.log(`  URI:         ${opts.uri || "(empty)"}\n`);

  const umi = createDasUmi(opts.rpc);
  setupSignerFromKeypair(umi, opts.keypair);

  try {
    switch (source) {
      case "bubblegum":
        await mintTestCollection(umi, {
          count,
          concurrency,
          delay,
          collectionName: opts.name,
          nftUri: opts.uri,
        });
        break;
      case "core":
        await mintCoreTestCollection(umi, {
          count,
          concurrency,
          delay,
          collectionName: opts.name,
          nftUri: opts.uri,
        });
        break;
      case "token22":
        console.log("Token-2022 test minting not yet implemented.");
        break;
    }
  } catch (err: any) {
    console.error(`\nFailed: ${err.message}`);
    process.exit(1);
  }
}
