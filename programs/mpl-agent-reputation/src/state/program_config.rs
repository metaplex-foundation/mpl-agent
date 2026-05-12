use bytemuck::{Pod, Zeroable};
use mpl_utils::{assert_derivation, create_or_allocate_account_raw};
use shank::ShankAccount;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::error::MplAgentReputationError;

use super::Key;

/// Singleton config PDA anchoring the program-managed reviews
/// infrastructure. Stores admin, the canonical reviews collection, the
/// canonical receipts collection (used to validate that incoming
/// LeaveReviewV1 calls reference a tools-program receipt), and the next
/// reviews-tree index.
///
/// # Layout (120 bytes, 8-byte aligned)
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable, ShankAccount)]
pub struct ReviewsConfigV1 {
    #[idl_type(Key)]
    pub key: u8,
    pub bump: u8,
    #[padding]
    pub _padding: [u8; 6],
    pub admin: Pubkey,
    /// Reviews collection (owned by this program config PDA).
    pub reviews_collection: Pubkey,
    /// Receipts collection (owned by mpl-agent-tools' program config PDA).
    /// LeaveReviewV1 verifies the receipt the caller references is in
    /// this collection — cross-program canonicalisation.
    pub receipts_collection: Pubkey,
    pub next_tree_index: u64,
}

const _: () = assert!(core::mem::size_of::<ReviewsConfigV1>() % 8 == 0);
const _: () = assert!(core::mem::size_of::<ReviewsConfigV1>() == 112);

impl ReviewsConfigV1 {
    pub const PREFIX: &'static [u8] = b"program_config";

    pub fn check_pda_derivation(address: &AccountInfo) -> Result<u8, ProgramError> {
        assert_derivation(
            &crate::ID,
            address,
            &[Self::PREFIX],
            MplAgentReputationError::InvalidProgramConfigDerivation,
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
            core::mem::size_of::<ReviewsConfigV1>(),
            &[Self::PREFIX, &[bump]],
        )
    }

    #[inline]
    pub fn initialize(
        &mut self,
        bump: u8,
        admin: &Pubkey,
        reviews_collection: &Pubkey,
        receipts_collection: &Pubkey,
    ) {
        self.key = Key::ReviewsConfigV1 as u8;
        self.bump = bump;
        self._padding = [0u8; 6];
        self.admin = *admin;
        self.reviews_collection = *reviews_collection;
        self.receipts_collection = *receipts_collection;
        self.next_tree_index = 0;
    }
}

pub const REVIEWS_TREE_PREFIX: &[u8] = b"reviews_tree";

pub fn check_reviews_tree_pda(address: &AccountInfo, index: u64) -> Result<u8, ProgramError> {
    assert_derivation(
        &crate::ID,
        address,
        &[REVIEWS_TREE_PREFIX, &index.to_le_bytes()],
        MplAgentReputationError::InvalidReviewsTreeDerivation,
    )
}
