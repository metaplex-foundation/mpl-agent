/* eslint-disable import/no-extraneous-dependencies */
import { createUmi as basecreateUmi } from '@metaplex-foundation/umi-bundle-tests';
import { generateSigner, PublicKey, Umi } from '@metaplex-foundation/umi';
import { create, createCollection } from '@metaplex-foundation/mpl-core';
import {
  mplAgentIdentity,
  mplAgentReputation,
  mplAgentValidation,
} from '../src';

export const createUmi = async () =>
  (await basecreateUmi())
    .use(mplAgentIdentity())
    .use(mplAgentReputation())
    .use(mplAgentValidation());

export async function createCollectionAndAsset(
  umi: Umi
): Promise<{ collection: PublicKey; asset: PublicKey }> {
  const collection = generateSigner(umi);
  const asset = generateSigner(umi);

  await createCollection(umi, {
    collection,
    name: 'Test Collection',
    uri: 'https://example.com/collection.json',
    plugins: [
      {
        type: 'Attributes',
        attributeList: [{ key: 'Test Attribute', value: 'Test Value' }],
      },
    ],
  }).sendAndConfirm(umi);

  await create(umi, {
    asset,
    name: 'Test Asset',
    uri: 'https://example.com/asset.json',
    collection,
  }).sendAndConfirm(umi);
  return { collection: collection.publicKey, asset: asset.publicKey };
}
