# ScholarStream

**"Prove you're a student. Get your scholarship instantly."**

A Soroban smart contract that turns scholarship disbursement from a
months-long paperwork chain into an instant, automatic transaction.

## Problem

Priya is a university student in rural India who qualified for an NGO
scholarship, but disbursement is delayed for months due to manual paperwork
verification and slow bank processing — she risks missing tuition deadlines
and losing her seat.

## Solution

Priya's university issues a verifiable on-chain credential confirming her
enrollment. Once the NGO's smart contract checks that credential, funds
release automatically and instantly to her wallet — no manual approval
chain, no waiting on paperwork to physically move between offices.

## Timeline

- **Day 1–2:** Contract design, `lib.rs` core logic, storage model
- **Day 3:** Test suite (`test.rs`), local testnet deploy
- **Day 4:** Wallet UX polish, demo script, README/pitch polish
- **Day 5:** Buffer + hackathon demo rehearsal

## Stellar Features Used

- **Soroban smart contracts** — programmable credential check + disbursement logic
- **USDC transfers** — the actual scholarship payout asset
- **Trustlines** — required for the student's wallet to hold USDC
- **Digital identity / credential check** — the university-issued enrollment credential gates payout

## Vision and Purpose

Education funding worldwide is chronically slow, opaque, and paperwork-heavy,
which disproportionately hurts the students who can least afford delays.
ScholarStream shows that when eligibility can be verified on-chain, payment
doesn't need a human approval chain at all — it can be instant, auditable,
and tamper-resistant, with every donor and NGO able to verify exactly which
students were paid and when.

## Prerequisites

- Rust (stable toolchain, `wasm32-unknown-unknown` target installed)
- Soroban CLI v21+ (`cargo install --locked soroban-cli`)
- soroban-sdk `21.0.0` (pinned in `Cargo.toml`)

## How to Build

```bash
soroban contract build
```

This produces the compiled Wasm binary at:
`target/wasm32-unknown-unknown/release/scholarstream.wasm`

## How to Test

```bash
cargo test
```

Runs all 5 tests: happy path, missing-credential failure, state
verification, duplicate-claim prevention, and unauthorized-issuer
rejection.

## How to Deploy to Testnet

```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/scholarstream.wasm \
  --source <YOUR_SECRET_KEY> \
  --network testnet
```

This returns a contract ID — save it for the CLI calls below.

## Sample CLI Invocations (dummy arguments)

**Initialize the contract (NGO admin, one-time setup):**

```bash
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source <NGO_ADMIN_SECRET_KEY> \
  --network testnet \
  -- \
  initialize \
  --admin <NGO_ADMIN_ADDRESS> \
  --university <UNIVERSITY_ADDRESS> \
  --token <USDC_TOKEN_CONTRACT_ID> \
  --amount 5000000000
```

**University issues Priya's enrollment credential:**

```bash
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source <UNIVERSITY_SECRET_KEY> \
  --network testnet \
  -- \
  issue_credential \
  --university <UNIVERSITY_ADDRESS> \
  --student <PRIYA_ADDRESS> \
  --verified true
```

**Priya claims her scholarship:**

```bash
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source <PRIYA_SECRET_KEY> \
  --network testnet \
  -- \
  claim_scholarship \
  --student <PRIYA_ADDRESS>
```

## License

MIT