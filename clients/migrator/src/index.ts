#!/usr/bin/env node

import { Command } from "commander";
import { fetchCommand } from "./commands/fetch";
import { migrateCommand } from "./commands/migrate";
import { statusCommand } from "./commands/status";
import { mintTestCommand } from "./commands/mint-test";

const program = new Command();

program
  .name("mpl-agent-migrator")
  .description(
    "Migrate existing Agent collections to the MPL Agent Registry standard"
  )
  .version("0.2.0");

program
  .command("fetch")
  .description("Fetch and display assets from a collection using DAS")
  .requiredOption("-c, --collection <address>", "Collection address")
  .requiredOption("-s, --source <standard>", "Source standard: bubblegum | token22 | core")
  .option("-l, --limit <number>", "Max assets to fetch", "100")
  .option("--rpc <url>", "Solana RPC URL", "https://api.mainnet-beta.solana.com")
  .option("--das <url>", "DAS API URL (defaults to RPC URL)")
  .action(fetchCommand);

program
  .command("migrate")
  .description("Migrate a collection to MPL Core with Agent Registry (or register existing Core assets)")
  .requiredOption("-c, --collection <address>", "Source collection address")
  .requiredOption("-s, --source <standard>", "Source standard: bubblegum | token22 | core")
  .option("-d, --destination <address>", "Destination MPL Core collection (creates new if omitted)")
  .option("-k, --keypair <path>", "Payer keypair file", "~/.config/solana/id.json")
  .option("--batch-size <number>", "Assets per processing batch", "2")
  .option("--delay <ms>", "Delay between transactions in ms", "1000")
  .option("--agent-uri <url>", "Agent registration URI (uses default if omitted)")
  .option("--burn", "Burn original compressed NFTs after migration", false)
  .option("--execute", "Actually send transactions (dry run by default)", false)
  .option("--rpc <url>", "Solana RPC URL", "https://api.mainnet-beta.solana.com")
  .option("--das <url>", "DAS API URL (defaults to RPC URL)")
  .action(migrateCommand);

program
  .command("status")
  .description("Check the status of a migration")
  .requiredOption("-c, --collection <address>", "Collection address")
  .option("--rpc <url>", "Solana RPC URL", "https://api.mainnet-beta.solana.com")
  .option("--das <url>", "DAS API URL (defaults to RPC URL)")
  .action(statusCommand);

program
  .command("mint-test")
  .description("Mint a test collection on devnet (Bubblegum or Core)")
  .requiredOption("-s, --source <standard>", "Source standard: bubblegum | core")
  .option("-k, --keypair <path>", "Payer keypair file", "~/.config/solana/id.json")
  .option("--rpc <url>", "Solana RPC URL", "https://api.devnet.solana.com")
  .option("--count <number>", "Number of NFTs to mint", "10")
  .option("--concurrency <number>", "Parallel transactions", "2")
  .option("--delay <ms>", "Delay between transactions in ms", "1000")
  .option("--name <string>", "Collection name", "Test Agent Collection")
  .option("--uri <url>", "Metadata URI for NFTs", "")
  .action(mintTestCommand);

program.parse();
