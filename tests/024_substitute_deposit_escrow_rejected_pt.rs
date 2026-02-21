// tests/024_substitute_deposit_escrow_rejected_pt.rs
#![forbid(unsafe_code)]

use borsh::BorshSerialize;
use solana_program::program_pack::Pack;
use solana_program_test::*;
use solana_sdk::{
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

#[tokio::test]
async fn substitute_deposit_escrow_rejected_pt() {
    let program_id = lockrion_issuance_v1_1::id();
    let pt = ProgramTest::new(
        "lockrion_issuance_v1_1",
        program_id,
        processor!(lockrion_issuance_v1_1::entrypoint::process_instruction),
    );

    let mut ctx = pt.start_with_context().await;
    let payer_pk = ctx.payer.pubkey();

    let c: solana_sdk::sysvar::clock::Clock = ctx.banks_client.get_sysvar().await.unwrap();
    let now: i64 = (c.slot as i64) / 2;

    let reserve_total: u128 = 1000;
    let deposit_amount: u64 = 100;

    let start_ts: i64 = now + 10;
    let maturity_ts: i64 = start_ts + 86_400;

    let (issuance_pda, _) = pda::derive_issuance_pda(&program_id, &payer_pk, start_ts, reserve_total);
    let (user_pda, _) = pda::derive_user_pda(&program_id, &issuance_pda, &payer_pk);

    // mints
    let lock_mint = Keypair::new();
    let reward_mint = Keypair::new();
    let mint_auth = Keypair::new();
    create_mint(&mut ctx, &lock_mint, &mint_auth.pubkey()).await;
    create_mint(&mut ctx, &reward_mint, &mint_auth.pubkey()).await;

    // escrows (REAL)
    let deposit_escrow = Keypair::new();
    let reward_escrow = Keypair::new();
    create_token_account(&mut ctx, &deposit_escrow, &lock_mint.pubkey(), &issuance_pda).await;
    create_token_account(&mut ctx, &reward_escrow, &reward_mint.pubkey(), &issuance_pda).await;

    // escrows (FAKE substitute)
    let fake_deposit_escrow = Keypair::new();
    create_token_account(&mut ctx, &fake_deposit_escrow, &lock_mint.pubkey(), &issuance_pda).await;

    // funding + participant
    let issuer_reward = Keypair::new();
    create_token_account(&mut ctx, &issuer_reward, &reward_mint.pubkey(), &payer_pk).await;

    let participant_lock = Keypair::new();
    create_token_account(&mut ctx, &participant_lock, &lock_mint.pubkey(), &payer_pk).await;

    let platform_treasury = Keypair::new();
    create_token_account(&mut ctx, &platform_treasury, &reward_mint.pubkey(), &payer_pk).await;

    mint_to(&mut ctx, &reward_mint.pubkey(), &issuer_reward.pubkey(), &mint_auth, reserve_total as u64).await;
    mint_to(&mut ctx, &lock_mint.pubkey(), &participant_lock.pubkey(), &mint_auth, deposit_amount).await;

    // init
    let init_ix = mk_ix(
        program_id,
        LockrionInstruction::InitIssuance { reserve_total, start_ts, maturity_ts }.try_to_vec().unwrap(),
        vec![
            AccountMeta::new(payer_pk, true),
            AccountMeta::new(issuance_pda, false),
            AccountMeta::new_readonly(lock_mint.pubkey(), false),
            AccountMeta::new_readonly(reward_mint.pubkey(), false),
            AccountMeta::new_readonly(deposit_escrow.pubkey(), false),
            AccountMeta::new_readonly(reward_escrow.pubkey(), false),
            AccountMeta::new_readonly(platform_treasury.pubkey(), false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );
    send_ok(&mut ctx, vec![init_ix], &[]).await;

    // fund
    let fund_ix = mk_ix(
        program_id,
        LockrionInstruction::FundReserve { amount: reserve_total as u64 }.try_to_vec().unwrap(),
        vec![
            AccountMeta::new(issuance_pda, false),
            AccountMeta::new(payer_pk, true),
            AccountMeta::new(issuer_reward.pubkey(), false),
            AccountMeta::new(reward_escrow.pubkey(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
    );
    send_ok(&mut ctx, vec![fund_ix], &[]).await;

    // deposit window open
    warp_to_ts(&mut ctx, start_ts).await;

    // deposit with WRONG deposit_escrow account
    let deposit_ix = mk_ix(
        program_id,
        LockrionInstruction::Deposit { amount: deposit_amount }.try_to_vec().unwrap(),
        vec![
            AccountMeta::new(issuance_pda, false),
            AccountMeta::new(user_pda, false),
            AccountMeta::new(payer_pk, true),
            AccountMeta::new(participant_lock.pubkey(), false),
            AccountMeta::new(fake_deposit_escrow.pubkey(), false), // WRONG
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );

    send_expect_custom_err(
        &mut ctx,
        vec![deposit_ix],
        LockrionError::InvalidEscrowAccount as u32,
    )
    .await;
}