mod agent_reputation;
mod review_record;
mod seeds;

pub use agent_reputation::*;
pub use review_record::*;
pub use seeds::*;

use shank::ShankType;

/// Account discriminator enum.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, ShankType)]
pub enum Key {
    Uninitialized,
    AgentReputationV1,
    ReviewRecordV1,
}

impl From<u8> for Key {
    fn from(value: u8) -> Self {
        match value {
            0 => Key::Uninitialized,
            1 => Key::AgentReputationV1,
            2 => Key::ReviewRecordV1,
            _ => Key::Uninitialized,
        }
    }
}

impl From<Key> for u8 {
    fn from(value: Key) -> Self {
        value as u8
    }
}
