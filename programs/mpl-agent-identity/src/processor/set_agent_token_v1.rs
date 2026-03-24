use bytemuck::{Pod, Zeroable};
use mpl_core::accounts::AssetSigner;
use mpl_core::types::Key as MplCoreKey;
use mpl_utils::token::SPL_TOKEN_PROGRAM_IDS;
use mpl_utils::{assert_owner_in, assert_signer, resize_or_reallocate_account_raw};
use podded::pod::{Nullable, OptionalPubkey};
use shank::ShankType;
use solana_program::program_error::ProgramError;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, system_program};
use spl_token_2022_interface::extension::StateWithExtensions;
use spl_token_2022_interface::state::Mint;

use crate::instruction::accounts::SetAgentTokenV1Accounts;
use crate::state::Key;
use crate::{error::MplAgentIdentityError, state::AgentIdentityV2};

impl<'a> SetAgentTokenV1Accounts<'a> {
    pub fn validate(&self) -> Result<(), ProgramError> {
        let Self {
            agent_identity,
            asset,
            agent_token,
            payer,
            authority,
            system_program,
        } = self;

        // Agent Identity
        let agent_identity_data = agent_identity.try_borrow_data()?;
        if agent_identity.owner != &crate::ID
            || agent_identity_data.len() == 0
            || (agent_identity_data[0] != Key::AgentIdentityV1 as u8
                && agent_identity_data[0] != Key::AgentIdentityV2 as u8)
        {
            return Err(MplAgentIdentityError::InvalidAgentIdentity.into());
        }

        let _ = AgentIdentityV2::check_pda_derivation(agent_identity, self.asset.key)?;

        // Asset
        // Assert that the asset exists and is a Core asset.
        if asset.owner != &mpl_core::ID || asset.try_borrow_data()?[0] != MplCoreKey::AssetV1 as u8
        {
            return Err(MplAgentIdentityError::InvalidCoreAsset.into());
        }

        // Agent Token
        // Assert that the agent token is owned by the SPL Token program.
        assert_owner_in(
            agent_token,
            &SPL_TOKEN_PROGRAM_IDS,
            MplAgentIdentityError::InvalidAgentToken,
        )?;
        // And that it is a mint account.
        let mint_data = agent_token.try_borrow_data()?;
        let _mint = StateWithExtensions::<Mint>::unpack(&mint_data)
            .map_err(|_| MplAgentIdentityError::InvalidAgentToken)?;

        // Payer
        assert_signer(payer)?;

        // Authority
        if authority.is_some() {
            assert_signer(authority.unwrap())?;
        }

        // System Program
        if *system_program.key != system_program::id() {
            return Err(MplAgentIdentityError::InvalidSystemProgram.into());
        }

        Ok(())
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable, ShankType)]
pub struct SetAgentTokenV1Args {
    /// Instruction discriminator (not included in IDL).
    #[skip]
    pub discriminator: u8,
    /// Padding for alignment.
    #[padding]
    pub _padding: [u8; 7],
}

// Compile-time assertion to ensure struct is properly sized.
const _: () = assert!(core::mem::size_of::<SetAgentTokenV1Args>() == 8);

pub fn set_agent_token_v1<'a>(
    accounts: &'a [AccountInfo<'a>],
    _instruction_data: &[u8],
) -> ProgramResult {
    /****************************************************/
    /****************** Account Setup *******************/
    /****************************************************/

    let ctx = SetAgentTokenV1Accounts::context(accounts)?;

    ctx.accounts.validate()?;

    // We also need to assert that the authority is the asset signer PDA.
    let asset_signer_pda = AssetSigner::find_pda(ctx.accounts.asset.key);
    if asset_signer_pda.0 != *ctx.accounts.authority.unwrap_or(ctx.accounts.payer).key {
        return Err(MplAgentIdentityError::OnlyAssetSignerCanSetAgentToken.into());
    }

    /****************************************************/
    /***************** Argument Guards ******************/
    /****************************************************/

    /****************************************************/
    /********************* Actions **********************/
    /****************************************************/
    // If agent_identity is an AgentIdentityV1, we need to upgrade it to an AgentIdentityV2.
    if ctx.accounts.agent_identity.try_borrow_data()?[0] == Key::AgentIdentityV1 as u8 {
        // Resize the account
        resize_or_reallocate_account_raw(
            ctx.accounts.agent_identity,
            ctx.accounts.payer,
            ctx.accounts.system_program,
            core::mem::size_of::<AgentIdentityV2>(),
        )?;

        // Set the discriminator to AgentIdentityV2
        ctx.accounts.agent_identity.try_borrow_mut_data()?[0] = Key::AgentIdentityV2 as u8;

        // The new bytes are zeroed so the new fields will be valid.
    }

    let mut agent_identity_data = ctx.accounts.agent_identity.try_borrow_mut_data()?;
    let agent_identity: &mut AgentIdentityV2 = bytemuck::from_bytes_mut(
        &mut agent_identity_data[..core::mem::size_of::<AgentIdentityV2>()],
    );

    // You can only set the agent token if it is not already set.
    if agent_identity.agent_token.is_some() {
        return Err(MplAgentIdentityError::AgentTokenAlreadySet.into());
    }

    agent_identity.agent_token = OptionalPubkey::new(*ctx.accounts.agent_token.key);

    Ok(())
}
