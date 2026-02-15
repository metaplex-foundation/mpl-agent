import { UmiPlugin } from '@metaplex-foundation/umi';
import { createMpl8004IdentityProgram } from './generated';

export const mpl8004Identity = (): UmiPlugin => ({
  install(umi) {
    umi.programs.add(createMpl8004IdentityProgram(), false);
  },
});
