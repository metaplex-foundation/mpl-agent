use bytemuck::{Pod, Zeroable};
use mpl_utils::{assert_derivation, create_or_allocate_account_raw};
use shank::ShankAccount;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::error::MplAgentReputationError;

use super::Key;

/// Per-agent lamport pool that funds review-record rent (and a small
/// reviewer tip) so the reviewer's wallet doesn't have to. The pool is
/// program-owned: only the on-chain logic can move lamports out, which
/// prevents the agent from censoring specific reviews after funding.
///
/// # Layout
/// - key: 1 byte
/// - bump: 1 byte
/// - _padding: 6 bytes
/// - agent_asset: 32 bytes
/// - withdraw_authority: 32 bytes (typically the agent asset's owner)
///
/// Total: 72 bytes (8-byte aligned). The remaining lamports above
/// rent-exempt minimum are the spendable subsidy budget.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable, ShankAccount)]
pub struct ReviewSubsidyPoolV1 {
    /// Account discriminator.
    #[idl_type(Key)]
    pub key: u8,
    /// PDA bump seed.
    pub bump: u8,
    /// Padding for 8-byte alignment.
    #[padding]
    pub _padding: [u8; 6],
    /// The agent asset whose reviews this pool subsidises.
    pub agent_asset: Pubkey,
    /// The wallet authorised to withdraw remaining funds.
    pub withdraw_authority: Pubkey,
}

const _: () = assert!(core::mem::size_of::<ReviewSubsidyPoolV1>() % 8 == 0);
const _: () = assert!(core::mem::size_of::<ReviewSubsidyPoolV1>() == 72);

impl ReviewSubsidyPoolV1 {
    pub const PREFIX: &'static [u8] = b"review_subsidy_pool";

    pub fn check_pda_derivation(
        address: &AccountInfo,
        agent_asset: &Pubkey,
    ) -> Result<u8, ProgramError> {
        assert_derivation(
            &crate::ID,
            address,
            &[Self::PREFIX, agent_asset.as_ref()],
            MplAgentReputationError::InvalidAccountData,
        )
    }

    pub fn create_account<'a>(
        account: &AccountInfo<'a>,
        system_program: &AccountInfo<'a>,
        payer: &AccountInfo<'a>,
        agent_asset: &Pubkey,
        bump: u8,
    ) -> ProgramResult {
        create_or_allocate_account_raw(
            crate::ID,
            account,
            system_program,
            payer,
            core::mem::size_of::<ReviewSubsidyPoolV1>(),
            &[Self::PREFIX, agent_asset.as_ref(), &[bump]],
        )
    }

    #[inline]
    pub fn initialize(&mut self, bump: u8, agent_asset: &Pubkey, withdraw_authority: &Pubkey) {
        self.key = Key::ReviewSubsidyPoolV1 as u8;
        self.bump = bump;
        self._padding = [0u8; 6];
        self.agent_asset = *agent_asset;
        self.withdraw_authority = *withdraw_authority;
    }
}
