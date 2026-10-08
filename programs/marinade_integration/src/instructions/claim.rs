use anchor_lang::prelude::*;
use marinade_cpi::cpi::accounts::Claim;
use marinade_cpi::cpi::claim;
// See order_unstake.rs for why this path is explicit.
use marinade_cpi::state::TicketAccountData;

use crate::constants::{MARINADE_PROGRAM_ID, MARINADE_STATE, TICKET_SEED};

#[derive(Accounts)]
#[instruction(ticket_id: u64)]
pub struct ClaimSol<'info> {
    /// CHECK: pinned to the Marinade state address.
    #[account(mut, address = MARINADE_STATE)]
    pub state: UncheckedAccount<'info>,

    /// CHECK: Marinade reserve PDA that pays out the claim; validated by Marinade.
    #[account(mut)]
    pub reserve_pda: UncheckedAccount<'info>,

    /// Ticket opened by `order_unstake`. `Account` enforces Marinade ownership
    /// and the ticket discriminator; the seeds bind it to this program.
    #[account(
        mut,
        seeds = [
            TICKET_SEED,
            state.key().as_ref(),
            transfer_sol_to.key().as_ref(),
            ticket_id.to_le_bytes().as_ref(),
        ],
        bump,
    )]
    pub ticket_account: Account<'info, TicketAccountData>,

    /// Ticket beneficiary. Claims are permissionless; SOL always goes here.
    #[account(mut, address = ticket_account.beneficiary)]
    pub transfer_sol_to: SystemAccount<'info>,

    pub clock: Sysvar<'info, Clock>,
    pub system_program: Program<'info, System>,

    /// CHECK: pinned to the Marinade program ID.
    #[account(address = MARINADE_PROGRAM_ID)]
    pub marinade_finance_program: UncheckedAccount<'info>,
}

impl ClaimSol<'_> {
    pub fn process(&mut self) -> Result<()> {
        let accounts = Claim {
            state: self.state.to_account_info(),
            reserve_pda: self.reserve_pda.to_account_info(),
            ticket_account: self.ticket_account.to_account_info(),
            transfer_sol_to: self.transfer_sol_to.to_account_info(),
            clock: self.clock.to_account_info(),
            system_program: self.system_program.to_account_info(),
        };

        claim(CpiContext::new(
            self.marinade_finance_program.to_account_info(),
            accounts,
        ))
    }
}
