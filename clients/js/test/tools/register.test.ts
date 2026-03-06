import test from 'ava';
import { generateSigner } from '@metaplex-foundation/umi';
import {
  fetchExecutorProfileV1,
  ExecutorProfileV1,
  Key,
  registerExecutorV1,
} from '../../src/generated/tools';
import { createUmi } from '../_setup';

test('it can register an executor profile', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const executorProfile = await generateSigner(umi);

  // When we register the executor profile.
  await registerExecutorV1(umi, {
    executorProfile,
  }).sendAndConfirm(umi);

  // Then the executor profile is created with the correct data.
  t.like(await fetchExecutorProfileV1(umi, executorProfile.publicKey), <
    ExecutorProfileV1
  >{
    key: Key.ExecutorProfileV1,
    authority: umi.identity.publicKey,
  });
});

test('it can register an executor profile with a custom authority', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const executorProfile = await generateSigner(umi);
  const authority = await generateSigner(umi);

  // When we register the executor profile with a custom authority.
  await registerExecutorV1(umi, {
    executorProfile,
    authority,
  }).sendAndConfirm(umi);

  // Then the executor profile authority is the custom authority.
  t.like(await fetchExecutorProfileV1(umi, executorProfile.publicKey), <
    ExecutorProfileV1
  >{
    key: Key.ExecutorProfileV1,
    authority: authority.publicKey,
  });
});
