const path = require("path");
const k = require("@metaplex-foundation/kinobi");

// Paths.
const clientDir = path.join(__dirname, "..", "clients");
const idlDir = path.join(__dirname, "..", "idls");

// Instantiate Kinobi.
const kinobi = k.createFromIdls([path.join(idlDir, "mpl_8004_identity.json")]);

// Update programs.
kinobi.update(
  new k.updateProgramsVisitor({
    mpl8004IdentityProgram: { name: "mpl8004Identity" },
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
    collectionConfigV1: {
      seeds: [
        k.constantPdaSeedNodeFromString("collection_config"),
        k.variablePdaSeedNode("collection", k.publicKeyTypeNode(), "The address of the collection"),
      ],
    },
  })
);

// Update instructions.
kinobi.update(
  new k.updateInstructionsVisitor({
    registerV1: {
      accounts: {
        agentIdentity: {defaultValue: k.pdaValueNode("agentIdentityV1")},
        collectionConfig: {defaultValue: k.pdaValueNode("collectionConfigV1")},
      },
    },
  })
);

// Set ShankAccount discriminator.
const key = (name) => ({ field: "key", value: k.enumValueNode("Key", name) });
kinobi.update(
  new k.setAccountDiscriminatorFromFieldVisitor({
    agentIdentityV1: key("AgentIdentityV1"),
    collectionConfigV1: key("CollectionConfigV1"),
  })
);

// Render JavaScript.
const jsDir = path.join(clientDir, "js", "src", "generated");
const prettier = require(path.join(clientDir, "js", ".prettierrc.json"));
kinobi.accept(new k.renderJavaScriptVisitor(jsDir, { prettier }));

// Render Rust.
const crateDir = path.join(clientDir, "rust");
const rustDir = path.join(clientDir, "rust", "src", "generated");
kinobi.accept(
  new k.renderRustVisitor(rustDir, {
    formatCode: true,
    crateFolder: crateDir,
  })
);
