use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use marinade_cpi::cpi::accounts::OrderUnstake;
use marinade_cpi::cpi::order_unstake;
// Explicit path: the crate root re-exports `TicketAccountData` through two
// glob imports, which current compilers reject as ambiguous.
use marinade_cpi::state::TicketAccountData;

use crate::constants::{MARINADE_PROGRAM_ID, MARINADE_STATE, MSOL_MINT, TICKET_SEED};
use crate::error::IntegrationError;

#[derive(Accounts)]
#[instruction(msol_amount: u64, ticket_id: u64)]
pub struct OrderUnstakeSol<'info> {
    /// CHECK: pinned to the Marinade state address.
    #[account(mut, address = MARINADE_STATE)]
    pub state: UncheckedAccount<'info>,

    #[account(mut, address = MSOL_MINT)]
    pub msol_mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = msol_mint,
        associated_token::authority = burn_msol_authority,
    )]
    pub burn_msol_from: Account<'info, TokenAccount>,

    pub burn_msol_authority: Signer<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    /// Zero-initialised ticket account, created here and handed to Marinade.
    /// `init` (not `init_if_needed`) makes reuse of a ticket ID fail up front.
    #[account(
        init,
        payer = payer,
        space = 8 + std::mem::size_of::<TicketAccountData>(),
        seeds = [
            TICKET_SEED,
            state.key().as_ref(),
            burn_msol_authority.key().as_ref(),
            ticket_id.to_le_bytes().as_ref(),
        ],
        bump,
        owner = marinade_finance_program.key(),
    )]
    pub new_ticket_account: Account<'info, TicketAccountData>,

    pub clock: Sysvar<'info, Clock>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,

    /// CHECK: pinned to the Marinade program ID.
    #[account(address = MARINADE_PROGRAM_ID)]
    pub marinade_finance_program: UncheckedAccount<'info>,
}

impl OrderUnstakeSol<'_> {
    pub fn process(&mut self, msol_amount: u64) -> Result<()> {
        require!(msol_amount > 0, IntegrationError::InvalidAmount);
        require!(
            self.burn_msol_from.amount >= msol_amount,
            IntegrationError::InsufficientMsol
        );

        let accounts = OrderUnstake {
            state: self.state.to_account_info(),
            msol_mint: self.msol_mint.to_account_info(),
            burn_msol_from: self.burn_msol_from.to_account_info(),
            burn_msol_authority: self.burn_msol_authority.to_account_info(),
            new_ticket_account: self.new_ticket_account.to_account_info(),
            clock: self.clock.to_account_info(),
            rent: self.rent.to_account_info(),
            token_program: self.token_program.to_account_info(),
        };

        order_unstake(
            CpiContext::new(self.marinade_finance_program.to_account_info(), accounts),
            msol_amount,
        )
    }
}
