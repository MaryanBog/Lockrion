# Contributing to Lockrion

Lockrion v1.1 is a deterministic, production-grade Solana protocol.

All contributions must preserve:

- Deterministic execution
- Immutability guarantees
- Escrow isolation
- Fixed-width arithmetic safety
- Seed model integrity

---

## Development Environment

Required:

- Rust (rustup)
- Solana CLI 3.1.8 (Agave)
- WSL2 (validator)
- Git Bash (operator layer)

Validator must run in WSL2 only:

solana-test-validator --reset

All operational commands must run from Git Bash.

Do NOT mix environments.

---

## Running Tests

Run full Rust test suite:

cargo test --features test-clock -- --nocapture

Run unit-only tests:

cargo test --test accumulator_unit
cargo test --test processor_unit

Verify production build:

cargo build-sbf --no-default-features

All tests must pass before submitting a PR.

---

## Pull Request Rules

Every PR must:

- Pass full test suite
- Not introduce dynamic behavior
- Not modify seed model
- Not modify state layout without version bump
- Preserve error surface immutability

Changes to:

- State layout
- PDA derivation
- Error codes
- Settlement logic

Require version increment and explicit documentation update.

---

## Commit Message Format

Recommended format:

type(scope): short description

Examples:

fix(accumulator): prevent overflow on day clamp
test(sweep): add boundary validation
docs(readme): update build instructions

---

## Determinism Rule

For identical:

- Input parameters
- Accounts
- State
- Timestamp

The program must produce identical output.

Any PR violating replay determinism will be rejected.