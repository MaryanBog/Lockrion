// ==============================
// tests/033_claim_exact_claim_window_end.rs
// ==============================
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
        let jump_slots = need.saturating_mul(2); // now = slot/2
        ctx.warp_to_slot(c.slot + jump_slots).unwrap();
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

    send_tx(ctx, vec![create, init], &[mint_kp]).await;
}

async fn create_token_account(
    ctx: &mut ProgramTestContext,
    acct_kp: &Keypair,
    mint: &Pubkey,
    owner: &Pubkey,
) {
    let rent = ctx.banks_client.get_rent().await.unwrap();
    let space = spl_token::state::Account::LEN;
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
    send_tx(ctx, vec![create, init], &[acct_kp]).await;
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
        &[] as &[&solana_sdk::pubkey::Pubkey],
        amount,
    )
    .unwrap();
    send_tx(ctx, vec![ix], &[mint_authority]).await;
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
async fn claim_exact_claim_window_end() {
    let program_id = lockrion_issuance_v1_1::id();
    let pt = ProgramTest::new(
        "lockrion_issuance_v1_1",
        program_id,
        processor!(lockrion_issuance_v1_1::entrypoint::process_instruction),
    );
    let mut ctx = pt.start_with_context().await;

    // ✅ STEP 1 — platform keypair
    let platform = read_keypair_file("platform-authority.json").unwrap();

    // ✅ STEP 2 — give platform lamports
    let airdrop_ix = system_instruction::transfer(
        &ctx.payer.pubkey(),
        &platform.pubkey(),
        5_000_000_000,
    );
    send_tx(&mut ctx, vec![airdrop_ix], &[]).await;

    let payer_pk = ctx.payer.pubkey();

    let c: solana_sdk::sysvar::clock::Clock = ctx.banks_client.get_sysvar().await.unwrap();
    let now: i64 = (c.slot as i64) / 2;

    let reserve_total: u128 = 1000;
    let deposit_amount: u64 = 100;

    let start_ts: i64 = now + 10;
    let maturity_ts: i64 = start_ts + 86_400;

    // ✅ STEP 3 — issuance PDA derived from PLATFORM
    let (issuance_pda, _bump) =
        pda::derive_issuance_pda(&program_id, &platform.pubkey(), start_ts, reserve_total);

    let lock_mint = Keypair::new();
    let reward_mint = Keypair::new();
    let mint_auth = Keypair::new();

    create_mint(&mut ctx, &lock_mint, &mint_auth.pubkey(), 0).await;
    create_mint(&mut ctx, &reward_mint, &mint_auth.pubkey(), 0).await;

    let deposit_escrow = Keypair::new();
    let reward_escrow = Keypair::new();
    create_token_account(&mut ctx, &deposit_escrow, &lock_mint.pubkey(), &issuance_pda).await;
    create_token_account(&mut ctx, &reward_escrow, &reward_mint.pubkey(), &issuance_pda).await;

    let issuer_reward = Keypair::new();
    // ✅ STEP 4 — issuer_reward owned by PLATFORM
    create_token_account(&mut ctx, &issuer_reward, &reward_mint.pubkey(), &platform.pubkey()).await;

    // platform_treasury must be a token account; keep as payer-owned (participant)
    let platform_treasury = Keypair::new();
    create_token_account(&mut ctx, &platform_treasury, &reward_mint.pubkey(), &payer_pk).await;

    let participant_lock = Keypair::new();
    create_token_account(&mut ctx, &participant_lock, &lock_mint.pubkey(), &payer_pk).await;

    let participant_reward = Keypair::new();
    create_token_account(&mut ctx, &participant_reward, &reward_mint.pubkey(), &payer_pk).await;

    mint_to(
        &mut ctx,
        &reward_mint.pubkey(),
        &issuer_reward.pubkey(),
        &mint_auth,
        reserve_total as u64,
    )
    .await;
    mint_to(
        &mut ctx,
        &lock_mint.pubkey(),
        &participant_lock.pubkey(),
        &mint_auth,
        deposit_amount,
    )
    .await;

    // init
    let init_ix = mk_ix(
        program_id,
        LockrionInstruction::InitIssuance {
            reserve_total,
            start_ts,
            maturity_ts,
        }
        .try_to_vec()
        .unwrap(),
        vec![
            // ✅ STEP 5 — init signer is PLATFORM
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
    send_tx(&mut ctx, vec![init_ix], &[&platform]).await;

    // fund
    let fund_ix = mk_ix(
        program_id,
        LockrionInstruction::FundReserve {
            amount: reserve_total as u64,
        }
        .try_to_vec()
        .unwrap(),
        vec![
            AccountMeta::new(issuance_pda, false),
            // ✅ STEP 6 — fund signer is PLATFORM
            AccountMeta::new(platform.pubkey(), true),
            AccountMeta::new(issuer_reward.pubkey(), false),
            AccountMeta::new(reward_escrow.pubkey(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
    );
    send_tx(&mut ctx, vec![fund_ix], &[&platform]).await;

    // deposit (participant remains ctx.payer)
    warp_until_ts(&mut ctx, start_ts).await;
    let (user_pda, _ub) = pda::derive_user_pda(&program_id, &issuance_pda, &payer_pk);

    let dep_ix = mk_ix(
        program_id,
        LockrionInstruction::Deposit {
            amount: deposit_amount,
        }
        .try_to_vec()
        .unwrap(),
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
    send_tx(&mut ctx, vec![dep_ix], &[]).await;

    // claim at EXACT end-1
    let claim_window: i64 = 90 * 86_400; // contract default in init_issuance
    let target = maturity_ts + claim_window - 1;
    warp_until_ts(&mut ctx, target).await;

    let before = token_balance(&mut ctx, &participant_reward.pubkey()).await;

    let claim_ix = mk_ix(
        program_id,
        LockrionInstruction::ClaimReward.try_to_vec().unwrap(),
        vec![
            AccountMeta::new(issuance_pda, false),
            AccountMeta::new(user_pda, false),
            AccountMeta::new(payer_pk, true),
            AccountMeta::new(participant_reward.pubkey(), false),
            AccountMeta::new(reward_escrow.pubkey(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
    );
    send_tx(&mut ctx, vec![claim_ix], &[]).await;

    let after = token_balance(&mut ctx, &participant_reward.pubkey()).await;
    assert!(after > before, "claim at claim_window_end-1 did not increase reward");
}