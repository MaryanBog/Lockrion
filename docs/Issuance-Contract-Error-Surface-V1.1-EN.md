# Lockrion Issuance Contract
## Error Surface v1.1

Status: Draft  
Scope: Deterministic error definitions  

All errors use u32 codes.

No dynamic error messages are permitted.

---

# 1. Error Code Mapping (Canonical v1.1)

Code | Error Name
-----|------------------------------
1000 | InvalidVersion
1001 | InvalidPDA
1002 | UnauthorizedCaller
1003 | InvalidMint
1004 | InvalidTokenProgram
1005 | InvalidEscrowAuthority
1006 | InvalidFundingAmount
1007 | ReserveAlreadyFunded
1008 | ReserveNotFunded
1009 | DepositWindowClosed
1010 | DepositWindowNotStarted
1011 | ClaimWindowNotStarted
1012 | ClaimWindowClosed
1013 | AlreadyClaimed
1014 | SweepAlreadyExecuted
1015 | ReclaimAlreadyExecuted
1016 | NoParticipation
1017 | InvalidAmount
1018 | ArithmeticOverflow
1019 | ArithmeticUnderflow
1020 | DivisionByZero
1021 | AccountSizeMismatch
1022 | ImmutableFieldMutation
1023 | InvalidTimestampOrder
1024 | InvalidPlatformTreasury
1025 | InvalidUserStateAccount
1026 | InvalidIssuer
1027 | InvariantViolation
1028 | InvalidEscrowAccount
1029 | InvalidAuthority
1030 | TimestampMisaligned
1031 | InvalidInstruction

---

# 2. Deterministic Error Rule

For identical:

- Input parameters
- Accounts
- State
- Timestamp

The program MUST produce
the same error code.

Errors MUST NOT depend on:

- Transaction ordering within same day
- Non-deterministic logic
- External CPI behavior

---

# 3. Error Surface Immutability

Error codes are part of protocol definition.

Codes MUST NOT change in v1.1.

New errors in future versions
require version increment.
