mod execution_delegate_record_v1;
mod executive_profile_v1;
mod x402_endpoint_v1;

pub use execution_delegate_record_v1::*;
pub use executive_profile_v1::*;
pub use x402_endpoint_v1::*;

use shank::ShankType;

/// Account discriminator enum.
/// Stored as a u8 in account data but represented as an enum for type safety.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, ShankType)]
pub enum Key {
    Uninitialized,
    ExecutiveProfileV1,
    ExecutionDelegateRecordV1,
    X402EndpointV1,
}

impl From<u8> for Key {
    fn from(value: u8) -> Self {
        match value {
            0 => Key::Uninitialized,
            1 => Key::ExecutiveProfileV1,
            2 => Key::ExecutionDelegateRecordV1,
            3 => Key::X402EndpointV1,
            _ => Key::Uninitialized,
        }
    }
}

impl From<Key> for u8 {
    fn from(value: Key) -> Self {
        value as u8
    }
}
