// tests/042_claim_zero_participation_rejected_pt.rs

#![forbid(unsafe_code)]

use borsh::BorshSerialize;
use solana_program::program_pack::Pack;
use solana_program_test::*;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_program,
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

fn mk_ix(program_id: Pubkey, data: Vec<u8>, metas: Vec<AccountMeta>) -> Instruction {
    Instruction { program_id, accounts: metas, data }
}

#[tokio::test]
async fn claim_zero_participation_rejected_pt() {
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

    // --- Init issuance ---
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
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(payer, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );

    send_tx_ok(&mut ctx, vec![init_ix], &[]).await;

    warp_until_ts(&mut ctx, maturity_ts).await;

    // --- No user_state created ---
    let (user_pda, _) =
        pda::derive_user_pda(&program_id, &issuance_pda, &payer);

    let claim_data = LockrionInstruction::ClaimReward.try_to_vec().unwrap();

    let claim_ix = mk_ix(
        program_id,
        claim_data,
        vec![
            AccountMeta::new(issuance_pda, false),
            AccountMeta::new(user_pda, false), // NOT created
            AccountMeta::new(payer, true),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
    );

    let err = send_tx_err(&mut ctx, vec![claim_ix], &[]).await;

    let expected = LockrionError::InvalidUserStateAccount as u32;

    match err {
        TransactionError::InstructionError(_, InstructionError::Custom(code)) => {
            assert_eq!(code, expected);
        }
        _ => panic!("unexpected error"),
    }
}