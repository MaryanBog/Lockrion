// tests/023_invalid_token_program_rejected_pt.rs
#![forbid(unsafe_code)]

use borsh::BorshSerialize;
use solana_program_test::*;
use solana_sdk::{
    instruction::{AccountMeta, Instruction, InstructionError},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_program,
    transaction::{Transaction, TransactionError},
};

use lockrion_issuance_v1_1::{
    error::LockrionError,
    instruction::LockrionInstruction,
    pda,
};

fn mk_ix(program_id: Pubkey, data: Vec<u8>, accounts: Vec<AccountMeta>) -> Instruction {
    Instruction { program_id, accounts, data }
}

async fn send_expect_custom_err(ctx: &mut ProgramTestContext, ixs: Vec<Instruction>, expected_code: u32) {
    let payer = ctx.payer.pubkey();
    let mut tx = Transaction::new_with_payer(&ixs, Some(&payer));
    let bh = ctx.banks_client.get_latest_blockhash().await.unwrap();
    tx.sign(&[&ctx.payer], bh);

    let err = ctx.banks_client.process_transaction(tx).await.unwrap_err();

    match err {
        BanksClientError::TransactionError(TransactionError::InstructionError(
            _,
            InstructionError::Custom(code),
        )) => {
            assert_eq!(code, expected_code, "wrong custom error code");
        }
        _ => panic!("unexpected error: {:?}", err),
    }
}

#[tokio::test]
async fn invalid_token_program_rejected_pt() {
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
    let start_ts: i64 = now + 10;
    let maturity_ts: i64 = start_ts + 86_400;

    let (issuance_pda, _) =
        pda::derive_issuance_pda(&program_id, &payer_pk, start_ts, reserve_total);

    // For this test we intentionally pass WRONG token program id
    let wrong_token_program = system_program::id(); // definitely not spl_token::id()

    let fund_ix = mk_ix(
        program_id,
        LockrionInstruction::FundReserve { amount: 1000 }
            .try_to_vec()
            .unwrap(),
        vec![
            AccountMeta::new(issuance_pda, false),
            AccountMeta::new(payer_pk, true),
            AccountMeta::new(Pubkey::new_unique(), false), // fake issuer_reward_ata
            AccountMeta::new(Pubkey::new_unique(), false), // fake reward_escrow
            AccountMeta::new_readonly(wrong_token_program, false), // WRONG TOKEN PROGRAM
        ],
    );

    send_expect_custom_err(
        &mut ctx,
        vec![fund_ix],
        LockrionError::InvalidTokenProgram as u32,
    )
    .await;
}