const path = require("path");
const k = require("@metaplex-foundation/kinobi");

// Paths.
const clientDir = path.join(__dirname, "..", "clients");
const idlDir = path.join(__dirname, "..", "idls");

// Instantiate Kinobi.
const kinobi = k.createFromIdls([
  path.join(idlDir, "mpl_agent_validation.json"),
]);

// Update programs.
kinobi.update(
  new k.updateProgramsVisitor({
    mplAgentValidationProgram: { name: "mplAgentValidation" },
  })
);

// Update accounts.
kinobi.update(
  new k.updateAccountsVisitor({
    agentValidationV1: {
      seeds: [
        k.constantPdaSeedNodeFromString("agent_validation"),
        k.variablePdaSeedNode("asset", k.publicKeyTypeNode(), "The address of the asset"),
      ],
    },
    collectionValidationConfigV1: {
      seeds: [
        k.constantPdaSeedNodeFromString("collection_validation_config"),
        k.variablePdaSeedNode("collection", k.publicKeyTypeNode(), "The address of the collection"),
      ],
    },
  })
);

// Update instructions.
kinobi.update(
  new k.updateInstructionsVisitor({
    registerValidationV1: {
      accounts: {
        agentValidation: {defaultValue: k.pdaValueNode("agentValidationV1")},
        collectionValidationConfig: {defaultValue: k.pdaValueNode("collectionValidationConfigV1")},
      },
    },
  })
);

// Render JavaScript.
const jsDir = path.join(clientDir, "js", "src", "generated", "validation");
const prettier = require(path.join(clientDir, "js", ".prettierrc.json"));
kinobi.accept(new k.renderJavaScriptVisitor(jsDir, { prettier }));

// Render Rust.
const crateDir = path.join(clientDir, "rust");
const rustDir = path.join(clientDir, "rust", "src", "generated", "validation");
kinobi.accept(
  new k.renderRustVisitor(rustDir, {
    formatCode: true,
    crateFolder: crateDir,
  })
);
