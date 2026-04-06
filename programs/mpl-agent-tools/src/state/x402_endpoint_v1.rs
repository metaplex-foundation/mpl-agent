use bytemuck::{Pod, Zeroable};
use mpl_utils::{assert_derivation, create_or_allocate_account_raw};
use shank::ShankAccount;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{error::MplAgentToolsError, instruction::accounts::RegisterX402V1Accounts, state::Key};

/// Maximum length of an x402 endpoint URL in bytes.
pub const MAX_URL_LEN: usize = 128;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable, ShankAccount)]
pub struct X402EndpointV1 {
    /// Account discriminator.
    #[idl_type(Key)]
    pub key: u8,
    /// PDA bump seed.
    pub bump: u8,
    /// Length of the URL stored in the url field.
    pub url_len: u8,
    /// Padding for 8-byte alignment.
    #[padding]
    pub _padding: [u8; 5],
    /// The address of the agent asset this endpoint is registered for.
    pub asset: Pubkey,
    /// The authority who registered the endpoint (asset owner at registration time).
    pub authority: Pubkey,
    /// The x402 endpoint URL (fixed-size buffer, use url_len for actual length).
    pub url: [u8; 128],
}

// Compile-time assertion to ensure struct is 8-byte aligned.
const _: () = assert!(core::mem::size_of::<X402EndpointV1>() % 8 == 0);
const _: () = assert!(core::mem::size_of::<X402EndpointV1>() == 200);

impl X402EndpointV1 {
    const PREFIX: &[u8] = b"x402_endpoint";

    // Check the PDA derivation.
    pub fn check_pda_derivation(address: &AccountInfo, asset: &Pubkey) -> Result<u8, ProgramError> {
        assert_derivation(
            &crate::ID,
            address,
            &[Self::PREFIX, asset.as_ref()],
            MplAgentToolsError::InvalidX402EndpointDerivation,
        )
    }

    // Create the account.
    pub fn create_account(accounts: &RegisterX402V1Accounts, bump: u8) -> ProgramResult {
        create_or_allocate_account_raw(
            crate::ID,
            accounts.x402_endpoint,
            accounts.system_program,
            accounts.payer,
            core::mem::size_of::<X402EndpointV1>(),
            &[Self::PREFIX, accounts.agent_asset.key.as_ref(), &[bump]],
        )
    }

    /// Initialize the account with the given data.
    #[inline]
    pub fn initialize(
        &mut self,
        bump: u8,
        asset: &Pubkey,
        authority: &Pubkey,
        url: &[u8],
        url_len: u8,
    ) {
        solana_program::msg!("Initializing x402 endpoint account");
        self.key = Key::X402EndpointV1 as u8;
        self.bump = bump;
        self.url_len = url_len;
        self._padding = [0u8; 5];
        self.asset = *asset;
        self.authority = *authority;
        self.url = [0u8; 128];
        self.url[..url_len as usize].copy_from_slice(&url[..url_len as usize]);
    }
}
