use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use shank::ShankType;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program::invoke,
    program_error::ProgramError, rent::Rent, system_instruction, system_program, sysvar::Sysvar,
};

use crate::{
    error::MplAgentReputationError,
    instruction::accounts::{DepositSubsidyV1Accounts, WithdrawSubsidyV1Accounts},
    state::ReviewSubsidyPoolV1,
};

#[derive(Clone, Debug, PartialEq, Eq, BorshSerialize, BorshDeserialize, ShankType)]
pub struct DepositSubsidyV1Args {
    /// Lamports to deposit on top of any rent required to first-init the pool.
    pub amount: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, BorshSerialize, BorshDeserialize, ShankType)]
pub struct WithdrawSubsidyV1Args {
    /// Lamports to withdraw. Must leave at least rent-exempt minimum.
    pub amount: u64,
}

/// Initialize the agent's subsidy pool (if uninitialized) and deposit
/// `amount` lamports of subsidy budget.
pub fn deposit_subsidy_v1<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: DepositSubsidyV1Args,
) -> ProgramResult {
    let ctx = DepositSubsidyV1Accounts::context(accounts)?;

    assert_signer(ctx.accounts.payer)?;
    if *ctx.accounts.system_program.key != system_program::id() {
        return Err(MplAgentReputationError::InvalidSystemProgram.into());
    }

    let bump = ReviewSubsidyPoolV1::check_pda_derivation(
        ctx.accounts.subsidy_pool,
        ctx.accounts.agent_asset.key,
    )?;

    let uninitialized = ctx.accounts.subsidy_pool.data_len() == 0
        || *ctx.accounts.subsidy_pool.owner == system_program::id();

    if uninitialized {
        // First init — allocate the pool and capture the withdraw authority.
        let withdraw_authority = ctx
            .accounts
            .withdraw_authority
            .unwrap_or(ctx.accounts.payer)
            .key;

        ReviewSubsidyPoolV1::create_account(
            ctx.accounts.subsidy_pool,
            ctx.accounts.system_program,
            ctx.accounts.payer,
            ctx.accounts.agent_asset.key,
            bump,
        )?;

        let mut data = ctx.accounts.subsidy_pool.try_borrow_mut_data()?;
        let pool: &mut ReviewSubsidyPoolV1 =
            bytemuck::from_bytes_mut(&mut data[..core::mem::size_of::<ReviewSubsidyPoolV1>()]);
        pool.initialize(bump, ctx.accounts.agent_asset.key, withdraw_authority);
    }

    // Top up the pool with `amount` extra lamports beyond rent.
    if args.amount > 0 {
        invoke(
            &system_instruction::transfer(
                ctx.accounts.payer.key,
                ctx.accounts.subsidy_pool.key,
                args.amount,
            ),
            &[
                ctx.accounts.payer.clone(),
                ctx.accounts.subsidy_pool.clone(),
                ctx.accounts.system_program.clone(),
            ],
        )?;
    }

    msg!(
        "Subsidy pool funded: agent={} added={} lamports",
        ctx.accounts.agent_asset.key,
        args.amount,
    );

    Ok(())
}

/// Withdraw `amount` lamports from the subsidy pool. The pool must remain
/// rent-exempt afterwards.
pub fn withdraw_subsidy_v1<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: WithdrawSubsidyV1Args,
) -> ProgramResult {
    let ctx = WithdrawSubsidyV1Accounts::context(accounts)?;

    assert_signer(ctx.accounts.withdraw_authority)?;

    let _bump = ReviewSubsidyPoolV1::check_pda_derivation(
        ctx.accounts.subsidy_pool,
        ctx.accounts.agent_asset.key,
    )?;

    if ctx.accounts.subsidy_pool.owner != &crate::ID
        || ctx.accounts.subsidy_pool.data_len() < core::mem::size_of::<ReviewSubsidyPoolV1>()
    {
        return Err(MplAgentReputationError::SubsidyPoolNotInitialized.into());
    }

    {
        let data = ctx.accounts.subsidy_pool.try_borrow_data()?;
        let pool: &ReviewSubsidyPoolV1 =
            bytemuck::from_bytes(&data[..core::mem::size_of::<ReviewSubsidyPoolV1>()]);
        if pool.withdraw_authority != *ctx.accounts.withdraw_authority.key {
            return Err(MplAgentReputationError::UnauthorizedSubsidyWithdrawal.into());
        }
        if pool.agent_asset != *ctx.accounts.agent_asset.key {
            return Err(MplAgentReputationError::InvalidAccountData.into());
        }
    }

    let rent_floor = Rent::get()?.minimum_balance(core::mem::size_of::<ReviewSubsidyPoolV1>());
    let pool_lamports_now = ctx.accounts.subsidy_pool.lamports();
    let spendable = pool_lamports_now.saturating_sub(rent_floor);
    if args.amount > spendable {
        return Err(ProgramError::InsufficientFunds);
    }

    // Move lamports directly — pool is program-owned so a system transfer
    // would fail. Mutate both accounts' lamport fields.
    let mut pool_lamports = ctx.accounts.subsidy_pool.try_borrow_mut_lamports()?;
    let mut dest_lamports = ctx.accounts.destination.try_borrow_mut_lamports()?;
    **pool_lamports = pool_lamports
        .checked_sub(args.amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    **dest_lamports = dest_lamports
        .checked_add(args.amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    Ok(())
}

/// Deserialize Deposit args.
pub fn deserialize_deposit_subsidy_args(data: &[u8]) -> Result<DepositSubsidyV1Args, ProgramError> {
    DepositSubsidyV1Args::try_from_slice(data)
        .map_err(|_| MplAgentReputationError::InvalidInstructionData.into())
}

/// Deserialize Withdraw args.
pub fn deserialize_withdraw_subsidy_args(
    data: &[u8],
) -> Result<WithdrawSubsidyV1Args, ProgramError> {
    WithdrawSubsidyV1Args::try_from_slice(data)
        .map_err(|_| MplAgentReputationError::InvalidInstructionData.into())
}
