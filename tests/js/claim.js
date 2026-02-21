// ==============================
// tests/js/claim.js
// ==============================
const {
  Connection,
  PublicKey,
  Keypair,
  Transaction,
  TransactionInstruction,
  sendAndConfirmTransaction,
  SendTransactionError,
} = require("@solana/web3.js");
const fs = require("fs");

const RPC = "http://127.0.0.1:8899";

[
  "PROGRAM_ID",
  "ISSUANCE_PDA",
  "PARTICIPANT_REWARD_ATA",
  "REWARD_ESCROW",
].forEach((k) => {
  if (!process.env[k] || !String(process.env[k]).trim()) {
    console.error("MISSING", k);
    process.exit(2);
  }
});

const programId = new PublicKey(process.env.PROGRAM_ID);
const issuancePda = new PublicKey(process.env.ISSUANCE_PDA);
const participantRewardAta = new PublicKey(process.env.PARTICIPANT_REWARD_ATA);
const rewardEscrow = new PublicKey(process.env.REWARD_ESCROW);

const payer = Keypair.fromSecretKey(
  Uint8Array.from(
    JSON.parse(fs.readFileSync("target/deploy/test-wallet.json", "utf8"))
  )
);

const tokenProgram = new PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

// Canonical UserState PDA: [ "user", issuance_pda, participant_pubkey ]
const [userPda] = PublicKey.findProgramAddressSync(
  [Buffer.from("user"), issuancePda.toBuffer(), payer.publicKey.toBuffer()],
  programId
);

(async () => {
  const c = new Connection(RPC, "confirmed");

  // Borsh enum variant index for LockrionInstruction::ClaimReward
  // Enum order (instruction.rs): Init=0, Fund=1, Deposit=2, Claim=3
  const data = Buffer.from([3]);

  const keys = [
    // 0 [writable] issuance_state (PDA)
    { pubkey: issuancePda, isSigner: false, isWritable: true },
    // 1 [writable] user_state (PDA)
    { pubkey: userPda, isSigner: false, isWritable: true },
    // 2 [signer] participant
    { pubkey: payer.publicKey, isSigner: true, isWritable: false },
    // 3 [writable] participant_reward_ata
    { pubkey: participantRewardAta, isSigner: false, isWritable: true },
    // 4 [writable] reward_escrow
    { pubkey: rewardEscrow, isSigner: false, isWritable: true },
    // 5 [] token_program
    { pubkey: tokenProgram, isSigner: false, isWritable: false },
  ];

  const ix = new TransactionInstruction({ programId, keys, data });
  const tx = new Transaction().add(ix);

  try {
    const sig = await sendAndConfirmTransaction(c, tx, [payer], {
      commitment: "confirmed",
    });
    console.log("sig", sig);
    console.log("user_pda", userPda.toBase58());
  } catch (e) {
    console.error("CLAIM FAILED:", e?.message || e);

    if (e instanceof SendTransactionError) {
      try {
        const logs = await e.getLogs(c);
        if (logs) console.error("LOGS:\n" + logs.join("\n"));
      } catch (_) {}
    } else if (e?.transactionLogs) {
      console.error("LOGS:\n" + e.transactionLogs.join("\n"));
    }

    process.exit(1);
  }
})();