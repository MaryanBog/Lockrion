// tests/046_seed_endianness_mutation_invalid_pda_pt.rs

#![forbid(unsafe_code)]

use borsh::BorshSerialize;
use solana_program_test::*;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Signer},
    system_program,
    transaction::Transaction,
};
use solana_sdk::instruction::InstructionError;
use solana_sdk::transaction::TransactionError;

use lockrion_issuance_v1_1::{
    error::LockrionError,
    instruction::LockrionInstruction,
};

async fn send_tx_err(
    ctx: &mut ProgramTestContext,
    ixs: Vec<Instruction>,
) -> TransactionError {
    let payer_pk = ctx.payer.pubkey();
    let mut tx = Transaction::new_with_payer(&ixs, Some(&payer_pk));
    let bh = ctx.banks_client.get_latest_blockhash().await.unwrap();
    tx.sign(&[&ctx.payer], bh);

    match ctx.banks_client.process_transaction(tx).await {
        Ok(_) => panic!("expected failure"),
        Err(e) => e.unwrap(),
    }
}

fn mk_ix(program_id: Pubkey, data: Vec<u8>, metas: Vec<AccountMeta>) -> Instruction {
    Instruction { program_id, accounts: metas, data }
}

#[tokio::test]
async fn seed_endianness_mutation_invalid_pda_pt() {
    let program_id = lockrion_issuance_v1_1::id();

    let pt = ProgramTest::new(
        "lockrion_issuance_v1_1",
        program_id,
        processor!(lockrion_issuance_v1_1::entrypoint::process_instruction),
    );

    let mut ctx = pt.start_with_context().await;
    let payer = ctx.payer.pubkey();

    let clock: solana_sdk::sysvar::clock::Clock =
        ctx.banks_client.get_sysvar().await.unwrap();
    let now = (clock.slot as i64) / 2;

    let reserve_total: u128 = 1000;
    let start_ts = now + 10;
    let maturity_ts = start_ts + 86400;

    // --- WRONG PDA: simulate BE instead of LE for start_ts ---
    let start_ts_be = start_ts.to_be_bytes();
    let reserve_le = reserve_total.to_le_bytes();

    let (mutated_pda, _) = Pubkey::find_program_address(
        &[
            b"issuance",
            payer.as_ref(),
            &start_ts_be,     // WRONG endian
            &reserve_le,
        ],
        &program_id,
    );

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
            AccountMeta::new(mutated_pda, false), // invalid PDA
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(payer, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );

    let err = send_tx_err(&mut ctx, vec![init_ix]).await;

    let expected = LockrionError::InvalidPda as u32;

    match err {
        TransactionError::InstructionError(_, InstructionError::Custom(code)) => {
            assert_eq!(code, expected);
        }
        _ => panic!("unexpected error"),
    }
}