import {
    Connection,
    Keypair,
    PublicKey,
    Transaction,
    TransactionInstruction,
    sendAndConfirmTransaction
  } from "@solana/web3.js";
  import fs from "fs";
  
  const RPC_URL = "http://127.0.0.1:8899";
  
  async function main() {
    console.log("INIT RUNNER STARTED");
  
    const connection = new Connection(RPC_URL, "confirmed");
  
    const programId = new PublicKey(process.env.PROGRAM_ID!);
    const issuancePda = new PublicKey(process.env.ISSUANCE_PDA!);
  
    const issuer = Keypair.fromSecretKey(
      Uint8Array.from(
        JSON.parse(fs.readFileSync(process.env.ISSUER_KEYPAIR!, "utf8"))
      )
    );
  
    const startTs = BigInt(process.env.START_TS!);
    const maturityTs = BigInt(process.env.MATURITY_TS!);
    const reserveTotal = BigInt(process.env.RESERVE_TOTAL!);
  
    const data = Buffer.alloc(1 + 16 + 8 + 8);
  
    // enum variant index 0 = InitIssuance
    data.writeUInt8(0, 0);
  
    // u128 reserve_total
    data.writeBigUInt64LE(reserveTotal, 1);
    data.writeBigUInt64LE(0n, 9);
  
    // i64 start_ts
    data.writeBigInt64LE(startTs, 17);
  
    // i64 maturity_ts
    data.writeBigInt64LE(maturityTs, 25);
  
    const ix = new TransactionInstruction({
      programId,
      keys: [
        { pubkey: issuer.publicKey, isSigner: true, isWritable: true },
        { pubkey: issuancePda, isSigner: false, isWritable: true },
        { pubkey: new PublicKey("11111111111111111111111111111111"), isSigner: false, isWritable: false },
      ],
      data,
    });
  
    const tx = new Transaction().add(ix);
  
    const sig = await sendAndConfirmTransaction(connection, tx, [issuer]);
  
    console.log("Init TX:", sig);
  }
  
  main().catch(console.error);