use bytemuck::{Pod, Zeroable};
use shank::ShankAccount;
use solana_program::pubkey::Pubkey;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable, ShankAccount)]
pub struct AgentIdentityV1 {
    /// Account discriminator.
    #[idl_type(Key)]
    pub key: u8,
    /// PDA bump seed.
    pub bump: u8,
    /// Padding for 8-byte alignment.
    #[padding]
    pub _padding: [u8; 6],
    /// The address of the asset.
    pub asset: Pubkey,
}

// Compile-time assertion to ensure struct is 8-byte aligned.
const _: () = assert!(core::mem::size_of::<AgentIdentityV1>() % 8 == 0);
const _: () = assert!(core::mem::size_of::<AgentIdentityV1>() == 40);
