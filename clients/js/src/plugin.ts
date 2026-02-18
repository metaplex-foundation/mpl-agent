import { UmiPlugin } from '@metaplex-foundation/umi';
import { createMplAgentIdentityProgram } from './generated/identity';
import { createMplAgentReputationProgram } from './generated/reputation';
import { createMplAgentValidationProgram } from './generated/validation';

export const mplAgentIdentity = (): UmiPlugin => ({
  install(umi) {
    umi.programs.add(createMplAgentIdentityProgram(), false);
  },
});

export const mplAgentReputation = (): UmiPlugin => ({
  install(umi) {
    umi.programs.add(createMplAgentReputationProgram(), false);
  },
});

export const mplAgentValidation = (): UmiPlugin => ({
  install(umi) {
    umi.programs.add(createMplAgentValidationProgram(), false);
  },
});
