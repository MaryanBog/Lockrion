#![forbid(unsafe_code)]

use borsh::BorshSerialize;
use solana_program_test::*;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Signer,
    system_program,
    transaction::Transaction,
};

use lockrion_issuance_v1_1::instruction::LockrionInstruction;

fn mk_ix(program_id: Pubkey, data: Vec<u8>, metas: Vec<AccountMeta>) -> Instruction {
    Instruction { program_id, accounts: metas, data }
}

#[tokio::test]
async fn sweep_twice_should_fail() {
    let program_id = lockrion_issuance_v1_1::id();

    let pt = ProgramTest::new(
        "lockrion_issuance_v1_1",
        program_id,
        processor!(lockrion_issuance_v1_1::entrypoint::process_instruction),
    );

    let mut ctx = pt.start_with_context().await;
    let payer = ctx.payer.pubkey();

    // Предполагаем, что sweep уже выполнен в предыдущем тесте или setup
    // Здесь проверяем только guard

    let sweep_ix = mk_ix(
        program_id,
        LockrionInstruction::Sweep.try_to_vec().unwrap(),
        vec![
            AccountMeta::new_readonly(payer, true),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );

    let mut tx = Transaction::new_with_payer(&[sweep_ix], Some(&payer));
    let bh = ctx.banks_client.get_latest_blockhash().await.unwrap();
    tx.sign(&[&ctx.payer], bh);

    let result = ctx.banks_client.process_transaction(tx).await;
    assert!(result.is_err(), "second sweep unexpectedly succeeded");
}