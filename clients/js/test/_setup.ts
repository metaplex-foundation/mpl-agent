/* eslint-disable import/no-extraneous-dependencies */
import { createUmi as basecreateUmi } from '@metaplex-foundation/umi-bundle-tests';
import { generateSigner, PublicKey, Umi } from '@metaplex-foundation/umi';
import {
  create,
  createCollection,
  mplCore,
} from '@metaplex-foundation/mpl-core';
import { mplToolbox } from '@metaplex-foundation/mpl-toolbox';
import {
  mplAgentIdentity,
  mplAgentReputation,
  mplAgentValidation,
  mplAgentTools,
} from '../src';

export const createUmi = async () =>
  (await basecreateUmi())
    .use(mplCore())
    .use(mplToolbox())
    .use(mplAgentIdentity())
    .use(mplAgentReputation())
    .use(mplAgentValidation())
    .use(mplAgentTools());

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
