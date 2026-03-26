// Client functions
export { mintAgent, mintAndSubmitAgent } from './client';

// Transaction helper
export { signAndSendAgentTransaction } from './transactionHelper';

// Error interfaces, factories, and type guards
export type {
  AgentApiError,
  AgentApiNetworkError,
  AgentValidationError,
} from './errors';
export {
  agentApiError,
  agentApiNetworkError,
  agentValidationError,
  isAgentApiError,
  isAgentApiNetworkError,
  isAgentValidationError,
} from './errors';

// Types
export type {
  AgentApiConfig,
  AgentMetadata,
  AgentRegistration,
  AgentService,
  MintAgentInput,
  MintAgentResponse,
  MintAndSubmitAgentResult,
  SignAndSendOptions,
  SvmNetwork,
} from './types';
