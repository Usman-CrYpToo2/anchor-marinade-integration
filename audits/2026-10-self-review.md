# Self-review: Anchor Marinade Integration

| | |
|---|---|
| **Scope** | `programs/` and `tests/` at commit `f3bfe17` |
| **Reference** | Marinade Finance program, `marinade-cpi` 0.4.0, Anchor 0.31.1 |
| **Date** | 2026-10 |

## Summary

| ID | Title | Severity | Status |
|---|---|---|---|
| MI-01 | Program does not compile on current Rust | High | Fixed |
| MI-02 | Test suite asserts nothing and cannot pass twice | Medium | Fixed |
| MI-03 | Marinade state account is not pinned | Low | Fixed |
| MI-04 | `init_if_needed` on the ticket account | Low | Fixed |
| MI-05 | Unused accounts and stale documentation | Informational | Fixed |
| MI-06 | Debug log left in `deposit` | Informational | Fixed |

The program holds no funds, so no finding allows theft. Severity reflects
correctness and reviewability.

---

## MI-01: Program does not compile on current Rust

**Severity:** High

```rust
use marinade_cpi::TicketAccountData;
```

`marinade-cpi` is generated from Marinade's IDL, and its crate root
re-exports `TicketAccountData` through two glob imports. Older compilers
accepted this with a future-compatibility warning; current stable rejects it,
so `cargo build` failed with 10 errors.

**Fix.** Import the type by its defining module:
`marinade_cpi::state::TicketAccountData`.

---

## MI-02: Test suite asserts nothing and cannot pass twice

**Severity:** Medium

The tests only logged transaction signatures, so they could not detect a
regression. They also ran on devnet with the author's local keypair, used a
fixed ticket ID (a second run fails on the existing ticket), and called
`claim` immediately after `order_unstake`, which Marinade always rejects
until the epoch ends.

**Fix.** The suite now runs on a local validator that clones Marinade from
mainnet-beta, uses a fresh funded keypair per run, and asserts balances and
account contents. The claim test asserts the early claim is rejected and
moves no SOL.

---

## MI-03: Marinade state account is not pinned

**Severity:** Low

`state` was an unchecked account, relying on Marinade to validate it. Marinade
does reject foreign mints, and the mSOL mint was pinned, so a substituted state
could not mint real mSOL. But `claim` has no mint account, and its ticket seeds
include the state key, so the program accepted any Marinade-owned state there.

**Fix.** `state` is pinned to `MARINADE_STATE` in every instruction.

---

## MI-04: `init_if_needed` on the ticket account

**Severity:** Low

Reusing a `ticket_id` loaded the existing ticket and passed it to Marinade,
which then failed on its own zero-account check with an opaque error.

**Fix.** `init`. A reused ID now fails with Anchor's "account already in use"
before any CPI.

---

## MI-05: Unused accounts and stale documentation

**Severity:** Informational

`liquid_unstake` and `order_unstake` required `associated_token_program`
without using it, which added an account to every transaction. Two signers
were marked `mut` without being written. Doc comments described a `bump`
argument that does not exist, and the program ID constant was named
`MARINADE_ID_DEVNET` although the address is the same on mainnet.

**Fix.** Removed the unused account and `mut` markers, corrected the
documentation, renamed the constant to `MARINADE_PROGRAM_ID`.

---

## MI-06: Debug log left in `deposit`

**Severity:** Informational

`msg!("enter Deposit::process {}", lamports)` spent compute units on every
deposit.

**Fix.** Removed.
