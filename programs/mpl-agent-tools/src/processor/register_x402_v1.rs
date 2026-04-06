use bytemuck::{Pod, Zeroable};
use mpl_core::{accounts::BaseAssetV1, types::Key as MplCoreKey};
use shank::ShankType;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, system_program};

use crate::{
    error::MplAgentToolsError,
    instruction::accounts::RegisterX402V1Accounts,
    state::{X402EndpointV1, MAX_URL_LEN},
};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable, ShankType)]
pub struct RegisterX402V1Args {
    /// Instruction discriminator (not included in IDL).
    #[skip]
    pub discriminator: u8,
    /// Padding for alignment.
    #[padding]
    pub _padding: [u8; 7],
    /// Length of the URL in the url field.
    pub url_len: u8,
    /// The x402 endpoint URL bytes (fixed-size buffer, padded with zeroes).
    pub url: [u8; 128],
}

// Compile-time assertion to ensure struct is properly sized.
const _: () = assert!(core::mem::size_of::<RegisterX402V1Args>() == 137);

pub fn register_x402_v1<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: &RegisterX402V1Args,
) -> ProgramResult {
    /****************************************************/
    /****************** Account Setup *******************/
    /****************************************************/

    let ctx = RegisterX402V1Accounts::context(accounts)?;

    /****************************************************/
    /****************** Account Guards ******************/
    /****************************************************/

    // Assert that the x402 endpoint account is not already initialized.
    if ctx.accounts.x402_endpoint.owner != &system_program::ID
        || ctx.accounts.x402_endpoint.data_len() != 0
    {
        return Err(MplAgentToolsError::X402EndpointMustBeUninitialized.into());
    }

    // Assert that the agent asset is a valid MPL Core asset.
    if ctx.accounts.agent_asset.owner != &mpl_core::ID
        || ctx.accounts.agent_asset.data_len() == 0
        || ctx.accounts.agent_asset.try_borrow_data()?[0] != MplCoreKey::AssetV1 as u8
    {
        return Err(MplAgentToolsError::InvalidCoreAsset.into());
    }

    // Assert that the signer is the asset owner.
    let asset = BaseAssetV1::try_from(ctx.accounts.agent_asset)?;
    let signer = ctx.accounts.authority.unwrap_or(ctx.accounts.payer);
    if asset.owner != *signer.key {
        return Err(MplAgentToolsError::AssetOwnerMustRegisterX402.into());
    }

    // Check PDA derivation.
    let bump = X402EndpointV1::check_pda_derivation(
        ctx.accounts.x402_endpoint,
        ctx.accounts.agent_asset.key,
    )?;

    // Validate system program.
    if *ctx.accounts.system_program.key != system_program::id() {
        return Err(MplAgentToolsError::InvalidSystemProgram.into());
    }

    /****************************************************/
    /***************** Argument Guards ******************/
    /****************************************************/

    // Validate URL length.
    let url_len = args.url_len as usize;
    if url_len == 0 || url_len > MAX_URL_LEN {
        return Err(MplAgentToolsError::InvalidUrlLength.into());
    }

    /****************************************************/
    /********************* Actions **********************/
    /****************************************************/

    // Create the x402 endpoint account.
    X402EndpointV1::create_account(&ctx.accounts, bump)?;

    // Initialize the account using zero-copy.
    let mut data = ctx.accounts.x402_endpoint.try_borrow_mut_data()?;
    let endpoint: &mut X402EndpointV1 =
        bytemuck::from_bytes_mut(&mut data[..core::mem::size_of::<X402EndpointV1>()]);

    endpoint.initialize(
        bump,
        ctx.accounts.agent_asset.key,
        signer.key,
        &args.url,
        args.url_len,
    );

    Ok(())
}
