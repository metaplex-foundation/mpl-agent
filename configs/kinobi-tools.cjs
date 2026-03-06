const path = require("path");
const k = require("@metaplex-foundation/kinobi");

// Paths.
const clientDir = path.join(__dirname, "..", "clients");
const idlDir = path.join(__dirname, "..", "idls");

// Instantiate Kinobi.
const kinobi = k.createFromIdls([
  path.join(idlDir, "mpl_agent_tools.json"),
]);

// Update programs.
kinobi.update(
  new k.updateProgramsVisitor({
    mplAgentToolsProgram: { name: "mplAgentTools" },
  })
);

// Update accounts.
kinobi.update(
  new k.updateAccountsVisitor({
    executionDelegateRecordV1: {
      seeds: [
        k.constantPdaSeedNodeFromString("execution_delegate_record"),
        k.variablePdaSeedNode("executorProfile", k.publicKeyTypeNode(), "The address of the executor profile"),
        k.variablePdaSeedNode("agentAsset", k.publicKeyTypeNode(), "The address of the agent asset"),
      ],
    },
  })
);

// Update instructions.
kinobi.update(
  new k.updateInstructionsVisitor({
    delegateExecutionV1: {
      accounts: {
        executionDelegateRecord: {defaultValue: k.pdaValueNode("executionDelegateRecordV1")},
      },
    },
  })
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
  })
);
