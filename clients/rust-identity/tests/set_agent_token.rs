#![cfg(feature = "test-sbf")]

mod setup;

use mpl_agent_identity::{
    accounts::{AgentIdentityV1, AgentIdentityV2},
    errors::MplAgentIdentityError,
    instructions::SetAgentTokenV1Builder,
    types::Key,
};
use mpl_core::instructions::ExecuteV1Builder;
use solana_program_test::tokio;
use solana_sdk::{
    account::AccountSharedData,
    instruction::AccountMeta,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

use crate::setup::create_genesis_account;

const SYSTEM_PROGRAM_ID: Pubkey = solana_program::pubkey!("11111111111111111111111111111111");

/// Build an ExecuteV1 instruction that wraps a SetAgentTokenV1 CPI.
fn build_set_agent_token_via_execute(
    asset: Pubkey,
    collection: Pubkey,
    agent_identity_pda: Pubkey,
    genesis_account: Pubkey,
    payer: Pubkey,
) -> solana_sdk::instruction::Instruction {
    let (asset_signer_pda, _) = Pubkey::find_program_address(
        &["mpl-core-execute".as_bytes(), asset.as_ref()],
        &setup::MPL_CORE_ID,
    );

    // SetAgentTokenV1 instruction data: discriminator 1 + 7 bytes padding.
    let instruction_data = vec![1, 0, 0, 0, 0, 0, 0, 0];

    ExecuteV1Builder::new()
        .asset(asset)
        .collection(Some(collection))
        .asset_signer(asset_signer_pda)
        .payer(payer, true)
        .program_id(mpl_agent_identity::ID)
        .instruction_data(instruction_data)
        .add_remaining_account(AccountMeta::new(agent_identity_pda, false))
        .add_remaining_account(AccountMeta::new_readonly(asset, false))
        .add_remaining_account(AccountMeta::new_readonly(genesis_account, false))
        .add_remaining_account(AccountMeta::new(payer, true))
        .add_remaining_account(AccountMeta::new_readonly(asset_signer_pda, false))
        .add_remaining_account(AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false))
        .instruction()
}

#[tokio::test]
async fn set_agent_token() {
    let mut context = setup::setup().start_with_context().await;

    let (collection, asset) = setup::create_collection_and_asset(&mut context).await;
    let agent_identity_pda = setup::register_identity(&mut context, asset, collection).await;

    let base_mint = Keypair::new().pubkey();
    let genesis_account = create_genesis_account(&mut context, base_mint, 0, 0).await;

    let ix = build_set_agent_token_via_execute(
        asset,
        collection,
        agent_identity_pda,
        genesis_account,
        context.payer.pubkey(),
    );

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Verify the agent identity has the base_mint set (NOT the genesis account address).
    let account = context
        .banks_client
        .get_account(agent_identity_pda)
        .await
        .unwrap()
        .unwrap();

    let agent_identity = AgentIdentityV2::from_bytes(&account.data).unwrap();
    assert_eq!(agent_identity.key, Key::AgentIdentityV2);
    assert_eq!(agent_identity.asset, asset);
    assert_eq!(agent_identity.agent_token, Some(base_mint));
}

#[tokio::test]
async fn cannot_set_agent_token_without_asset_signer() {
    let mut context = setup::setup().start_with_context().await;

    let (collection, asset) = setup::create_collection_and_asset(&mut context).await;
    let agent_identity_pda = setup::register_identity(&mut context, asset, collection).await;

    let base_mint = Keypair::new().pubkey();
    let genesis_account = create_genesis_account(&mut context, base_mint, 0, 0).await;

    // Call SetAgentTokenV1 directly (not via Execute), so payer is the authority.
    let ix = SetAgentTokenV1Builder::new()
        .agent_identity(agent_identity_pda)
        .asset(asset)
        .genesis_account(genesis_account)
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

    setup::assert_custom_error(
        err,
        MplAgentIdentityError::OnlyAssetSignerCanSetAgentToken as u32,
    );
}

#[tokio::test]
async fn cannot_set_agent_token_twice() {
    let mut context = setup::setup().start_with_context().await;

    let (collection, asset) = setup::create_collection_and_asset(&mut context).await;
    let agent_identity_pda = setup::register_identity(&mut context, asset, collection).await;

    let base_mint = Keypair::new().pubkey();
    let genesis_account = create_genesis_account(&mut context, base_mint, 0, 0).await;

    // First set succeeds.
    let ix = build_set_agent_token_via_execute(
        asset,
        collection,
        agent_identity_pda,
        genesis_account,
        context.payer.pubkey(),
    );

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Second set with a different genesis account should fail.
    let base_mint_2 = Keypair::new().pubkey();
    let genesis_account_2 = create_genesis_account(&mut context, base_mint_2, 0, 0).await;

    let ix = build_set_agent_token_via_execute(
        asset,
        collection,
        agent_identity_pda,
        genesis_account_2,
        context.payer.pubkey(),
    );

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

    setup::assert_custom_error(err, MplAgentIdentityError::AgentTokenAlreadySet as u32);
}

#[tokio::test]
async fn cannot_set_agent_token_with_invalid_genesis_account() {
    let mut context = setup::setup().start_with_context().await;

    let (collection, asset) = setup::create_collection_and_asset(&mut context).await;
    let agent_identity_pda = setup::register_identity(&mut context, asset, collection).await;

    // Create an account NOT owned by the Genesis program.
    let fake_account_keypair = Keypair::new();
    let fake_account_data = AccountSharedData::new(1_000_000, 136, &SYSTEM_PROGRAM_ID);
    context.set_account(&fake_account_keypair.pubkey(), &fake_account_data);

    let ix = build_set_agent_token_via_execute(
        asset,
        collection,
        agent_identity_pda,
        fake_account_keypair.pubkey(),
        context.payer.pubkey(),
    );

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

    setup::assert_custom_error(err, MplAgentIdentityError::InvalidGenesisAccount as u32);
}

#[tokio::test]
async fn cannot_set_agent_token_with_transfer_funded_genesis() {
    let mut context = setup::setup().start_with_context().await;

    let (collection, asset) = setup::create_collection_and_asset(&mut context).await;
    let agent_identity_pda = setup::register_identity(&mut context, asset, collection).await;

    // Create a genesis account with funding_mode = Transfer (1).
    let base_mint = Keypair::new().pubkey();
    let genesis_account = create_genesis_account(&mut context, base_mint, 0, 1).await;

    let ix = build_set_agent_token_via_execute(
        asset,
        collection,
        agent_identity_pda,
        genesis_account,
        context.payer.pubkey(),
    );

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

    setup::assert_custom_error(err, MplAgentIdentityError::GenesisNotMintFunded as u32);
}

#[tokio::test]
async fn cannot_set_agent_token_with_invalid_genesis_discriminator() {
    let mut context = setup::setup().start_with_context().await;

    let (collection, asset) = setup::create_collection_and_asset(&mut context).await;
    let agent_identity_pda = setup::register_identity(&mut context, asset, collection).await;

    // Create an account owned by the Genesis program but with wrong discriminator.
    let bad_genesis_keypair = Keypair::new();
    let mut data = vec![0u8; 136];
    data[0] = 99; // Wrong discriminator (should be 18)
    data[128] = 0; // funding_mode = Mint

    let rent = context.banks_client.get_rent().await.unwrap();
    let lamports = rent.minimum_balance(136);
    let mut account_data = AccountSharedData::new(lamports, 136, &setup::GENESIS_PROGRAM_ID);
    account_data.set_data_from_slice(&data);
    context.set_account(&bad_genesis_keypair.pubkey(), &account_data);

    let ix = build_set_agent_token_via_execute(
        asset,
        collection,
        agent_identity_pda,
        bad_genesis_keypair.pubkey(),
        context.payer.pubkey(),
    );

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

    setup::assert_custom_error(err, MplAgentIdentityError::InvalidGenesisAccount as u32);
}

#[tokio::test]
async fn cannot_set_agent_token_on_unregistered_identity() {
    let mut context = setup::setup().start_with_context().await;

    let (collection, asset) = setup::create_collection_and_asset(&mut context).await;

    // Don't register identity — PDA won't be initialized.
    let (agent_identity_pda, _) = AgentIdentityV2::find_pda(&asset);
    let base_mint = Keypair::new().pubkey();
    let genesis_account = create_genesis_account(&mut context, base_mint, 0, 0).await;

    let ix = build_set_agent_token_via_execute(
        asset,
        collection,
        agent_identity_pda,
        genesis_account,
        context.payer.pubkey(),
    );

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

    setup::assert_custom_error(err, MplAgentIdentityError::InvalidAgentIdentity as u32);
}

/// Downgrade an AgentIdentityV2 PDA to V1 format (40 bytes, key=1).
/// This simulates a legacy account that existed before the V2 migration.
async fn downgrade_to_v1(
    context: &mut solana_program_test::ProgramTestContext,
    agent_identity_pda: Pubkey,
    asset: Pubkey,
) {
    // Read the current V2 account to get the bump.
    let v2_account = context
        .banks_client
        .get_account(agent_identity_pda)
        .await
        .unwrap()
        .unwrap();
    let v2 = AgentIdentityV2::from_bytes(&v2_account.data).unwrap();
    let bump = v2.bump;

    // Construct V1 data: key=1 (AgentIdentityV1), bump, padding[6], asset[32]
    let mut v1_data = vec![0u8; AgentIdentityV1::LEN]; // 40 bytes
    v1_data[0] = 1; // Key::AgentIdentityV1
    v1_data[1] = bump;
    // v1_data[2..8] = padding (already zero)
    v1_data[8..40].copy_from_slice(asset.as_ref());

    let rent = context.banks_client.get_rent().await.unwrap();
    // Keep enough rent for the eventual V1->V2 realloc performed by SetAgentToken.
    let lamports = rent.minimum_balance(AgentIdentityV2::LEN);

    let mut account_data =
        AccountSharedData::new(lamports, AgentIdentityV1::LEN, &mpl_agent_identity::ID);
    account_data.set_data_from_slice(&v1_data);
    context.set_account(&agent_identity_pda, &account_data);
}

#[tokio::test]
#[ignore = "legacy V1->V2 realloc is not supported under current Solana 3 runtime"]
async fn set_agent_token_migrates_v1_to_v2() {
    let mut context = setup::setup().start_with_context().await;

    let (collection, asset) = setup::create_collection_and_asset(&mut context).await;
    let agent_identity_pda = setup::register_identity(&mut context, asset, collection).await;

    // Downgrade the PDA to V1 format to simulate a legacy account.
    downgrade_to_v1(&mut context, agent_identity_pda, asset).await;

    // Verify the discriminator is V1 now.
    let account = context
        .banks_client
        .get_account(agent_identity_pda)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(account.data[0], 1); // Key::AgentIdentityV1

    // Set agent token via Execute CPI — this should migrate V1 -> V2.
    let base_mint = Keypair::new().pubkey();
    let genesis_account = create_genesis_account(&mut context, base_mint, 0, 0).await;

    let ix = build_set_agent_token_via_execute(
        asset,
        collection,
        agent_identity_pda,
        genesis_account,
        context.payer.pubkey(),
    );

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Verify the account was migrated to V2 and the token was set.
    let account = context
        .banks_client
        .get_account(agent_identity_pda)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(account.data.len(), AgentIdentityV2::LEN);
    let agent_identity = AgentIdentityV2::from_bytes(&account.data).unwrap();
    assert_eq!(agent_identity.key, Key::AgentIdentityV2);
    assert_eq!(agent_identity.asset, asset);
    assert_eq!(agent_identity.agent_token, Some(base_mint));
}

#[tokio::test]
#[ignore = "legacy V1->V2 realloc is not supported under current Solana 3 runtime"]
async fn set_agent_token_on_v1_cannot_set_twice() {
    let mut context = setup::setup().start_with_context().await;

    let (collection, asset) = setup::create_collection_and_asset(&mut context).await;
    let agent_identity_pda = setup::register_identity(&mut context, asset, collection).await;

    // Downgrade to V1.
    downgrade_to_v1(&mut context, agent_identity_pda, asset).await;

    // First set migrates V1 -> V2 and sets the token.
    let base_mint = Keypair::new().pubkey();
    let genesis_account = create_genesis_account(&mut context, base_mint, 0, 0).await;

    let ix = build_set_agent_token_via_execute(
        asset,
        collection,
        agent_identity_pda,
        genesis_account,
        context.payer.pubkey(),
    );

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Second set should fail (now it's V2 with token already set).
    let base_mint_2 = Keypair::new().pubkey();
    let genesis_account_2 = create_genesis_account(&mut context, base_mint_2, 0, 0).await;

    let ix = build_set_agent_token_via_execute(
        asset,
        collection,
        agent_identity_pda,
        genesis_account_2,
        context.payer.pubkey(),
    );

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

    setup::assert_custom_error(err, MplAgentIdentityError::AgentTokenAlreadySet as u32);
}
