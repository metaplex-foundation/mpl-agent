use bytemuck::{Pod, Zeroable};
use mpl_utils::{assert_derivation, create_or_allocate_account_raw};
use shank::ShankAccount;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::error::MplAgentToolsError;

use super::Key;

/// Singleton config PDA that anchors the program-managed receipts
/// infrastructure: which account is the canonical receipts collection,
/// how many trees have been registered so far, and who is allowed to
/// register more.
///
/// # Layout (88 bytes, 8-byte aligned)
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable, ShankAccount)]
pub struct ToolsConfigV1 {
    #[idl_type(Key)]
    pub key: u8,
    pub bump: u8,
    #[padding]
    pub _padding: [u8; 6],
    /// Authority allowed to register new trees and rotate the admin.
    pub admin: Pubkey,
    /// Canonical receipts collection. Every receipt cNFT minted by this
    /// program lives in this single collection so off-chain consumers can
    /// filter receipts by one address.
    pub collection: Pubkey,
    /// Monotonically increasing counter. The next call to
    /// `RegisterReceiptsTreeV1` mints a tree at PDA `["receipts_tree",
    /// next_tree_index_le]` and bumps this counter.
    pub next_tree_index: u64,
}

const _: () = assert!(core::mem::size_of::<ToolsConfigV1>() % 8 == 0);
const _: () = assert!(core::mem::size_of::<ToolsConfigV1>() == 80);

impl ToolsConfigV1 {
    pub const PREFIX: &'static [u8] = b"program_config";

    pub fn check_pda_derivation(address: &AccountInfo) -> Result<u8, ProgramError> {
        assert_derivation(
            &crate::ID,
            address,
            &[Self::PREFIX],
            MplAgentToolsError::InvalidProgramConfigDerivation,
        )
    }

    pub fn create_account<'a>(
        config: &AccountInfo<'a>,
        system_program: &AccountInfo<'a>,
        payer: &AccountInfo<'a>,
        bump: u8,
    ) -> ProgramResult {
        create_or_allocate_account_raw(
            crate::ID,
            config,
            system_program,
            payer,
            core::mem::size_of::<ToolsConfigV1>(),
            &[Self::PREFIX, &[bump]],
        )
    }

    #[inline]
    pub fn initialize(&mut self, bump: u8, admin: &Pubkey, collection: &Pubkey) {
        self.key = Key::ToolsConfigV1 as u8;
        self.bump = bump;
        self._padding = [0u8; 6];
        self.admin = *admin;
        self.collection = *collection;
        self.next_tree_index = 0;
    }
}

/// Seeds for the per-tree PDA. Together with the program id and the
/// little-endian-encoded tree index, this yields a deterministic merkle
/// tree address for index N.
pub const RECEIPTS_TREE_PREFIX: &[u8] = b"receipts_tree";

/// Derive a receipts tree PDA bump for a given index. The address itself
/// is whatever `find_program_address` returns at the same time.
pub fn check_receipts_tree_pda(address: &AccountInfo, index: u64) -> Result<u8, ProgramError> {
    assert_derivation(
        &crate::ID,
        address,
        &[RECEIPTS_TREE_PREFIX, &index.to_le_bytes()],
        MplAgentToolsError::InvalidReceiptsTreeDerivation,
    )
}
