#![cfg(feature = "test-sbf")]

use mpl_agent_identity::{accounts::AgentIdentityV2, instructions::RegisterIdentityV1Builder};
use mpl_agent_tools::{
    accounts::{ExecutionDelegateRecordV1, ExecutiveProfileV1},
    errors::MplAgentToolsError,
    instructions::{DelegateExecutionV1Builder, RegisterExecutiveV1Builder},
    types::Key,
};
use mpl_core::instructions::{CreateCollectionV1Builder, CreateV1Builder, ExecuteV1Builder};
use solana_program::instruction::InstructionError;
use solana_program_test::{tokio, BanksClientError, ProgramTest};
use solana_sdk::{
    instruction::AccountMeta,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::{Transaction, TransactionError},
};

fn assert_custom_error(error: BanksClientError, expected_code: u32) {
    match error.unwrap() {
        TransactionError::InstructionError(_, InstructionError::Custom(code)) => {
            assert_eq!(code, expected_code);
        }
        err => panic!("Expected InstructionError::Custom({expected_code}), got: {err:?}"),
    }
}

const MPL_CORE_ID: Pubkey = solana_program::pubkey!("CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d");
const SPL_NOOP_ID: Pubkey = solana_program::pubkey!("noopb9bkMVfRPU8AsbpTUg8AQkHtKwMYZiFUjNRtMmV");
const SYSTEM_PROGRAM_ID: Pubkey = solana_program::pubkey!("11111111111111111111111111111111");

fn setup() -> ProgramTest {
    let mut program_test = ProgramTest::new("mpl_agent_tools_program", mpl_agent_tools::ID, None);
    program_test.add_program("mpl_agent_identity_program", mpl_agent_identity::ID, None);
    program_test.add_program("mpl_core", MPL_CORE_ID, None);
    program_test.add_program("spl_noop", SPL_NOOP_ID, None);
    program_test
}

async fn create_collection_and_asset(
    context: &mut solana_program_test::ProgramTestContext,
) -> (Pubkey, Pubkey) {
    let collection = Keypair::new();
    let asset = Keypair::new();

    // Create collection.
    let create_collection_ix = CreateCollectionV1Builder::new()
        .collection(collection.pubkey())
        .payer(context.payer.pubkey())
        .name("Test Collection".to_string())
        .uri("https://example.com/collection.json".to_string())
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[create_collection_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &collection],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Create asset in collection.
    let create_asset_ix = CreateV1Builder::new()
        .asset(asset.pubkey())
        .collection(Some(collection.pubkey()))
        .payer(context.payer.pubkey())
        .name("Test Asset".to_string())
        .uri("https://example.com/asset.json".to_string())
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[create_asset_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &asset],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    (collection.pubkey(), asset.pubkey())
}

async fn register_identity(
    context: &mut solana_program_test::ProgramTestContext,
    asset: Pubkey,
    collection: Pubkey,
) -> Pubkey {
    let (agent_identity_pda, _) = AgentIdentityV2::find_pda(&asset);

    let ix = RegisterIdentityV1Builder::new()
        .agent_identity(agent_identity_pda)
        .asset(asset)
        .collection(Some(collection))
        .payer(context.payer.pubkey())
        .agent_registration_uri("https://example.com/agent.json".to_string())
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    agent_identity_pda
}

async fn register_executive(
    context: &mut solana_program_test::ProgramTestContext,
    authority: Option<&Keypair>,
) -> Pubkey {
    let authority_pubkey = authority
        .map(|k| k.pubkey())
        .unwrap_or(context.payer.pubkey());

    let (executive_profile_pda, _) = ExecutiveProfileV1::find_pda(&authority_pubkey);

    let ix = RegisterExecutiveV1Builder::new()
        .executive_profile(executive_profile_pda)
        .payer(context.payer.pubkey())
        .authority(authority.map(|k| k.pubkey()))
        .instruction();

    let mut signers: Vec<&Keypair> = vec![&context.payer];
    if let Some(auth) = authority {
        signers.push(auth);
    }

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &signers,
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    executive_profile_pda
}

/// Equivalent of JS test: tools/delegateExecution.test.ts - "it can delegate execution to an executor"
#[tokio::test]
async fn delegate_execution() {
    let mut context = setup().start_with_context().await;

    // Create a collection and asset.
    let (collection, asset) = create_collection_and_asset(&mut context).await;

    // Register the asset's identity.
    let agent_identity_pda = register_identity(&mut context, asset, collection).await;

    // Register an executor profile (default authority = payer).
    let executive_profile_pda = register_executive(&mut context, None).await;

    // Derive the delegate record PDA.
    let (delegate_record_pda, expected_bump) =
        ExecutionDelegateRecordV1::find_pda(&executive_profile_pda, &asset);

    // When we delegate execution to the executor for the agent asset.
    let ix = DelegateExecutionV1Builder::new()
        .executive_profile(executive_profile_pda)
        .agent_asset(asset)
        .agent_identity(agent_identity_pda)
        .execution_delegate_record(delegate_record_pda)
        .payer(context.payer.pubkey())
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the execution delegate record is created.
    let account = context
        .banks_client
        .get_account(delegate_record_pda)
        .await
        .unwrap()
        .unwrap();

    let delegate_record = ExecutionDelegateRecordV1::from_bytes(&account.data).unwrap();
    assert_eq!(delegate_record.key, Key::ExecutionDelegateRecordV1);
    assert_eq!(delegate_record.bump, expected_bump);
    assert_eq!(delegate_record.executive_profile, executive_profile_pda);
    assert_eq!(delegate_record.authority, context.payer.pubkey());
    assert_eq!(delegate_record.agent_asset, asset);
}

/// Equivalent of JS test: tools/delegateExecution.test.ts -
/// "it can execute as the execution delegate without the owner signing"
#[tokio::test]
async fn execute_as_delegate_without_owner() {
    let mut context = setup().start_with_context().await;

    // Create a collection and asset (owner = payer).
    let (collection, asset) = create_collection_and_asset(&mut context).await;

    // Register identity on the asset.
    register_identity(&mut context, asset, collection).await;

    // Create a separate authority for the executive (different from asset owner).
    let executive_authority = Keypair::new();
    let executive_profile_pda = register_executive(&mut context, Some(&executive_authority)).await;

    // Owner delegates execution to the executive for the agent asset.
    let agent_identity_pda = AgentIdentityV2::find_pda(&asset).0;
    let (delegate_record_pda, _) =
        ExecutionDelegateRecordV1::find_pda(&executive_profile_pda, &asset);

    let ix = DelegateExecutionV1Builder::new()
        .executive_profile(executive_profile_pda)
        .agent_asset(asset)
        .agent_identity(agent_identity_pda)
        .execution_delegate_record(delegate_record_pda)
        .payer(context.payer.pubkey())
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Execute as the delegate (NOT the owner).
    // The delegate record must be the first remaining account for the
    // AgentIdentity plugin's validate_execute to verify it. Core strips it
    // before the inner CPI.
    let (asset_signer_pda, _) = Pubkey::find_program_address(
        &["mpl-core-execute".as_bytes(), asset.as_ref()],
        &MPL_CORE_ID,
    );

    let execute_ix = ExecuteV1Builder::new()
        .asset(asset)
        .collection(Some(collection))
        .asset_signer(asset_signer_pda)
        .payer(context.payer.pubkey(), true)
        .authority(Some(executive_authority.pubkey()))
        .program_id(SPL_NOOP_ID)
        .instruction_data(vec![])
        .add_remaining_account(AccountMeta::new_readonly(delegate_record_pda, false))
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[execute_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &executive_authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();
}

/// Equivalent of JS test: tools/delegateExecution.test.ts -
/// "it can transfer SOL via delegate execution"
#[tokio::test]
async fn transfer_sol_via_delegate_execution() {
    let mut context = setup().start_with_context().await;

    let (collection, asset) = create_collection_and_asset(&mut context).await;
    register_identity(&mut context, asset, collection).await;

    let executive_authority = Keypair::new();
    let executive_profile_pda = register_executive(&mut context, Some(&executive_authority)).await;

    let agent_identity_pda = AgentIdentityV2::find_pda(&asset).0;
    let (delegate_record_pda, _) =
        ExecutionDelegateRecordV1::find_pda(&executive_profile_pda, &asset);

    let ix = DelegateExecutionV1Builder::new()
        .executive_profile(executive_profile_pda)
        .agent_asset(asset)
        .agent_identity(agent_identity_pda)
        .execution_delegate_record(delegate_record_pda)
        .payer(context.payer.pubkey())
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Fund the asset signer PDA so it has SOL to transfer.
    let (asset_signer_pda, _) = Pubkey::find_program_address(
        &["mpl-core-execute".as_bytes(), asset.as_ref()],
        &MPL_CORE_ID,
    );

    // System Transfer instruction data: u32 type (2) + u64 lamports.
    let mut fund_data = vec![0u8; 12];
    fund_data[..4].copy_from_slice(&2u32.to_le_bytes()); // Transfer = 2
    fund_data[4..12].copy_from_slice(&1_000_000_000u64.to_le_bytes()); // 1 SOL
    let fund_ix = solana_sdk::instruction::Instruction {
        program_id: SYSTEM_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(context.payer.pubkey(), true),
            AccountMeta::new(asset_signer_pda, false),
        ],
        data: fund_data,
    };

    let tx = Transaction::new_signed_with_payer(
        &[fund_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Execute a system transfer as the delegate.
    // The delegate record is the first remaining account for the AgentIdentity
    // plugin check. Core strips it before the inner CPI so the system transfer
    // correctly receives [assetSigner, recipient].
    let recipient = Keypair::new();

    // System Transfer instruction data: u32 type (2) + u64 lamports
    let mut transfer_data = vec![0u8; 12];
    transfer_data[..4].copy_from_slice(&2u32.to_le_bytes()); // Transfer = 2
    transfer_data[4..12].copy_from_slice(&500_000_000u64.to_le_bytes()); // 0.5 SOL

    let execute_ix = ExecuteV1Builder::new()
        .asset(asset)
        .collection(Some(collection))
        .asset_signer(asset_signer_pda)
        .payer(context.payer.pubkey(), true)
        .authority(Some(executive_authority.pubkey()))
        .program_id(SYSTEM_PROGRAM_ID)
        .instruction_data(transfer_data)
        .add_remaining_account(AccountMeta::new_readonly(delegate_record_pda, false))
        .add_remaining_account(AccountMeta::new(asset_signer_pda, false))
        .add_remaining_account(AccountMeta::new(recipient.pubkey(), false))
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[execute_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &executive_authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Verify the recipient received the SOL.
    let recipient_account = context
        .banks_client
        .get_account(recipient.pubkey())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(recipient_account.lamports, 500_000_000);
}

/// Equivalent of JS test: tools/delegateExecution.test.ts -
/// "the owner can still execute even with the AgentIdentity plugin"
#[tokio::test]
async fn owner_can_still_execute() {
    let mut context = setup().start_with_context().await;

    // Create a collection and asset (owner = payer).
    let (collection, asset) = create_collection_and_asset(&mut context).await;

    // Register identity on the asset (adds AgentIdentity plugin with execute lifecycle check).
    register_identity(&mut context, asset, collection).await;

    // The owner can still execute without any delegate record.
    // The AgentIdentity plugin abstains when no valid delegate record is at index 7,
    // and the owner is approved by Core's default ownership check.
    let (asset_signer_pda, _) = Pubkey::find_program_address(
        &["mpl-core-execute".as_bytes(), asset.as_ref()],
        &MPL_CORE_ID,
    );

    let execute_ix = ExecuteV1Builder::new()
        .asset(asset)
        .collection(Some(collection))
        .asset_signer(asset_signer_pda)
        .payer(context.payer.pubkey(), true)
        .program_id(SPL_NOOP_ID)
        .instruction_data(vec![])
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[execute_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();
}

/// Equivalent of JS test: tools/delegateExecution.test.ts -
/// "it cannot execute without a valid execution delegate"
#[tokio::test]
async fn cannot_execute_without_delegate() {
    let mut context = setup().start_with_context().await;

    // Create a collection and asset (owner = payer).
    let (collection, asset) = create_collection_and_asset(&mut context).await;

    // Register identity on the asset.
    register_identity(&mut context, asset, collection).await;

    // Create a non-owner signer who has NOT been delegated execution.
    let attacker = Keypair::new();

    // Attempt to execute as the attacker without a delegate record.
    let (asset_signer_pda, _) = Pubkey::find_program_address(
        &["mpl-core-execute".as_bytes(), asset.as_ref()],
        &MPL_CORE_ID,
    );

    let execute_ix = ExecuteV1Builder::new()
        .asset(asset)
        .collection(Some(collection))
        .asset_signer(asset_signer_pda)
        .payer(context.payer.pubkey(), true)
        .authority(Some(attacker.pubkey()))
        .program_id(SPL_NOOP_ID)
        .instruction_data(vec![])
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[execute_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &attacker],
        context.last_blockhash,
    );
    let result = context.banks_client.process_transaction(tx).await;

    // The execute should fail (error 0x1a = NoApprovals in MPL Core).
    assert!(
        result.is_err(),
        "Expected execute to fail for non-delegate attacker"
    );
}

/// Equivalent of JS test: tools/delegateExecution.test.ts -
/// "it cannot delegate execution if not the asset owner"
#[tokio::test]
async fn cannot_delegate_if_not_owner() {
    let mut context = setup().start_with_context().await;

    let (collection, asset) = create_collection_and_asset(&mut context).await;
    register_identity(&mut context, asset, collection).await;

    // Register an executive profile for a separate authority.
    let executive_authority = Keypair::new();
    let executive_profile_pda = register_executive(&mut context, Some(&executive_authority)).await;

    // A non-owner tries to delegate execution.
    let non_owner = Keypair::new();
    let agent_identity_pda = AgentIdentityV2::find_pda(&asset).0;
    let (delegate_record_pda, _) =
        ExecutionDelegateRecordV1::find_pda(&executive_profile_pda, &asset);

    let ix = DelegateExecutionV1Builder::new()
        .executive_profile(executive_profile_pda)
        .agent_asset(asset)
        .agent_identity(agent_identity_pda)
        .execution_delegate_record(delegate_record_pda)
        .payer(context.payer.pubkey())
        .authority(Some(non_owner.pubkey()))
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &non_owner],
        context.last_blockhash,
    );
    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    assert_custom_error(
        err,
        MplAgentToolsError::AssetOwnerMustBeTheOneToDelegateExecution as u32,
    );
}

/// Equivalent of JS test: tools/delegateExecution.test.ts -
/// "it cannot delegate execution without a registered identity"
#[tokio::test]
async fn cannot_delegate_without_identity() {
    let mut context = setup().start_with_context().await;

    // Create a collection and asset but do NOT register identity.
    let (_collection, asset) = create_collection_and_asset(&mut context).await;

    // Register an executive profile.
    let executive_profile_pda = register_executive(&mut context, None).await;

    // Use the agent identity PDA (it won't be initialized).
    let agent_identity_pda = AgentIdentityV2::find_pda(&asset).0;
    let (delegate_record_pda, _) =
        ExecutionDelegateRecordV1::find_pda(&executive_profile_pda, &asset);

    let ix = DelegateExecutionV1Builder::new()
        .executive_profile(executive_profile_pda)
        .agent_asset(asset)
        .agent_identity(agent_identity_pda)
        .execution_delegate_record(delegate_record_pda)
        .payer(context.payer.pubkey())
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    assert_custom_error(err, MplAgentToolsError::AgentIdentityNotRegistered as u32);
}

/// Equivalent of JS test: tools/delegateExecution.test.ts -
/// "it cannot delegate execution twice to the same executive"
#[tokio::test]
async fn cannot_delegate_twice() {
    let mut context = setup().start_with_context().await;

    let (collection, asset) = create_collection_and_asset(&mut context).await;
    register_identity(&mut context, asset, collection).await;

    let executive_profile_pda = register_executive(&mut context, None).await;

    let agent_identity_pda = AgentIdentityV2::find_pda(&asset).0;
    let (delegate_record_pda, _) =
        ExecutionDelegateRecordV1::find_pda(&executive_profile_pda, &asset);

    // First delegation succeeds.
    let ix = DelegateExecutionV1Builder::new()
        .executive_profile(executive_profile_pda)
        .agent_asset(asset)
        .agent_identity(agent_identity_pda)
        .execution_delegate_record(delegate_record_pda)
        .payer(context.payer.pubkey())
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Second delegation to the same executive for the same asset fails.
    let ix = DelegateExecutionV1Builder::new()
        .executive_profile(executive_profile_pda)
        .agent_asset(asset)
        .agent_identity(agent_identity_pda)
        .execution_delegate_record(delegate_record_pda)
        .payer(context.payer.pubkey())
        .instruction();

    // Need a fresh blockhash since the prior tx consumed the old one for this payer+ix combo.
    let recent_blockhash = context.banks_client.get_latest_blockhash().await.unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        recent_blockhash,
    );
    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    assert_custom_error(
        err,
        MplAgentToolsError::ExecutionDelegateRecordMustBeUninitialized as u32,
    );
}

/// Equivalent of JS test: tools/delegateExecution.test.ts -
/// "it cannot delegate execution with an uninitialized executive profile"
#[tokio::test]
async fn cannot_delegate_with_uninitialized_profile() {
    let mut context = setup().start_with_context().await;

    let (collection, asset) = create_collection_and_asset(&mut context).await;
    register_identity(&mut context, asset, collection).await;

    // Use a PDA for an executive profile that was never registered.
    let fake_authority = Keypair::new();
    let (uninitialized_profile, _) = ExecutiveProfileV1::find_pda(&fake_authority.pubkey());

    let agent_identity_pda = AgentIdentityV2::find_pda(&asset).0;
    let (delegate_record_pda, _) =
        ExecutionDelegateRecordV1::find_pda(&uninitialized_profile, &asset);

    let ix = DelegateExecutionV1Builder::new()
        .executive_profile(uninitialized_profile)
        .agent_asset(asset)
        .agent_identity(agent_identity_pda)
        .execution_delegate_record(delegate_record_pda)
        .payer(context.payer.pubkey())
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    assert_custom_error(
        err,
        MplAgentToolsError::ExecutiveProfileMustBeUninitialized as u32,
    );
}
