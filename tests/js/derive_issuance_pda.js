const { PublicKey } = require("@solana/web3.js");

try {
  const programId = new PublicKey(process.argv[2]);
  const issuer    = new PublicKey(process.argv[3]);
  const startTs   = BigInt(process.argv[4]);
  const reserve   = BigInt(process.argv[5]);

  const startBuf = Buffer.alloc(8);
  startBuf.writeBigInt64LE(startTs);

  const reserveBuf = Buffer.alloc(16);
  reserveBuf.writeBigUInt64LE(reserve, 0);
  reserveBuf.writeBigUInt64LE(0n, 8);

  const [pda] = PublicKey.findProgramAddressSync(
    [Buffer.from("issuance"), issuer.toBuffer(), startBuf, reserveBuf],
    programId
  );

  console.log(pda.toBase58());
} catch (e) {
  console.error(e);
  process.exit(1);
}
