// tests/045_seed_order_mutation_invalid_pda_pt.rs
#![forbid(unsafe_code)]

use borsh::BorshSerialize;
use solana_program::program_pack::Pack;
use solana_program_test::*;
use solana_sdk::{
    instruction::{AccountMeta, Instruction, InstructionError},
    pubkey::Pubkey,
    signature::{read_keypair_file, Keypair, Signer},
    system_instruction, system_program,
    transaction::{Transaction, TransactionError},
};
use spl_token::state::{Account as TokenAccount, Mint};

use lockrion_issuance_v1_1::{
    error::LockrionError,
    instruction::LockrionInstruction,
    pda,
};

async fn send_expect_custom(
    ctx: &mut ProgramTestContext,
    ixs: Vec<Instruction>,
    extra_signers: &[&Keypair],
) -> u32 {
    let payer_pk = ctx.payer.pubkey();
    let mut tx = Transaction::new_with_payer(&ixs, Some(&payer_pk));
    let bh = ctx.banks_client.get_latest_blockhash().await.unwrap();

    let mut signers: Vec<&Keypair> = Vec::with_capacity(1 + extra_signers.len());
    signers.push(&ctx.payer);
    signers.extend_from_slice(extra_signers);

    tx.sign(&signers, bh);

    let err = ctx
        .banks_client
        .process_transaction(tx)
        .await
        .err()
        .expect("tx must fail");

    match err {
        BanksClientError::TransactionError(TransactionError::InstructionError(
            _idx,
            InstructionError::Custom(code),
        )) => code,
        other => panic!("unexpected error: {other:?}"),
    }
}

async fn send_tx_ok(ctx: &mut ProgramTestContext, ixs: Vec<Instruction>, extra_signers: &[&Keypair]) {
    let payer_pk = ctx.payer.pubkey();
    let mut tx = Transaction::new_with_payer(&ixs, Some(&payer_pk));
    let bh = ctx.banks_client.get_latest_blockhash().await.unwrap();

    let mut signers: Vec<&Keypair> = Vec::with_capacity(1 + extra_signers.len());
    signers.push(&ctx.payer);
    signers.extend_from_slice(extra_signers);

    tx.sign(&signers, bh);
    ctx.banks_client.process_transaction(tx).await.unwrap();
}

async fn create_mint(ctx: &mut ProgramTestContext, mint_kp: &Keypair, mint_authority: &Pubkey, decimals: u8) {
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
        decimals,
    )
    .unwrap();

    send_tx_ok(ctx, vec![create, init], &[mint_kp]).await;
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

    send_tx_ok(ctx, vec![create, init], &[acct_kp]).await;
}

fn mk_ix(program_id: Pubkey, data: Vec<u8>, metas: Vec<AccountMeta>) -> Instruction {
    Instruction {
        program_id,
        accounts: metas,
        data,
    }
}

#[tokio::test]
async fn seed_order_mutation_invalid_pda_pt() {
    let program_id = lockrion_issuance_v1_1::id();

    let pt = ProgramTest::new(
        "lockrion_issuance_v1_1",
        program_id,
        processor!(lockrion_issuance_v1_1::entrypoint::process_instruction),
    );

    let mut ctx = pt.start_with_context().await;

    // -----------------------------
    // PLATFORM-CONTROLLED ISSUANCE
    // -----------------------------
    let platform = read_keypair_file("platform-authority.json").unwrap();
    let platform_pk = platform.pubkey();

    // give platform lamports (fee payer is ctx.payer, platform signs the instruction)
    let fund_platform_ix = system_instruction::transfer(&ctx.payer.pubkey(), &platform_pk, 5_000_000_000);
    send_tx_ok(&mut ctx, vec![fund_platform_ix], &[]).await;

    // -------- times --------
    let clock: solana_sdk::sysvar::clock::Clock = ctx.banks_client.get_sysvar().await.unwrap();
    let now: i64 = (clock.slot as i64) / 2;

    let reserve_total: u128 = 1000;
    let start_ts: i64 = now + 10;
    let maturity_ts: i64 = start_ts + 86_400;

    // CORRECT PDA (для справки)
    let (_correct_pda, _) = pda::derive_issuance_pda(&program_id, &platform_pk, start_ts, reserve_total);

    // MUTATED PDA: деривим на НЕПРАВИЛЬНОМ timestamp (maturity_ts вместо start_ts)
    // Это и есть "seed mutation" для PDA
    let (mutated_pda, _) = pda::derive_issuance_pda(&program_id, &platform_pk, maturity_ts, reserve_total);

    // -------- create real mints / token accounts (чтобы не словить другую ошибку раньше InvalidPda) --------
    let lock_mint = Keypair::new();
    let reward_mint = Keypair::new();
    let mint_auth = Keypair::new();
    create_mint(&mut ctx, &lock_mint, &mint_auth.pubkey(), 0).await;
    create_mint(&mut ctx, &reward_mint, &mint_auth.pubkey(), 0).await;

    // IMPORTANT:
    // escrow owners должны соответствовать issuance account, который мы передаём в InitIssuance.
    // Здесь мы специально используем mutated_pda, чтобы "всё остальное" выглядело консистентно,
    // и контракт упал именно на проверке PDA == derive(...)
    let deposit_escrow = Keypair::new();
    let reward_escrow = Keypair::new();
    create_token_account(&mut ctx, &deposit_escrow, &lock_mint.pubkey(), &mutated_pda).await;
    create_token_account(&mut ctx, &reward_escrow, &reward_mint.pubkey(), &mutated_pda).await;

    let platform_treasury = Keypair::new();
    create_token_account(&mut ctx, &platform_treasury, &reward_mint.pubkey(), &platform_pk).await;

    // -------- InitIssuance with WRONG issuance PDA --------
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
            AccountMeta::new(platform_pk, true),                 // signer = PLATFORM
            AccountMeta::new(mutated_pda, false),                // WRONG issuance PDA
            AccountMeta::new_readonly(lock_mint.pubkey(), false),
            AccountMeta::new_readonly(reward_mint.pubkey(), false),
            AccountMeta::new_readonly(deposit_escrow.pubkey(), false),
            AccountMeta::new_readonly(reward_escrow.pubkey(), false),
            AccountMeta::new_readonly(platform_treasury.pubkey(), false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );

    let code = send_expect_custom(&mut ctx, vec![init_ix], &[&platform]).await;

    // Expect: InvalidPda
    let expected = LockrionError::InvalidPda as u32;
    assert_eq!(code, expected, "unexpected custom error code: got={code} expected={expected}");
}