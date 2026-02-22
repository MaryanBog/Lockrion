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
use solana_sdk::instruction::InstructionError;
use solana_sdk::transaction::TransactionError;

use spl_token::state::{Account as TokenAccount, Mint};

use lockrion_issuance_v1_1::{
    error::LockrionError,
    instruction::LockrionInstruction,
    pda,
    state::IssuanceState,
};

async fn send_tx_ok(
    ctx: &mut ProgramTestContext,
    ixs: Vec<Instruction>,
    extra_signers: &[&Keypair],
) {
    let payer_pk = ctx.payer.pubkey();
    let mut tx = Transaction::new_with_payer(&ixs, Some(&payer_pk));
    let bh = ctx.banks_client.get_latest_blockhash().await.unwrap();

    let mut signers: Vec<&Keypair> = Vec::with_capacity(1 + extra_signers.len());
    signers.push(&ctx.payer);
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

    let mut signers: Vec<&Keypair> = Vec::with_capacity(1 + extra_signers.len());
    signers.push(&ctx.payer);
    signers.extend_from_slice(extra_signers);

    tx.sign(&signers, bh);

    match ctx.banks_client.process_transaction(tx).await {
        Ok(_) => panic!("expected transaction failure, but it succeeded"),
        Err(e) => e.unwrap(),
    }
}

async fn create_mint(
    ctx: &mut ProgramTestContext,
    mint_kp: &Keypair,
    mint_authority: &Pubkey,
    decimals: u8,
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
        decimals,
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

async fn token_balance(ctx: &mut ProgramTestContext, token_acc: &Pubkey) -> u64 {
    let acc = ctx.banks_client.get_account(*token_acc).await.unwrap().unwrap();
    let ta = TokenAccount::unpack_from_slice(&acc.data).unwrap();
    ta.amount
}

fn mk_ix(program_id: Pubkey, data: Vec<u8>, metas: Vec<AccountMeta>) -> Instruction {
    Instruction {
        program_id,
        accounts: metas,
        data,
    }
}

#[tokio::test]
async fn fund_reserve_wrong_amount_rejected_pt() {
    let program_id = lockrion_issuance_v1_1::id();

    let pt = ProgramTest::new(
        "lockrion_issuance_v1_1",
        program_id,
        processor!(lockrion_issuance_v1_1::entrypoint::process_instruction),
    );

    let mut ctx = pt.start_with_context().await;

    // ✅ platform authority (issuer)
    let platform = read_keypair_file("platform-authority.json").unwrap();

    // ✅ give platform lamports for fees
    let airdrop_ix = system_instruction::transfer(
        &ctx.payer.pubkey(),
        &platform.pubkey(),
        5_000_000_000,
    );
    send_tx_ok(&mut ctx, vec![airdrop_ix], &[]).await;

    let payer_pk = ctx.payer.pubkey();
    let issuer_pk = platform.pubkey();

    // -------- params --------
    let c: solana_sdk::sysvar::clock::Clock = ctx.banks_client.get_sysvar().await.unwrap();
    let now: i64 = (c.slot as i64) / 2;

    let reserve_total: u128 = 1000;
    let wrong_amount: u64 = (reserve_total as u64).saturating_sub(1);

    // fund_reserve requires now < start_ts
    let start_ts: i64 = now + 10;
    let maturity_ts: i64 = start_ts + 86_400;

    // ✅ issuance PDA derived from PLATFORM
    let (issuance_pda, _bump) =
        pda::derive_issuance_pda(&program_id, &issuer_pk, start_ts, reserve_total);

    // -------- mints --------
    let lock_mint = Keypair::new();
    let reward_mint = Keypair::new();
    let mint_auth = Keypair::new();

    create_mint(&mut ctx, &lock_mint, &mint_auth.pubkey(), 0).await;
    create_mint(&mut ctx, &reward_mint, &mint_auth.pubkey(), 0).await;

    // -------- token accounts --------
    let deposit_escrow = Keypair::new();
    let reward_escrow = Keypair::new();

    create_token_account(&mut ctx, &deposit_escrow, &lock_mint.pubkey(), &issuance_pda).await;
    create_token_account(&mut ctx, &reward_escrow, &reward_mint.pubkey(), &issuance_pda).await;

    let issuer_reward = Keypair::new();
    // ✅ issuer_reward must be owned by PLATFORM
    create_token_account(&mut ctx, &issuer_reward, &reward_mint.pubkey(), &issuer_pk).await;

    // mint issuer reward balance (enough to fund even the correct amount)
    mint_to(
        &mut ctx,
        &reward_mint.pubkey(),
        &issuer_reward.pubkey(),
        &mint_auth,
        reserve_total as u64,
    )
    .await;

    // -------- init_issuance (signer PLATFORM) --------
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
            AccountMeta::new(issuer_pk, true), // ✅ platform signer
            AccountMeta::new(issuance_pda, false),
            AccountMeta::new_readonly(lock_mint.pubkey(), false),
            AccountMeta::new_readonly(reward_mint.pubkey(), false),
            AccountMeta::new_readonly(deposit_escrow.pubkey(), false),
            AccountMeta::new_readonly(reward_escrow.pubkey(), false),
            AccountMeta::new_readonly(payer_pk, false), // platform_treasury (ok for this test)
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );
    send_tx_ok(&mut ctx, vec![init_ix], &[&platform]).await;

    // -------- fund_reserve (WRONG amount, signer PLATFORM) --------
    let fund_data = LockrionInstruction::FundReserve { amount: wrong_amount }
        .try_to_vec()
        .unwrap();

    let fund_ix = mk_ix(
        program_id,
        fund_data,
        vec![
            AccountMeta::new(issuance_pda, false),
            AccountMeta::new(issuer_pk, true),                // ✅ platform signer
            AccountMeta::new(issuer_reward.pubkey(), false),
            AccountMeta::new(reward_escrow.pubkey(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
    );

    let err = send_tx_err(&mut ctx, vec![fund_ix], &[&platform]).await;

    // Expect: InvalidFundingAmount
    let expected_code = LockrionError::InvalidFundingAmount as u32;

    match err {
        TransactionError::InstructionError(_idx, InstructionError::Custom(code)) => {
            assert_eq!(
                code, expected_code,
                "unexpected custom error code: got={code} expected={expected_code}"
            );
        }
        other => panic!("unexpected transaction error: {other:?}"),
    }

    // -------- postconditions --------
    // reserve_funded remains false
    let iss_acc = ctx.banks_client.get_account(issuance_pda).await.unwrap().unwrap();
    let iss = IssuanceState::unpack(&iss_acc.data).unwrap();
    assert_eq!(iss.reserve_funded, 0, "reserve_funded mutated on failed funding");

    // reward escrow balance unchanged (0)
    let esc_bal = token_balance(&mut ctx, &reward_escrow.pubkey()).await;
    assert_eq!(esc_bal, 0, "reward escrow balance changed on failed funding");
}