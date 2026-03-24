mod agent_identity_v1;
mod agent_identity_v2;

pub use agent_identity_v1::*;
pub use agent_identity_v2::*;

use shank::ShankType;

/// Account discriminator enum.
/// Stored as a u8 in account data but represented as an enum for type safety.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, ShankType)]
pub enum Key {
    Uninitialized,
    AgentIdentityV1,
    AgentIdentityV2,
}

impl From<u8> for Key {
    fn from(value: u8) -> Self {
        match value {
            0 => Key::Uninitialized,
            1 => Key::AgentIdentityV1,
            2 => Key::AgentIdentityV2,
            _ => Key::Uninitialized,
        }
    }
}

impl From<Key> for u8 {
    fn from(value: Key) -> Self {
        value as u8
    }
}
