export * from './api';
export * from './plugin';

// Convenience aliases
export { MPL_AGENT_IDENTITY_PROGRAM_ID as IDENTITY_ID } from './generated/identity';
export { MPL_AGENT_REPUTATION_PROGRAM_ID as REPUTATION_ID } from './generated/reputation';
export { MPL_AGENT_VALIDATION_PROGRAM_ID as VALIDATION_ID } from './generated/validation';
export { MPL_AGENT_TOOLS_PROGRAM_ID as TOOLS_ID } from './generated/tools';

// Identity
export * from './generated/identity/accounts';
export * from './generated/identity/instructions';
export * from './generated/identity/programs';
export {
  Key as IdentityKey,
  KeyArgs as IdentityKeyArgs,
  getKeySerializer as getIdentityKeySerializer,
} from './generated/identity/types';
export {
  InvalidSystemProgramError as IdentityInvalidSystemProgramError,
  InvalidInstructionDataError as IdentityInvalidInstructionDataError,
  InvalidAccountDataError as IdentityInvalidAccountDataError,
  InvalidMplCoreProgramError as IdentityInvalidMplCoreProgramError,
  InvalidCoreAssetError as IdentityInvalidCoreAssetError,
  InvalidAgentTokenError,
  OnlyAssetSignerCanSetAgentTokenError,
  AgentTokenAlreadySetError,
  InvalidAgentIdentityError,
  AgentIdentityAlreadyRegisteredError,
  InvalidGenesisAccountError,
  GenesisNotMintFundedError,
  getMplAgentIdentityErrorFromCode,
  getMplAgentIdentityErrorFromName,
} from './generated/identity/errors';

// Reputation
export * from './generated/reputation/accounts';
export * from './generated/reputation/instructions';
export * from './generated/reputation/programs';
export {
  Key as ReputationKey,
  KeyArgs as ReputationKeyArgs,
  getKeySerializer as getReputationKeySerializer,
} from './generated/reputation/types';
export {
  getMplAgentReputationErrorFromCode,
  getMplAgentReputationErrorFromName,
} from './generated/reputation/errors';

// Validation
export * from './generated/validation/accounts';
export * from './generated/validation/instructions';
export * from './generated/validation/programs';
export {
  Key as ValidationKey,
  KeyArgs as ValidationKeyArgs,
  getKeySerializer as getValidationKeySerializer,
} from './generated/validation/types';
export {
  getMplAgentValidationErrorFromCode,
  getMplAgentValidationErrorFromName,
} from './generated/validation/errors';

// Tools
export * from './generated/tools/accounts';
export * from './generated/tools/instructions';
export * from './generated/tools/programs';
export {
  Key as ToolsKey,
  KeyArgs as ToolsKeyArgs,
  getKeySerializer as getToolsKeySerializer,
} from './generated/tools/types';
export {
  ExecutiveProfileMustBeUninitializedError,
  InvalidExecutionDelegateRecordDerivationError,
  ExecutionDelegateRecordMustBeUninitializedError,
  InvalidAgentIdentityError as ToolsInvalidAgentIdentityError,
  AgentIdentityNotRegisteredError,
  AssetOwnerMustBeTheOneToDelegateExecutionError,
  InvalidExecutiveProfileDerivationError,
  ExecutionDelegateRecordMustBeInitializedError,
  UnauthorizedRevokeError,
  getMplAgentToolsErrorFromCode,
  getMplAgentToolsErrorFromName,
} from './generated/tools/errors';
