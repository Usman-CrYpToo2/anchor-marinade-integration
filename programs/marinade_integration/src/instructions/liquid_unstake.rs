use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use marinade_cpi::cpi::accounts::LiquidUnstake;
use marinade_cpi::cpi::liquid_unstake;

use crate::constants::{MARINADE_PROGRAM_ID, MARINADE_STATE, MSOL_MINT};
use crate::error::IntegrationError;

#[derive(Accounts)]
pub struct LiquidUnstakeSol<'info> {
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

    /// CHECK: Marinade treasury mSOL account that receives the fee; validated by Marinade.
    #[account(mut)]
    pub treasury_msol_account: UncheckedAccount<'info>,

    #[account(
        mut,
        associated_token::mint = msol_mint,
        associated_token::authority = get_msol_from_authority,
    )]
    pub get_msol_from: Account<'info, TokenAccount>,

    pub get_msol_from_authority: Signer<'info>,

    #[account(mut)]
    pub transfer_sol_to: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,

    /// CHECK: pinned to the Marinade program ID.
    #[account(address = MARINADE_PROGRAM_ID)]
    pub marinade_finance_program: UncheckedAccount<'info>,
}

impl LiquidUnstakeSol<'_> {
    pub fn process(&mut self, msol_amount: u64) -> Result<()> {
        require!(msol_amount > 0, IntegrationError::InvalidAmount);
        require!(
            self.get_msol_from.amount >= msol_amount,
            IntegrationError::InsufficientMsol
        );

        let accounts = LiquidUnstake {
            state: self.state.to_account_info(),
            msol_mint: self.msol_mint.to_account_info(),
            liq_pool_sol_leg_pda: self.liq_pool_sol_leg_pda.to_account_info(),
            liq_pool_msol_leg: self.liq_pool_msol_leg.to_account_info(),
            treasury_msol_account: self.treasury_msol_account.to_account_info(),
            get_msol_from: self.get_msol_from.to_account_info(),
            get_msol_from_authority: self.get_msol_from_authority.to_account_info(),
            transfer_sol_to: self.transfer_sol_to.to_account_info(),
            system_program: self.system_program.to_account_info(),
            token_program: self.token_program.to_account_info(),
        };

        liquid_unstake(
            CpiContext::new(self.marinade_finance_program.to_account_info(), accounts),
            msol_amount,
        )
    }
}
