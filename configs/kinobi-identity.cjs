const fs = require("fs");
const path = require("path");
const { execFileSync } = require("child_process");
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
    agentIdentityV2: {
      seeds: [
        k.constantPdaSeedNodeFromString("agent_identity"),
        k.variablePdaSeedNode("asset", k.publicKeyTypeNode(), "The address of the asset"),
      ],
    },
  })
);

// Update instructions.
kinobi.update(
  new k.updateInstructionsVisitor({
    registerIdentityV1: {
      accounts: {
        agentIdentity: {defaultValue: k.pdaValueNode("agentIdentityV2")},
      },
    },
    setAgentTokenV1: {
      accounts: {
        agentIdentity: {defaultValue: k.pdaValueNode("agentIdentityV2")},
      },
    },
  })
);

// Render JavaScript.
const jsDir = path.join(clientDir, "js", "src", "generated", "identity");
const prettier = require(path.join(clientDir, "js", ".prettierrc.json"));
kinobi.accept(new k.renderJavaScriptVisitor(jsDir, { prettier }));

// Render Rust.
const crateDir = path.join(clientDir, "rust-identity");
const rustDir = path.join(clientDir, "rust-identity", "src", "generated");
kinobi.accept(
  new k.renderRustVisitor(rustDir, {
    formatCode: true,
    crateFolder: crateDir,
  })
);

// Kinobi currently emits `borsh::maybestd::io` for fixed-size option serializers,
// but this workspace uses borsh versions that expose `borsh::io`.
const agentIdentityV2Path = path.join(rustDir, "accounts", "agent_identity_v2.rs");
const agentIdentityV2Content = fs.readFileSync(agentIdentityV2Path, "utf8");
const normalizedAgentIdentityV2Content = agentIdentityV2Content.replace(
  /borsh::maybestd::io/g,
  "borsh::io"
);
if (normalizedAgentIdentityV2Content !== agentIdentityV2Content) {
  fs.writeFileSync(agentIdentityV2Path, normalizedAgentIdentityV2Content);
  execFileSync("rustfmt", ["--edition", "2021", agentIdentityV2Path], {
    stdio: "inherit",
  });
}
