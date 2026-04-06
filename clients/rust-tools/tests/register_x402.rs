#![cfg(feature = "test-sbf")]

use mpl_agent_tools::{
    accounts::X402EndpointV1,
    errors::MplAgentToolsError,
    instructions::{RegisterX402V1Builder, RegisterX402V1InstructionArgs},
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
    program_test.add_program("mpl_core", MPL_CORE_ID, None);
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

/// It can register an x402 endpoint for an agent asset.
#[tokio::test]
async fn register_x402_endpoint() {
    let mut context = setup().start_with_context().await;

    let (_collection, asset) = create_collection_and_asset(&mut context).await;

    let url = "https://example.com/x402/pay";
    let (x402_endpoint_pda, _) = X402EndpointV1::find_pda(&asset);

    let ix = RegisterX402V1Builder::new()
        .x402_endpoint(x402_endpoint_pda)
        .agent_asset(asset)
        .payer(context.payer.pubkey())
        .url(url.to_string())
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Verify the account was created with correct data.
    let account = context
        .banks_client
        .get_account(x402_endpoint_pda)
        .await
        .unwrap()
        .unwrap();

    let endpoint = X402EndpointV1::from_bytes(&account.data).unwrap();
    assert_eq!(endpoint.key, Key::X402EndpointV1);
    assert_eq!(endpoint.asset, asset);
    assert_eq!(endpoint.authority, context.payer.pubkey());
    assert_eq!(endpoint.url, url);
}

/// It can register an x402 endpoint with a custom authority (asset owner).
#[tokio::test]
async fn register_x402_endpoint_custom_authority() {
    let mut context = setup().start_with_context().await;

    // Create asset owned by a custom authority.
    let collection = Keypair::new();
    let asset = Keypair::new();
    let authority = Keypair::new();

    // Fund the authority.
    let fund_ix = solana_sdk::system_instruction::transfer(
        &context.payer.pubkey(),
        &authority.pubkey(),
        1_000_000_000,
    );
    let tx = Transaction::new_signed_with_payer(
        &[fund_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

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

    // Create asset with custom owner.
    let create_asset_ix = CreateV1Builder::new()
        .asset(asset.pubkey())
        .collection(Some(collection.pubkey()))
        .payer(context.payer.pubkey())
        .owner(Some(authority.pubkey()))
        .name("Test Asset".to_string())
        .uri("https://example.com/asset.json".to_string())
        .instruction();
    let recent_blockhash = context.banks_client.get_latest_blockhash().await.unwrap();
    let tx = Transaction::new_signed_with_payer(
        &[create_asset_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &asset],
        recent_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    let url = "https://api.agent.example.com/x402";
    let (x402_endpoint_pda, _) = X402EndpointV1::find_pda(&asset.pubkey());

    let ix = RegisterX402V1Builder::new()
        .x402_endpoint(x402_endpoint_pda)
        .agent_asset(asset.pubkey())
        .payer(context.payer.pubkey())
        .authority(Some(authority.pubkey()))
        .url(url.to_string())
        .instruction();

    let recent_blockhash = context.banks_client.get_latest_blockhash().await.unwrap();
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &authority],
        recent_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    let account = context
        .banks_client
        .get_account(x402_endpoint_pda)
        .await
        .unwrap()
        .unwrap();

    let endpoint = X402EndpointV1::from_bytes(&account.data).unwrap();
    assert_eq!(endpoint.key, Key::X402EndpointV1);
    assert_eq!(endpoint.authority, authority.pubkey());
    assert_eq!(endpoint.url, url);
}

/// It cannot register an x402 endpoint if not the asset owner.
#[tokio::test]
async fn cannot_register_x402_if_not_owner() {
    let mut context = setup().start_with_context().await;

    let (_collection, asset) = create_collection_and_asset(&mut context).await;

    let non_owner = Keypair::new();
    let url = "https://example.com/x402/pay";
    let (x402_endpoint_pda, _) = X402EndpointV1::find_pda(&asset);

    let ix = RegisterX402V1Builder::new()
        .x402_endpoint(x402_endpoint_pda)
        .agent_asset(asset)
        .payer(context.payer.pubkey())
        .authority(Some(non_owner.pubkey()))
        .url(url.to_string())
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

    assert_custom_error(err, MplAgentToolsError::AssetOwnerMustRegisterX402 as u32);
}

/// It cannot register an x402 endpoint twice for the same asset.
#[tokio::test]
async fn cannot_register_x402_twice() {
    let mut context = setup().start_with_context().await;

    let (_collection, asset) = create_collection_and_asset(&mut context).await;

    let url = "https://example.com/x402/pay";
    let (x402_endpoint_pda, _) = X402EndpointV1::find_pda(&asset);

    let ix = RegisterX402V1Builder::new()
        .x402_endpoint(x402_endpoint_pda)
        .agent_asset(asset)
        .payer(context.payer.pubkey())
        .url(url.to_string())
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Second registration should fail.
    let ix = RegisterX402V1Builder::new()
        .x402_endpoint(x402_endpoint_pda)
        .agent_asset(asset)
        .payer(context.payer.pubkey())
        .url(url.to_string())
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
        MplAgentToolsError::X402EndpointMustBeUninitialized as u32,
    );
}
