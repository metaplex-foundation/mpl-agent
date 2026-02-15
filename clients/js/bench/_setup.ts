/* eslint-disable import/no-extraneous-dependencies */
import { createUmi as basecreateUmi } from '@metaplex-foundation/umi-bundle-tests';
import {
  mpl8004Identity,
} from '../src';

export const createUmi = async () => (await basecreateUmi()).use(mpl8004Identity());
