use bytemuck::{Pod, Zeroable};
use mpl_core::instructions::{
    AddExternalPluginAdapterV1Cpi, AddExternalPluginAdapterV1InstructionArgs,
};
use mpl_core::types::{
    AgentIdentityInitInfo, ExternalPluginAdapterInitInfo, HookableLifecycleEvent, Key as MplCoreKey,
};
use mpl_core::ExternalCheckResultBits;
use mpl_utils::assert_signer;
use shank::ShankType;
use solana_program::program_error::ProgramError;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, system_program};

use crate::{
    error::MplAgentIdentityError, instruction::accounts::RegisterIdentityV1Accounts,
    state::AgentIdentityV2,
};

impl<'a> RegisterIdentityV1Accounts<'a> {
    pub fn validate(&self) -> Result<u8, ProgramError> {
        let Self {
            agent_identity,
            asset,
            collection: _,
            payer,
            authority,
            mpl_core_program,
            system_program,
        } = self;

        // Agent Identity
        let agent_identity_bump =
            AgentIdentityV2::check_pda_derivation(agent_identity, self.asset.key)?;

        if agent_identity.data_len() != 0 || *agent_identity.owner != system_program::id() {
            return Err(MplAgentIdentityError::AgentIdentityAlreadyRegistered.into());
        }

        // Asset
        // Assert that the asset exists and is a Core asset.
        if asset.owner != &mpl_core::ID || asset.try_borrow_data()?[0] != MplCoreKey::AssetV1 as u8
        {
            return Err(MplAgentIdentityError::InvalidCoreAsset.into());
        }

        // Collection
        // SAFE: Checked by the Core program.

        // Payer
        assert_signer(payer)?;

        // Authority
        if authority.is_some() {
            assert_signer(authority.unwrap())?;
        }

        // MPL Core Program
        if *mpl_core_program.key != mpl_core::ID {
            return Err(MplAgentIdentityError::InvalidMplCoreProgram.into());
        }

        // System Program
        if *system_program.key != system_program::id() {
            return Err(MplAgentIdentityError::InvalidSystemProgram.into());
        }

        Ok(agent_identity_bump)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable, ShankType)]
pub struct RegisterIdentityV1Args {
    /// Instruction discriminator (not included in IDL).
    #[skip]
    pub discriminator: u8,
    /// Padding for alignment.
    #[padding]
    pub _padding: [u8; 7],
    /// The URI of the Agent Registration JSON file.
    /// We parse this manually from a string representation in the IDL.
    #[idl_type("String")]
    agent_registration_uri: [u8; 0],
}

// Compile-time assertion to ensure struct is properly sized.
const _: () = assert!(core::mem::size_of::<RegisterIdentityV1Args>() == 8);

pub fn register_identity_v1<'a>(
    accounts: &'a [AccountInfo<'a>],
    instruction_data: &[u8],
) -> ProgramResult {
    let (_, string_data) =
        instruction_data.split_at(core::mem::size_of::<RegisterIdentityV1Args>());

    let uri_length: u32 = u32::from_le_bytes(string_data[..4].try_into().unwrap());
    let uri = String::from_utf8(string_data[4..4 + uri_length as usize].to_vec()).unwrap();
    /****************************************************/
    /****************** Account Setup *******************/
    /****************************************************/

    let ctx = RegisterIdentityV1Accounts::context(accounts)?;
    let agent_identity_bump = ctx.accounts.validate()?;

    /****************************************************/
    /***************** Argument Guards ******************/
    /****************************************************/

    /****************************************************/
    /********************* Actions **********************/
    /****************************************************/
    // Create the agent identity account.
    AgentIdentityV2::create_account(&ctx.accounts, agent_identity_bump)?;

    // Initialize the account using zero-copy.
    // Borrow the account data mutably and cast to our struct.
    let mut data = ctx.accounts.agent_identity.try_borrow_mut_data()?;
    let agent_identity: &mut AgentIdentityV2 =
        bytemuck::from_bytes_mut(&mut data[..core::mem::size_of::<AgentIdentityV2>()]);

    agent_identity.initialize(agent_identity_bump, ctx.accounts.asset.key);

    // Drop the agent identity account data.
    drop(data);

    // Add the Agent Identity External Plugin Adapter to the asset.
    AddExternalPluginAdapterV1Cpi {
        __program: ctx.accounts.mpl_core_program,
        asset: ctx.accounts.asset,
        collection: ctx.accounts.collection,
        payer: ctx.accounts.payer,
        authority: ctx.accounts.authority,
        system_program: ctx.accounts.system_program,
        log_wrapper: None,
        __args: AddExternalPluginAdapterV1InstructionArgs {
            init_info: ExternalPluginAdapterInitInfo::AgentIdentity(AgentIdentityInitInfo {
                uri,
                init_plugin_authority: None,
                lifecycle_checks: vec![
                    (
                        HookableLifecycleEvent::Transfer,
                        ExternalCheckResultBits::new()
                            .with_can_approve(true)
                            .with_can_listen(true)
                            .with_can_reject(true)
                            .into(),
                    ),
                    (
                        HookableLifecycleEvent::Update,
                        ExternalCheckResultBits::new()
                            .with_can_approve(true)
                            .with_can_listen(true)
                            .with_can_reject(true)
                            .into(),
                    ),
                    (
                        HookableLifecycleEvent::Execute,
                        ExternalCheckResultBits::new()
                            .with_can_approve(true)
                            .with_can_listen(true)
                            .with_can_reject(true)
                            .into(),
                    ),
                ],
            }),
        },
    }
    .invoke_signed_with_remaining_accounts(
        &[&[
            AgentIdentityV2::PREFIX,
            ctx.accounts.asset.key.as_ref(),
            &[agent_identity_bump],
        ]],
        &[(ctx.accounts.agent_identity, true, false)],
    )?;

    Ok(())
}
