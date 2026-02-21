// tests/028_arithmetic_overflow_rejected_pt.rs
#![forbid(unsafe_code)]

use borsh::BorshSerialize;
use solana_program::program_pack::Pack;
use solana_program_test::*;
use solana_sdk::{
    account::Account,
    instruction::{AccountMeta, Instruction, InstructionError},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction, system_program,
    transaction::{Transaction, TransactionError},
};
use spl_token::state::{Account as TokenAccount, Mint};

use lockrion_issuance_v1_1::{
    error::LockrionError,
    instruction::LockrionInstruction,
    pda,
    state::IssuanceState,
};

fn mk_ix(program_id: Pubkey, data: Vec<u8>, accounts: Vec<AccountMeta>) -> Instruction {
    Instruction { program_id, accounts, data }
}

async fn send_ok(ctx: &mut ProgramTestContext, ixs: Vec<Instruction>, extra_signers: &[&Keypair]) {
    let payer = ctx.payer.pubkey();
    let mut tx = Transaction::new_with_payer(&ixs, Some(&payer));
    let bh = ctx.banks_client.get_latest_blockhash().await.unwrap();

    let mut signers: Vec<&Keypair> = Vec::new();
    signers.push(&ctx.payer);
    signers.extend_from_slice(extra_signers);

    tx.sign(&signers, bh);
    ctx.banks_client.process_transaction(tx).await.unwrap();
}

async fn send_expect_custom_err(ctx: &mut ProgramTestContext, ixs: Vec<Instruction>, expected_code: u32) {
    let payer = ctx.payer.pubkey();
    let mut tx = Transaction::new_with_payer(&ixs, Some(&payer));
    let bh = ctx.banks_client.get_latest_blockhash().await.unwrap();
    tx.sign(&[&ctx.payer], bh);

    let err = ctx.banks_client.process_transaction(tx).await.unwrap_err();
    match err {
        BanksClientError::TransactionError(TransactionError::InstructionError(_, InstructionError::Custom(code))) => {
            assert_eq!(code, expected_code, "wrong custom error code");
        }
        _ => panic!("unexpected error: {:?}", err),
    }
}

async fn warp_to_ts(ctx: &mut ProgramTestContext, target_ts: i64) {
    loop {
        let c: solana_sdk::sysvar::clock::Clock = ctx.banks_client.get_sysvar().await.unwrap();
        let now: i64 = (c.slot as i64) / 2; // feature test-clock
        if now >= target_ts {
            return;
        }
        let need = (target_ts - now) as u64;
        ctx.warp_to_slot(c.slot + need.saturating_mul(2) + 10).unwrap();
    }
}

async fn create_mint(ctx: &mut ProgramTestContext, mint_kp: &Keypair, mint_authority: &Pubkey) {
    let rent = ctx.banks_client.get_rent().await.unwrap();
    let space = Mint::LEN;
    let lamports = rent.minimum_balance(space);

    let create = system_instruction::create_account(
        &ctx.payer.pubkey(),
        &mint_kp.pubkey(),
        lamports,
        space as u64,
        &spl_token::id(),
    );

    let init = spl_token::instruction::initialize_mint(
        &spl_token::id(),
        &mint_kp.pubkey(),
        mint_authority,
        None,
        0,
    )
    .unwrap();

    send_ok(ctx, vec![create, init], &[mint_kp]).await;
}

async fn create_token_account(ctx: &mut ProgramTestContext, acct_kp: &Keypair, mint: &Pubkey, owner: &Pubkey) {
    let rent = ctx.banks_client.get_rent().await.unwrap();
    let space = TokenAccount::LEN;
    let lamports = rent.minimum_balance(space);

    let create = system_instruction::create_account(
        &ctx.payer.pubkey(),
        &acct_kp.pubkey(),
        lamports,
        space as u64,
        &spl_token::id(),
    );

    let init = spl_token::instruction::initialize_account(
        &spl_token::id(),
        &acct_kp.pubkey(),
        mint,
        owner,
    )
    .unwrap();

    send_ok(ctx, vec![create, init], &[acct_kp]).await;
}

async fn mint_to(ctx: &mut ProgramTestContext, mint: &Pubkey, dst: &Pubkey, mint_authority: &Keypair, amount: u64) {
    let ix = spl_token::instruction::mint_to(
        &spl_token::id(),
        mint,
        dst,
        &mint_authority.pubkey(),
        &[],
        amount,
    )
    .unwrap();

    send_ok(ctx, vec![ix], &[mint_authority]).await;
}

async fn token_balance(ctx: &mut ProgramTestContext, token_acc: &Pubkey) -> u64 {
    let acc = ctx.banks_client.get_account(*token_acc).await.unwrap().unwrap();
    let ta = TokenAccount::unpack_from_slice(&acc.data).unwrap();
    ta.amount
}

#[tokio::test]
async fn arithmetic_overflow_rejected_pt() {
    let program_id = lockrion_issuance_v1_1::id();

    // Pre-create keys so IssuanceState can reference them
    let issuer_pubkey = Pubkey::new_unique();
    let lock_mint = Keypair::new();
    let reward_mint = Keypair::new();
    let deposit_escrow = Keypair::new();
    let reward_escrow = Keypair::new();
    let platform_treasury = Keypair::new();

    // Fixed timestamps so we can preload issuance PDA before ctx exists
    let reserve_total: u128 = 1000;
    let start_ts: i64 = 10;
    let maturity_ts: i64 = start_ts + 86_400;

    let (issuance_pda, bump) =
        pda::derive_issuance_pda(&program_id, &issuer_pubkey, start_ts, reserve_total);

    // Preload issuance account with total_locked = u128::MAX so any deposit overflows
    let final_day_index: u64 = ((maturity_ts - start_ts) / 86_400) as u64;

    let issuance = IssuanceState {
        version: 1,
        bump,
        issuer_address: issuer_pubkey,

        lock_mint: lock_mint.pubkey(),
        reward_mint: reward_mint.pubkey(),
        deposit_escrow: deposit_escrow.pubkey(),
        reward_escrow: reward_escrow.pubkey(),
        platform_treasury: platform_treasury.pubkey(),

        reserve_total,
        start_ts,
        maturity_ts,
        claim_window: 90 * 86_400,
        final_day_index,

        total_locked: u128::MAX, // <-- will trigger ArithmeticOverflow on checked_add
        total_weight_accum: 0,
        last_day_index: 0,

        reserve_funded: 1, // allow deposit
        sweep_executed: 0,
        reclaim_executed: 0,
        reserved_padding: [0u8; 7],
    };

    let mut issuance_data = vec![0u8; lockrion_issuance_v1_1::state::ISSUANCE_STATE_SIZE];
    issuance.pack(&mut issuance_data).unwrap();

    let mut pt = ProgramTest::new(
        "lockrion_issuance_v1_1",
        program_id,
        processor!(lockrion_issuance_v1_1::entrypoint::process_instruction),
    );

    pt.add_account(
        issuance_pda,
        Account {
            lamports: 10_000_000_000,
            data: issuance_data,
            owner: program_id,
            executable: false,
            rent_epoch: 0,
        },
    );

    let mut ctx = pt.start_with_context().await;
    let payer_pk = ctx.payer.pubkey();

    // Create mints + token accounts matching those pubkeys
    let mint_auth = Keypair::new();
    create_mint(&mut ctx, &lock_mint, &mint_auth.pubkey()).await;
    create_mint(&mut ctx, &reward_mint, &mint_auth.pubkey()).await;

    create_token_account(&mut ctx, &deposit_escrow, &lock_mint.pubkey(), &issuance_pda).await;
    create_token_account(&mut ctx, &reward_escrow, &reward_mint.pubkey(), &issuance_pda).await;
    create_token_account(&mut ctx, &platform_treasury, &reward_mint.pubkey(), &payer_pk).await;

    // participant lock ATA (token account) owned by payer
    let participant_lock = Keypair::new();
    create_token_account(&mut ctx, &participant_lock, &lock_mint.pubkey(), &payer_pk).await;

    // give participant some lock tokens (so CPI would succeed if reached)
    let deposit_amount: u64 = 1;
    mint_to(&mut ctx, &lock_mint.pubkey(), &participant_lock.pubkey(), &mint_auth, deposit_amount).await;

    // open deposit window (start_ts=10)
    warp_to_ts(&mut ctx, start_ts).await;

    let (user_pda, _) = pda::derive_user_pda(&program_id, &issuance_pda, &payer_pk);

    // snapshot escrow before
    let escrow_before = token_balance(&mut ctx, &deposit_escrow.pubkey()).await;

    // deposit must fail with ArithmeticOverflow (60)
    let deposit_ix = mk_ix(
        program_id,
        LockrionInstruction::Deposit { amount: deposit_amount }.try_to_vec().unwrap(),
        vec![
            AccountMeta::new(issuance_pda, false),
            AccountMeta::new(user_pda, false),
            AccountMeta::new(payer_pk, true),
            AccountMeta::new(participant_lock.pubkey(), false),
            AccountMeta::new(deposit_escrow.pubkey(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );

    send_expect_custom_err(
        &mut ctx,
        vec![deposit_ix],
        LockrionError::ArithmeticOverflow as u32,
    )
    .await;

    // escrow must remain unchanged (atomic rollback)
    let escrow_after = token_balance(&mut ctx, &deposit_escrow.pubkey()).await;
    assert_eq!(escrow_after, escrow_before, "escrow changed despite overflow");

    // issuance.total_locked must remain u128::MAX
    let iss_acc = ctx.banks_client.get_account(issuance_pda).await.unwrap().unwrap();
    let iss_after = IssuanceState::unpack(&iss_acc.data).unwrap();
    assert_eq!(iss_after.total_locked, u128::MAX, "total_locked changed despite overflow");

    // user state must NOT exist (creation rolled back)
    let user_acc = ctx.banks_client.get_account(user_pda).await.unwrap();
    assert!(user_acc.is_none(), "user state exists despite overflow rollback");
}