# Lockrion Issuance Contract
## State Layout v1.1

Status: Draft  
Standard: Lockrion Issuance Contract v1  
Applies To: Raw Solana Program (no Anchor)  
Scope: Binary account layout specification  

---

# 1. Design Principles

This document defines the exact binary layout of all on-chain accounts
used by Lockrion Issuance Contract v1.1.

The State Layout is part of the protocol definition.
Any modification to this layout requires a new contract version.

The layout is defined byte-by-byte and is binding.

---

## 1.1 Layout Determinism

All account structures:

- Use fixed-size fields only.
- Use fixed ordering.
- Contain no variable-length data.
- Contain no dynamic resizing capability.
- Contain no optional fields.
- Contain no feature-gated compilation differences.

Binary layout MUST remain identical across all builds.

---

## 1.2 Serialization Model

The program uses manual Borsh-compatible serialization
with explicit field ordering.

No automatic macro-based layout generation is permitted.

All fields are serialized in declared order.
No padding is assumed unless explicitly defined.

---

## 1.3 Endianness

All integer fields use little-endian encoding.

This applies to:

- u8
- u64
- u128
- i64

Pubkey fields are stored as raw 32-byte arrays.

---

## 1.4 Alignment and Padding Policy

The layout uses packed representation.

No implicit compiler padding is permitted.

If padding bytes are required,
they MUST be explicitly declared as reserved fields.

---

## 1.5 Versioning Rule

Each account begins with:

- version: u8

The version field defines layout compatibility.

State Layout v1.1 corresponds to:

version = 1

Any future layout modification requires:

- version increment
- new deployment standard

---

# 2. IssuanceState Account Layout

This section defines the exact binary layout of the
IssuanceState account.

The IssuanceState account represents a single issuance instance.
All immutable parameters and global accounting variables
are stored in this account.

The layout below is binding and byte-exact.

---

## 2.1 Field Order and Offsets

Offset | Field                     | Type    | Size (bytes)
-------|--------------------------|---------|--------------
0      | version                  | u8      | 1
1      | bump                     | u8      | 1
2      | issuer_address           | Pubkey  | 32
34     | lock_mint                | Pubkey  | 32
66     | reward_mint              | Pubkey  | 32
98     | deposit_escrow           | Pubkey  | 32
130    | reward_escrow            | Pubkey  | 32
162    | platform_treasury        | Pubkey  | 32
194    | reserve_total            | u128    | 16
210    | start_ts                 | i64     | 8
218    | maturity_ts              | i64     | 8
226    | claim_window             | i64     | 8
234    | final_day_index          | u64     | 8
242    | total_locked             | u128    | 16
258    | total_weight_accum       | u128    | 16
274    | last_day_index           | u64     | 8
282    | reserve_funded           | u8      | 1
283    | sweep_executed           | u8      | 1
284    | reclaim_executed         | u8      | 1
285    | reserved_padding         | [u8; 7] | 7

---

## 2.2 Total Account Size

IssuanceState total size:

292 bytes

This includes:

- All immutable parameters
- All mutable accounting variables
- All settlement flags
- Explicit padding to maintain alignment stability

No additional fields may be appended in v1.1.

---

## 2.3 Field Semantics

version  
Must equal 1 for State Layout v1.1.

bump  
Stores the PDA bump used for issuance account derivation.

issuer_address  
Immutable issuer authority.

lock_mint  
Mint used for participant deposits.

reward_mint  
Mint used for reward distribution.

deposit_escrow  
Token account holding participant deposits.

reward_escrow  
Token account holding reward reserve.

platform_treasury  
Destination account for sweep of unclaimed rewards.

reserve_total  
Total reward amount required to be funded before start.

start_ts  
Issuance start timestamp (UTC).

maturity_ts  
Issuance maturity timestamp (UTC).

claim_window  
Duration after maturity during which claims are allowed.

final_day_index  
Computed as:
(maturity_ts - start_ts) / 86400

total_locked  
Sum of all participant locked amounts.

total_weight_accum  
Global accumulated weight across all days.

last_day_index  
Last day index at which global accumulator was updated.

reserve_funded  
Set to 1 after successful funding of reward escrow.

sweep_executed  
Set to 1 after sweep() execution.

reclaim_executed  
Set to 1 after zero_participation_reclaim() execution.

reserved_padding  
Explicit padding for layout stability.
Must be zero-initialized.
Must not be used for logic.

---

# 3. UserState Account Layout

This section defines the exact binary layout of the
UserState account.

The UserState account represents a single participant
within a specific issuance instance.

Each (issuance, participant) pair MUST have exactly one
UserState account derived via PDA.

The layout below is binding and byte-exact.

---

## 3.1 Field Order and Offsets

Offset | Field                  | Type    | Size (bytes)
-------|-----------------------|---------|--------------
0      | version               | u8      | 1
1      | bump                  | u8      | 1
2      | issuance              | Pubkey  | 32
34     | participant           | Pubkey  | 32
66     | locked_amount         | u128    | 16
82     | user_weight_accum     | u128    | 16
98     | user_last_day_index   | u64     | 8
106    | reward_claimed        | u8      | 1
107    | reserved_padding      | [u8; 5] | 5

---

## 3.2 Total Account Size

UserState total size:

112 bytes

This includes:

- Identity binding to issuance
- Participant binding
- Locked amount tracking
- Accumulator state
- Settlement flag
- Explicit padding for stability

No additional fields may be appended in v1.1.

---

## 3.3 Field Semantics

version  
Must equal 1 for State Layout v1.1.

bump  
Stores the PDA bump used for UserState derivation.

issuance  
Public key of the IssuanceState account.
Prevents cross-issuance substitution.

participant  
Public key of the participant.

locked_amount  
Current amount locked by participant.

user_weight_accum  
Accumulated weight for this participant.

user_last_day_index  
Last day index at which user accumulator was updated.

reward_claimed  
Set to 1 after successful claim_reward() execution.
Irreversible.

reserved_padding  
Explicit padding for layout stability.
Must be zero-initialized.
Must not be used for logic.

---

# 4. PDA Seed Model

This section defines the exact PDA derivation rules
for all program-derived accounts used by
Lockrion Issuance Contract v1.1.

All PDA derivations are deterministic.
All seeds are fixed and part of the protocol definition.

Any modification to seed structure requires a new contract version.

---

## 4.1 IssuanceState PDA

The IssuanceState account MUST be derived using:

Seeds:

- "issuance"
- issuer_address (Pubkey)
- start_ts (i64, little-endian bytes)
- reserve_total (u128, little-endian bytes)

Derivation rule:

PDA = Pubkey::find_program_address(
    [
        b"issuance",
        issuer_address.as_ref(),
        start_ts.to_le_bytes(),
        reserve_total.to_le_bytes()
    ],
    program_id
)

The bump returned by derivation MUST be stored in:

IssuanceState.bump

The program MUST recompute the PDA inside every instruction
and verify that the provided IssuanceState account matches
the derived PDA.

Any mismatch MUST cause immediate abort.

---

## 4.2 UserState PDA

Each participant MUST have exactly one UserState account
per issuance.

Seeds:

- "user"
- issuance_pubkey (Pubkey)
- participant_pubkey (Pubkey)

Derivation rule:

PDA = Pubkey::find_program_address(
    [
        b"user",
        issuance_pubkey.as_ref(),
        participant_pubkey.as_ref()
    ],
    program_id
)

The bump returned by derivation MUST be stored in:

UserState.bump

The program MUST recompute the PDA inside every
participant instruction and verify equality.

Cross-issuance substitution MUST be impossible.

---

## 4.3 Escrow Authority Model

The authority of both:

- deposit_escrow
- reward_escrow

MUST be the IssuanceState PDA.

Escrow token accounts are NOT PDAs of this program,
but MUST:

- Have owner == SPL Token Program
- Have authority == issuance PDA
- Have mint matching expected mint

The program MUST verify these conditions on every use.

---

## 4.4 platform_treasury Binding

The platform_treasury account:

- Is NOT a PDA.
- Is stored immutably in IssuanceState.
- MUST match exactly the account passed to sweep().

The program MUST reject any sweep() call
where provided platform_treasury
does not match stored value.

---

## 4.5 Seed Stability Rule

Seed structure MUST NOT change in v1.1.

Specifically:

- String prefixes MUST remain identical.
- Seed order MUST remain identical.
- Byte encoding MUST remain little-endian.

Any modification to seed model requires:

- New program deployment
- Version increment
- New State Layout document

---

# 5. Size Constants and Allocation Rules

This section defines the exact allocation requirements
for all accounts defined in State Layout v1.1.

Account sizes are fixed and MUST NOT vary at runtime.

---

## 5.1 IssuanceState Allocation Size

IssuanceState total size:

292 bytes

When creating the IssuanceState account:

- space MUST equal exactly 292 bytes
- no extra bytes are permitted
- no smaller allocation is permitted

The account MUST be rent-exempt at creation.

Rent exemption MUST be calculated using:

Rent::get()?.minimum_balance(292)

Failure to allocate correct size MUST cause instruction failure.

---

## 5.2 UserState Allocation Size

UserState total size:

112 bytes

When creating a UserState account:

- space MUST equal exactly 112 bytes
- no dynamic resizing is permitted
- account MUST be rent-exempt

Rent exemption MUST be calculated using:

Rent::get()?.minimum_balance(112)

Under-allocation or over-allocation MUST cause failure.

---

## 5.3 Token Escrow Accounts

deposit_escrow and reward_escrow:

- Are standard SPL Token accounts
- Must follow canonical SPL Token layout
- Must be rent-exempt at creation

The program does NOT create escrow accounts automatically.
They MUST be created prior to issuance initialization
or within the initialization instruction.

Escrow accounts MUST:

- Have owner == SPL Token Program
- Have authority == issuance PDA
- Have mint matching expected mint

---

## 5.4 No Reallocation Rule

Neither IssuanceState nor UserState:

- May be reallocated
- May be resized
- May use realloc()
- May append data

Any attempt to change account size invalidates v1.1 compatibility.

---

## 5.5 Zero Initialization Requirement

All accounts MUST be zero-initialized before first write.

Specifically:

- reserved_padding fields MUST be zero
- flags MUST initialize to 0
- total_locked MUST initialize to 0
- total_weight_accum MUST initialize to 0
- last_day_index MUST initialize to 0
- user_weight_accum MUST initialize to 0
- user_last_day_index MUST initialize to 0
- reward_claimed MUST initialize to 0

Non-zero unexpected data MUST cause initialization failure.

---

## 5.6 Immutable Parameter Rule

The following fields are immutable after initialization:

IssuanceState:

- issuer_address
- lock_mint
- reward_mint
- deposit_escrow
- reward_escrow
- platform_treasury
- reserve_total
- start_ts
- maturity_ts
- claim_window
- final_day_index
- bump
- version

Any mutation of these fields after initialization
is a protocol violation and MUST NOT be permitted.

---

# 6. Layout Finalization and Immutability Clause

This section defines the finalization boundary
for State Layout v1.1.

The binary layout defined in Sections 2–5
is part of the protocol definition.

It is immutable once the contract is deployed.

---

## 6.1 Binding Nature of Layout

The State Layout v1.1 document:

- Defines byte offsets.
- Defines field ordering.
- Defines total account sizes.
- Defines seed model.
- Defines allocation rules.

All implementations of Issuance Contract v1.1
MUST strictly conform to this layout.

Deviation from this layout invalidates compatibility.

---

## 6.2 Version Locking

The version field at offset 0 of each account:

version = 1

MUST be verified during deserialization.

If version != 1:

- The program MUST reject the account.
- No implicit migration is permitted.

There is no automatic upgrade path in v1.1.

---

## 6.3 Upgrade Prohibition

Issuance Contract v1.1 is defined as:

- Non-upgradeable
- Immutable after deployment
- Layout-stable

Any modification to:

- Field ordering
- Field types
- Seed structure
- Account size
- Arithmetic domain

Requires:

- New program deployment
- New version number
- New State Layout document

---

## 6.4 Backward Compatibility Policy

State Layout v1.1:

- Does not guarantee forward compatibility.
- Does not support in-place migrations.
- Does not support layout extension.

If a future version (v1.2+) modifies layout:

- Old accounts remain valid only for v1.1 program.
- New program MUST use a distinct deployment.

---

## 6.5 Canonical Size and Offset Authority

The following values are authoritative:

IssuanceState size: 292 bytes  
UserState size: 112 bytes  

Offsets defined in Sections 2 and 3
are the single source of truth.

No compiler struct representation
may override these definitions.

Manual serialization and deserialization
MUST respect exact byte offsets.

---

## 6.6 Finalization Statement

State Layout v1.1 is now fully specified.

It defines:

- Deterministic binary structure
- Stable PDA derivation model
- Immutable account sizing
- Explicit serialization rules

Any change to this specification
constitutes a protocol version change.

State Layout v1.1 is complete.
