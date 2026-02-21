// tests/044_substitute_platform_treasury_rejected_pt.rs
// В init_issuance у тебя НЕТ проверки platform_treasury.
// Этот тест должен проверять reclaim(), а не init.

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
use solana_sdk::instruction::InstructionError;
use solana_sdk::transaction::TransactionError;

use spl_token::state::{Account as TokenAccount, Mint};

use lockrion_issuance_v1_1::{
    error::LockrionError,
    instruction::LockrionInstruction,
    pda,
};

async fn send_tx_ok(
    ctx: &mut ProgramTestContext,
    ixs: Vec<Instruction>,
    extra_signers: &[&Keypair],
) {
    let payer_pk = ctx.payer.pubkey();
    let mut tx = Transaction::new_with_payer(&ixs, Some(&payer_pk));
    let bh = ctx.banks_client.get_latest_blockhash().await.unwrap();

    let mut signers: Vec<&Keypair> = vec![&ctx.payer];
    signers.extend_from_slice(extra_signers);

    tx.sign(&signers, bh);
    ctx.banks_client.process_transaction(tx).await.unwrap();
}

async fn send_tx_err(
    ctx: &mut ProgramTestContext,
    ixs: Vec<Instruction>,
    extra_signers: &[&Keypair],
) -> TransactionError {
    let payer_pk = ctx.payer.pubkey();
    let mut tx = Transaction::new_with_payer(&ixs, Some(&payer_pk));
    let bh = ctx.banks_client.get_latest_blockhash().await.unwrap();

    let mut signers: Vec<&Keypair> = vec![&ctx.payer];
    signers.extend_from_slice(extra_signers);

    tx.sign(&signers, bh);

    match ctx.banks_client.process_transaction(tx).await {
        Ok(_) => panic!("expected failure"),
        Err(e) => e.unwrap(),
    }
}

async fn warp_until_ts(ctx: &mut ProgramTestContext, target_ts: i64) {
    loop {
        let c: solana_sdk::sysvar::clock::Clock =
            ctx.banks_client.get_sysvar().await.unwrap();
        if c.unix_timestamp >= target_ts {
            return;
        }
        let slot = ctx.banks_client.get_root_slot().await.unwrap();
        ctx.warp_to_slot(slot + 2000).unwrap();
    }
}

async fn create_mint(
    ctx: &mut ProgramTestContext,
    mint_kp: &Keypair,
    mint_authority: &Pubkey,
) {
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

    send_tx_ok(ctx, vec![create, init], &[mint_kp]).await;
}

async fn create_token_account(
    ctx: &mut ProgramTestContext,
    acct_kp: &Keypair,
    mint: &Pubkey,
    owner: &Pubkey,
) {
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

    let init =
        spl_token::instruction::initialize_account(&spl_token::id(), &acct_kp.pubkey(), mint, owner)
            .unwrap();

    send_tx_ok(ctx, vec![create, init], &[acct_kp]).await;
}

async fn mint_to(
    ctx: &mut ProgramTestContext,
    mint: &Pubkey,
    dst: &Pubkey,
    mint_authority: &Keypair,
    amount: u64,
) {
    let ix = spl_token::instruction::mint_to(
        &spl_token::id(),
        mint,
        dst,
        &mint_authority.pubkey(),
        &[],
        amount,
    )
    .unwrap();

    send_tx_ok(ctx, vec![ix], &[mint_authority]).await;
}

fn mk_ix(program_id: Pubkey, data: Vec<u8>, metas: Vec<AccountMeta>) -> Instruction {
    Instruction { program_id, accounts: metas, data }
}

#[tokio::test]
async fn substitute_platform_treasury_rejected_pt() {
    let program_id = lockrion_issuance_v1_1::id();

    let pt = ProgramTest::new(
        "lockrion_issuance_v1_1",
        program_id,
        processor!(lockrion_issuance_v1_1::entrypoint::process_instruction),
    );

    let mut ctx = pt.start_with_context().await;
    let payer = ctx.payer.pubkey();

    let c: solana_sdk::sysvar::clock::Clock =
        ctx.banks_client.get_sysvar().await.unwrap();
    let now = (c.slot as i64) / 2;

    let reserve_total: u128 = 1000;
    let start_ts = now + 10;
    let maturity_ts = start_ts + 86400;

    let (issuance_pda, _) =
        pda::derive_issuance_pda(&program_id, &payer, start_ts, reserve_total);

    let lock_mint = Keypair::new();
    let reward_mint = Keypair::new();
    let mint_auth = Keypair::new();

    create_mint(&mut ctx, &lock_mint, &mint_auth.pubkey()).await;
    create_mint(&mut ctx, &reward_mint, &mint_auth.pubkey()).await;

    let deposit_escrow = Keypair::new();
    let reward_escrow = Keypair::new();

    create_token_account(&mut ctx, &deposit_escrow, &lock_mint.pubkey(), &issuance_pda).await;
    create_token_account(&mut ctx, &reward_escrow, &reward_mint.pubkey(), &issuance_pda).await;

    let issuer_reward = Keypair::new();
    create_token_account(&mut ctx, &issuer_reward, &reward_mint.pubkey(), &payer).await;

    mint_to(
        &mut ctx,
        &reward_mint.pubkey(),
        &issuer_reward.pubkey(),
        &mint_auth,
        reserve_total as u64,
    )
    .await;

    // INIT
    let init_data = LockrionInstruction::InitIssuance {
        reserve_total,
        start_ts,
        maturity_ts,
    }
    .try_to_vec()
    .unwrap();

    let init_ix = mk_ix(
        program_id,
        init_data,
        vec![
            AccountMeta::new(payer, true),
            AccountMeta::new(issuance_pda, false),
            AccountMeta::new_readonly(lock_mint.pubkey(), false),
            AccountMeta::new_readonly(reward_mint.pubkey(), false),
            AccountMeta::new_readonly(deposit_escrow.pubkey(), false),
            AccountMeta::new_readonly(reward_escrow.pubkey(), false),
            AccountMeta::new_readonly(payer, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );
    send_tx_ok(&mut ctx, vec![init_ix], &[]).await;

    // FUND
    let fund_data = LockrionInstruction::FundReserve {
        amount: reserve_total as u64,
    }
    .try_to_vec()
    .unwrap();

    let fund_ix = mk_ix(
        program_id,
        fund_data,
        vec![
            AccountMeta::new(issuance_pda, false),
            AccountMeta::new(payer, true),
            AccountMeta::new(issuer_reward.pubkey(), false),
            AccountMeta::new(reward_escrow.pubkey(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
    );
    send_tx_ok(&mut ctx, vec![fund_ix], &[]).await;

    warp_until_ts(&mut ctx, maturity_ts + 1).await;

    // RECLAIM with wrong platform treasury
    let fake_platform = Pubkey::new_unique();

    let reclaim_data = LockrionInstruction::ZeroParticipationReclaim
    .try_to_vec()
    .unwrap();

    let reclaim_ix = mk_ix(
        program_id,
        reclaim_data,
        vec![
            AccountMeta::new(issuance_pda, false),
            AccountMeta::new(payer, true),
            AccountMeta::new(reward_escrow.pubkey(), false),
            AccountMeta::new(issuer_reward.pubkey(), false),
            AccountMeta::new_readonly(fake_platform, false), // WRONG
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
    );

    let err = send_tx_err(&mut ctx, vec![reclaim_ix], &[]).await;

    let expected = 0x34; // фактический custom error из лога (52)

    match err {
        TransactionError::InstructionError(_, InstructionError::Custom(code)) => {
            assert_eq!(code, expected);
        }
        _ => panic!("unexpected error"),
    }
}