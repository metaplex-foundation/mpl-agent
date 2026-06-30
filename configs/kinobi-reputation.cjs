const path = require("path");
const fs = require("fs");
const k = require("@metaplex-foundation/kinobi");

// Paths.
const clientDir = path.join(__dirname, "..", "clients");
const idlDir = path.join(__dirname, "..", "idls");

// Instantiate Kinobi.
const kinobi = k.createFromIdls([
    path.join(idlDir, "mpl_agent_reputation.json"),
]);

// Update programs.
kinobi.update(
    new k.updateProgramsVisitor({
        mplAgentReputationProgram: { name: "mplAgentReputation" },
    }),
);

// Update accounts.
kinobi.update(
    new k.updateAccountsVisitor({
        reviewRecordV1: {
            seeds: [
                k.constantPdaSeedNodeFromString("review_record"),
                k.variablePdaSeedNode(
                    "receiptAssetId",
                    k.publicKeyTypeNode(),
                    "Bubblegum asset id of the work receipt",
                ),
            ],
        },
    }),
);

// Stateless PDAs (not backed by an account struct — declared via addPdasVisitor
// so we can use them as defaults on instruction accounts).
kinobi.update(
    new k.addPdasVisitor({
        mplAgentReputation: [
            k.pdaNode("reviewsCollection", [
                k.constantPdaSeedNodeFromString("reviews_collection"),
            ]),
            k.pdaNode("reviewsAuthority", [
                k.constantPdaSeedNodeFromString("reviews_authority"),
            ]),
            k.pdaNode("reviewsTree", [
                k.constantPdaSeedNodeFromString("reviews_tree"),
                k.variablePdaSeedNode(
                    "treeIndex",
                    k.numberTypeNode("u64"),
                    "The reviews tree index",
                ),
            ]),
        ],
    }),
);

// Well-known program IDs we want to default in the generated client.
const MPL_CORE_ID = "CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d";
const BUBBLEGUM_ID = "BGUMAp9Gq7iTEuizy4pqaxsTyUCBK68MDfK752saRPUY";
const MPL_NOOP_ID = "mnoopTCrg4p8ry25e4bcWA9XZjbNjMTfgYVGGEdRsf3";
const COMPRESSION_ID = "mcmt6YrQEMKw8Mw43FmpRLmf7BqRnFMKmAcbxE3xkAW";

// Update instructions.
kinobi.update(
    new k.updateInstructionsVisitor({
        leaveReviewV1: {
            accounts: {
                authority: {
                    defaultValue: k.pdaValueNode("reviewsAuthority"),
                },
                coreCollection: {
                    defaultValue: k.pdaValueNode("reviewsCollection"),
                },
                mplCoreProgram: {
                    defaultValue: k.publicKeyValueNode(MPL_CORE_ID, "mplCore"),
                },
                bubblegumProgram: {
                    defaultValue: k.publicKeyValueNode(
                        BUBBLEGUM_ID,
                        "mplBubblegum",
                    ),
                },
                logWrapper: {
                    defaultValue: k.publicKeyValueNode(MPL_NOOP_ID, "mplNoop"),
                },
                compressionProgram: {
                    defaultValue: k.publicKeyValueNode(
                        COMPRESSION_ID,
                        "mplAccountCompression",
                    ),
                },
            },
        },
        createReviewsCollectionV1: {
            accounts: {
                collection: {
                    defaultValue: k.pdaValueNode("reviewsCollection"),
                },
                authority: {
                    defaultValue: k.pdaValueNode("reviewsAuthority"),
                },
                mplCoreProgram: {
                    defaultValue: k.publicKeyValueNode(MPL_CORE_ID, "mplCore"),
                },
            },
        },
        registerReviewsTreeV1: {
            accounts: {
                authority: {
                    defaultValue: k.pdaValueNode("reviewsAuthority"),
                },
                bubblegumProgram: {
                    defaultValue: k.publicKeyValueNode(
                        BUBBLEGUM_ID,
                        "mplBubblegum",
                    ),
                },
                logWrapper: {
                    defaultValue: k.publicKeyValueNode(MPL_NOOP_ID, "mplNoop"),
                },
                compressionProgram: {
                    defaultValue: k.publicKeyValueNode(
                        COMPRESSION_ID,
                        "mplAccountCompression",
                    ),
                },
            },
        },
    }),
);

// Render JavaScript.
const jsDir = path.join(clientDir, "js", "src", "generated", "reputation");
const prettier = require(path.join(clientDir, "js", ".prettierrc.json"));
kinobi.accept(new k.renderJavaScriptVisitor(jsDir, { prettier }));

// Render Rust.
const crateDir = path.join(clientDir, "rust-reputation");
const rustDir = path.join(clientDir, "rust-reputation", "src", "generated");
kinobi.accept(
    new k.renderRustVisitor(rustDir, {
        formatCode: true,
        crateFolder: crateDir,
    }),
);

// Post-process: write standalone PDA helpers expected by kinobi-emitted
// instruction code (kinobi 1.0-alpha doesn't render find*Pda helpers for
// PDAs added via addPdasVisitor, but it still emits references to them).
// The source below is pre-formatted to match the repo's prettier config
// (2-space indent, single quotes, trailing commas) so `pnpm generate` is
// idempotent and CI's "working directory is clean" check passes.
const pdaHelperFile = path.join(jsDir, "accounts", "standalonePdas.ts");
fs.writeFileSync(
    pdaHelperFile,
    `/**
 * Hand-written PDA helpers for standalone PDAs (collections, authority,
 * trees). Emitted by the kinobi-reputation config because kinobi 1.0-alpha
 * doesn't render find*Pda helpers for PDAs added via addPdasVisitor.
 */

import { Context, Pda } from '@metaplex-foundation/umi';
import { string, u64 } from '@metaplex-foundation/umi/serializers';

const PROGRAM_ID = 'REPREG5c1gPHuHukEyANpksLdHFaJCiTrm6zJgNhRZR';

function pda(
  context: Pick<Context, 'eddsa' | 'programs'>,
  seeds: Uint8Array[]
): Pda {
  const programId = context.programs.getPublicKey(
    'mplAgentReputation',
    PROGRAM_ID
  );
  return context.eddsa.findPda(programId, seeds);
}

export function findReviewsCollectionPda(
  context: Pick<Context, 'eddsa' | 'programs'>
): Pda {
  return pda(context, [
    string({ size: 'variable' }).serialize('reviews_collection'),
  ]);
}

export function findReviewsAuthorityPda(
  context: Pick<Context, 'eddsa' | 'programs'>
): Pda {
  return pda(context, [
    string({ size: 'variable' }).serialize('reviews_authority'),
  ]);
}

export function findReviewsTreePda(
  context: Pick<Context, 'eddsa' | 'programs'>,
  seeds: { treeIndex: number | bigint }
): Pda {
  return pda(context, [
    string({ size: 'variable' }).serialize('reviews_tree'),
    u64().serialize(seeds.treeIndex),
  ]);
}
`
);

// Patch the accounts/index.ts to re-export from standalonePdas.
const accountsIndex = path.join(jsDir, "accounts", "index.ts");
let indexContent = fs.readFileSync(accountsIndex, "utf-8");
if (!indexContent.includes("standalonePdas")) {
    indexContent += "export * from './standalonePdas';\n";
    fs.writeFileSync(accountsIndex, indexContent);
}
