use bytemuck::{Pod, Zeroable};
use mpl_core::accounts::BaseCollectionV1;
use mpl_core::fetch_wrapped_external_plugin_adapter;
use mpl_core::instructions::{
    AddCollectionExternalPluginAdapterV1Cpi, AddCollectionExternalPluginAdapterV1InstructionArgs,
};
use mpl_core::types::{
    ExternalPluginAdapterInitInfo, ExternalPluginAdapterKey, ExternalPluginAdapterSchema,
    Key as MplCoreKey, LinkedAppDataInitInfo, PluginAuthority,
};
use shank::ShankType;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, system_program};

use crate::{
    error::Mpl8004IdentityError,
    instruction::accounts::RegisterV1Accounts,
    state::{AgentIdentityV1, CollectionConfigV1},
};

/// Arguments for the Register instruction.
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
pub struct RegisterV1Args {
    /// Instruction discriminator (not included in IDL).
    #[skip]
    pub discriminator: u8,
    /// Padding for alignment.
    #[padding]
    pub _padding: [u8; 7],
}

// Compile-time assertion to ensure struct is properly sized.
const _: () = assert!(core::mem::size_of::<RegisterV1Args>() == 8);

/// Register a new Agent Identity.
///
/// # Accounts
/// 0. `[writable, signer]` agent_identity - The address of the new agent identity
/// 1. `[]` authority - The authority of the agent identity
/// 2. `[writable, signer]` payer - The account paying for the storage fees
/// 3. `[]` system_program - The system program
///
/// # Arguments
/// * `accounts` - The accounts required for the instruction
/// * `args` - The instruction arguments (zero-copy reference)
pub fn register_v1<'a>(accounts: &'a [AccountInfo<'a>], _args: &RegisterV1Args) -> ProgramResult {
    /****************************************************/
    /****************** Account Setup *******************/
    /****************************************************/

    let ctx = RegisterV1Accounts::context(accounts)?;

    /****************************************************/
    /****************** Account Guards ******************/
    /****************************************************/

    let agent_identity_bump =
        AgentIdentityV1::check_pda_derivation(ctx.accounts.agent_identity, ctx.accounts.asset.key)?;

    let collection_config_bump = CollectionConfigV1::check_pda_derivation(
        ctx.accounts.collection_config,
        ctx.accounts.collection.key,
    )?;

    // Assert that the asset exists and is a Core asset.
    if ctx.accounts.asset.owner != &mpl_core::ID
        || ctx.accounts.asset.try_borrow_data()?[0] != MplCoreKey::AssetV1 as u8
    {
        return Err(Mpl8004IdentityError::InvalidCoreAsset.into());
    }

    // Validate the MPL Core program.
    if *ctx.accounts.mpl_core_program.key != mpl_core::ID {
        return Err(Mpl8004IdentityError::InvalidMplCoreProgram.into());
    }

    // Validate system program.
    if *ctx.accounts.system_program.key != system_program::id() {
        return Err(Mpl8004IdentityError::InvalidSystemProgram.into());
    }

    /****************************************************/
    /***************** Argument Guards ******************/
    /****************************************************/

    // Add any argument validation here.
    // Example: if args.arg1 == 0 { return Err(Mpl8004IdentityError::InvalidArgument.into()); }

    /****************************************************/
    /********************* Actions **********************/
    /****************************************************/
    // Create the agent identity account.
    AgentIdentityV1::create_account(&ctx.accounts, agent_identity_bump)?;

    // Initialize the account using zero-copy.
    // Borrow the account data mutably and cast to our struct.
    let mut data = ctx.accounts.agent_identity.try_borrow_mut_data()?;
    let agent_identity: &mut AgentIdentityV1 =
        bytemuck::from_bytes_mut(&mut data[..core::mem::size_of::<AgentIdentityV1>()]);

    agent_identity.initialize(agent_identity_bump, ctx.accounts.asset.key);

    // Create the collection config account.
    CollectionConfigV1::create_account(&ctx.accounts, collection_config_bump)?;

    // Initialize the account using zero-copy.
    // Borrow the account data mutably and cast to our struct.
    let mut data = ctx.accounts.collection_config.try_borrow_mut_data()?;
    let collection_config: &mut CollectionConfigV1 =
        bytemuck::from_bytes_mut(&mut data[..core::mem::size_of::<CollectionConfigV1>()]);

    collection_config.initialize(collection_config_bump, ctx.accounts.collection.key);

    // Check if the collection already has a LinkedAppData plugin.
    let result = fetch_wrapped_external_plugin_adapter::<BaseCollectionV1>(
        ctx.accounts.collection,
        None,
        &ExternalPluginAdapterKey::LinkedAppData(PluginAuthority::Address {
            address: *ctx.accounts.collection_config.key,
        }),
    );

    // If the collection already has a LinkedAppData plugin, move on, otherwise create it.
    if result.is_err() {
        AddCollectionExternalPluginAdapterV1Cpi {
            __program: ctx.accounts.mpl_core_program,
            collection: ctx.accounts.collection,
            payer: ctx.accounts.payer,
            authority: ctx.accounts.authority,
            system_program: ctx.accounts.system_program,
            log_wrapper: None,
            __args: AddCollectionExternalPluginAdapterV1InstructionArgs {
                init_info: ExternalPluginAdapterInitInfo::LinkedAppData(LinkedAppDataInitInfo {
                    data_authority: PluginAuthority::Address {
                        address: *ctx.accounts.collection_config.key,
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
