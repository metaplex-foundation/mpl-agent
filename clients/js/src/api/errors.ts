// ─── API Error ───────────────────────────────────────────────────────────────

export interface AgentApiError extends Error {
  readonly name: 'AgentApiError';
  readonly statusCode: number;
  readonly responseBody: unknown;
}

export function agentApiError(
  message: string,
  statusCode: number,
  responseBody: unknown
): AgentApiError {
  const error = new Error(message) as AgentApiError;
  (error as { name: string }).name = 'AgentApiError';
  (error as { statusCode: number }).statusCode = statusCode;
  (error as { responseBody: unknown }).responseBody = responseBody;
  return error;
}

export function isAgentApiError(err: unknown): err is AgentApiError {
  return err instanceof Error && err.name === 'AgentApiError';
}

// ─── Network Error ──────────────────────────────────────────────────────────

export interface AgentApiNetworkError extends Error {
  readonly name: 'AgentApiNetworkError';
  readonly cause: Error;
}

export function agentApiNetworkError(
  message: string,
  cause: Error
): AgentApiNetworkError {
  const error = new Error(message) as AgentApiNetworkError;
  (error as { name: string }).name = 'AgentApiNetworkError';
  (error as { cause: Error }).cause = cause;
  return error;
}

export function isAgentApiNetworkError(
  err: unknown
): err is AgentApiNetworkError {
  return err instanceof Error && err.name === 'AgentApiNetworkError';
}

// ─── Validation Error ───────────────────────────────────────────────────────

export interface AgentValidationError extends Error {
  readonly name: 'AgentValidationError';
  readonly field: string;
}

export function agentValidationError(
  message: string,
  field: string
): AgentValidationError {
  const error = new Error(message) as AgentValidationError;
  (error as { name: string }).name = 'AgentValidationError';
  (error as { field: string }).field = field;
  return error;
}

export function isAgentValidationError(
  err: unknown
): err is AgentValidationError {
  return err instanceof Error && err.name === 'AgentValidationError';
}
