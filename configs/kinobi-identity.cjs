const fs = require("fs");
const path = require("path");
const { execFileSync } = require("child_process");
const k = require("@metaplex-foundation/kinobi");

// Paths.
const clientDir = path.join(__dirname, "..", "clients");
const idlDir = path.join(__dirname, "..", "idls");

// Instantiate Kinobi.
const kinobi = k.createFromIdls([path.join(idlDir, "mpl_agent_identity.json")]);

// Update programs.
kinobi.update(
    new k.updateProgramsVisitor({
        mplAgentIdentityProgram: { name: "mplAgentIdentity" },
    }),
);

// Update accounts.
kinobi.update(
    new k.updateAccountsVisitor({
        agentIdentityV1: {
            seeds: [
                k.constantPdaSeedNodeFromString("agent_identity"),
                k.variablePdaSeedNode(
                    "asset",
                    k.publicKeyTypeNode(),
                    "The address of the asset",
                ),
            ],
        },
        agentIdentityV2: {
            seeds: [
                k.constantPdaSeedNodeFromString("agent_identity"),
                k.variablePdaSeedNode(
                    "asset",
                    k.publicKeyTypeNode(),
                    "The address of the asset",
                ),
            ],
        },
    }),
);

// Update instructions.
kinobi.update(
    new k.updateInstructionsVisitor({
        registerIdentityV1: {
            accounts: {
                agentIdentity: {
                    defaultValue: k.pdaValueNode("agentIdentityV2"),
                },
            },
        },
        setAgentTokenV1: {
            accounts: {
                agentIdentity: {
                    defaultValue: k.pdaValueNode("agentIdentityV2"),
                },
            },
        },
    }),
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
    }),
);

// Kinobi (through 1.0.0-alpha.3) emits `borsh::maybestd::io` for fixed-size
// option serializers and other Borsh impls, but this workspace uses borsh 1.x
// which exposes `borsh::io`. Rewrite any generated account files that reference
// the old path. This can be removed once kinobi no longer emits `maybestd`.
const accountsDir = path.join(rustDir, "accounts");
if (fs.existsSync(accountsDir)) {
    for (const entry of fs.readdirSync(accountsDir)) {
        if (!entry.endsWith(".rs")) continue;
        const filePath = path.join(accountsDir, entry);
        const original = fs.readFileSync(filePath, "utf8");
        const patched = original.replace(/borsh::maybestd::io/g, "borsh::io");
        if (patched !== original) {
            fs.writeFileSync(filePath, patched);
            execFileSync("rustfmt", ["--edition", "2021", filePath], {
                stdio: "inherit",
            });
        }
    }
}
