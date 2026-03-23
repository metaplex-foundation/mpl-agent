import { createDasUmi, validateSource, validatePublicKey } from "../setup";
import { migrateBubblegum } from "../migrate/bubblegum";
import { migrateToken22 } from "../migrate/token22";
import { migrateCore } from "../migrate/core";
import type { MigrateOptions } from "../types";

export async function migrateCommand(opts: MigrateOptions): Promise<void> {
  const source = validateSource(opts.source);
  const collection = validatePublicKey(opts.collection, "collection");
  if (opts.destination) {
    validatePublicKey(opts.destination, "destination");
  }
  const batchSize = parseInt(opts.batchSize, 10);
  const delay = parseInt(opts.delay, 10);
  const dasUrl = opts.das ?? opts.rpc;

  console.log(`\n=== MPL Agent Migrator ===\n`);
  console.log(`Source collection:  ${collection}`);
  console.log(`Source standard:    ${source}`);
  if (source !== "core") {
    console.log(`Destination:        ${opts.destination ?? "(new collection)"}`);
  }
  console.log(`Batch size:         ${batchSize}`);
  console.log(`TX delay:           ${delay}ms`);
  console.log(`Agent URI:          ${opts.agentUri ?? "(default)"}`);
  if (source !== "core") {
    console.log(`Burn originals:     ${opts.burn}`);
  }
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
        delay,
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
        delay,
      });
      break;
    case "core":
      await migrateCore(umi, {
        sourceCollection: collection,
        keypairPath: opts.keypair,
        agentUri: opts.agentUri,
        execute: opts.execute,
        batchSize,
        delay,
      });
      break;
  }
}
