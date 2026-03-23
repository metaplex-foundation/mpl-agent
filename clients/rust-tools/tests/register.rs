#![cfg(feature = "test-sbf")]

use mpl_agent_tools::{
    accounts::ExecutiveProfileV1, instructions::RegisterExecutiveV1Builder, types::Key,
};
use solana_program_test::{tokio, ProgramTest};
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
};

fn setup() -> ProgramTest {
    ProgramTest::new("mpl_agent_tools_program", mpl_agent_tools::ID, None)
}

/// Equivalent of JS test: tools/register.test.ts - "it can register an executive profile"
#[tokio::test]
async fn register_executive_profile() {
    let context = setup().start_with_context().await;

    // Derive the executive profile PDA using the payer as authority.
    let (executive_profile_pda, _) = ExecutiveProfileV1::find_pda(&context.payer.pubkey());

    // When we register the executive profile.
    let ix = RegisterExecutiveV1Builder::new()
        .executive_profile(executive_profile_pda)
        .payer(context.payer.pubkey())
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the executive profile is created with the correct data.
    let account = context
        .banks_client
        .get_account(executive_profile_pda)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(account.data.len(), ExecutiveProfileV1::LEN);

    let profile = ExecutiveProfileV1::from_bytes(&account.data).unwrap();
    assert_eq!(profile.key, Key::ExecutiveProfileV1);
    assert_eq!(profile.authority, context.payer.pubkey());
}

/// Equivalent of JS test: tools/register.test.ts - "it can register an executive profile with a custom authority"
#[tokio::test]
async fn register_executive_profile_custom_authority() {
    let context = setup().start_with_context().await;

    let authority = Keypair::new();

    // Derive the executive profile PDA using the custom authority.
    let (executive_profile_pda, _) = ExecutiveProfileV1::find_pda(&authority.pubkey());

    // When we register the executive profile with a custom authority.
    let ix = RegisterExecutiveV1Builder::new()
        .executive_profile(executive_profile_pda)
        .payer(context.payer.pubkey())
        .authority(Some(authority.pubkey()))
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the executive profile authority is the custom authority.
    let account = context
        .banks_client
        .get_account(executive_profile_pda)
        .await
        .unwrap()
        .unwrap();

    let profile = ExecutiveProfileV1::from_bytes(&account.data).unwrap();
    assert_eq!(profile.key, Key::ExecutiveProfileV1);
    assert_eq!(profile.authority, authority.pubkey());
}
