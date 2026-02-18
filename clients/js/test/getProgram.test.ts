import test from 'ava';
import {
  IDENTITY_ID as MPL_AGENT_IDENTITY_PROGRAM_ID,
  REPUTATION_ID as MPL_AGENT_REPUTATION_PROGRAM_ID,
  VALIDATION_ID as MPL_AGENT_VALIDATION_PROGRAM_ID,
} from '../src';
import { createUmi } from './_setup';

test('it registers the identity program', async (t) => {
  // Given a Umi instance using the project's plugin.
  const umi = await createUmi();

  // When we fetch the registered program.
  const program = umi.programs.get('mplAgentIdentity');

  // Then we expect it to be the same as the program ID constant.
  t.true(program.publicKey === MPL_AGENT_IDENTITY_PROGRAM_ID);
});

test('it registers the reputation program', async (t) => {
  // Given a Umi instance using the project's plugin.
  const umi = await createUmi();

  // When we fetch the registered program.
  const program = umi.programs.get('mplAgentReputation');

  // Then we expect it to be the same as the program ID constant.
  t.true(program.publicKey === MPL_AGENT_REPUTATION_PROGRAM_ID);
});

test('it registers the validation program', async (t) => {
  // Given a Umi instance using the project's plugin.
  const umi = await createUmi();

  // When we fetch the registered program.
  const program = umi.programs.get('mplAgentValidation');

  // Then we expect it to be the same as the program ID constant.
  t.true(program.publicKey === MPL_AGENT_VALIDATION_PROGRAM_ID);
});
