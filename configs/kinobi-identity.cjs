const path = require("path");
const k = require("@metaplex-foundation/kinobi");

// Paths.
const clientDir = path.join(__dirname, "..", "clients");
const idlDir = path.join(__dirname, "..", "idls");

// Instantiate Kinobi.
const kinobi = k.createFromIdls([
  path.join(idlDir, "mpl_agent_identity.json"),
]);

// Update programs.
kinobi.update(
  new k.updateProgramsVisitor({
    mplAgentIdentityProgram: { name: "mplAgentIdentity" },
  })
);

// Update accounts.
kinobi.update(
  new k.updateAccountsVisitor({
    agentIdentityV1: {
      seeds: [
        k.constantPdaSeedNodeFromString("agent_identity"),
        k.variablePdaSeedNode("asset", k.publicKeyTypeNode(), "The address of the asset"),
      ],
    },
    collectionIdentityConfigV1: {
      seeds: [
        k.constantPdaSeedNodeFromString("collection_identity_config"),
        k.variablePdaSeedNode("collection", k.publicKeyTypeNode(), "The address of the collection"),
      ],
    },
  })
);

// Update instructions.
kinobi.update(
  new k.updateInstructionsVisitor({
    registerIdentityV1: {
      accounts: {
        agentIdentity: {defaultValue: k.pdaValueNode("agentIdentityV1")},
        collectionIdentityConfig: {defaultValue: k.pdaValueNode("collectionIdentityConfigV1")},
      },
    },
  })
);

// // Set ShankAccount discriminator.
// const key = (name) => ({ field: "key", value: k.enumValueNode("Key", name) });
// kinobi.update(
//   new k.setAccountDiscriminatorFromFieldVisitor({
//     agentIdentityV1: key("AgentIdentityV1"),
//     collectionIdentityConfigV1: key("CollectionIdentityConfigV1"),
//   })
// );

// Render JavaScript.
const jsDir = path.join(clientDir, "js", "src", "generated", "identity");
const prettier = require(path.join(clientDir, "js", ".prettierrc.json"));
kinobi.accept(new k.renderJavaScriptVisitor(jsDir, { prettier }));

// Render Rust.
const crateDir = path.join(clientDir, "rust");
const rustDir = path.join(clientDir, "rust", "src", "generated", "identity");
kinobi.accept(
  new k.renderRustVisitor(rustDir, {
    formatCode: true,
    crateFolder: crateDir,
  })
);
