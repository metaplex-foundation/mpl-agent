#![cfg(feature = "test-sbf")]

use mpl_agent_identity::{accounts::AgentIdentityV2, instructions::RegisterIdentityV1Builder};
use mpl_agent_tools::{
    accounts::{ExecutionDelegateRecordV1, ExecutiveProfileV1},
    errors::MplAgentToolsError,
    instructions::{
        DelegateExecutionV1Builder, RegisterExecutiveV1Builder, RevokeExecutionV1Builder,
    },
    types::Key,
};
use mpl_core::instructions::{CreateCollectionV1Builder, CreateV1Builder};
use solana_program::instruction::InstructionError;
use solana_program_test::{tokio, BanksClientError, ProgramTest};
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::{Transaction, TransactionError},
};

const MPL_CORE_ID: Pubkey = solana_program::pubkey!("CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d");
const SPL_NOOP_ID: Pubkey = solana_program::pubkey!("noopb9bkMVfRPU8AsbpTUg8AQkHtKwMYZiFUjNRtMmV");

fn assert_custom_error(error: BanksClientError, expected_code: u32) {
    match error.unwrap() {
        TransactionError::InstructionError(_, InstructionError::Custom(code)) => {
            assert_eq!(code, expected_code);
        }
        err => panic!("Expected InstructionError::Custom({expected_code}), got: {err:?}"),
    }
}

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

async fn delegate_execution(
    context: &mut solana_program_test::ProgramTestContext,
    executive_profile_pda: Pubkey,
    asset: Pubkey,
    agent_identity_pda: Pubkey,
) -> Pubkey {
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

    delegate_record_pda
}

/// Owner delegates then revokes. The delegate record account should be closed.
#[tokio::test]
async fn revoke_execution_by_owner() {
    let mut context = setup().start_with_context().await;

    let (collection, asset) = create_collection_and_asset(&mut context).await;
    let agent_identity_pda = register_identity(&mut context, asset, collection).await;
    let executive_profile_pda = register_executive(&mut context, None).await;
    let delegate_record_pda = delegate_execution(
        &mut context,
        executive_profile_pda,
        asset,
        agent_identity_pda,
    )
    .await;

    // Verify the delegate record exists.
    let account = context
        .banks_client
        .get_account(delegate_record_pda)
        .await
        .unwrap();
    assert!(account.is_some(), "Delegate record should exist");

    // Revoke as the owner (payer is the owner).
    let ix = RevokeExecutionV1Builder::new()
        .execution_delegate_record(delegate_record_pda)
        .agent_asset(asset)
        .destination(context.payer.pubkey())
        .payer(context.payer.pubkey())
        .instruction();

    let recent_blockhash = context.banks_client.get_latest_blockhash().await.unwrap();
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        recent_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // The delegate record should be closed (account no longer exists).
    let account = context
        .banks_client
        .get_account(delegate_record_pda)
        .await
        .unwrap();
    assert!(account.is_none(), "Delegate record should be closed");
}

/// Owner delegates to a separate executive, then the executive authority revokes.
#[tokio::test]
async fn revoke_execution_by_executive() {
    let mut context = setup().start_with_context().await;

    let (collection, asset) = create_collection_and_asset(&mut context).await;
    let agent_identity_pda = register_identity(&mut context, asset, collection).await;

    // Create a separate executive authority.
    let executive_authority = Keypair::new();
    let executive_profile_pda = register_executive(&mut context, Some(&executive_authority)).await;
    let delegate_record_pda = delegate_execution(
        &mut context,
        executive_profile_pda,
        asset,
        agent_identity_pda,
    )
    .await;

    // Revoke as the executive authority.
    let ix = RevokeExecutionV1Builder::new()
        .execution_delegate_record(delegate_record_pda)
        .agent_asset(asset)
        .destination(context.payer.pubkey())
        .payer(context.payer.pubkey())
        .authority(Some(executive_authority.pubkey()))
        .instruction();

    let recent_blockhash = context.banks_client.get_latest_blockhash().await.unwrap();
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &executive_authority],
        recent_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // The delegate record should be closed.
    let account = context
        .banks_client
        .get_account(delegate_record_pda)
        .await
        .unwrap();
    assert!(account.is_none(), "Delegate record should be closed");
}

/// Attempt to revoke a non-existent delegate record should fail.
#[tokio::test]
async fn cannot_revoke_uninitialized_record() {
    let mut context = setup().start_with_context().await;

    let (collection, asset) = create_collection_and_asset(&mut context).await;
    register_identity(&mut context, asset, collection).await;
    let executive_profile_pda = register_executive(&mut context, None).await;

    // Derive the delegate record PDA but don't delegate.
    let (delegate_record_pda, _) =
        ExecutionDelegateRecordV1::find_pda(&executive_profile_pda, &asset);

    let ix = RevokeExecutionV1Builder::new()
        .execution_delegate_record(delegate_record_pda)
        .agent_asset(asset)
        .destination(context.payer.pubkey())
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
        MplAgentToolsError::ExecutionDelegateRecordMustBeInitialized as u32,
    );
}

/// A third party (neither owner nor executive) tries to revoke — should fail.
#[tokio::test]
async fn cannot_revoke_as_unauthorized_party() {
    let mut context = setup().start_with_context().await;

    let (collection, asset) = create_collection_and_asset(&mut context).await;
    let agent_identity_pda = register_identity(&mut context, asset, collection).await;

    let executive_authority = Keypair::new();
    let executive_profile_pda = register_executive(&mut context, Some(&executive_authority)).await;
    let delegate_record_pda = delegate_execution(
        &mut context,
        executive_profile_pda,
        asset,
        agent_identity_pda,
    )
    .await;

    // A random unauthorized party tries to revoke.
    let attacker = Keypair::new();

    let ix = RevokeExecutionV1Builder::new()
        .execution_delegate_record(delegate_record_pda)
        .agent_asset(asset)
        .destination(context.payer.pubkey())
        .payer(context.payer.pubkey())
        .authority(Some(attacker.pubkey()))
        .instruction();

    let recent_blockhash = context.banks_client.get_latest_blockhash().await.unwrap();
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &attacker],
        recent_blockhash,
    );
    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    assert_custom_error(err, MplAgentToolsError::UnauthorizedRevoke as u32);
}

/// Passing a mismatched agent_asset (different from the one in the record) should fail.
#[tokio::test]
async fn cannot_revoke_with_mismatched_agent_asset() {
    let mut context = setup().start_with_context().await;

    let (collection, asset) = create_collection_and_asset(&mut context).await;
    let agent_identity_pda = register_identity(&mut context, asset, collection).await;
    let executive_profile_pda = register_executive(&mut context, None).await;
    let delegate_record_pda = delegate_execution(
        &mut context,
        executive_profile_pda,
        asset,
        agent_identity_pda,
    )
    .await;

    // Create a second asset to use as the wrong agent_asset.
    let other_asset = Keypair::new();
    let create_asset_ix = mpl_core::instructions::CreateV1Builder::new()
        .asset(other_asset.pubkey())
        .collection(Some(collection))
        .payer(context.payer.pubkey())
        .name("Other Asset".to_string())
        .uri("https://example.com/other.json".to_string())
        .instruction();

    let recent_blockhash = context.banks_client.get_latest_blockhash().await.unwrap();
    let tx = Transaction::new_signed_with_payer(
        &[create_asset_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &other_asset],
        recent_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Try to revoke with the delegate_record for `asset` but pass `other_asset`.
    let ix = RevokeExecutionV1Builder::new()
        .execution_delegate_record(delegate_record_pda)
        .agent_asset(other_asset.pubkey())
        .destination(context.payer.pubkey())
        .payer(context.payer.pubkey())
        .instruction();

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
        MplAgentToolsError::InvalidExecutionDelegateRecordDerivation as u32,
    );
}

/// Owner delegates, revokes, then delegates again — second delegation should succeed.
#[tokio::test]
async fn can_redelegate_after_revoke() {
    let mut context = setup().start_with_context().await;

    let (collection, asset) = create_collection_and_asset(&mut context).await;
    let agent_identity_pda = register_identity(&mut context, asset, collection).await;
    let executive_profile_pda = register_executive(&mut context, None).await;

    // First delegation.
    let delegate_record_pda = delegate_execution(
        &mut context,
        executive_profile_pda,
        asset,
        agent_identity_pda,
    )
    .await;

    // Revoke.
    let ix = RevokeExecutionV1Builder::new()
        .execution_delegate_record(delegate_record_pda)
        .agent_asset(asset)
        .destination(context.payer.pubkey())
        .payer(context.payer.pubkey())
        .instruction();

    let recent_blockhash = context.banks_client.get_latest_blockhash().await.unwrap();
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        recent_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Re-delegate — should succeed.
    let (delegate_record_pda_2, expected_bump) =
        ExecutionDelegateRecordV1::find_pda(&executive_profile_pda, &asset);
    assert_eq!(delegate_record_pda, delegate_record_pda_2);

    let ix = DelegateExecutionV1Builder::new()
        .executive_profile(executive_profile_pda)
        .agent_asset(asset)
        .agent_identity(agent_identity_pda)
        .execution_delegate_record(delegate_record_pda_2)
        .payer(context.payer.pubkey())
        .instruction();

    let recent_blockhash = context.banks_client.get_latest_blockhash().await.unwrap();
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        recent_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Verify the new delegate record is correct.
    let account = context
        .banks_client
        .get_account(delegate_record_pda_2)
        .await
        .unwrap()
        .unwrap();

    let delegate_record = ExecutionDelegateRecordV1::from_bytes(&account.data).unwrap();
    assert_eq!(delegate_record.key, Key::ExecutionDelegateRecordV1);
    assert_eq!(delegate_record.bump, expected_bump);
    assert_eq!(delegate_record.executive_profile, executive_profile_pda);
    assert_eq!(delegate_record.agent_asset, asset);
}
