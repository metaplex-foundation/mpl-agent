export * from './api';
export * from './plugin';

// Full namespace exports (includes everything: types, errors, shared helpers)
export * as identity from './generated/identity';
export * as reputation from './generated/reputation';
export * as validation from './generated/validation';
export * as tools from './generated/tools';

// Flat re-exports for non-conflicting names (accounts, instructions, programs)
export * from './generated/identity/accounts';
export * from './generated/identity/instructions';
export * from './generated/identity/programs';

export * from './generated/reputation/accounts';
export * from './generated/reputation/instructions';
export * from './generated/reputation/programs';

export * from './generated/validation/accounts';
export * from './generated/validation/instructions';
export * from './generated/validation/programs';

export * from './generated/tools/accounts';
export * from './generated/tools/instructions';
export * from './generated/tools/programs';
