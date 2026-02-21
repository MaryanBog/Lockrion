// tests/017_zero_participation_reclaim_happy_pt.rs
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

use lockrion_issuance_v1_1::{
    instruction::LockrionInstruction,
    pda,
};

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

async fn warp_until_ts(ctx: &mut ProgramTestContext, target_ts: i64) {
    loop {
        let c: solana_sdk::sysvar::clock::Clock = ctx.banks_client.get_sysvar().await.unwrap();
        let now: i64 = (c.slot as i64) / 2; // must match feature test-clock

        if now >= target_ts {
            return;
        }

        let need = (target_ts - now) as u64;
        let jump_slots = need.saturating_mul(2);
        ctx.warp_to_slot(c.slot + jump_slots + 10).unwrap();
    }
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

    let init =
        spl_token::instruction::initialize_account(&spl_token::id(), &acct_kp.pubkey(), mint, owner).unwrap();

    send_tx_ok(ctx, vec![create, init], &[acct_kp]).await;
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

    send_tx_ok(ctx, vec![ix], &[mint_authority]).await;
}

async fn token_balance(ctx: &mut ProgramTestContext, token_acc: &Pubkey) -> u64 {
    let acc = ctx.banks_client.get_account(*token_acc).await.unwrap().unwrap();
    let ta = TokenAccount::unpack_from_slice(&acc.data).unwrap();
    ta.amount
}

fn mk_ix(program_id: Pubkey, data: Vec<u8>, metas: Vec<AccountMeta>) -> Instruction {
    Instruction { program_id, accounts: metas, data }
}

#[tokio::test]
async fn zero_participation_reclaim_happy_program_test() {
    let program_id = lockrion_issuance_v1_1::id();

    let pt = ProgramTest::new(
        "lockrion_issuance_v1_1",
        program_id,
        processor!(lockrion_issuance_v1_1::entrypoint::process_instruction),
    );

    let mut ctx = pt.start_with_context().await;
    let payer_pk = ctx.payer.pubkey();

    // now matches feature test-clock: slot/2
    let c: solana_sdk::sysvar::clock::Clock = ctx.banks_client.get_sysvar().await.unwrap();
    let now: i64 = (c.slot as i64) / 2;

    let reserve_total: u128 = 1000;

    // NO deposit => total_weight_accum stays 0 => reclaim should succeed after maturity
    let start_ts: i64 = now + 10;
    let maturity_ts: i64 = start_ts + 86_400;

    let (issuance_pda, _bump) = pda::derive_issuance_pda(&program_id, &payer_pk, start_ts, reserve_total);

    // mints
    let reward_mint = Keypair::new();
    let lock_mint = Keypair::new();
    let mint_auth = Keypair::new();
    create_mint(&mut ctx, &reward_mint, &mint_auth.pubkey(), 0).await;
    create_mint(&mut ctx, &lock_mint, &mint_auth.pubkey(), 0).await;

    // escrows (authority MUST be issuance_pda)
    let reward_escrow = Keypair::new();
    create_token_account(&mut ctx, &reward_escrow, &reward_mint.pubkey(), &issuance_pda).await;

    // deposit_escrow still required by init (but unused)
    let deposit_escrow = Keypair::new();
    create_token_account(&mut ctx, &deposit_escrow, &lock_mint.pubkey(), &issuance_pda).await;

    // issuer reward token account (for initial mint + reclaim destination)
    let issuer_reward = Keypair::new();
    create_token_account(&mut ctx, &issuer_reward, &reward_mint.pubkey(), &payer_pk).await;

    // mint reserve into issuer reward, then fund reserve into escrow
    mint_to(&mut ctx, &reward_mint.pubkey(), &issuer_reward.pubkey(), &mint_auth, reserve_total as u64).await;

    // init_issuance (platform_treasury = payer_pk OK, reclaim doesn't validate it)
    let init_data = LockrionInstruction::InitIssuance { reserve_total, start_ts, maturity_ts }
        .try_to_vec()
        .unwrap();

    let init_ix = mk_ix(
        program_id,
        init_data,
        vec![
            AccountMeta::new(payer_pk, true),
            AccountMeta::new(issuance_pda, false),
            AccountMeta::new_readonly(lock_mint.pubkey(), false),
            AccountMeta::new_readonly(reward_mint.pubkey(), false),
            AccountMeta::new_readonly(deposit_escrow.pubkey(), false),
            AccountMeta::new_readonly(reward_escrow.pubkey(), false),
            AccountMeta::new_readonly(payer_pk, false), // platform_treasury
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );
    send_tx_ok(&mut ctx, vec![init_ix], &[]).await;

    // fund_reserve before start_ts
    let fund_ix = mk_ix(
        program_id,
        LockrionInstruction::FundReserve { amount: reserve_total as u64 }
            .try_to_vec()
            .unwrap(),
        vec![
            AccountMeta::new(issuance_pda, false),
            AccountMeta::new(payer_pk, true),
            AccountMeta::new(issuer_reward.pubkey(), false),
            AccountMeta::new(reward_escrow.pubkey(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
    );
    send_tx_ok(&mut ctx, vec![fund_ix], &[]).await;

    // warp to maturity
    warp_until_ts(&mut ctx, maturity_ts).await;

    let issuer_before = token_balance(&mut ctx, &issuer_reward.pubkey()).await;
    let escrow_before = token_balance(&mut ctx, &reward_escrow.pubkey()).await;
    assert!(escrow_before > 0, "escrow must hold funded reserve");

    // reclaim: issuance, issuer(signer), issuer_reward_ata, reward_escrow, token_program
    let reclaim_ix = mk_ix(
        program_id,
        LockrionInstruction::ZeroParticipationReclaim.try_to_vec().unwrap(),
        vec![
            AccountMeta::new(issuance_pda, false),
            AccountMeta::new(payer_pk, true),
            AccountMeta::new(issuer_reward.pubkey(), false),
            AccountMeta::new(reward_escrow.pubkey(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
    );
    send_tx_ok(&mut ctx, vec![reclaim_ix], &[]).await;

    let issuer_after = token_balance(&mut ctx, &issuer_reward.pubkey()).await;
    let escrow_after = token_balance(&mut ctx, &reward_escrow.pubkey()).await;

    assert_eq!(escrow_after, 0, "escrow must be empty after reclaim");
    assert_eq!(issuer_after, issuer_before + escrow_before, "issuer must receive full escrow balance");
}