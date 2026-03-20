import test from 'ava';
import { generateSigner } from '@metaplex-foundation/umi';
import {
  fetchExecutiveProfileV1,
  Key,
  registerExecutiveV1,
  findExecutiveProfileV1Pda,
} from '../../src/generated/tools';
import { createUmi } from '../_setup';

test('it can register an executive profile', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();

  // When we register the executive profile.
  await registerExecutiveV1(umi, {}).sendAndConfirm(umi);

  // Then the executive profile is created with the correct data.
  t.like(
    await fetchExecutiveProfileV1(
      umi,
      findExecutiveProfileV1Pda(umi, { authority: umi.identity.publicKey })
    ),
    {
      key: Key.ExecutiveProfileV1,
      authority: umi.identity.publicKey,
    }
  );
});

test('it can register an executive profile with a custom authority', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const authority = await generateSigner(umi);

  // When we register the executive profile with a custom authority.
  await registerExecutiveV1(umi, {
    authority,
  }).sendAndConfirm(umi);

  // Then the executive profile authority is the custom authority.
  t.like(
    await fetchExecutiveProfileV1(
      umi,
      findExecutiveProfileV1Pda(umi, { authority: authority.publicKey })
    ),
    {
      key: Key.ExecutiveProfileV1,
      authority: authority.publicKey,
    }
  );
});
