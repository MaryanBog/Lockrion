const { Connection, PublicKey } = require("@solana/web3.js");

const RPC_URL = "http://127.0.0.1:8899";
const issuancePdaStr = process.env.ISSUANCE_PDA;

if (!issuancePdaStr || !issuancePdaStr.trim()) {
  console.error("ISSUANCE_PDA is empty");
  process.exit(2);
}

const issuancePda = new PublicKey(issuancePdaStr.trim());

(async () => {
  const c = new Connection(RPC_URL, "confirmed");
  const ai = await c.getAccountInfo(issuancePda);
  if (!ai) {
    console.error("AccountNotFound");
    process.exit(3);
  }
  const d = Buffer.from(ai.data);

  const pk = (a, b) => new PublicKey(d.subarray(a, b)).toBase58();

  console.log("issuance_pda", issuancePda.toBase58());
  console.log("owner_program", ai.owner.toBase58());
  console.log("reward_mint", pk(66, 98));
  console.log("reward_escrow", pk(130, 162));
  console.log("deposit_escrow", pk(98, 130));
  console.log("reserve_total_u128_le", d.subarray(194, 210).toString("hex"));
  console.log("start_ts_i64_le", d.subarray(210, 218).toString("hex"));
  console.log("maturity_ts_i64_le", d.subarray(218, 226).toString("hex"));
  console.log("reserve_funded", d.readUInt8(282));
})();
