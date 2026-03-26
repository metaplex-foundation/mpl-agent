/**
 * Live devnet test for the Agent API SDK.
 *
 * NOT included in `pnpm test` (excluded in ava config).
 * Run manually:
 *   1. pnpm build
 *   2. npx ava dist/test/api/mintAgent.test.js --serial --timeout=120s
 *
 * Set KEYPAIR env var to override the keypair path (defaults to ~/.config/solana/id.json).
 */
import test from 'ava';
import fs from 'fs';
import path from 'path';
import { createUmi } from '@metaplex-foundation/umi-bundle-defaults';
import { keypairIdentity } from '@metaplex-foundation/umi';

import { mintAgent, isAgentApiError } from '../../src/api';
import { mplAgentIdentity } from '../../src/plugin';

const DEVNET_RPC = 'https://api.devnet.solana.com';

function loadKeypairFile(): Uint8Array {
  const keypairPath =
    process.env.KEYPAIR ??
    path.join(process.env.HOME ?? '/home/user', '.config', 'solana', 'id.json');
  const raw = fs.readFileSync(keypairPath, 'utf-8');
  return Uint8Array.from(JSON.parse(raw));
}

function setupUmi() {
  const umi = createUmi(DEVNET_RPC).use(mplAgentIdentity());
  const secretKey = loadKeypairFile();
  const kp = umi.eddsa.createKeypairFromSecretKey(secretKey);
  umi.use(keypairIdentity(kp));
  return umi;
}

test('mintAgent returns transaction for a valid devnet request', async (t) => {
  const umi = setupUmi();
  const walletStr = umi.identity.publicKey.toString();
  t.log(`Wallet: ${walletStr}`);

  const result = await mintAgent(
    umi,
    {},
    {
      wallet: umi.identity.publicKey,
      network: 'solana-devnet',
      name: 'SDK Test Agent',
      uri: 'https://example.com/test-agent-metadata.json',
      agentMetadata: {
        type: 'agent',
        name: 'SDK Test Agent',
        description: 'Created by mpl-agent-registry SDK test',
        services: [
          { name: 'test-service', endpoint: 'https://example.com/test' },
        ],
        registrations: [],
        supportedTrust: [],
      },
    }
  );

  t.log(`Asset address: ${result.assetAddress}`);
  t.log(`Blockhash: ${JSON.stringify(result.blockhash)}`);

  t.truthy(result.assetAddress, 'should return an asset address');
  t.truthy(result.transaction, 'should return a transaction');
  t.truthy(result.blockhash, 'should return a blockhash');
});

test('mintAgent rejects invalid input with an API error', async (t) => {
  const umi = setupUmi();

  const error = await t.throwsAsync(() =>
    mintAgent(
      umi,
      {},
      {
        wallet: umi.identity.publicKey,
        network: 'solana-devnet',
        name: '',
        uri: '',
        agentMetadata: {} as any,
      }
    )
  );

  t.truthy(error, 'should throw an error for invalid input');
  if (isAgentApiError(error)) {
    t.log(`API error (${error.statusCode}): ${error.message}`);
    t.log(`Response body: ${JSON.stringify(error.responseBody)}`);
  }
});
