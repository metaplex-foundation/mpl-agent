import type { DasUmi } from "../setup";

export interface Token22MigrateOptions {
  sourceCollection: string;
  destinationCollection?: string;
  keypairPath: string;
  agentUri?: string;
  burn: boolean;
  execute: boolean;
  batchSize: number;
  delay: number;
}

export async function migrateToken22(
  umi: DasUmi,
  opts: Token22MigrateOptions
): Promise<void> {
  console.log("Token-2022 migration not yet implemented.");
}
