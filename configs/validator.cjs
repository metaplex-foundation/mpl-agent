const path = require("path");

const programDir = path.join(__dirname, "..", "programs");

function getProgram(programBinary) {
  return path.join(programDir, ".bin", programBinary);
}

module.exports = {
  validator: {
    commitment: "processed",
    programs: [
      {
        label: "Mpl Agent Identity",
        programId: "1DREGFgysWYxLnRnKQnwrxnJQeSMk2HmGaC6whw2B2p",
        deployPath: getProgram("mpl_agent_identity_program.so"),
      },
      {
        label: "Mpl Agent Reputation",
        programId: "REPREG5c1gPHuHukEyANpksLdHFaJCiTrm6zJgNhRZR",
        deployPath: getProgram("mpl_agent_reputation_program.so"),
      },
      {
        label: "Mpl Agent Validation",
        programId: "VALREGY66A9ieJfFUNs5GrxFTy498KUoSU7TbmSePQi",
        deployPath: getProgram("mpl_agent_validation_program.so"),
      },
      // Below are external programs that should be included in the local validator.
      // You may configure which ones to fetch from the cluster when building
      // programs within the `configs/program-scripts/dump.sh` script.
      {
        label: "Token Metadata",
        programId: "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s",
        deployPath: getProgram("mpl_token_metadata.so"),
      },
      {
        label: "SPL Noop",
        programId: "noopb9bkMVfRPU8AsbpTUg8AQkHtKwMYZiFUjNRtMmV",
        deployPath: getProgram("spl_noop.so"),
      },
      {
        label: "Mpl Core",
        programId: "CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d",
        deployPath: path.join(__dirname, "..", "..", "..", "security", "core", "programs", ".bin", "mpl_core_program.so"),
      }
    ],
  },
};
