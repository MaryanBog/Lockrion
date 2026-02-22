// tests/027_deposit_atomicity_cpi_fail_pt.rs
#![forbid(unsafe_code)]

use borsh::BorshSerialize;
use solana_program::program_pack::Pack;
use solana_program_test::*;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{read_keypair_file, Keypair, Signer},
    system_instruction, system_program,
    transaction::Transaction,
};
use spl_token::state::{Account as TokenAccount, Mint};

use lockrion_issuance_v1_1::{
    instruction::LockrionInstruction,
    pda,
    state::{IssuanceState, UserState},
};

fn mk_ix(program_id: Pubkey, data: Vec<u8>, accounts: Vec<AccountMeta>) -> Instruction {
    Instruction { program_id, accounts, data }
}

async fn send_ok(ctx: &mut ProgramTestContext, ixs: Vec<Instruction>, extra_signers: &[&Keypair]) {
    let payer = ctx.payer.pubkey();
    let mut tx = Transaction::new_with_payer(&ixs, Some(&payer));
    let bh = ctx.banks_client.get_latest_blockhash().await.unwrap();

    let mut signers: Vec<&Keypair> = Vec::with_capacity(1 + extra_signers.len());
    signers.push(&ctx.payer);
    signers.extend_from_slice(extra_signers);

    tx.sign(&signers, bh);
    ctx.banks_client.process_transaction(tx).await.unwrap();
}

async fn send_expect_fail_any(ctx: &mut ProgramTestContext, ixs: Vec<Instruction>, extra_signers: &[&Keypair]) {
    let payer = ctx.payer.pubkey();
    let mut tx = Transaction::new_with_payer(&ixs, Some(&payer));
    let bh = ctx.banks_client.get_latest_blockhash().await.unwrap();

    let mut signers: Vec<&Keypair> = Vec::with_capacity(1 + extra_signers.len());
    signers.push(&ctx.payer);
    signers.extend_from_slice(extra_signers);

    tx.sign(&signers, bh);
    let _err = ctx.banks_client.process_transaction(tx).await.unwrap_err();
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

    let init = spl_token::instruction::initialize_mint(&spl_token::id(), &mint_kp.pubkey(), mint_authority, None, 0)
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

    let init = spl_token::instruction::initialize_account(&spl_token::id(), &acct_kp.pubkey(), mint, owner).unwrap();

    send_ok(ctx, vec![create, init], &[acct_kp]).await;
}

async fn mint_to(ctx: &mut ProgramTestContext, mint: &Pubkey, dst: &Pubkey, mint_authority: &Keypair, amount: u64) {
    let ix = spl_token::instruction::mint_to(&spl_token::id(), mint, dst, &mint_authority.pubkey(), &[], amount).unwrap();
    send_ok(ctx, vec![ix], &[mint_authority]).await;
}

async fn token_balance(ctx: &mut ProgramTestContext, token_acc: &Pubkey) -> u64 {
    let acc = ctx.banks_client.get_account(*token_acc).await.unwrap().unwrap();
    let ta = TokenAccount::unpack_from_slice(&acc.data).unwrap();
    ta.amount
}

async fn read_issuance(ctx: &mut ProgramTestContext, issuance_pda: &Pubkey) -> IssuanceState {
    let acc = ctx.banks_client.get_account(*issuance_pda).await.unwrap().unwrap();
    IssuanceState::unpack(&acc.data).unwrap()
}

async fn read_user_opt(ctx: &mut ProgramTestContext, user_pda: &Pubkey) -> Option<UserState> {
    let acc_opt = ctx.banks_client.get_account(*user_pda).await.unwrap();
    acc_opt.map(|acc| UserState::unpack(&acc.data).unwrap())
}

#[tokio::test]
async fn deposit_atomicity_cpi_fail_pt() {
    let program_id = lockrion_issuance_v1_1::id();

    let pt = ProgramTest::new(
        "lockrion_issuance_v1_1",
        program_id,
        processor!(lockrion_issuance_v1_1::entrypoint::process_instruction),
    );

    let mut ctx = pt.start_with_context().await;

    // -------- PLATFORM --------
    let platform = read_keypair_file("platform-authority.json").unwrap();

    // fund platform lamports
    let airdrop_ix = system_instruction::transfer(&ctx.payer.pubkey(), &platform.pubkey(), 5_000_000_000);
    send_ok(&mut ctx, vec![airdrop_ix], &[]).await;

    let participant_pk = ctx.payer.pubkey();

    let c: solana_sdk::sysvar::clock::Clock = ctx.banks_client.get_sysvar().await.unwrap();
    let now: i64 = (c.slot as i64) / 2;

    let reserve_total: u128 = 1000;
    let deposit_amount: u64 = 100;

    let start_ts: i64 = now + 10;
    let maturity_ts: i64 = start_ts + 86_400;

    let (issuance_pda, _) = pda::derive_issuance_pda(&program_id, &platform.pubkey(), start_ts, reserve_total);
    let (user_pda, _) = pda::derive_user_pda(&program_id, &issuance_pda, &participant_pk);

    // mints
    let lock_mint = Keypair::new();
    let reward_mint = Keypair::new();
    let mint_auth = Keypair::new();
    create_mint(&mut ctx, &lock_mint, &mint_auth.pubkey()).await;
    create_mint(&mut ctx, &reward_mint, &mint_auth.pubkey()).await;

    // escrows
    let deposit_escrow = Keypair::new();
    let reward_escrow = Keypair::new();
    create_token_account(&mut ctx, &deposit_escrow, &lock_mint.pubkey(), &issuance_pda).await;
    create_token_account(&mut ctx, &reward_escrow, &reward_mint.pubkey(), &issuance_pda).await;

    // funding source (platform-owned)
    let issuer_reward = Keypair::new();
    create_token_account(&mut ctx, &issuer_reward, &reward_mint.pubkey(), &platform.pubkey()).await;

    // platform treasury (required by init; reward mint)
    let platform_treasury = Keypair::new();
    create_token_account(&mut ctx, &platform_treasury, &reward_mint.pubkey(), &platform.pubkey()).await;

    // trick: token account has correct mint, but owner != participant signer (payer)
    let wrong_owner = Keypair::new();
    let participant_lock_wrong_owner = Keypair::new();
    create_token_account(
        &mut ctx,
        &participant_lock_wrong_owner,
        &lock_mint.pubkey(),
        &wrong_owner.pubkey(), // NOT participant_pk
    )
    .await;

    // balances
    mint_to(&mut ctx, &reward_mint.pubkey(), &issuer_reward.pubkey(), &mint_auth, reserve_total as u64).await;
    mint_to(
        &mut ctx,
        &lock_mint.pubkey(),
        &participant_lock_wrong_owner.pubkey(),
        &mint_auth,
        deposit_amount,
    )
    .await;

    // init (signer = PLATFORM)
    let init_ix = mk_ix(
        program_id,
        LockrionInstruction::InitIssuance { reserve_total, start_ts, maturity_ts }
            .try_to_vec()
            .unwrap(),
        vec![
            AccountMeta::new(platform.pubkey(), true),
            AccountMeta::new(issuance_pda, false),
            AccountMeta::new_readonly(lock_mint.pubkey(), false),
            AccountMeta::new_readonly(reward_mint.pubkey(), false),
            AccountMeta::new_readonly(deposit_escrow.pubkey(), false),
            AccountMeta::new_readonly(reward_escrow.pubkey(), false),
            AccountMeta::new_readonly(platform_treasury.pubkey(), false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );
    send_ok(&mut ctx, vec![init_ix], &[&platform]).await;

    // fund (signer = PLATFORM)
    let fund_ix = mk_ix(
        program_id,
        LockrionInstruction::FundReserve { amount: reserve_total as u64 }
            .try_to_vec()
            .unwrap(),
        vec![
            AccountMeta::new(issuance_pda, false),
            AccountMeta::new(platform.pubkey(), true),
            AccountMeta::new(issuer_reward.pubkey(), false),
            AccountMeta::new(reward_escrow.pubkey(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
    );
    send_ok(&mut ctx, vec![fund_ix], &[&platform]).await;

    // open deposit window
    warp_to_ts(&mut ctx, start_ts).await;

    // snapshot BEFORE
    let iss_before = read_issuance(&mut ctx, &issuance_pda).await;
    let dep_escrow_before = token_balance(&mut ctx, &deposit_escrow.pubkey()).await;
    let user_before = read_user_opt(&mut ctx, &user_pda).await;

    // deposit: will FAIL at CPI transfer (authority = participant signer, but token account owner != authority)
    let deposit_ix = mk_ix(
        program_id,
        LockrionInstruction::Deposit { amount: deposit_amount }
            .try_to_vec()
            .unwrap(),
        vec![
            AccountMeta::new(issuance_pda, false),
            AccountMeta::new(user_pda, false),
            AccountMeta::new(participant_pk, true),
            AccountMeta::new(participant_lock_wrong_owner.pubkey(), false),
            AccountMeta::new(deposit_escrow.pubkey(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );
    send_expect_fail_any(&mut ctx, vec![deposit_ix], &[]).await;

    // verify AFTER: state identical, escrow unchanged
    let iss_after = read_issuance(&mut ctx, &issuance_pda).await;
    let dep_escrow_after = token_balance(&mut ctx, &deposit_escrow.pubkey()).await;
    let user_after = read_user_opt(&mut ctx, &user_pda).await;

    assert_eq!(iss_after.total_locked, iss_before.total_locked);
    assert_eq!(iss_after.total_weight_accum, iss_before.total_weight_accum);
    assert_eq!(iss_after.last_day_index, iss_before.last_day_index);
    assert_eq!(dep_escrow_after, dep_escrow_before);

    assert_eq!(user_after.is_some(), user_before.is_some());
    if let (Some(a), Some(b)) = (user_after, user_before) {
        assert_eq!(a, b);
    }
}