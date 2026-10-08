use anchor_lang::prelude::*;

#[error_code]
pub enum IntegrationError {
    #[msg("Amount must be greater than zero")]
    InvalidAmount,
    #[msg("Insufficient SOL balance")]
    InsufficientSol,
    #[msg("Insufficient mSOL balance")]
    InsufficientMsol,
}
