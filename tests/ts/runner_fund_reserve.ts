console.log("RUNNER STARTED");

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
    const connection = new Connection(RPC_URL, "confirmed");
  
    const programId = new PublicKey(process.env.PROGRAM_ID!);
    const issuancePda = new PublicKey(process.env.ISSUANCE_PDA!);
    const issuer = Keypair.fromSecretKey(
      Uint8Array.from(JSON.parse(fs.readFileSync(process.env.ISSUER_KEYPAIR!, "utf8")))
    );
    const issuerRewardAta = new PublicKey(process.env.ISSUER_REWARD_ATA!);
    const rewardEscrow = new PublicKey(process.env.REWARD_ESCROW!);
    const tokenProgram = new PublicKey(
      "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
    );
  
    const amount = BigInt(process.env.AMOUNT!);
  
    // Build instruction data: variant=0 + u64 LE
    const data = Buffer.alloc(9);
    data.writeUInt8(1, 0);
    data.writeBigUInt64LE(amount, 1);
  
    const ix = new TransactionInstruction({
      programId,
      keys: [
        { pubkey: issuancePda, isSigner: false, isWritable: true },
        { pubkey: issuer.publicKey, isSigner: true, isWritable: false },
        { pubkey: issuerRewardAta, isSigner: false, isWritable: true },
        { pubkey: rewardEscrow, isSigner: false, isWritable: true },
        { pubkey: tokenProgram, isSigner: false, isWritable: false },
      ],
      data,
    });
  
    const tx = new Transaction().add(ix);
  
    const sig = await sendAndConfirmTransaction(
      connection,
      tx,
      [issuer]
    );
  
    console.log("FundReserve TX:", sig);
  }
  
  main().catch((e) => {
    console.error(e);
    process.exit(1);
  });