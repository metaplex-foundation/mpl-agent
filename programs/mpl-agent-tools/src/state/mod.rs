mod execution_delegate_record_v1;
mod executive_profile_v1;
mod seeds;

pub use execution_delegate_record_v1::*;
pub use executive_profile_v1::*;
pub use seeds::*;

use shank::ShankType;

/// Account discriminator enum for the two on-chain record accounts in
/// this program. Receipts collection/authority/tree are stateless PDAs
/// without their own discriminator.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, ShankType)]
pub enum Key {
    Uninitialized,
    ExecutiveProfileV1,
    ExecutionDelegateRecordV1,
}

impl From<u8> for Key {
    fn from(value: u8) -> Self {
        match value {
            0 => Key::Uninitialized,
            1 => Key::ExecutiveProfileV1,
            2 => Key::ExecutionDelegateRecordV1,
            _ => Key::Uninitialized,
        }
    }
}

impl From<Key> for u8 {
    fn from(value: Key) -> Self {
        value as u8
    }
}
