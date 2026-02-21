#![forbid(unsafe_code)]

use borsh::BorshSerialize;
use solana_program::program_pack::Pack;
use solana_program_test::*;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, SeedDerivable, Signer},
    system_instruction, system_program,
    transaction::Transaction,
};
use spl_token::state::{Account as TokenAccount, Mint};

use lockrion_issuance_v1_1::{
    instruction::LockrionInstruction,
    pda,
    state::{IssuanceState, UserState},
};

fn kp(n: u8) -> Keypair {
    Keypair::from_seed(&[n; 32]).unwrap()
}

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

async fn read_issuance(ctx: &mut ProgramTestContext, issuance: &Pubkey) -> IssuanceState {
    let acc = ctx.banks_client.get_account(*issuance).await.unwrap().unwrap();
    IssuanceState::unpack(&acc.data).unwrap()
}

async fn read_user(ctx: &mut ProgramTestContext, user: &Pubkey) -> UserState {
    let acc = ctx.banks_client.get_account(*user).await.unwrap().unwrap();
    UserState::unpack(&acc.data).unwrap()
}

fn mk_ix(program_id: Pubkey, data: Vec<u8>, metas: Vec<AccountMeta>) -> Instruction {
    Instruction { program_id, accounts: metas, data }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct IssuanceSig {
    reserve_total: u128,
    start_ts: i64,
    maturity_ts: i64,
    claim_window: i64,
    final_day_index: u64,
    total_locked: u128,
    total_weight_accum: u128,
    last_day_index: u64,
    reserve_funded: u8,
    sweep_executed: u8,
    reclaim_executed: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct UserSig {
    locked_amount: u128,
    user_weight_accum: u128,
    user_last_day_index: u64,
    reward_claimed: u8,
}

async fn run_scenario() -> (IssuanceSig, UserSig, u64, u64) {
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
    let deposit_amount: u64 = 7;

    let (issuance_pda, _) = pda::derive_issuance_pda(&program_id, &issuer_pk, start_ts, reserve_total);

    // seeded (детерминированные) аккаунты, кроме payer (payer всегда новый)
    let lock_mint = kp(1);
    let reward_mint = kp(2);
    let mint_auth = kp(3);

    create_mint(&mut ctx, &lock_mint, &mint_auth.pubkey()).await;
    create_mint(&mut ctx, &reward_mint, &mint_auth.pubkey()).await;

    let deposit_escrow = kp(4);
    let reward_escrow = kp(5);
    create_token_account(&mut ctx, &deposit_escrow, &lock_mint.pubkey(), &issuance_pda).await;
    create_token_account(&mut ctx, &reward_escrow, &reward_mint.pubkey(), &issuance_pda).await;

    let issuer_reward_ata = kp(6);
    create_token_account(&mut ctx, &issuer_reward_ata, &reward_mint.pubkey(), &issuer_pk).await;

    let platform_treasury = kp(7);
    create_token_account(&mut ctx, &platform_treasury, &reward_mint.pubkey(), &issuer_pk).await;

    let participant_lock = kp(8);
    create_token_account(&mut ctx, &participant_lock, &lock_mint.pubkey(), &issuer_pk).await;

    let participant_reward = kp(9);
    create_token_account(&mut ctx, &participant_reward, &reward_mint.pubkey(), &issuer_pk).await;

    mint_to(&mut ctx, &reward_mint.pubkey(), &issuer_reward_ata.pubkey(), &mint_auth, reserve_total as u64).await;
    mint_to(&mut ctx, &lock_mint.pubkey(), &participant_lock.pubkey(), &mint_auth, deposit_amount).await;

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

    // fund reserve
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

    // deposit (user_pda зависит от issuer_pk => между сессиями он будет разный, это ОК)
    warp_until_ts(&mut ctx, start_ts).await;
    let (user_pda, _) = pda::derive_user_pda(&program_id, &issuance_pda, &issuer_pk);

    let dep_ix = mk_ix(
        program_id,
        LockrionInstruction::Deposit { amount: deposit_amount }.try_to_vec().unwrap(),
        vec![
            AccountMeta::new(issuance_pda, false),
            AccountMeta::new(user_pda, false),
            AccountMeta::new(issuer_pk, true),
            AccountMeta::new(participant_lock.pubkey(), false),
            AccountMeta::new(deposit_escrow.pubkey(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );
    send_tx(&mut ctx, vec![dep_ix], &[]).await;

    // claim
    warp_until_ts(&mut ctx, maturity_ts).await;
    let claim_ix = mk_ix(
        program_id,
        LockrionInstruction::ClaimReward.try_to_vec().unwrap(),
        vec![
            AccountMeta::new(issuance_pda, false),
            AccountMeta::new(user_pda, false),
            AccountMeta::new(issuer_pk, true),
            AccountMeta::new(participant_reward.pubkey(), false),
            AccountMeta::new(reward_escrow.pubkey(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
    );
    send_tx(&mut ctx, vec![claim_ix], &[]).await;

    let iss = read_issuance(&mut ctx, &issuance_pda).await;
    let user = read_user(&mut ctx, &user_pda).await;

    let iss_sig = IssuanceSig {
        reserve_total: iss.reserve_total,
        start_ts: iss.start_ts,
        maturity_ts: iss.maturity_ts,
        claim_window: iss.claim_window,
        final_day_index: iss.final_day_index,
        total_locked: iss.total_locked,
        total_weight_accum: iss.total_weight_accum,
        last_day_index: iss.last_day_index,
        reserve_funded: iss.reserve_funded,
        sweep_executed: iss.sweep_executed,
        reclaim_executed: iss.reclaim_executed,
    };

    let user_sig = UserSig {
        locked_amount: user.locked_amount,
        user_weight_accum: user.user_weight_accum,
        user_last_day_index: user.user_last_day_index,
        reward_claimed: user.reward_claimed,
    };

    let reward_bal = token_balance(&mut ctx, &participant_reward.pubkey()).await;
    let escrow_bal = token_balance(&mut ctx, &reward_escrow.pubkey()).await;

    (iss_sig, user_sig, reward_bal, escrow_bal)
}

#[tokio::test]
async fn same_sequence_same_result() {
    let a = run_scenario().await;
    let b = run_scenario().await;
    assert_eq!(a, b);
}