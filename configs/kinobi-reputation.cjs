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
    collectionReputationConfigV1: {
      seeds: [
        k.constantPdaSeedNodeFromString("collection_reputation_config"),
        k.variablePdaSeedNode("collection", k.publicKeyTypeNode(), "The address of the collection"),
      ],
    },
  })
);

// Update instructions.
kinobi.update(
  new k.updateInstructionsVisitor({
    registerReputationV1: {
      accounts: {
        agentReputation: {defaultValue: k.pdaValueNode("agentReputationV1")},
        collectionReputationConfig: {defaultValue: k.pdaValueNode("collectionReputationConfigV1")},
      },
    },
  })
);

// Render JavaScript.
const jsDir = path.join(clientDir, "js", "src", "generated", "reputation");
const prettier = require(path.join(clientDir, "js", ".prettierrc.json"));
kinobi.accept(new k.renderJavaScriptVisitor(jsDir, { prettier }));

// Render Rust.
const crateDir = path.join(clientDir, "rust");
const rustDir = path.join(clientDir, "rust", "src", "generated", "reputation");
kinobi.accept(
  new k.renderRustVisitor(rustDir, {
    formatCode: true,
    crateFolder: crateDir,
  })
);
