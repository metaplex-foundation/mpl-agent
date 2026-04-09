// Allow lints from generated code (num_derive, kinobi codegen).
#![allow(non_local_definitions)]
#![allow(clippy::new_without_default)]
#![allow(unexpected_cfgs)]

mod generated;

pub use generated::programs::MPL_AGENT_IDENTITY_ID as ID;
pub use generated::*;
