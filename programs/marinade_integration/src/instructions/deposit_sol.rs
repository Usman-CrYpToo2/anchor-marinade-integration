use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};
use marinade_cpi::cpi::accounts::Deposit;
use marinade_cpi::cpi::deposit;

use crate::constants::{MARINADE_PROGRAM_ID, MARINADE_STATE, MSOL_MINT};
use crate::error::IntegrationError;

#[derive(Accounts)]
pub struct DepositSol<'info> {
    /// CHECK: pinned to the Marinade state address.
    #[account(mut, address = MARINADE_STATE)]
    pub state: UncheckedAccount<'info>,

    #[account(mut, address = MSOL_MINT)]
    pub msol_mint: Account<'info, Mint>,

    /// CHECK: Marinade liquidity pool SOL leg; validated against `state` by Marinade.
    #[account(mut)]
    pub liq_pool_sol_leg_pda: UncheckedAccount<'info>,

    /// CHECK: Marinade liquidity pool mSOL leg; validated against `state` by Marinade.
    #[account(mut)]
    pub liq_pool_msol_leg: UncheckedAccount<'info>,

    /// CHECK: PDA authority of the mSOL leg; validated by Marinade.
    pub liq_pool_msol_leg_authority: UncheckedAccount<'info>,

    /// CHECK: Marinade reserve PDA; validated by Marinade.
    #[account(mut)]
    pub reserve_pda: UncheckedAccount<'info>,

    #[account(mut)]
    pub transfer_from: Signer<'info>,

    #[account(
        init_if_needed,
        payer = transfer_from,
        associated_token::mint = msol_mint,
        associated_token::authority = transfer_from,
    )]
    pub mint_to: Account<'info, TokenAccount>,

    /// CHECK: PDA mint authority of mSOL; validated by Marinade.
    pub msol_mint_authority: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,

    /// CHECK: pinned to the Marinade program ID.
    #[account(address = MARINADE_PROGRAM_ID)]
    pub marinade_finance_program: UncheckedAccount<'info>,
}

impl DepositSol<'_> {
    pub fn process(&mut self, lamports: u64) -> Result<()> {
        require!(lamports > 0, IntegrationError::InvalidAmount);
        require!(
            self.transfer_from.lamports() >= lamports,
            IntegrationError::InsufficientSol
        );

        let accounts = Deposit {
            state: self.state.to_account_info(),
            msol_mint: self.msol_mint.to_account_info(),
            liq_pool_sol_leg_pda: self.liq_pool_sol_leg_pda.to_account_info(),
            liq_pool_msol_leg: self.liq_pool_msol_leg.to_account_info(),
            liq_pool_msol_leg_authority: self.liq_pool_msol_leg_authority.to_account_info(),
            reserve_pda: self.reserve_pda.to_account_info(),
            transfer_from: self.transfer_from.to_account_info(),
            mint_to: self.mint_to.to_account_info(),
            msol_mint_authority: self.msol_mint_authority.to_account_info(),
            system_program: self.system_program.to_account_info(),
            token_program: self.token_program.to_account_info(),
        };

        deposit(
            CpiContext::new(self.marinade_finance_program.to_account_info(), accounts),
            lamports,
        )
    }
}
