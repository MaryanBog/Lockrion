# Lockrion Issuance Contract
## PDA Seed Model v1.1

Status: Draft  
Applies To: Raw Solana Program  
Scope: Deterministic PDA derivation rules  

---

# 1. General Rule

All PDAs used in Issuance Contract v1.1
are deterministic and part of protocol definition.

Seed structure is immutable.

Seed ordering MUST NOT change.

All numeric values used in seeds
MUST be encoded in little-endian format.

---

## 2. Issuance PDA (Issuance State Address + Escrow Authority)

In v1.1, the issuance PDA is canonical and serves two roles:

1) The Issuance State account address (single global state per issuance)
2) The sole escrow authority for both deposit_escrow and reward_escrow

Seeds:

- b"issuance"
- issuer_address (Pubkey)
- start_ts (i64, little-endian)
- reserve_total (u128, little-endian)

Derivation:

Pubkey::find_program_address(
    [
        b"issuance",
        issuer_address.as_ref(),
        start_ts.to_le_bytes(),
        reserve_total.to_le_bytes()
    ],
    program_id
)

Seed ordering MUST NOT change in v1.1.
All numeric values used in seeds MUST be encoded in little-endian format.

---

# 3. UserState PDA

Seeds:

- b"user"
- issuance_pubkey (Pubkey)
- participant_pubkey (Pubkey)

Derivation:

Pubkey::find_program_address(
    [
        b"user",
        issuance_pubkey.as_ref(),
        participant_pubkey.as_ref()
    ],
    program_id
)

The resulting bump:

- MUST be stored in UserState.bump
- MUST be verified in every participant instruction

Cross-issuance substitution MUST be impossible.

---

# 4. Escrow Authority Model

deposit_escrow and reward_escrow:

- Are NOT PDAs.
- MUST have authority == issuance PDA.

The program MUST verify:

token_account.owner == SPL Token Program
token_account.mint == expected mint
token_account.authority == issuance PDA

Authority mismatch MUST abort execution.

---

# 5. Seed Stability Rule

The following MUST NOT change in v1.1:

- Seed string literals
- Seed order
- Seed count
- Encoding format

Any modification requires:

- New program deployment
- New version number
