const path = require("path");
const fs = require("fs");
const k = require("@metaplex-foundation/kinobi");

// Paths.
const clientDir = path.join(__dirname, "..", "clients");
const idlDir = path.join(__dirname, "..", "idls");

// Instantiate Kinobi.
const kinobi = k.createFromIdls([path.join(idlDir, "mpl_agent_tools.json")]);

// Update programs.
kinobi.update(
    new k.updateProgramsVisitor({
        mplAgentToolsProgram: { name: "mplAgentTools" },
    }),
);

// Update accounts.
kinobi.update(
    new k.updateAccountsVisitor({
        executiveProfileV1: {
            seeds: [
                k.constantPdaSeedNodeFromString("executive_profile"),
                k.variablePdaSeedNode(
                    "authority",
                    k.publicKeyTypeNode(),
                    "The address of the authority",
                ),
            ],
        },
        executionDelegateRecordV1: {
            seeds: [
                k.constantPdaSeedNodeFromString("execution_delegate_record"),
                k.variablePdaSeedNode(
                    "executiveProfile",
                    k.publicKeyTypeNode(),
                    "The address of the executive profile",
                ),
                k.variablePdaSeedNode(
                    "agentAsset",
                    k.publicKeyTypeNode(),
                    "The address of the agent asset",
                ),
            ],
        },
    }),
);

// Stateless PDAs (not backed by an account struct — declared via addPdasVisitor
// so we can use them as defaults on instruction accounts).
kinobi.update(
    new k.addPdasVisitor({
        mplAgentTools: [
            k.pdaNode("receiptsCollection", [
                k.constantPdaSeedNodeFromString("receipts_collection"),
            ]),
            k.pdaNode("receiptsAuthority", [
                k.constantPdaSeedNodeFromString("receipts_authority"),
            ]),
            k.pdaNode("receiptsTree", [
                k.constantPdaSeedNodeFromString("receipts_tree"),
                k.variablePdaSeedNode(
                    "treeIndex",
                    k.numberTypeNode("u64"),
                    "The receipts tree index",
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
        registerExecutiveV1: {
            accounts: {
                executiveProfile: {
                    defaultValue: k.conditionalValueNode({
                        condition: k.accountValueNode("authority"),
                        ifTrue: k.pdaValueNode("executiveProfileV1", [
                            k.pdaSeedValueNode(
                                "authority",
                                k.accountValueNode("authority"),
                            ),
                        ]),
                        ifFalse: k.pdaValueNode("executiveProfileV1", [
                            k.pdaSeedValueNode(
                                "authority",
                                k.accountValueNode("payer"),
                            ),
                        ]),
                    }),
                },
            },
        },
        delegateExecutionV1: {
            accounts: {
                executionDelegateRecord: {
                    defaultValue: k.pdaValueNode("executionDelegateRecordV1"),
                },
            },
        },
        revokeExecutionV1: {},
        mintWorkReceiptV1: {
            accounts: {
                authority: {
                    defaultValue: k.pdaValueNode("receiptsAuthority"),
                },
                coreCollection: {
                    defaultValue: k.pdaValueNode("receiptsCollection"),
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
        createReceiptsCollectionV1: {
            accounts: {
                collection: {
                    defaultValue: k.pdaValueNode("receiptsCollection"),
                },
                authority: {
                    defaultValue: k.pdaValueNode("receiptsAuthority"),
                },
                mplCoreProgram: {
                    defaultValue: k.publicKeyValueNode(MPL_CORE_ID, "mplCore"),
                },
            },
        },
        registerReceiptsTreeV1: {
            accounts: {
                authority: {
                    defaultValue: k.pdaValueNode("receiptsAuthority"),
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
const jsDir = path.join(clientDir, "js", "src", "generated", "tools");
const prettier = require(path.join(clientDir, "js", ".prettierrc.json"));
kinobi.accept(new k.renderJavaScriptVisitor(jsDir, { prettier }));

// Render Rust.
const crateDir = path.join(clientDir, "rust-tools");
const rustDir = path.join(clientDir, "rust-tools", "src", "generated");
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
 * trees). Emitted by the kinobi-tools config because kinobi 1.0-alpha
 * doesn't render find*Pda helpers for PDAs added via addPdasVisitor.
 */

import { Context, Pda } from '@metaplex-foundation/umi';
import { string, u64 } from '@metaplex-foundation/umi/serializers';

const PROGRAM_ID = 'TLREGni9ZEyGC3vnPZtqUh95xQ8oPqJSvNjvB7FGK8S';

function pda(
  context: Pick<Context, 'eddsa' | 'programs'>,
  seeds: Uint8Array[]
): Pda {
  const programId = context.programs.getPublicKey('mplAgentTools', PROGRAM_ID);
  return context.eddsa.findPda(programId, seeds);
}

export function findReceiptsCollectionPda(
  context: Pick<Context, 'eddsa' | 'programs'>
): Pda {
  return pda(context, [
    string({ size: 'variable' }).serialize('receipts_collection'),
  ]);
}

export function findReceiptsAuthorityPda(
  context: Pick<Context, 'eddsa' | 'programs'>
): Pda {
  return pda(context, [
    string({ size: 'variable' }).serialize('receipts_authority'),
  ]);
}

export function findReceiptsTreePda(
  context: Pick<Context, 'eddsa' | 'programs'>,
  seeds: { treeIndex: number | bigint }
): Pda {
  return pda(context, [
    string({ size: 'variable' }).serialize('receipts_tree'),
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
