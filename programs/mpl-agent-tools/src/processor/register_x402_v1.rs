use bytemuck::{Pod, Zeroable};
use mpl_core::{accounts::BaseAssetV1, types::Key as MplCoreKey};
use shank::ShankType;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, system_program};

use crate::{
    error::MplAgentToolsError, instruction::accounts::RegisterX402V1Accounts, state::X402EndpointV1,
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
    /// The x402 endpoint URL.
    /// Parsed manually from the Borsh string representation (u32 length + bytes).
    #[idl_type("String")]
    url: [u8; 0],
}

// Compile-time assertion to ensure struct is properly sized.
const _: () = assert!(core::mem::size_of::<RegisterX402V1Args>() == 8);

pub fn register_x402_v1<'a>(
    accounts: &'a [AccountInfo<'a>],
    instruction_data: &[u8],
) -> ProgramResult {
    // Parse the URL string from instruction data (after the 8-byte header).
    let (_, string_data) = instruction_data.split_at(core::mem::size_of::<RegisterX402V1Args>());

    let url_length = u32::from_le_bytes(string_data[..4].try_into().unwrap()) as usize;
    let url_bytes = &string_data[4..4 + url_length];

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
    if url_length == 0 {
        return Err(MplAgentToolsError::InvalidUrlLength.into());
    }

    /****************************************************/
    /********************* Actions **********************/
    /****************************************************/

    // Create the x402 endpoint account with space for the trailing URL string.
    X402EndpointV1::create_account(&ctx.accounts, bump, url_length)?;

    // Initialize the fixed portion of the account using zero-copy.
    let mut data = ctx.accounts.x402_endpoint.try_borrow_mut_data()?;
    let endpoint: &mut X402EndpointV1 =
        bytemuck::from_bytes_mut(&mut data[..core::mem::size_of::<X402EndpointV1>()]);

    endpoint.initialize(bump, ctx.accounts.agent_asset.key, signer.key);

    // Write the trailing URL string (u32 length + bytes).
    let url_offset = core::mem::size_of::<X402EndpointV1>();
    data[url_offset..url_offset + 4].copy_from_slice(&(url_length as u32).to_le_bytes());
    data[url_offset + 4..url_offset + 4 + url_length].copy_from_slice(url_bytes);

    Ok(())
}
