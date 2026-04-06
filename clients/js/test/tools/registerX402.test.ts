import test from 'ava';
import { generateSigner } from '@metaplex-foundation/umi';
import {
  fetchX402EndpointV1,
  findX402EndpointV1Pda,
  Key,
  registerX402V1,
} from '../../src/generated/tools';
import { createCollectionAndAsset, createUmi } from '../_setup';

test('it can register an x402 endpoint for an agent asset', async (t) => {
  // Given a Umi instance and a collection with an asset.
  const umi = await createUmi();
  const { asset } = await createCollectionAndAsset(umi);

  const url = 'https://example.com/x402/pay';

  // When we register the x402 endpoint.
  await registerX402V1(umi, {
    agentAsset: asset,
    url,
  }).sendAndConfirm(umi);

  // Then the x402 endpoint is created with the correct data.
  const endpointPda = findX402EndpointV1Pda(umi, { asset });
  const endpoint = await fetchX402EndpointV1(umi, endpointPda);

  t.is(endpoint.key, Key.X402EndpointV1);
  t.is(endpoint.asset, asset);
  t.is(endpoint.authority, umi.identity.publicKey);
  t.is(endpoint.url, url);
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
    url,
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
    url,
  }).sendAndConfirm(umi);

  // Second registration fails.
  const result = registerX402V1(umi, {
    agentAsset: asset,
    url,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result);
});
