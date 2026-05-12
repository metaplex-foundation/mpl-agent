mod agent_reputation;
mod program_config;
mod review_record;
mod review_subsidy_pool;

pub use agent_reputation::*;
pub use program_config::*;
pub use review_record::*;
pub use review_subsidy_pool::*;

use shank::ShankType;

/// Account discriminator enum.
/// Stored as a u8 in account data but represented as an enum for type safety.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, ShankType)]
pub enum Key {
    Uninitialized,
    AgentReputationV1,
    ReviewRecordV1,
    ReviewSubsidyPoolV1,
    ReviewsConfigV1,
}

impl From<u8> for Key {
    fn from(value: u8) -> Self {
        match value {
            0 => Key::Uninitialized,
            1 => Key::AgentReputationV1,
            2 => Key::ReviewRecordV1,
            3 => Key::ReviewSubsidyPoolV1,
            4 => Key::ReviewsConfigV1,
            _ => Key::Uninitialized,
        }
    }
}

impl From<Key> for u8 {
    fn from(value: Key) -> Self {
        value as u8
    }
}
