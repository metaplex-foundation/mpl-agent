#![cfg(feature = "test-sbf")]

use mpl_agent_identity::{
    accounts::AgentIdentityV1, instructions::RegisterIdentityV1Builder, types::Key,
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
    let mut program_test =
        ProgramTest::new("mpl_agent_identity_program", mpl_agent_identity::ID, None);
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

/// Equivalent of JS test: identity/register.test.ts - "it can register an asset"
#[tokio::test]
async fn register_identity() {
    let mut context = setup().start_with_context().await;

    // Create the collection and asset.
    let (collection, asset) = create_collection_and_asset(&mut context).await;

    // Derive the agent identity PDA.
    let (agent_identity_pda, expected_bump) = AgentIdentityV1::find_pda(&asset);

    let agent_registration_uri = "https://example.com/agent.json";

    // When we register the asset.
    let ix = RegisterIdentityV1Builder::new()
        .agent_identity(agent_identity_pda)
        .asset(asset)
        .collection(Some(collection))
        .payer(context.payer.pubkey())
        .agent_registration_uri(agent_registration_uri.to_string())
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then there's an Agent Identity PDA with the correct data.
    let account = context
        .banks_client
        .get_account(agent_identity_pda)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(account.data.len(), AgentIdentityV1::LEN);

    let agent_identity = AgentIdentityV1::from_bytes(&account.data).unwrap();
    assert_eq!(agent_identity.key, Key::AgentIdentityV1);
    assert_eq!(agent_identity.bump, expected_bump);
    assert_eq!(agent_identity.asset, asset);
}
