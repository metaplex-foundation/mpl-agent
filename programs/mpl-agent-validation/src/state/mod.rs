mod agent_validation;
mod collection_validation_config;

pub use agent_validation::*;
pub use collection_validation_config::*;

use shank::ShankType;

/// Account discriminator enum.
/// Stored as a u8 in account data but represented as an enum for type safety.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, ShankType)]
pub enum Key {
    Uninitialized,
    AgentValidationV1,
    CollectionValidationConfigV1,
}

impl From<u8> for Key {
    fn from(value: u8) -> Self {
        match value {
            0 => Key::Uninitialized,
            1 => Key::AgentValidationV1,
            2 => Key::CollectionValidationConfigV1,
            _ => Key::Uninitialized,
        }
    }
}

impl From<Key> for u8 {
    fn from(value: Key) -> Self {
        value as u8
    }
}
