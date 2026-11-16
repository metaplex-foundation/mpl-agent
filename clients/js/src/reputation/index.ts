/**
 * Constants and helpers that complement the auto-generated reputation client.
 */

import { PublicKey } from '@metaplex-foundation/umi';

/** Maximum length, in bytes, of the feedback URI accepted by the program. */
export const MAX_FEEDBACK_URI_LEN = 200;

// Useful constants for callers wiring up the LeaveReviewV1 instruction.
export const MPL_BUBBLEGUM_PROGRAM_ID =
  'BGUMAp9Gq7iTEuizy4pqaxsTyUCBK68MDfK752saRPUY' as PublicKey;
export const MPL_NOOP_PROGRAM_ID =
  'mnoopTCrg4p8ry25e4bcWA9XZjbNjMTfgYVGGEdRsf3' as PublicKey;
export const MPL_ACCOUNT_COMPRESSION_PROGRAM_ID =
  'mcmt6YrQEMKw8Mw43FmpRLmf7BqRnFMKmAcbxE3xkAW' as PublicKey;
