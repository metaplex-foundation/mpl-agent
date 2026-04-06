import test from 'ava';
import { generateSigner } from '@metaplex-foundation/umi';
import {
  fetchX402EndpointV1,
  findX402EndpointV1Pda,
  Key,
  registerX402V1,
} from '../../src/generated/tools';
import { createCollectionAndAsset, createUmi } from '../_setup';

function urlToBytes(url: string): number[] {
  const bytes = new Array(128).fill(0);
  for (let i = 0; i < url.length; i++) {
    bytes[i] = url.charCodeAt(i);
  }
  return bytes;
}

test('it can register an x402 endpoint for an agent asset', async (t) => {
  // Given a Umi instance and a collection with an asset.
  const umi = await createUmi();
  const { asset } = await createCollectionAndAsset(umi);

  const url = 'https://example.com/x402/pay';

  // When we register the x402 endpoint.
  await registerX402V1(umi, {
    agentAsset: asset,
    urlLen: url.length,
    url: urlToBytes(url),
  }).sendAndConfirm(umi);

  // Then the x402 endpoint is created with the correct data.
  const endpointPda = findX402EndpointV1Pda(umi, { asset });
  const endpoint = await fetchX402EndpointV1(umi, endpointPda);

  t.is(endpoint.key, Key.X402EndpointV1);
  t.is(endpoint.urlLen, url.length);
  t.is(endpoint.asset, asset);
  t.is(endpoint.authority, umi.identity.publicKey);

  // Verify URL bytes match.
  const storedUrl = String.fromCharCode(
    ...endpoint.url.slice(0, endpoint.urlLen)
  );
  t.is(storedUrl, url);
});

test('it cannot register an x402 endpoint if not the asset owner', async (t) => {
  // Given a Umi instance and a collection with an asset.
  const umi = await createUmi();
  const { asset } = await createCollectionAndAsset(umi);

  const url = 'https://example.com/x402/pay';
  const nonOwner = generateSigner(umi);

  // When a non-owner tries to register the x402 endpoint, it fails.
  const result = registerX402V1(umi, {
    agentAsset: asset,
    authority: nonOwner,
    urlLen: url.length,
    url: urlToBytes(url),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result);
});

test('it cannot register an x402 endpoint twice for the same asset', async (t) => {
  // Given a Umi instance and a collection with an asset.
  const umi = await createUmi();
  const { asset } = await createCollectionAndAsset(umi);

  const url = 'https://example.com/x402/pay';

  // First registration succeeds.
  await registerX402V1(umi, {
    agentAsset: asset,
    urlLen: url.length,
    url: urlToBytes(url),
  }).sendAndConfirm(umi);

  // Second registration fails.
  const result = registerX402V1(umi, {
    agentAsset: asset,
    urlLen: url.length,
    url: urlToBytes(url),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result);
});
