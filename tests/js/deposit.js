const {
  Connection,
  PublicKey,
  Keypair,
  Transaction,
  TransactionInstruction,
  sendAndConfirmTransaction
} = require("@solana/web3.js");

const fs = require("fs");

const RPC = "http://127.0.0.1:8899";

[
  "PROGRAM_ID",
  "ISSUANCE_PDA",
  "PARTICIPANT_LOCK_ATA",
  "DEPOSIT_ESCROW",
  "AMOUNT"
].forEach(k=>{
  if(!process.env[k] || !String(process.env[k]).trim()){
    console.error("MISSING",k);
    process.exit(2);
  }
});

const programId = new PublicKey(process.env.PROGRAM_ID);
const issuancePda = new PublicKey(process.env.ISSUANCE_PDA);
const participantLockAta = new PublicKey(process.env.PARTICIPANT_LOCK_ATA);
const depositEscrow = new PublicKey(process.env.DEPOSIT_ESCROW);
const amount = BigInt(process.env.AMOUNT);

// PARTICIPANT = владелец participantLockAta
const participant = Keypair.fromSecretKey(
  Uint8Array.from(
    JSON.parse(
      fs.readFileSync("platform-authority.json","utf8")
    )
  )
);

const tokenProgram = new PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
const systemProgram = new PublicKey("11111111111111111111111111111111");

// User PDA = ["user", issuance_pda, participant]
const [userPda] = PublicKey.findProgramAddressSync(
  [
    Buffer.from("user"),
    issuancePda.toBuffer(),
    participant.publicKey.toBuffer()
  ],
  programId
);

(async()=>{
  const c = new Connection(RPC,"confirmed");

  const data = Buffer.alloc(1+8);
  data.writeUInt8(2,0); // Deposit discriminant
  data.writeBigUInt64LE(amount,1);

  const keys = [
    {pubkey: issuancePda,          isSigner:false, isWritable:true},
    {pubkey: userPda,              isSigner:false, isWritable:true},
    {pubkey: participant.publicKey,isSigner:true,  isWritable:false},
    {pubkey: participantLockAta,   isSigner:false, isWritable:true},
    {pubkey: depositEscrow,        isSigner:false, isWritable:true},
    {pubkey: tokenProgram,         isSigner:false, isWritable:false},
    {pubkey: systemProgram,        isSigner:false, isWritable:false},
  ];

  const ix = new TransactionInstruction({ programId, keys, data });
  const tx = new Transaction().add(ix);

  const sig = await sendAndConfirmTransaction(c, tx, [participant]);

  console.log("sig", sig);
  console.log("user_pda", userPda.toBase58());
})();