export type SourceStandard = "bubblegum" | "token22" | "core";

export interface FetchOptions {
  collection: string;
  source: SourceStandard;
  limit: string;
  rpc: string;
  das?: string;
}

export interface MigrateOptions {
  collection: string;
  source: SourceStandard;
  destination?: string;
  keypair: string;
  batchSize: string;
  delay: string;
  agentUri?: string;
  burn: boolean;
  execute: boolean;
  rpc: string;
  das?: string;
}

export interface StatusOptions {
  collection: string;
  rpc: string;
  das?: string;
}

export interface MintTestOptions {
  source: string;
  keypair: string;
  rpc: string;
  count: string;
  name: string;
  uri: string;
  concurrency: string;
  delay: string;
}
