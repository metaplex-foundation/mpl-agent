import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import { dasApi, DasApiInterface } from "@metaplex-foundation/digital-asset-standard-api";
import { mplAgentIdentity } from "@metaplex-foundation/mpl-agent-registry";
import { irysUploader } from "@metaplex-foundation/umi-uploader-irys";
import { keypairIdentity, publicKey } from "@metaplex-foundation/umi";
import type { Umi, RpcInterface, Keypair } from "@metaplex-foundation/umi";
import bs58 from "bs58";
import * as fs from "fs";
import * as path from "path";

export type DasUmi = Umi & { rpc: RpcInterface & DasApiInterface };

export function createDasUmi(rpcUrl: string, dasUrl?: string): DasUmi {
  const umi = createUmi(rpcUrl)
    .use(dasApi())
    .use(mplAgentIdentity())
    .use(irysUploader());

  return umi as DasUmi;
}

export function loadKeypair(keypairPath: string): Keypair {
  const resolved = keypairPath.replace(/^~/, process.env.HOME ?? "~");
  const absolute = path.resolve(resolved);
  const secretKey = new Uint8Array(JSON.parse(fs.readFileSync(absolute, "utf-8")));
  return {
    publicKey: publicKey(bs58.encode(secretKey.slice(32))),
    secretKey,
  };
}

export function setupSignerFromKeypair(umi: DasUmi, keypairPath: string): DasUmi {
  const kp = loadKeypair(keypairPath);
  umi.use(keypairIdentity(kp));
  return umi;
}

export function validateSource(source: string): "bubblegum" | "token22" {
  const valid = ["bubblegum", "token22"];
  if (!valid.includes(source)) {
    console.error(`Invalid source standard: "${source}". Must be one of: ${valid.join(", ")}`);
    process.exit(1);
  }
  return source as "bubblegum" | "token22";
}

export function validatePublicKey(address: string, label: string): string {
  if (!/^[1-9A-HJ-NP-Za-km-z]{32,44}$/.test(address)) {
    console.error(`Invalid ${label} address: "${address}"`);
    process.exit(1);
  }
  return address;
}
