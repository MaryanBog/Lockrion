const {Connection,PublicKey,Keypair,Transaction,TransactionInstruction,sendAndConfirmTransaction} = require("@solana/web3.js");
const fs = require("fs");

const RPC="http://127.0.0.1:8899";

["PROGRAM_ID","ISSUANCE_PDA","ISSUER_REWARD_ATA","REWARD_ESCROW","AMOUNT"].forEach(k=>{
  if(!process.env[k] || !String(process.env[k]).trim()){ console.error("MISSING",k); process.exit(2); }
});

const programId=new PublicKey(process.env.PROGRAM_ID);
const issuancePda=new PublicKey(process.env.ISSUANCE_PDA);
const issuerRewardAta=new PublicKey(process.env.ISSUER_REWARD_ATA);
const rewardEscrow=new PublicKey(process.env.REWARD_ESCROW);
const amount=BigInt(process.env.AMOUNT);

const payer=Keypair.fromSecretKey(Uint8Array.from(JSON.parse(fs.readFileSync("target/deploy/test-wallet.json","utf8"))));
const tokenProgram=new PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

(async()=>{
  const c=new Connection(RPC,"confirmed");

  const data=Buffer.alloc(1+8);
  data.writeUInt8(1,0);
  data.writeBigUInt64LE(amount,1);

  const keys=[
    {pubkey: issuancePda,      isSigner:false, isWritable:true},
    {pubkey: payer.publicKey,  isSigner:true,  isWritable:false},
    {pubkey: issuerRewardAta,  isSigner:false, isWritable:true},
    {pubkey: rewardEscrow,     isSigner:false, isWritable:true},
    {pubkey: tokenProgram,     isSigner:false, isWritable:false},
  ];

  const ix=new TransactionInstruction({programId, keys, data});
  const tx=new Transaction().add(ix);
  const sig=await sendAndConfirmTransaction(c, tx, [payer]);
  console.log("sig",sig);
})();
