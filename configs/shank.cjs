const path = require("path");
const { generateIdl } = require("@metaplex-foundation/shank-js");

const idlDir = path.join(__dirname, "..", "idls");
const binaryInstallDir = path.join(__dirname, "..", ".crates");
const programDir = path.join(__dirname, "..", "programs");

generateIdl({
  generator: "shank",
  programName: "mpl_agent_identity_program",
  programId: "1DREGFgysWYxLnRnKQnwrxnJQeSMk2HmGaC6whw2B2p",
  idlDir,
  idlName: "mpl_agent_identity",
  binaryInstallDir,
  programDir: path.join(programDir, "mpl-agent-identity"),
});

generateIdl({
  generator: "shank",
  programName: "mpl_agent_reputation_program",
  programId: "REPREG5c1gPHuHukEyANpksLdHFaJCiTrm6zJgNhRZR",
  idlDir,
  idlName: "mpl_agent_reputation",
  binaryInstallDir,
  programDir: path.join(programDir, "mpl-agent-reputation"),
});

generateIdl({
  generator: "shank",
  programName: "mpl_agent_validation_program",
  programId: "VALREGY66A9ieJfFUNs5GrxFTy498KUoSU7TbmSePQi",
  idlDir,
  idlName: "mpl_agent_validation",
  binaryInstallDir,
  programDir: path.join(programDir, "mpl-agent-validation"),
});