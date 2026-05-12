const path = require("path");
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
  })
);

// Update accounts.
kinobi.update(
  new k.updateAccountsVisitor({
    agentReputationV1: {
      seeds: [
        k.constantPdaSeedNodeFromString("agent_reputation"),
        k.variablePdaSeedNode("asset", k.publicKeyTypeNode(), "The address of the asset"),
      ],
    },
    reviewRecordV1: {
      seeds: [
        k.constantPdaSeedNodeFromString("review_record"),
        k.variablePdaSeedNode("receiptAssetId", k.publicKeyTypeNode(), "Bubblegum asset id of the work receipt"),
      ],
    },
    reviewSubsidyPoolV1: {
      seeds: [
        k.constantPdaSeedNodeFromString("review_subsidy_pool"),
        k.variablePdaSeedNode("agentAsset", k.publicKeyTypeNode(), "The agent's Core asset"),
      ],
    },
    reviewsConfigV1: {
      seeds: [k.constantPdaSeedNodeFromString("program_config")],
    },
  })
);

// Well-known program IDs we want to default in the generated client.
const MPL_CORE_ID = "CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d";
const BUBBLEGUM_ID = "BGUMAp9Gq7iTEuizy4pqaxsTyUCBK68MDfK752saRPUY";
const MPL_NOOP_ID = "mnoopTCrg4p8ry25e4bcWA9XZjbNjMTfgYVGGEdRsf3";
const COMPRESSION_ID = "mcmt6YrQEMKw8Mw43FmpRLmf7BqRnFMKmAcbxE3xkAW";

// Update instructions.
kinobi.update(
  new k.updateInstructionsVisitor({
    registerReputationV1: {
      accounts: {
        agentReputation: { defaultValue: k.pdaValueNode("agentReputationV1") },
      },
    },
    leaveReviewV1: {
      accounts: {
        programConfig: { defaultValue: k.pdaValueNode("reviewsConfigV1") },
        // Auto-derive subsidy pool from the agent being reviewed.
        subsidyPool: {
          defaultValue: k.pdaValueNode("reviewSubsidyPoolV1", [
            k.pdaSeedValueNode("agentAsset", k.accountValueNode("asset")),
          ]),
        },
        mplCoreProgram: { defaultValue: k.publicKeyValueNode(MPL_CORE_ID, "mplCore") },
        bubblegumProgram: { defaultValue: k.publicKeyValueNode(BUBBLEGUM_ID, "mplBubblegum") },
        logWrapper: { defaultValue: k.publicKeyValueNode(MPL_NOOP_ID, "mplNoop") },
        compressionProgram: { defaultValue: k.publicKeyValueNode(COMPRESSION_ID, "mplAccountCompression") },
      },
    },
    depositSubsidyV1: {
      accounts: {
        subsidyPool: { defaultValue: k.pdaValueNode("reviewSubsidyPoolV1") },
      },
    },
    withdrawSubsidyV1: {
      accounts: {
        subsidyPool: { defaultValue: k.pdaValueNode("reviewSubsidyPoolV1") },
      },
    },
    initializeReviewsConfigV1: {
      accounts: {
        programConfig: { defaultValue: k.pdaValueNode("reviewsConfigV1") },
        mplCoreProgram: { defaultValue: k.publicKeyValueNode(MPL_CORE_ID, "mplCore") },
      },
    },
    registerReviewsTreeV1: {
      accounts: {
        programConfig: { defaultValue: k.pdaValueNode("reviewsConfigV1") },
        bubblegumProgram: { defaultValue: k.publicKeyValueNode(BUBBLEGUM_ID, "mplBubblegum") },
        logWrapper: { defaultValue: k.publicKeyValueNode(MPL_NOOP_ID, "mplNoop") },
        compressionProgram: { defaultValue: k.publicKeyValueNode(COMPRESSION_ID, "mplAccountCompression") },
      },
    },
  })
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
  })
);
