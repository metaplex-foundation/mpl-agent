use bytemuck::{Pod, Zeroable};
use mpl_core::accounts::BaseAssetV1;
use mpl_core::fetch_wrapped_external_plugin_adapter;
use mpl_core::instructions::{
    AddExternalPluginAdapterV1Cpi, AddExternalPluginAdapterV1InstructionArgs,
};
use mpl_core::types::{
    AppDataInitInfo, ExternalPluginAdapterInitInfo, ExternalPluginAdapterKey,
    ExternalPluginAdapterSchema, Key as MplCoreKey, PluginAuthority,
};
use shank::ShankType;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, system_program};

use crate::{
    error::MplAgentReputationError, instruction::accounts::RegisterReputationV1Accounts,
    state::AgentReputationV1,
};

/// Arguments for the RegisterReputationV1 instruction.
///
/// # Layout
/// - discriminator: 1 byte (instruction discriminant, excluded from IDL)
/// - _padding: 1 byte (alignment)
/// - arg1: 2 bytes
/// - arg2: 4 bytes
///
/// Total: 8 bytes (8-byte aligned)
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable, ShankType)]
pub struct RegisterReputationV1Args {
    /// Instruction discriminator (not included in IDL).
    #[skip]
    pub discriminator: u8,
    /// Padding for alignment.
    #[padding]
    pub _padding: [u8; 7],
}

// Compile-time assertion to ensure struct is properly sized.
const _: () = assert!(core::mem::size_of::<RegisterReputationV1Args>() == 8);

/// RegisterReputationV1 a new Agent Reputation.
///
/// # Accounts
/// 0. `[writable, signer]` agent_reputation - The address of the new agent reputation
/// 1. `[]` authority - The authority of the agent reputation
/// 2. `[writable, signer]` payer - The account paying for the storage fees
/// 3. `[]` system_program - The system program
///
/// # Arguments
/// * `accounts` - The accounts required for the instruction
/// * `args` - The instruction arguments (zero-copy reference)
pub fn register_reputation_v1<'a>(
    accounts: &'a [AccountInfo<'a>],
    _args: &RegisterReputationV1Args,
) -> ProgramResult {
    /****************************************************/
    /****************** Account Setup *******************/
    /****************************************************/

    let ctx = RegisterReputationV1Accounts::context(accounts)?;

    /****************************************************/
    /****************** Account Guards ******************/
    /****************************************************/

    let agent_reputation_bump = AgentReputationV1::check_pda_derivation(
        ctx.accounts.agent_reputation,
        ctx.accounts.asset.key,
    )?;

    // Assert that the asset exists and is a Core asset.
    if ctx.accounts.asset.owner != &mpl_core::ID
        || ctx.accounts.asset.try_borrow_data()?[0] != MplCoreKey::AssetV1 as u8
    {
        return Err(MplAgentReputationError::InvalidCoreAsset.into());
    }

    // Validate the MPL Core program.
    if *ctx.accounts.mpl_core_program.key != mpl_core::ID {
        return Err(MplAgentReputationError::InvalidMplCoreProgram.into());
    }

    // Validate system program.
    if *ctx.accounts.system_program.key != system_program::id() {
        return Err(MplAgentReputationError::InvalidSystemProgram.into());
    }

    /****************************************************/
    /***************** Argument Guards ******************/
    /****************************************************/

    // Add any argument validation here.
    // Example: if args.arg1 == 0 { return Err(MplAgentReputationError::InvalidArgument.into()); }

    /****************************************************/
    /********************* Actions **********************/
    /****************************************************/
    // Create the agent identity account.
    AgentReputationV1::create_account(&ctx.accounts, agent_reputation_bump)?;

    // Initialize the account using zero-copy.
    // Borrow the account data mutably and cast to our struct.
    let mut data = ctx.accounts.agent_reputation.try_borrow_mut_data()?;
    let agent_reputation: &mut AgentReputationV1 =
        bytemuck::from_bytes_mut(&mut data[..core::mem::size_of::<AgentReputationV1>()]);

    agent_reputation.initialize(agent_reputation_bump, ctx.accounts.asset.key);

    // Check if the asset already has a AppData plugin.
    let result = fetch_wrapped_external_plugin_adapter::<BaseAssetV1>(
        ctx.accounts.asset,
        None,
        &ExternalPluginAdapterKey::AppData(PluginAuthority::Address {
            address: *ctx.accounts.agent_reputation.key,
        }),
    );

    // If the asset already has a AppData plugin, move on, otherwise create it.
    if result.is_err() {
        AddExternalPluginAdapterV1Cpi {
            __program: ctx.accounts.mpl_core_program,
            asset: ctx.accounts.asset,
            collection: ctx.accounts.collection,
            payer: ctx.accounts.payer,
            authority: ctx.accounts.authority,
            system_program: ctx.accounts.system_program,
            log_wrapper: None,
            __args: AddExternalPluginAdapterV1InstructionArgs {
                init_info: ExternalPluginAdapterInitInfo::AppData(AppDataInitInfo {
                    data_authority: PluginAuthority::Address {
                        address: *ctx.accounts.agent_reputation.key,
                    },
                    init_plugin_authority: None,
                    schema: Some(ExternalPluginAdapterSchema::Binary),
                }),
            },
        }
        .invoke()?;
    }

    Ok(())
}
