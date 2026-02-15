const path = require("path");
const { generateIdl } = require("@metaplex-foundation/shank-js");

const idlDir = path.join(__dirname, "..", "idls");
const binaryInstallDir = path.join(__dirname, "..", ".crates");
const programDir = path.join(__dirname, "..", "programs");

generateIdl({
  generator: "shank",
  programName: "mpl_8004_identity_program",
  programId: "8oo41DdXLnERYxrjU26Byuh3kii6YQY6eqVZUae1Tndk",
  idlDir,
  idlName: "mpl_8004_identity",
  binaryInstallDir,
  programDir: path.join(programDir, "mpl-8004-identity"),
});
