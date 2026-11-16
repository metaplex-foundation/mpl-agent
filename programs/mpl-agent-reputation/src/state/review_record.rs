use bytemuck::{Pod, Zeroable};
use mpl_utils::{assert_derivation, create_or_allocate_account_raw};
use shank::ShankAccount;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::error::MplAgentReputationError;

use super::Key;

/// PDA recorded once per (work-receipt cNFT, review) pair so the program can
/// prove "this receipt has been reviewed" without trusting any off-chain
/// signal. The account's existence itself is the gate — a second
/// `LeaveReviewV1` against the same receipt fails because
/// `create_or_allocate_account_raw` cannot create over a non-system
/// account.
///
/// # Layout
/// - key: 1 byte
/// - bump: 1 byte
/// - _padding: 6 bytes
/// - reviewer: 32 bytes
/// - receipt_asset_id: 32 bytes
///
/// Total: 72 bytes (8-byte aligned).
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable, ShankAccount)]
pub struct ReviewRecordV1 {
    /// Account discriminator.
    #[idl_type(Key)]
    pub key: u8,
    /// PDA bump seed.
    pub bump: u8,
    /// Padding for 8-byte alignment.
    #[padding]
    pub _padding: [u8; 6],
    /// The wallet that left the review.
    pub reviewer: Pubkey,
    /// The Bubblegum asset id of the work receipt this review references.
    pub receipt_asset_id: Pubkey,
}

const _: () = assert!(core::mem::size_of::<ReviewRecordV1>() % 8 == 0);
const _: () = assert!(core::mem::size_of::<ReviewRecordV1>() == 72);

impl ReviewRecordV1 {
    /// Seed prefix.
    pub const PREFIX: &'static [u8] = b"review_record";

    /// Verify that the supplied account is at the canonical PDA for the
    /// given receipt asset id, and return the bump.
    pub fn check_pda_derivation(
        address: &AccountInfo,
        receipt_asset_id: &Pubkey,
    ) -> Result<u8, ProgramError> {
        assert_derivation(
            &crate::ID,
            address,
            &[Self::PREFIX, receipt_asset_id.as_ref()],
            MplAgentReputationError::InvalidAccountData,
        )
    }

    /// Allocate and own a new ReviewRecord account, signed by PDA seeds.
    pub fn create_account<'a>(
        account: &AccountInfo<'a>,
        system_program: &AccountInfo<'a>,
        payer: &AccountInfo<'a>,
        receipt_asset_id: &Pubkey,
        bump: u8,
    ) -> ProgramResult {
        create_or_allocate_account_raw(
            crate::ID,
            account,
            system_program,
            payer,
            core::mem::size_of::<ReviewRecordV1>(),
            &[Self::PREFIX, receipt_asset_id.as_ref(), &[bump]],
        )
    }

    /// Initialize the account fields.
    #[inline]
    pub fn initialize(&mut self, bump: u8, reviewer: &Pubkey, receipt_asset_id: &Pubkey) {
        self.key = Key::ReviewRecordV1 as u8;
        self.bump = bump;
        self._padding = [0u8; 6];
        self.reviewer = *reviewer;
        self.receipt_asset_id = *receipt_asset_id;
    }
}
