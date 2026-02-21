["PROGRAM_ID","LOCK_MINT","REWARD_MINT","DEPOSIT_ESCROW","REWARD_ESCROW","PLATFORM_TREASURY","START_TS","MATURITY_TS","RESERVE_TOTAL"]
  .forEach(k=>{ if(!process.env[k] || !String(process.env[k]).trim()){ console.error("MISSING",k); process.exit(2);} });

const {Connection,Keypair,PublicKey,Transaction,TransactionInstruction,sendAndConfirmTransaction} = require("@solana/web3.js");
const fs = require("fs");

const RPC="http://127.0.0.1:8899";
const programId = new PublicKey(process.env.PROGRAM_ID);
const payer = Keypair.fromSecretKey(Uint8Array.from(JSON.parse(fs.readFileSync("target/deploy/test-wallet.json","utf8"))));

const reserveTotal = BigInt(process.env.RESERVE_TOTAL);
const startTs = BigInt(process.env.START_TS);
const maturityTs = BigInt(process.env.MATURITY_TS);

const lockMint = new PublicKey(process.env.LOCK_MINT);
const rewardMint = new PublicKey(process.env.REWARD_MINT);
const depositEscrow = new PublicKey(process.env.DEPOSIT_ESCROW);
const rewardEscrow = new PublicKey(process.env.REWARD_ESCROW);
const platformTreasury = new PublicKey(process.env.PLATFORM_TREASURY);

const seed1=Buffer.from("issuance");
const seed2=payer.publicKey.toBuffer();
const seed3=Buffer.alloc(8); seed3.writeBigInt64LE(startTs);
const seed4=Buffer.alloc(16); seed4.writeBigUInt64LE(reserveTotal,0); seed4.writeBigUInt64LE(0n,8);
const [issuancePda] = PublicKey.findProgramAddressSync([seed1,seed2,seed3,seed4], programId);

// === instruction data ===
// ТУТ НУЖНО СООТВЕТСТВИЕ ТВОЕМУ enum/discriminant!
// Если у тебя Instruction::InitIssuance { reserve_total, start_ts, maturity_ts }
// и первый байт = 0, тогда так:
const data = Buffer.alloc(1+16+8+8);
data.writeUInt8(0,0); // DISCRIMINANT INIT = 0 (ПРОВЕРЬ!)
data.writeBigUInt64LE(reserveTotal,1);
data.writeBigUInt64LE(0n,1+8);
data.writeBigInt64LE(startTs,1+16);
data.writeBigInt64LE(maturityTs,1+16+8);

const keys = [
  {pubkey: payer.publicKey, isSigner: true, isWritable: true},
  {pubkey: issuancePda, isSigner: false, isWritable: true},
  {pubkey: lockMint, isSigner: false, isWritable: false},
  {pubkey: rewardMint, isSigner: false, isWritable: false},
  {pubkey: depositEscrow, isSigner: false, isWritable: false},
  {pubkey: rewardEscrow, isSigner: false, isWritable: false},
  {pubkey: platformTreasury, isSigner: false, isWritable: false},
  {pubkey: new PublicKey("11111111111111111111111111111111"), isSigner: false, isWritable: false},
];

(async()=>{
  const c=new Connection(RPC,"confirmed");
  const ix=new TransactionInstruction({programId, keys, data});
  const tx=new Transaction().add(ix);
  const sig=await sendAndConfirmTransaction(c, tx, [payer]);
  console.log("sig",sig);
  console.log("issuance_pda",issuancePda.toBase58());
})();
