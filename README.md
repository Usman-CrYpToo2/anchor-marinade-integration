# Anchor Marinade Integration

[![CI](https://github.com/Usman-CrYpToo2/anchor-marinade-integration/actions/workflows/ci.yml/badge.svg)](https://github.com/Usman-CrYpToo2/anchor-marinade-integration/actions/workflows/ci.yml)

An Anchor program that integrates [Marinade Finance](https://marinade.finance)
liquid staking through cross-program invocation (CPI). It exposes Marinade's
four user flows (stake, instant unstake, delayed unstake, claim) behind a
single program interface.

The program is non-custodial: it never holds SOL or mSOL. Each instruction
validates the Marinade addresses it depends on and forwards the caller's
accounts to Marinade.

## Instructions

| Instruction | Marinade CPI | Effect |
|---|---|---|
| `deposit(lamports)` | `deposit` | Stakes SOL and mints mSOL to the caller's associated token account, creating it if needed |
| `liquid_unstake(msol_amount)` | `liquid_unstake` | Swaps mSOL for SOL through the liquidity pool; settles immediately, pays the pool fee |
| `order_unstake(msol_amount, ticket_id)` | `order_unstake` | Burns mSOL and opens an unstake ticket; no pool fee, claimable after the epoch ends |
| `claim(ticket_id)` | `claim` | Pays a matured ticket's SOL to its beneficiary |

## Design

| Concern | Approach |
|---|---|
| Marinade addresses | Program ID, state and mSOL mint are pinned in [`constants.rs`](programs/marinade_integration/src/constants.rs). The remaining Marinade accounts are derived from the state and validated by Marinade. |
| Ticket addresses | PDA of this program: `["ticket", state, authority, ticket_id]`. One user can hold many open tickets, and a ticket can be found again from the user and ID alone. |
| Ticket creation | The program creates the PDA zero-initialised, assigns it to Marinade, and Marinade writes it during `order_unstake`. `init` rejects a reused `ticket_id` before the CPI. |
| Claims | Permissionless, as in Marinade. SOL always goes to the ticket's beneficiary, enforced by `address = ticket_account.beneficiary`. |
| Input checks | Zero amounts and amounts above the caller's balance fail with a program error before the CPI. |

## Deployment

| Network | Program ID |
|---|---|
| Devnet | [`Bfjb9g895pAyGfuFGgkf1JcUzDeKdxYk2sDJC76RZ89r`](https://explorer.solana.com/address/Bfjb9g895pAyGfuFGgkf1JcUzDeKdxYk2sDJC76RZ89r?cluster=devnet) |

Devnet transactions from the original deployment:
[deposit](https://explorer.solana.com/tx/m5gUy4kjzqFipgFYE9L8CNGpM3DDBcLni6T5NSmtjegTA9fndokSV6JL6HKdKKMLK6wkP2LqUzSUqN6f9qJmCep?cluster=devnet),
[liquid unstake](https://explorer.solana.com/tx/2w6CWaNUH78aCiFCey4ibYCaVA4JC71S7ZER2LuNjJqjuvFLt13fKHUEtfSv9y7eUzJf63fbacDq92EATE9u7AiJ?cluster=devnet),
[order unstake](https://explorer.solana.com/tx/2PFkCVyzkJeWFaPUoUzGx5r1psEBBCEJbhSyiQfHzqaG3LimFNwJnk1pspAQuHzHSmS2acbMLkqpz5VEoxyPS1TS?cluster=devnet),
[claim](https://explorer.solana.com/tx/3n1q6xe8pQ7PdUWfT2nYPHoFXNFyeyQnFrF7b4KuxK6H55MamQXjyHRL4Y1mHdoJG5EgTGvj59TjMxBPb3ehoShW?cluster=devnet).

## Usage

Requirements: Rust, [Solana CLI](https://docs.anza.xyz/cli/install) 2.1,
[Anchor](https://www.anchor-lang.com/docs/installation) 0.31.1, Node.js and Yarn.

```bash
yarn install
anchor build
anchor test
```

`anchor test` starts a local validator that clones the Marinade program and
the accounts these instructions use from mainnet-beta (see `[test.validator]`
in [`Anchor.toml`](Anchor.toml)), so the suite runs against real Marinade
state with no devnet wallet or funds. On a fresh clone, run `anchor keys sync`
once after the first build so the declared program ID matches your local
program keypair.

## Testing

[`tests/marinade_integration.ts`](tests/marinade_integration.ts) asserts
balances and account state, not just transaction success:

| Test | Asserts |
|---|---|
| Deposit | mSOL minted, and fewer mSOL than SOL deposited (mSOL trades above SOL) |
| Liquid unstake | Exact mSOL burned, SOL received |
| Order unstake | Exact mSOL burned; ticket owned by Marinade with the right state, beneficiary and value |
| Claim | Rejected before the ticket matures, with no SOL moved |
| Input validation | Zero amount, excess mSOL and a reused ticket ID are rejected |

CI runs `cargo fmt`, `clippy -D warnings` and Prettier on every push. The
Anchor suite needs a Solana toolchain and mainnet-beta access, so it runs
locally with `anchor test`.

## Repository structure

| Path | Contents |
|---|---|
| [`programs/marinade_integration/src/instructions/`](programs/marinade_integration/src/instructions) | One module per instruction: account constraints and CPI |
| [`programs/marinade_integration/src/constants.rs`](programs/marinade_integration/src/constants.rs) | Pinned Marinade addresses and the ticket seed |
| [`tests/`](tests) | Integration tests and Marinade account addresses |
| [`audits/`](audits) | Security self-review |

## Origin

Built in July 2025 as a take-home assessment for Drox, then revised for
current toolchains, with stricter account validation and a deterministic test
suite.

## Security

A self-review found 6 issues, including a build failure on current Rust and a
test suite that asserted nothing. All are fixed. See
[`audits/2026-10-self-review.md`](audits/2026-10-self-review.md).

## License

MIT, see [`LICENSE`](LICENSE).
