const {PublicKey} = require("@solana/web3.js");

["MINT","OWNER"].forEach(k=>{
  if(!process.env[k] || !String(process.env[k]).trim()){
    console.error("MISSING",k);
    process.exit(2);
  }
});

const mint = new PublicKey(process.env.MINT);
const owner = new PublicKey(process.env.OWNER);

// Associated Token Program + SPL Token Program
const ATA_PROGRAM = new PublicKey("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
const TOKEN_PROGRAM = new PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

const [ata] = PublicKey.findProgramAddressSync(
  [owner.toBuffer(), TOKEN_PROGRAM.toBuffer(), mint.toBuffer()],
  ATA_PROGRAM
);

process.stdout.write(ata.toBase58());