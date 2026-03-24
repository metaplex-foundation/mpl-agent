#![cfg(feature = "test-sbf")]

mod setup;

use mpl_agent_identity::{
    accounts::AgentIdentityV2, errors::MplAgentIdentityError,
    instructions::RegisterIdentityV1Builder, types::Key,
};
use solana_program_test::tokio;
use solana_sdk::{signature::Signer, transaction::Transaction};

/// Equivalent of JS test: identity/register.test.ts - "it can register an asset"
#[tokio::test]
async fn register_identity() {
    let mut context = setup::setup().start_with_context().await;

    // Create the collection and asset.
    let (collection, asset) = setup::create_collection_and_asset(&mut context).await;

    // Derive the agent identity PDA.
    let (agent_identity_pda, expected_bump) = AgentIdentityV2::find_pda(&asset);

    // When we register the asset.
    setup::register_identity(&mut context, asset, collection).await;

    // Then there's an Agent Identity PDA with the correct data.
    let account = context
        .banks_client
        .get_account(agent_identity_pda)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(account.data.len(), AgentIdentityV2::LEN);

    let agent_identity = AgentIdentityV2::from_bytes(&account.data).unwrap();
    assert_eq!(agent_identity.key, Key::AgentIdentityV2);
    assert_eq!(agent_identity.bump, expected_bump);
    assert_eq!(agent_identity.asset, asset);
}

#[tokio::test]
async fn cannot_register_identity_twice() {
    let mut context = setup::setup().start_with_context().await;

    let (collection, asset) = setup::create_collection_and_asset(&mut context).await;

    // First registration succeeds.
    setup::register_identity(&mut context, asset, collection).await;

    // Second registration should fail.
    let (agent_identity_pda, _) = AgentIdentityV2::find_pda(&asset);

    let ix = RegisterIdentityV1Builder::new()
        .agent_identity(agent_identity_pda)
        .asset(asset)
        .collection(Some(collection))
        .payer(context.payer.pubkey())
        .agent_registration_uri("https://example.com/agent2.json".to_string())
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

    setup::assert_custom_error(
        err,
        MplAgentIdentityError::AgentIdentityAlreadyRegistered as u32,
    );
}
