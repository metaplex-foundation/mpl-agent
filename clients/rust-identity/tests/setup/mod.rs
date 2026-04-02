use mpl_agent_identity::{accounts::AgentIdentityV2, instructions::RegisterIdentityV1Builder};
use mpl_core::instructions::{CreateCollectionV1Builder, CreateV1Builder};
use solana_program::instruction::InstructionError;
use solana_program_test::BanksClientError;
use solana_program_test::ProgramTest;
use solana_sdk::transaction::TransactionError;
use solana_sdk::{
    account::AccountSharedData,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

/// Asserts that a BanksClientError is a custom program error matching the
/// expected error variant. Callers pass the error enum variant cast as u32,
/// e.g. `assert_custom_error(err, MplAgentIdentityError::SomeError as u32)`.
pub fn assert_custom_error(error: BanksClientError, expected_code: u32) {
    match error.unwrap() {
        TransactionError::InstructionError(_, InstructionError::Custom(code)) => {
            assert_eq!(code, expected_code);
        }
        err => panic!("Expected InstructionError::Custom({expected_code}), got: {err:?}"),
    }
}

pub const MPL_CORE_ID: Pubkey =
    solana_program::pubkey!("CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d");

pub const GENESIS_PROGRAM_ID: Pubkey =
    solana_program::pubkey!("GNS1S5J5AspKXgpjz6SvKL66kPaKWAhaGRhCqPRxii2B");

pub fn setup() -> ProgramTest {
    let mut program_test =
        ProgramTest::new("mpl_agent_identity_program", mpl_agent_identity::ID, None);
    program_test.add_program("mpl_core", MPL_CORE_ID, None);
    program_test
}

pub async fn create_collection_and_asset(
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

pub async fn register_identity(
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

/// Create a fake GenesisAccountV2 account in the test environment.
///
/// Layout (136 bytes total):
///   offset 0:   key (u8) = 18 (GenesisAccountV2 discriminator)
///   offset 4:   finalized (u8)
///   offset 8:   authority (Pubkey, 32 bytes)
///   offset 40:  base_mint (Pubkey, 32 bytes)
///   offset 128: funding_mode (u8) — 0=Mint, 1=Transfer
///
/// Returns the address of the genesis account.
#[allow(dead_code)]
pub async fn create_genesis_account(
    context: &mut solana_program_test::ProgramTestContext,
    base_mint: Pubkey,
    authority: Pubkey,
    finalized: u8,
    funding_mode: u8,
) -> Pubkey {
    let genesis_keypair = Keypair::new();
    let genesis_address = genesis_keypair.pubkey();

    let mut data = vec![0u8; 136];
    // key (discriminator) at offset 0
    data[0] = 18;
    // finalized at offset 4
    data[4] = finalized;
    // authority at offset 8
    data[8..40].copy_from_slice(authority.as_ref());
    // base_mint at offset 40
    data[40..72].copy_from_slice(base_mint.as_ref());
    // funding_mode at offset 128
    data[128] = funding_mode;

    let rent = context.banks_client.get_rent().await.unwrap();
    let lamports = rent.minimum_balance(136);

    let mut account_data = AccountSharedData::new(lamports, 136, &GENESIS_PROGRAM_ID);
    account_data.set_data_from_slice(&data);
    context.set_account(&genesis_address, &account_data);

    genesis_address
}
