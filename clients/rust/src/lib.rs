#[path = "generated/identity/mod.rs"]
mod identity;
#[path = "generated/reputation/mod.rs"]
mod reputation;
#[path = "generated/validation/mod.rs"]
mod validation;

pub use identity::programs::MPL_AGENT_IDENTITY_ID as ID;
// pub use identity::*;
pub use reputation::programs::MPL_AGENT_REPUTATION_ID as REPUTATION_ID;
// pub use reputation::*;
pub use validation::programs::MPL_AGENT_VALIDATION_ID as VALIDATION_ID;
// pub use validation::*;
