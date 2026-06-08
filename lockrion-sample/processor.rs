// ==============================
// src/processor.rs (dispatch + canonical order skeleton)
// ==============================
#![forbid(unsafe_code)]

use solana_program::{program::invoke_signed, system_instruction, system_program, rent::Rent};

use borsh::BorshDeserialize;
use solana_program::program_pack::Pack;

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::Sysvar,
};

use spl_token::state::Account as TokenAccount;

use crate::{
    accumulator,
    error::LockrionError,
    instruction::LockrionInstruction,
    pda,
    state::{IssuanceState, UserState},
};

// Platform-only init gate (hardcoded authority)
pub const PLATFORM_AUTHORITY: solana_program::pubkey::Pubkey =
    solana_program::pubkey!("B9xmmg2zPMSwPg7iX7a9J2j6SK5LcopZ8abRDj9ughxw");

pub struct Processor;

impl Processor {
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], ix_data: &[u8]) -> ProgramResult {
        let ix = LockrionInstruction::try_from_slice(ix_data).map_err(|_| LockrionError::InvalidInstruction)?;
        match ix {
            LockrionInstruction::InitIssuance { reserve_total, start_ts, maturity_ts } =>
            Self::init_issuance(program_id, accounts, reserve_total, start_ts, maturity_ts),
            LockrionInstruction::FundReserve { amount } => Self::fund_reserve(program_id, accounts, amount),
            LockrionInstruction::Deposit { amount } => Self::deposit(program_id, accounts, amount),
            LockrionInstruction::ClaimReward => Self::claim_reward(program_id, accounts),
            LockrionInstruction::WithdrawDeposit => Self::withdraw_deposit(program_id, accounts),
            LockrionInstruction::Sweep => Self::sweep(program_id, accounts),
            LockrionInstruction::ZeroParticipationReclaim => Self::reclaim(program_id, accounts),
        }
    }

    // ---------------------------------------------------------------------
    // fund_reserve()
    // Accounts (suggested RAW order):
    // 0 [writable] issuance_state (PDA)
    // 1 [signer]   issuer
    // 2 [writable] issuer_reward_ata (USDC)
    // 3 [writable] reward_escrow (USDC)
    // 4 []         token_program
    // ---------------------------------------------------------------------
    fn fund_reserve(program_id: &Pubkey, accounts: &[AccountInfo], amount: u64) -> ProgramResult {
        let acc_iter = &mut accounts.iter();
        let issuance_ai = next_account_info(acc_iter)?;
        let issuer_ai = next_account_info(acc_iter)?;
        let issuer_reward_ata_ai = next_account_info(acc_iter)?;
        let reward_escrow_ai = next_account_info(acc_iter)?;
        let token_program_ai = next_account_info(acc_iter)?;

        Self::validate_token_program(token_program_ai)?;

        // Load state
        if issuance_ai.owner != program_id {
            return Err(LockrionError::InvalidEscrowAccount.into());
        }
        let mut issuance = IssuanceState::unpack(&issuance_ai.try_borrow_data()?)?;

        // PDA validation (issuance account address MUST equal canonical PDA)
        let (issuance_pda, bump) = pda::derive_issuance_pda(program_id, &issuance.issuer_address, issuance.start_ts, issuance.reserve_total);
        if issuance_ai.key != &issuance_pda || issuance.bump != bump {
            return Err(LockrionError::InvalidPda.into());
        }

        // Preconditions
        if issuance.is_reserve_funded() {
            return Err(LockrionError::ReserveAlreadyFunded.into());
        }
        if !issuer_ai.is_signer {
            return Err(LockrionError::UnauthorizedCaller.into());
        }
        if issuer_ai.key != &issuance.issuer_address {
            return Err(LockrionError::UnauthorizedCaller.into());
        }

        let now = Self::now_ts();
        if now >= issuance.start_ts {
            return Err(LockrionError::FundingWindowClosed.into());
        }             

        // Amount must equal reserve_total exactly
        let amt_u128 = u128::from(amount);
        if amt_u128 != issuance.reserve_total {
            return Err(LockrionError::InvalidFundingAmount.into());
        }

        // Validate reward escrow is correct and mint matches reward_mint
        Self::validate_token_account_mint(reward_escrow_ai, &issuance.reward_mint)?;
        Self::validate_token_account_mint(issuer_reward_ata_ai, &issuance.reward_mint)?;

        // Authority of reward escrow MUST be issuance PDA
        Self::validate_token_account_authority(reward_escrow_ai, &issuance_pda)?;

        // CPI transfer issuer -> reward_escrow (issuer signs)
        Self::spl_transfer(
            token_program_ai,
            issuer_reward_ata_ai,
            reward_escrow_ai,
            issuer_ai,
            &[], // signer seeds none (issuer signs)
            amount,
        )?;

        // Verify escrow balance == reserve_total (optional strict check)
        let escrow = TokenAccount::unpack(&reward_escrow_ai.try_borrow_data()?)?;
        let bal_u128 = u128::from(escrow.amount);
        if bal_u128 != issuance.reserve_total {
            return Err(LockrionError::InvariantViolation.into());
        }

        // Set reserve_funded = true
        issuance.reserve_funded = 1;
        issuance.pack(&mut issuance_ai.try_borrow_mut_data()?)?;

        Ok(())
    }

    // ---------------------------------------------------------------------
    // deposit(amount)
    // Accounts:
    // 0 [writable] issuance_state (PDA)
    // 1 [writable] user_state (PDA)        (may be uninitialized; created here)
    // 2 [signer]   participant             (payer for UserState creation)
    // 3 [writable] participant_lock_ata
    // 4 [writable] deposit_escrow
    // 5 []         token_program
    // 6 []         system_program
    // ---------------------------------------------------------------------
    fn deposit(program_id: &Pubkey, accounts: &[AccountInfo], amount: u64) -> ProgramResult {
        if amount == 0 {
            return Err(LockrionError::InvalidAmount.into());
        }

        let acc_iter = &mut accounts.iter();
        let issuance_ai = next_account_info(acc_iter)?;
        let user_state_ai = next_account_info(acc_iter)?;
        let participant_ai = next_account_info(acc_iter)?;
        let participant_lock_ata_ai = next_account_info(acc_iter)?;
        let deposit_escrow_ai = next_account_info(acc_iter)?;
        let token_program_ai = next_account_info(acc_iter)?;

        let system_program_ai = next_account_info(acc_iter)?;

        if system_program_ai.key != &system_program::ID {
           return Err(LockrionError::InvalidInstruction.into());
        }

        Self::validate_token_program(token_program_ai)?;

        if issuance_ai.owner != program_id {
            return Err(LockrionError::InvalidEscrowAccount.into());
        }
        let mut issuance = IssuanceState::unpack(&issuance_ai.try_borrow_data()?)?;

        // Validate issuance PDA
        let (issuance_pda, bump) = pda::derive_issuance_pda(program_id, &issuance.issuer_address, issuance.start_ts, issuance.reserve_total);
        if issuance_ai.key != &issuance_pda || issuance.bump != bump {
            return Err(LockrionError::InvalidPda.into());
        }

        // Reserve funded gate
        if !issuance.is_reserve_funded() {
            return Err(LockrionError::ReserveNotFunded.into());
        }

        let now = Self::now_ts();
        if now < issuance.start_ts {
            return Err(LockrionError::DepositWindowNotStarted.into());
        }
        
        if now >= issuance.maturity_ts {
            return Err(LockrionError::DepositWindowClosed.into());
        }

        // Validate deposit escrow matches stored pubkey + mint + authority
        if deposit_escrow_ai.key != &issuance.deposit_escrow {
            return Err(LockrionError::InvalidEscrowAccount.into());
        }
        Self::validate_token_account_mint(deposit_escrow_ai, &issuance.lock_mint)?;
        Self::validate_token_account_mint(participant_lock_ata_ai, &issuance.lock_mint)?;
        Self::validate_token_account_authority(deposit_escrow_ai, &issuance_pda)?;

        // Validate user_state PDA for (issuance, participant)
        let (user_pda, user_bump) = pda::derive_user_pda(program_id, &issuance_pda, participant_ai.key);
        if user_state_ai.key != &user_pda {
            return Err(LockrionError::InvalidPda.into());
        }
        // If user_state is not initialized yet — create it (payer = participant)
        Self::create_user_state_if_needed(
          program_id,
          &issuance_pda,
          user_state_ai,
          participant_ai,
          user_bump,
        )?;

        // If freshly created or uninitialized: initialize EXACT bytes per State Layout v1.1
        // UserState layout offsets:
        // 0 version(u8)=1
        // 1 bump(u8)
        // 2 issuance(Pubkey)[32]
        // 34 participant(Pubkey)[32]
        // 66 locked_amount(u128)=0
        // 82 user_weight_accum(u128)=0
        // 98 user_last_day_index(u64)=issuance.last_day_index
        // 106 reward_claimed(u8)=0
        // 107..111 padding[5]=0
    {
    let mut d = user_state_ai.try_borrow_mut_data()?;

    // initialize only if version != 1 (fresh account will be all-zero)
    if d[0] != 1 {
        // version
        d[0] = 1;
        // bump
        d[1] = user_bump;

        // issuance pubkey bytes
        d[2..34].copy_from_slice(issuance_ai.key.as_ref());

        // participant pubkey bytes
        d[34..66].copy_from_slice(participant_ai.key.as_ref());

        // locked_amount + user_weight_accum already zero (leave as-is)
        // user_last_day_index = issuance.last_day_index
        d[98..106].copy_from_slice(&issuance.last_day_index.to_le_bytes());

        // reward_claimed = 0 (leave)
        // padding = 0 (leave)
    }
}

        // Load user state (assumes already created/initialized by separate init instruction OR off-chain create)
        // NOTE: v1 spec set exposes only 6 instructions; значит UserState должен существовать заранее,
        // либо создаётся вне контракта. Мы здесь только читаем/пишем. :contentReference[oaicite:5]{index=5}
        let mut user = UserState::unpack(&user_state_ai.try_borrow_data()?)?;
        if user.bump != user_bump {
            return Err(LockrionError::InvalidPda.into());
        }
        // Cross-binding guards
        if &user.issuance != issuance_ai.key || &user.participant != participant_ai.key {
            return Err(LockrionError::InvalidUserStateAccount.into());
        }
        if !participant_ai.is_signer {
            return Err(LockrionError::UnauthorizedCaller.into());
        }

        // 1) accumulator update (global then user) BEFORE mutation :contentReference[oaicite:6]{index=6}
        Self::apply_accumulators(&mut issuance, &mut user, now)?;

        // 2) state mutation BEFORE CPI transfer (defensive order) :contentReference[oaicite:7]{index=7}
        let amt_u128 = u128::from(amount);
        issuance.total_locked = issuance
            .total_locked
            .checked_add(amt_u128)
            .ok_or(LockrionError::ArithmeticOverflow)?;
        user.locked_amount = user
            .locked_amount
            .checked_add(amt_u128)
            .ok_or(LockrionError::ArithmeticOverflow)?;

        issuance.pack(&mut issuance_ai.try_borrow_mut_data()?)?;
        user.pack(&mut user_state_ai.try_borrow_mut_data()?)?;

        // 3) CPI transfer participant -> deposit_escrow (participant signs)
        Self::spl_transfer(
            token_program_ai,
            participant_lock_ata_ai,
            deposit_escrow_ai,
            participant_ai,
            &[],
            amount,
        )?;

        Ok(())
    }

    // claim_reward / withdraw_deposit / sweep / reclaim — оставляю как каркас,
    // дальше заполним по одному (строго по canonical order + flags-before-transfer).
    fn claim_reward(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let acc_iter = &mut accounts.iter();
        let issuance_ai = next_account_info(acc_iter)?;
        let user_state_ai = next_account_info(acc_iter)?;
        let participant_ai = next_account_info(acc_iter)?;
        let participant_reward_ata_ai = next_account_info(acc_iter)?;
        let reward_escrow_ai = next_account_info(acc_iter)?;
        let token_program_ai = next_account_info(acc_iter)?;
    
        Self::validate_token_program(token_program_ai)?;
    
        if issuance_ai.owner != program_id {
            return Err(LockrionError::InvalidEscrowAccount.into());
        }
        if user_state_ai.owner != program_id {
            return Err(LockrionError::InvalidUserStateAccount.into());
        }
        if !participant_ai.is_signer {
            return Err(LockrionError::UnauthorizedCaller.into());
        }
    
        // Load states
        let mut issuance = IssuanceState::unpack(&issuance_ai.try_borrow_data()?)?;
        let mut user = UserState::unpack(&user_state_ai.try_borrow_data()?)?;
    
        // Validate issuance PDA
        let (issuance_pda, bump) = pda::derive_issuance_pda(
            program_id,
            &issuance.issuer_address,
            issuance.start_ts,
            issuance.reserve_total,
        );
        if issuance_ai.key != &issuance_pda || issuance.bump != bump {
            return Err(LockrionError::InvalidPda.into());
        }
    
        // Validate user PDA + binding
        let (user_pda, user_bump) = pda::derive_user_pda(program_id, &issuance_pda, participant_ai.key);
        if user_state_ai.key != &user_pda || user.bump != user_bump {
            return Err(LockrionError::InvalidPda.into());
        }
        if &user.issuance != issuance_ai.key || &user.participant != participant_ai.key {
            return Err(LockrionError::InvalidUserStateAccount.into());
        }
    
        // Escrow must match stored reward_escrow
        if reward_escrow_ai.key != &issuance.reward_escrow {
            return Err(LockrionError::InvalidEscrowAccount.into());
        }
    
        // Mint checks
        Self::validate_token_account_mint(reward_escrow_ai, &issuance.reward_mint)?;
        Self::validate_token_account_mint(participant_reward_ata_ai, &issuance.reward_mint)?;
    
        // Escrow authority must be issuance PDA
        Self::validate_token_account_authority(reward_escrow_ai, &issuance_pda)?;
    
        // Window checks
        let now = Self::now_ts();
    
        if now < issuance.maturity_ts {
            return Err(LockrionError::ClaimWindowNotStarted.into());
        }
        let claim_end = issuance
            .maturity_ts
            .checked_add(issuance.claim_window)
            .ok_or(LockrionError::ArithmeticOverflow)?;
        if now >= claim_end {
            return Err(LockrionError::ClaimWindowClosed.into());
        }
    
        // User flag check
        if user.is_reward_claimed() {
            return Err(LockrionError::AlreadyClaimed.into());
        }
    
        // Finalize accumulators (global then user) BEFORE reward calc :contentReference[oaicite:2]{index=2}
        Self::apply_accumulators(&mut issuance, &mut user, now)?;
    
        if issuance.total_weight_accum == 0 {
            return Err(LockrionError::NoParticipation.into());
        }
    
        // SPL token amounts are u64; enforce representability deterministically
        if issuance.reserve_total > (u64::MAX as u128) {
            return Err(LockrionError::InvariantViolation.into());
        }
    
        // reward = reserve_total * user_weight_accum / total_weight_accum  (u128 checked)
        let numerator = issuance
            .reserve_total
            .checked_mul(user.user_weight_accum)
            .ok_or(LockrionError::ArithmeticOverflow)?;
        let reward_u128 = numerator
            .checked_div(issuance.total_weight_accum)
            .ok_or(LockrionError::DivisionByZero)?;
        if reward_u128 > (u64::MAX as u128) {
            return Err(LockrionError::ArithmeticOverflow.into());
        }
        let reward_u64 = reward_u128 as u64;
    
        // Defensive order: set flag BEFORE transfer :contentReference[oaicite:3]{index=3}
        user.reward_claimed = 1;
    
        // Persist state BEFORE CPI (atomic if CPI fails) :contentReference[oaicite:4]{index=4}
        issuance.pack(&mut issuance_ai.try_borrow_mut_data()?)?;
        user.pack(&mut user_state_ai.try_borrow_mut_data()?)?;
    
        // invoke_signed transfer from reward_escrow -> participant_reward_ata
        let start_ts_le = issuance.start_ts.to_le_bytes();
        let reserve_total_le = issuance.reserve_total.to_le_bytes();
        let bump_seed = [issuance.bump];
    
        let seeds: &[&[u8]] = &[
            pda::SEED_ISSUANCE,
            issuance.issuer_address.as_ref(),
            &start_ts_le,
            &reserve_total_le,
            &bump_seed,
        ];
        let signer_seeds: &[&[&[u8]]] = &[seeds];
    
        Self::spl_transfer(
            token_program_ai,
            reward_escrow_ai,
            participant_reward_ata_ai,
            issuance_ai,      // authority = issuance PDA account
            signer_seeds,     // PDA signs
            reward_u64,
        )?;
    
        Ok(())
    }
    
    fn withdraw_deposit(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let acc_iter = &mut accounts.iter();
        let issuance_ai = next_account_info(acc_iter)?;
        let user_state_ai = next_account_info(acc_iter)?;
        let participant_ai = next_account_info(acc_iter)?;
        let participant_lock_ata_ai = next_account_info(acc_iter)?;
        let deposit_escrow_ai = next_account_info(acc_iter)?;
        let token_program_ai = next_account_info(acc_iter)?;
    
        Self::validate_token_program(token_program_ai)?;
    
        if issuance_ai.owner != program_id {
            return Err(LockrionError::InvalidEscrowAccount.into());
        }
        if user_state_ai.owner != program_id {
            return Err(LockrionError::InvalidUserStateAccount.into());
        }
        if !participant_ai.is_signer {
            return Err(LockrionError::UnauthorizedCaller.into());
        }
    
        // Load state
        let mut issuance = IssuanceState::unpack(&issuance_ai.try_borrow_data()?)?;
        let mut user = UserState::unpack(&user_state_ai.try_borrow_data()?)?;
    
        // Validate issuance PDA (canonical seeds)
        let (issuance_pda, bump) = pda::derive_issuance_pda(
            program_id,
            &issuance.issuer_address,
            issuance.start_ts,
            issuance.reserve_total,
        );
        if issuance_ai.key != &issuance_pda || issuance.bump != bump {
            return Err(LockrionError::InvalidPda.into());
        }
    
        // Validate user PDA + binding
        let (user_pda, user_bump) = pda::derive_user_pda(program_id, &issuance_pda, participant_ai.key);
        if user_state_ai.key != &user_pda || user.bump != user_bump {
            return Err(LockrionError::InvalidPda.into());
        }
        if &user.issuance != issuance_ai.key || &user.participant != participant_ai.key {
            return Err(LockrionError::InvalidUserStateAccount.into());
        }
    
        // Validate deposit escrow matches stored + mint + authority
        if deposit_escrow_ai.key != &issuance.deposit_escrow {
            return Err(LockrionError::InvalidEscrowAccount.into());
        }
        Self::validate_token_account_mint(deposit_escrow_ai, &issuance.lock_mint)?;
        Self::validate_token_account_mint(participant_lock_ata_ai, &issuance.lock_mint)?;
        Self::validate_token_account_authority(deposit_escrow_ai, &issuance_pda)?;
    
        // Time gate: only after maturity
        let now = Self::now_ts();
        if now < issuance.maturity_ts {
            return Err(LockrionError::DepositWindowNotClosed.into());
        }
    
        // Must have something to withdraw
        if user.locked_amount == 0 {
            return Err(LockrionError::InvalidAmount.into());
        }
    
        // Canonical order: finalize accumulators (global then user) BEFORE clearing locked_amount :contentReference[oaicite:1]{index=1} :contentReference[oaicite:2]{index=2}
        Self::apply_accumulators(&mut issuance, &mut user, now)?;
    
        // Defensive mutation-before-transfer:
        // amount = user.locked_amount; total_locked -= amount; user.locked_amount = 0 :contentReference[oaicite:3]{index=3}
        let amount_u128 = user.locked_amount;
    
        issuance.total_locked = issuance
            .total_locked
            .checked_sub(amount_u128)
            .ok_or(LockrionError::ArithmeticUnderflow)?;
    
        user.locked_amount = 0;
    
        // Persist state before CPI (atomic revert on CPI failure) :contentReference[oaicite:4]{index=4}
        issuance.pack(&mut issuance_ai.try_borrow_mut_data()?)?;
        user.pack(&mut user_state_ai.try_borrow_mut_data()?)?;
    
        // SPL transfer amount must fit u64
        if amount_u128 > (u64::MAX as u128) {
            return Err(LockrionError::ArithmeticOverflow.into());
        }
        let amount_u64 = amount_u128 as u64;
    
        // invoke_signed (deposit_escrow -> participant_lock_ata), authority = issuance PDA
        let start_ts_le = issuance.start_ts.to_le_bytes();
        let reserve_total_le = issuance.reserve_total.to_le_bytes();
        let bump_seed = [issuance.bump];
    
        let seeds: &[&[u8]] = &[
            pda::SEED_ISSUANCE,
            issuance.issuer_address.as_ref(),
            &start_ts_le,
            &reserve_total_le,
            &bump_seed,
        ];
        let signer_seeds: &[&[&[u8]]] = &[seeds];
    
        Self::spl_transfer(
            token_program_ai,
            deposit_escrow_ai,
            participant_lock_ata_ai,
            issuance_ai,      // PDA authority
            signer_seeds,     // PDA signs
            amount_u64,
        )?;
    
        Ok(())
    }
    
    fn sweep(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let acc_iter = &mut accounts.iter();
        let issuance_ai = next_account_info(acc_iter)?;
        let reward_escrow_ai = next_account_info(acc_iter)?;
        let platform_treasury_ai = next_account_info(acc_iter)?;
        let token_program_ai = next_account_info(acc_iter)?;
    
        Self::validate_token_program(token_program_ai)?;
    
        if issuance_ai.owner != program_id {
            return Err(LockrionError::InvalidEscrowAccount.into());
        }
    
        // Load state
        let mut issuance = IssuanceState::unpack(&issuance_ai.try_borrow_data()?)?;
    
        // Validate issuance PDA
        let (issuance_pda, bump) = pda::derive_issuance_pda(
            program_id,
            &issuance.issuer_address,
            issuance.start_ts,
            issuance.reserve_total,
        );
        if issuance_ai.key != &issuance_pda || issuance.bump != bump {
            return Err(LockrionError::InvalidPda.into());
        }
    
        // Validate platform treasury binding (must match stored immutable)
        if platform_treasury_ai.key != &issuance.platform_treasury {
            return Err(LockrionError::InvalidPlatformTreasury.into());
        }
    
        // Validate reward escrow matches stored
        if reward_escrow_ai.key != &issuance.reward_escrow {
            return Err(LockrionError::InvalidEscrowAccount.into());
        }
    
        // Mint checks (both must be reward_mint)
        Self::validate_token_account_mint(reward_escrow_ai, &issuance.reward_mint)?;
        Self::validate_token_account_mint(platform_treasury_ai, &issuance.reward_mint)?;
    
        // Authority of reward escrow MUST be issuance PDA
        Self::validate_token_account_authority(reward_escrow_ai, &issuance_pda)?;
    
        // Preconditions
        if issuance.total_weight_accum == 0 {
            return Err(LockrionError::NoParticipation.into());
        }
        if issuance.is_sweep_executed() {
            return Err(LockrionError::SweepAlreadyExecuted.into());
        }
    
        let now = Self::now_ts();
        let sweep_start = issuance
            .maturity_ts
            .checked_add(issuance.claim_window)
            .ok_or(LockrionError::ArithmeticOverflow)?;
        if now < sweep_start {
            return Err(LockrionError::ClaimWindowClosed.into()); // not ideal naming, but you already have it
            // If хочешь идеально: добавим Error::SweepWindowNotStarted, но это не требуется протоколом.
        }
    
        // Accumulator finalization MUST still run before validation decisions in some profiles,
        // but here total_weight_accum is already nonzero gate; still, we finalize deterministically. :contentReference[oaicite:2]{index=2}
        // (No user state here; only global finalize.)
        // We reuse apply_accumulators by fabricating a dummy user? NO.
        // Instead: do minimal global finalize inline using the same logic.
    
        let _current = Self::finalize_global(&mut issuance, now)?;
    
        // Determine escrow balance and transfer entire balance
        let escrow = TokenAccount::unpack(&reward_escrow_ai.try_borrow_data()?)?;
        let bal = escrow.amount;
        if bal == 0 {
            return Ok(()); // spec says "reward escrow balance > 0" as precondition; returning Ok is harmless deterministic no-op
        }
    
        // Defensive order: set flag BEFORE transfer :contentReference[oaicite:3]{index=3}
        issuance.sweep_executed = 1;
    
        // Persist state before CPI (atomic revert on CPI failure) :contentReference[oaicite:4]{index=4}
        issuance.pack(&mut issuance_ai.try_borrow_mut_data()?)?;
    
        // invoke_signed transfer Reward Escrow -> platform_treasury for full balance
        let start_ts_le = issuance.start_ts.to_le_bytes();
        let reserve_total_le = issuance.reserve_total.to_le_bytes();
        let bump_seed = [issuance.bump];
    
        let seeds: &[&[u8]] = &[
            pda::SEED_ISSUANCE,
            issuance.issuer_address.as_ref(),
            &start_ts_le,
            &reserve_total_le,
            &bump_seed,
        ];
        let signer_seeds: &[&[&[u8]]] = &[seeds];
    
        Self::spl_transfer(
            token_program_ai,
            reward_escrow_ai,
            platform_treasury_ai,
            issuance_ai,      // PDA authority
            signer_seeds,     // PDA signs
            bal,
        )?;
    
        Ok(())
    }
    
    fn reclaim(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let acc_iter = &mut accounts.iter();
        let issuance_ai = next_account_info(acc_iter)?;
        let issuer_ai = next_account_info(acc_iter)?;
        let issuer_reward_ata_ai = next_account_info(acc_iter)?;
        let reward_escrow_ai = next_account_info(acc_iter)?;
        let token_program_ai = next_account_info(acc_iter)?;
    
        Self::validate_token_program(token_program_ai)?;
    
        if issuance_ai.owner != program_id {
            return Err(LockrionError::InvalidEscrowAccount.into());
        }
        if !issuer_ai.is_signer {
            return Err(LockrionError::UnauthorizedCaller.into());
        }
    
        // Load state
        let mut issuance = IssuanceState::unpack(&issuance_ai.try_borrow_data()?)?;
    
        // Validate issuance PDA (canonical)
        let (issuance_pda, bump) = pda::derive_issuance_pda(
            program_id,
            &issuance.issuer_address,
            issuance.start_ts,
            issuance.reserve_total,
        );
        if issuance_ai.key != &issuance_pda || issuance.bump != bump {
            return Err(LockrionError::InvalidPda.into());
        }
    
        // Caller must be issuer_address
        if issuer_ai.key != &issuance.issuer_address {
            return Err(LockrionError::UnauthorizedCaller.into());
        }
    
        // One-shot gate
        if issuance.is_reclaim_executed() {
            return Err(LockrionError::ReclaimAlreadyExecuted.into());
        }
    
        // Reward escrow must match stored, mint must match, authority must be issuance PDA
        if reward_escrow_ai.key != &issuance.reward_escrow {
            return Err(LockrionError::InvalidEscrowAccount.into());
        }
        Self::validate_token_account_mint(reward_escrow_ai, &issuance.reward_mint)?;
        Self::validate_token_account_authority(reward_escrow_ai, &issuance_pda)?;
    
        // Destination must be a token account with reward_mint (USDC)
        Self::validate_token_account_mint(issuer_reward_ata_ai, &issuance.reward_mint)?;
    
        // Time gate: only after maturity
        let now = Self::now_ts();
        if now < issuance.maturity_ts {
            return Err(LockrionError::ClaimWindowNotStarted.into()); // reclaim not available pre-maturity
        }
        
        let _current = Self::finalize_global(&mut issuance, now)?;

        if issuance.total_weight_accum != 0 {
            return Err(LockrionError::NoParticipation.into());
        }
        
        // Transfer entire escrow balance (must be > 0)
        let escrow = TokenAccount::unpack(&reward_escrow_ai.try_borrow_data()?)?;
        let bal = escrow.amount;
        if bal == 0 {
            return Err(LockrionError::InvalidAmount.into());
        }
    
        // Defensive order: set reclaim_executed BEFORE transfer :contentReference[oaicite:4]{index=4}
        issuance.reclaim_executed = 1;
    
        // Persist state before CPI (atomic revert on CPI failure) :contentReference[oaicite:5]{index=5}
        issuance.pack(&mut issuance_ai.try_borrow_mut_data()?)?;
    
        // invoke_signed: Reward Escrow -> issuer_reward_ata
        let start_ts_le = issuance.start_ts.to_le_bytes();
        let reserve_total_le = issuance.reserve_total.to_le_bytes();
        let bump_seed = [issuance.bump];
    
        let seeds: &[&[u8]] = &[
            pda::SEED_ISSUANCE,
            issuance.issuer_address.as_ref(),
            &start_ts_le,
            &reserve_total_le,
            &bump_seed,
        ];
        let signer_seeds: &[&[&[u8]]] = &[seeds];
    
        Self::spl_transfer(
            token_program_ai,
            reward_escrow_ai,
            issuer_reward_ata_ai,
            issuance_ai,      // PDA authority
            signer_seeds,     // PDA signs
            bal,
        )?;
    
        Ok(())
    }    

    fn init_issuance(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        reserve_total: u128,
        start_ts: i64,
        maturity_ts: i64,
    ) -> ProgramResult {
        let acc_iter = &mut accounts.iter();
    
        let payer_ai = next_account_info(acc_iter)?;
        let issuance_ai = next_account_info(acc_iter)?;
        let lock_mint_ai = next_account_info(acc_iter)?;
        let reward_mint_ai = next_account_info(acc_iter)?;
        let deposit_escrow_ai = next_account_info(acc_iter)?;
        let reward_escrow_ai = next_account_info(acc_iter)?;
        let platform_treasury_ai = next_account_info(acc_iter)?;
        let system_program_ai = next_account_info(acc_iter)?;
    
        // --- Platform-only gate ---
        if !payer_ai.is_signer {
            return Err(LockrionError::UnauthorizedCaller.into());
        }
        if payer_ai.key != &PLATFORM_AUTHORITY {
            return Err(LockrionError::UnauthorizedCaller.into());
        }
    
        if system_program_ai.key != &solana_program::system_program::id() {
            // keep existing error enum usage to avoid changing error surface
            return Err(LockrionError::InvalidTokenProgram.into());
        }
    
        if reserve_total == 0 {
            return Err(LockrionError::InvalidAmount.into());
        }
    
        let (issuance_pda, bump) =
            pda::derive_issuance_pda(program_id, payer_ai.key, start_ts, reserve_total);
    
        if issuance_ai.key != &issuance_pda {
            return Err(LockrionError::InvalidPda.into());
        }
    
        // must be uninitialized before create_account
        if issuance_ai.owner != &solana_program::system_program::id() || issuance_ai.data_len() != 0 {
            return Err(LockrionError::InvalidEscrowAccount.into());
        }
    
        let rent = solana_program::rent::Rent::get()?;
        let lamports = rent.minimum_balance(crate::state::ISSUANCE_STATE_SIZE);
    
        solana_program::program::invoke_signed(
            &solana_program::system_instruction::create_account(
                payer_ai.key,
                issuance_ai.key,
                lamports,
                crate::state::ISSUANCE_STATE_SIZE as u64,
                program_id,
            ),
            &[payer_ai.clone(), issuance_ai.clone(), system_program_ai.clone()],
            &[&[
                pda::SEED_ISSUANCE,
                payer_ai.key.as_ref(),
                &start_ts.to_le_bytes(),
                &reserve_total.to_le_bytes(),
                &[bump],
            ]],
        )?;
    
        let final_day_index = if maturity_ts > start_ts {
            ((maturity_ts - start_ts) / 86400) as u64
        } else {
            0u64
        };
    
        let issuance = IssuanceState {
            version: crate::state::STATE_VERSION,
            bump,
            issuer_address: *payer_ai.key,
    
            lock_mint: *lock_mint_ai.key,
            reward_mint: *reward_mint_ai.key,
            deposit_escrow: *deposit_escrow_ai.key,
            reward_escrow: *reward_escrow_ai.key,
            platform_treasury: *platform_treasury_ai.key,
    
            reserve_total,
            start_ts,
            maturity_ts,
            claim_window: 90 * 86400,
            final_day_index,
    
            total_locked: 0,
            total_weight_accum: 0,
            last_day_index: 0,
    
            reserve_funded: 0,
            sweep_executed: 0,
            reclaim_executed: 0,
            reserved_padding: [0u8; 7],
        };
    
        issuance.pack(&mut issuance_ai.try_borrow_mut_data()?)?;
    
        Ok(())
    }

    // ---------------------------------------------------------------------
    // Helpers
    // ---------------------------------------------------------------------

    fn now_ts() -> i64 {
        let c = Clock::get().unwrap();
    
        #[cfg(feature = "test-clock")]
        {
            (c.slot as i64) / 2
        }
    
        #[cfg(not(feature = "test-clock"))]
        {
            c.unix_timestamp
        }
    }

    fn create_user_state_if_needed<'a>(
        program_id: &Pubkey,
        issuance_pda: &Pubkey,
        user_state_ai: &AccountInfo<'a>,
        participant_ai: &AccountInfo<'a>,
        user_bump: u8,
    ) -> ProgramResult {
        // already program-owned => exists
        if user_state_ai.owner == program_id {
            return Ok(());
        }
    
        // must be system-owned placeholder (uninitialized)
        if user_state_ai.owner != &system_program::ID {
            return Err(LockrionError::InvalidUserStateAccount.into());
        }
    
        // Create PDA account (rent-exempt) of exact size 112 bytes
        let space: u64 = 112;
        let lamports = Rent::get()?.minimum_balance(space as usize);
    
        let ix = system_instruction::create_account(
            participant_ai.key,
            user_state_ai.key,
            lamports,
            space,
            program_id,
        );
    
        let bump_seed = [user_bump];
        let seeds: &[&[u8]] = &[
            pda::SEED_USER,              // b"user"
            issuance_pda.as_ref(),
            participant_ai.key.as_ref(),
            &bump_seed,
        ];
    
        invoke_signed(
            &ix,
            &[participant_ai.clone(), user_state_ai.clone()],
            &[seeds],
        )?;
    
        Ok(())
    }

    fn validate_token_program(token_program_ai: &AccountInfo) -> ProgramResult {
        if token_program_ai.key != &spl_token::id() {
            return Err(LockrionError::InvalidTokenProgram.into());
        }
        Ok(())
    }

    fn validate_token_account_mint(token_ai: &AccountInfo, expected_mint: &Pubkey) -> ProgramResult {
        let ta = TokenAccount::unpack(&token_ai.try_borrow_data()?)?;
        if &ta.mint != expected_mint {
            return Err(LockrionError::InvalidMint.into());
        }
        Ok(())
    }

    fn validate_token_account_authority(token_ai: &AccountInfo, expected_authority: &Pubkey) -> ProgramResult {
        let ta = TokenAccount::unpack(&token_ai.try_borrow_data()?)?;
        let auth = ta.owner; // SPL Token Account's "owner" field = authority
        if &auth != expected_authority {
            return Err(LockrionError::InvalidAuthority.into());
        }
        Ok(())
    }

    fn spl_transfer<'a>(
        token_program_ai: &AccountInfo<'a>,
        source_ai: &AccountInfo<'a>,
        dest_ai: &AccountInfo<'a>,
        authority_ai: &AccountInfo<'a>,
        signer_seeds: &[&[&[u8]]], // invoke_signed seeds if PDA
        amount: u64,
    ) -> ProgramResult {
        let ix = spl_token::instruction::transfer(
            token_program_ai.key,
            source_ai.key,
            dest_ai.key,
            authority_ai.key,
            &[] as &[&Pubkey],
            amount,
        )?;
    
        if signer_seeds.is_empty() {
            solana_program::program::invoke(
                &ix,
                &[
                    source_ai.clone(),
                    dest_ai.clone(),
                    authority_ai.clone(),
                    token_program_ai.clone(),
                ],
            )?;
        } else {
            solana_program::program::invoke_signed(
                &ix,
                &[
                    source_ai.clone(),
                    dest_ai.clone(),
                    authority_ai.clone(),
                    token_program_ai.clone(),
                ],
                signer_seeds,
            )?;
        }
    
        Ok(())
    }

    fn finalize_global(issuance: &mut IssuanceState, now: i64) -> Result<u64, ProgramError> {
        // Compute bounded current_day_index
        let raw = accumulator::raw_day_index(now, issuance.start_ts).map_err(ProgramError::from)?;
        let current = accumulator::bounded_day_index(raw, issuance.final_day_index);
    
        // Global accumulator update
        if current > issuance.last_day_index {
            let days_elapsed = current
                .checked_sub(issuance.last_day_index)
                .ok_or(LockrionError::ArithmeticUnderflow)?;
            let inc = issuance
                .total_locked
                .checked_mul(days_elapsed as u128)
                .ok_or(LockrionError::ArithmeticOverflow)?;
            issuance.total_weight_accum = issuance
                .total_weight_accum
                .checked_add(inc)
                .ok_or(LockrionError::ArithmeticOverflow)?;
            issuance.last_day_index = current;
        }
    
        if issuance.last_day_index > issuance.final_day_index {
            return Err(LockrionError::InvariantViolation.into());
        }
    
        Ok(current)
    }
    
    fn update_user_with_current(user: &mut UserState, current: u64) -> Result<(), ProgramError> {
        if current > user.user_last_day_index {
            let days_elapsed_user = current
                .checked_sub(user.user_last_day_index)
                .ok_or(LockrionError::ArithmeticUnderflow)?;
            let inc = user
                .locked_amount
                .checked_mul(days_elapsed_user as u128)
                .ok_or(LockrionError::ArithmeticOverflow)?;
            user.user_weight_accum = user
                .user_weight_accum
                .checked_add(inc)
                .ok_or(LockrionError::ArithmeticOverflow)?;
            user.user_last_day_index = current;
        }
        Ok(())
    }    

    fn apply_accumulators(
        issuance: &mut IssuanceState,
        user: &mut UserState,
        now: i64,
    ) -> Result<(), ProgramError> {
        let current = Self::finalize_global(issuance, now)?;
        Self::update_user_with_current(user, current)?;
        Ok(())
    }    

}