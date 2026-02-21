// ==============================
// tests/038_two_users_proportional_distribution.rs
// ==============================
#![forbid(unsafe_code)]

use borsh::BorshSerialize;
use solana_program::program_pack::Pack;
use solana_program_test::*;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction, system_program,
    transaction::Transaction,
};
use spl_token::state::{Account as TokenAccount, Mint};

use lockrion_issuance_v1_1::{instruction::LockrionInstruction, pda};

async fn send_tx(ctx: &mut ProgramTestContext, ixs: Vec<Instruction>, extra_signers: &[&Keypair]) {
    let payer_pk = ctx.payer.pubkey();
    let mut tx = Transaction::new_with_payer(&ixs, Some(&payer_pk));
    let bh = ctx.banks_client.get_latest_blockhash().await.unwrap();

    let mut signers: Vec<&Keypair> = Vec::with_capacity(1 + extra_signers.len());
    signers.push(&ctx.payer);
    signers.extend_from_slice(extra_signers);

    tx.sign(&signers, bh);
    ctx.banks_client.process_transaction(tx).await.unwrap();
}

async fn warp_until_ts(ctx: &mut ProgramTestContext, target_ts: i64) {
    loop {
        let c: solana_sdk::sysvar::clock::Clock = ctx.banks_client.get_sysvar().await.unwrap();
        let now: i64 = (c.slot as i64) / 2;
        if now >= target_ts {
            return;
        }
        let need = (target_ts - now) as u64;
        ctx.warp_to_slot(c.slot + need.saturating_mul(2)).unwrap();
    }
}

async fn fund_system_account(ctx: &mut ProgramTestContext, to: &Pubkey, lamports: u64) {
    let ix = system_instruction::transfer(&ctx.payer.pubkey(), to, lamports);
    send_tx(ctx, vec![ix], &[]).await;
}

async fn create_mint(ctx: &mut ProgramTestContext, mint_kp: &Keypair, mint_authority: &Pubkey) {
    let rent = ctx.banks_client.get_rent().await.unwrap();
    let lamports = rent.minimum_balance(Mint::LEN);

    let create = system_instruction::create_account(
        &ctx.payer.pubkey(),
        &mint_kp.pubkey(),
        lamports,
        Mint::LEN as u64,
        &spl_token::id(),
    );
    let init = spl_token::instruction::initialize_mint(&spl_token::id(), &mint_kp.pubkey(), mint_authority, None, 0).unwrap();
    send_tx(ctx, vec![create, init], &[mint_kp]).await;
}

async fn create_token_account(ctx: &mut ProgramTestContext, acct_kp: &Keypair, mint: &Pubkey, owner: &Pubkey) {
    let rent = ctx.banks_client.get_rent().await.unwrap();
    let lamports = rent.minimum_balance(TokenAccount::LEN);

    let create = system_instruction::create_account(
        &ctx.payer.pubkey(),
        &acct_kp.pubkey(),
        lamports,
        TokenAccount::LEN as u64,
        &spl_token::id(),
    );
    let init = spl_token::instruction::initialize_account(&spl_token::id(), &acct_kp.pubkey(), mint, owner).unwrap();
    send_tx(ctx, vec![create, init], &[acct_kp]).await;
}

async fn mint_to(ctx: &mut ProgramTestContext, mint: &Pubkey, dst: &Pubkey, mint_authority: &Keypair, amount: u64) {
    let ix = spl_token::instruction::mint_to(&spl_token::id(), mint, dst, &mint_authority.pubkey(), &[], amount).unwrap();
    send_tx(ctx, vec![ix], &[mint_authority]).await;
}

async fn token_balance(ctx: &mut ProgramTestContext, token_acc: &Pubkey) -> u64 {
    let acc = ctx.banks_client.get_account(*token_acc).await.unwrap().unwrap();
    TokenAccount::unpack_from_slice(&acc.data).unwrap().amount
}

fn mk_ix(program_id: Pubkey, data: Vec<u8>, metas: Vec<AccountMeta>) -> Instruction {
    Instruction { program_id, accounts: metas, data }
}

#[tokio::test]
async fn two_users_proportional_distribution() {
    let program_id = lockrion_issuance_v1_1::id();
    let pt = ProgramTest::new(
        "lockrion_issuance_v1_1",
        program_id,
        processor!(lockrion_issuance_v1_1::entrypoint::process_instruction),
    );
    let mut ctx = pt.start_with_context().await;

    let issuer_pk = ctx.payer.pubkey();

    let start_ts: i64 = 10;
    let maturity_ts: i64 = start_ts + 86_400;
    let reserve_total: u128 = 1000;

    let (issuance_pda, _bump) = pda::derive_issuance_pda(&program_id, &issuer_pk, start_ts, reserve_total);

    let lock_mint = Keypair::new();
    let reward_mint = Keypair::new();
    let mint_auth = Keypair::new();

    create_mint(&mut ctx, &lock_mint, &mint_auth.pubkey()).await;
    create_mint(&mut ctx, &reward_mint, &mint_auth.pubkey()).await;

    let deposit_escrow = Keypair::new();
    let reward_escrow = Keypair::new();
    create_token_account(&mut ctx, &deposit_escrow, &lock_mint.pubkey(), &issuance_pda).await;
    create_token_account(&mut ctx, &reward_escrow, &reward_mint.pubkey(), &issuance_pda).await;

    let issuer_reward_ata = Keypair::new();
    create_token_account(&mut ctx, &issuer_reward_ata, &reward_mint.pubkey(), &issuer_pk).await;

    let platform_treasury = Keypair::new();
    create_token_account(&mut ctx, &platform_treasury, &reward_mint.pubkey(), &issuer_pk).await;

    // mint reward to issuer then fund reserve
    mint_to(&mut ctx, &reward_mint.pubkey(), &issuer_reward_ata.pubkey(), &mint_auth, reserve_total as u64).await;

    // init
    let init_ix = mk_ix(
        program_id,
        LockrionInstruction::InitIssuance { reserve_total, start_ts, maturity_ts }.try_to_vec().unwrap(),
        vec![
            AccountMeta::new(issuer_pk, true),
            AccountMeta::new(issuance_pda, false),
            AccountMeta::new_readonly(lock_mint.pubkey(), false),
            AccountMeta::new_readonly(reward_mint.pubkey(), false),
            AccountMeta::new_readonly(deposit_escrow.pubkey(), false),
            AccountMeta::new_readonly(reward_escrow.pubkey(), false),
            AccountMeta::new_readonly(platform_treasury.pubkey(), false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );
    send_tx(&mut ctx, vec![init_ix], &[]).await;

    // fund before start
    warp_until_ts(&mut ctx, start_ts - 1).await;
    let fund_ix = mk_ix(
        program_id,
        LockrionInstruction::FundReserve { amount: reserve_total as u64 }.try_to_vec().unwrap(),
        vec![
            AccountMeta::new(issuance_pda, false),
            AccountMeta::new(issuer_pk, true),
            AccountMeta::new(issuer_reward_ata.pubkey(), false),
            AccountMeta::new(reward_escrow.pubkey(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
    );
    send_tx(&mut ctx, vec![fund_ix], &[]).await;

    // two users
    let u1 = Keypair::new();
    let u2 = Keypair::new();
    fund_system_account(&mut ctx, &u1.pubkey(), 2_000_000_000).await;
    fund_system_account(&mut ctx, &u2.pubkey(), 2_000_000_000).await;

    let u1_lock = Keypair::new();
    let u2_lock = Keypair::new();
    create_token_account(&mut ctx, &u1_lock, &lock_mint.pubkey(), &u1.pubkey()).await;
    create_token_account(&mut ctx, &u2_lock, &lock_mint.pubkey(), &u2.pubkey()).await;

    let u1_reward = Keypair::new();
    let u2_reward = Keypair::new();
    create_token_account(&mut ctx, &u1_reward, &reward_mint.pubkey(), &u1.pubkey()).await;
    create_token_account(&mut ctx, &u2_reward, &reward_mint.pubkey(), &u2.pubkey()).await;

    let amt1: u64 = 100;
    let amt2: u64 = 300;
    mint_to(&mut ctx, &lock_mint.pubkey(), &u1_lock.pubkey(), &mint_auth, amt1).await;
    mint_to(&mut ctx, &lock_mint.pubkey(), &u2_lock.pubkey(), &mint_auth, amt2).await;

    // deposits at start
    warp_until_ts(&mut ctx, start_ts).await;

    let (u1_pda, _) = pda::derive_user_pda(&program_id, &issuance_pda, &u1.pubkey());
    let (u2_pda, _) = pda::derive_user_pda(&program_id, &issuance_pda, &u2.pubkey());

    let dep1_ix = mk_ix(
        program_id,
        LockrionInstruction::Deposit { amount: amt1 }.try_to_vec().unwrap(),
        vec![
            AccountMeta::new(issuance_pda, false),
            AccountMeta::new(u1_pda, false),
            AccountMeta::new(u1.pubkey(), true),
            AccountMeta::new(u1_lock.pubkey(), false),
            AccountMeta::new(deposit_escrow.pubkey(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );
    send_tx(&mut ctx, vec![dep1_ix], &[&u1]).await;

    let dep2_ix = mk_ix(
        program_id,
        LockrionInstruction::Deposit { amount: amt2 }.try_to_vec().unwrap(),
        vec![
            AccountMeta::new(issuance_pda, false),
            AccountMeta::new(u2_pda, false),
            AccountMeta::new(u2.pubkey(), true),
            AccountMeta::new(u2_lock.pubkey(), false),
            AccountMeta::new(deposit_escrow.pubkey(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );
    send_tx(&mut ctx, vec![dep2_ix], &[&u2]).await;

    // claim at maturity
    warp_until_ts(&mut ctx, maturity_ts).await;

    let claim1_ix = mk_ix(
        program_id,
        LockrionInstruction::ClaimReward.try_to_vec().unwrap(),
        vec![
            AccountMeta::new(issuance_pda, false),
            AccountMeta::new(u1_pda, false),
            AccountMeta::new(u1.pubkey(), true),
            AccountMeta::new(u1_reward.pubkey(), false),
            AccountMeta::new(reward_escrow.pubkey(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
    );
    send_tx(&mut ctx, vec![claim1_ix], &[&u1]).await;

    let claim2_ix = mk_ix(
        program_id,
        LockrionInstruction::ClaimReward.try_to_vec().unwrap(),
        vec![
            AccountMeta::new(issuance_pda, false),
            AccountMeta::new(u2_pda, false),
            AccountMeta::new(u2.pubkey(), true),
            AccountMeta::new(u2_reward.pubkey(), false),
            AccountMeta::new(reward_escrow.pubkey(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
    );
    send_tx(&mut ctx, vec![claim2_ix], &[&u2]).await;

    let r1 = token_balance(&mut ctx, &u1_reward.pubkey()).await;
    let r2 = token_balance(&mut ctx, &u2_reward.pubkey()).await;

    // expected proportional, one full day weight: w1=amt1, w2=amt2
    let exp1: u64 = (reserve_total as u64) * amt1 / (amt1 + amt2);
    let exp2: u64 = (reserve_total as u64) * amt2 / (amt1 + amt2);

    assert_eq!(r1, exp1);
    assert_eq!(r2, exp2);
    assert_eq!(r1 + r2, reserve_total as u64);
}