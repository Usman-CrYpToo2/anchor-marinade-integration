use anchor_lang::prelude::*;

/// Marinade Finance program. Same address on mainnet-beta and devnet.
pub const MARINADE_PROGRAM_ID: Pubkey =
    Pubkey::from_str_const("MarBmsSgKXdrN1egZf5sqe1TMai9K1rChYNDJgjq7aD");

/// Marinade main state account. Same address on mainnet-beta and devnet.
pub const MARINADE_STATE: Pubkey =
    Pubkey::from_str_const("8szGkuLTAux9XMgZ2vtY39jVSowEcpBfFfD8hXSEqdGC");

/// mSOL mint. Same address on mainnet-beta and devnet.
pub const MSOL_MINT: Pubkey = Pubkey::from_str_const("mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So");

/// Seed prefix for unstake ticket PDAs.
pub const TICKET_SEED: &[u8] = b"ticket";
