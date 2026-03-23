#![cfg(feature = "test-sbf")]

use mpl_agent_validation::{
    accounts::AgentValidationV1, instructions::RegisterValidationV1Builder, types::Key,
};
use mpl_core::instructions::{CreateCollectionV1Builder, CreateV1Builder};
use solana_program_test::{tokio, ProgramTest};
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

const MPL_CORE_ID: Pubkey = solana_program::pubkey!("CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d");

fn setup() -> ProgramTest {
    let mut program_test = ProgramTest::new(
        "mpl_agent_validation_program",
        mpl_agent_validation::ID,
        None,
    );
    program_test.add_program("mpl_core", MPL_CORE_ID, None);
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

/// Equivalent of JS test: validation/register.test.ts - "it can register an asset"
#[tokio::test]
async fn register_validation() {
    let mut context = setup().start_with_context().await;

    // Create the collection and asset.
    let (collection, asset) = create_collection_and_asset(&mut context).await;

    // Derive the agent validation PDA.
    let (agent_validation_pda, expected_bump) = AgentValidationV1::find_pda(&asset);

    // When we register the asset.
    let ix = RegisterValidationV1Builder::new()
        .agent_validation(agent_validation_pda)
        .asset(asset)
        .collection(Some(collection))
        .payer(context.payer.pubkey())
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then there's an Agent Validation PDA with the correct data.
    let account = context
        .banks_client
        .get_account(agent_validation_pda)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(account.data.len(), AgentValidationV1::LEN);

    let agent_validation = AgentValidationV1::from_bytes(&account.data).unwrap();
    assert_eq!(agent_validation.key, Key::AgentValidationV1);
    assert_eq!(agent_validation.bump, expected_bump);
    assert_eq!(agent_validation.asset, asset);
}
