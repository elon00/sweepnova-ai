use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

declare_id!("SwpNva1111111111111111111111111111111111111");

pub mod errors;
pub mod state;

use errors::*;
use state::*;

#[program]
pub mod sweepnova {
    use super::*;

    pub fn initialize_protocol(
        ctx: Context<InitializeProtocol>,
        protocol_fee_bps: u16,
    ) -> Result<()> {
        let config = &mut ctx.accounts.protocol_config;
        config.admin = ctx.accounts.admin.key();
        config.emergency_multisig = ctx.accounts.emergency_multisig.key();
        config.fee_treasury = ctx.accounts.fee_treasury.key();
        config.is_paused = false;
        config.protocol_fee_bps = protocol_fee_bps;
        config.bump = ctx.bumps.protocol_config;
        Ok(())
    }

    pub fn create_user_vault(ctx: Context<CreateUserVault>) -> Result<()> {
        let vault = &mut ctx.accounts.user_vault;
        vault.owner = ctx.accounts.owner.key();
        vault.mint = ctx.accounts.mint.key();
        vault.cumulative_deposited = 0;
        vault.active_positions_count = 0;
        vault.bump = ctx.bumps.user_vault;
        Ok(())
    }

    pub fn deposit_fixed_position(
        ctx: Context<DepositFixedPosition>,
        position_id: u64,
        amount: u64,
        duration_seconds: i64,
        interest_rate_bps: u32,
    ) -> Result<()> {
        require!(amount > 0, SweepNovaError::ZeroDeposit);
        require!(duration_seconds >= 86400, SweepNovaError::InvalidDuration);

        // Transfer tokens from user to PDA vault token account
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.vault_token_account.to_account_info(),
            authority: ctx.accounts.owner.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        token::transfer(CpiContext::new(cpi_program, cpi_accounts), amount)?;

        let now = Clock::get()?.unix_timestamp;
        let position = &mut ctx.accounts.deposit_position;
        position.position_id = position_id;
        position.user_vault = ctx.accounts.user_vault.key();
        position.principal_amount = amount;
        position.interest_rate_bps = interest_rate_bps;
        position.creation_timestamp = now;
        position.maturity_timestamp = now + duration_seconds;
        position.status = PositionStatus::Active;
        position.withdrawn_amount = 0;
        position.bump = ctx.bumps.deposit_position;

        let user_vault = &mut ctx.accounts.user_vault;
        user_vault.cumulative_deposited = user_vault.cumulative_deposited.checked_add(amount).unwrap();
        user_vault.active_positions_count = user_vault.active_positions_count.checked_add(1).unwrap();

        Ok(())
    }

    pub fn execute_bounded_sweep(
        ctx: Context<ExecuteBoundedSweep>,
        amount_to_sweep: u64,
    ) -> Result<()> {
        let mandate = &mut ctx.accounts.sweep_mandate;
        require!(mandate.is_active, SweepNovaError::MandateInactive);
        let now = Clock::get()?.unix_timestamp;
        require!(now <= mandate.expiration_timestamp, SweepNovaError::MandateExpired);
        require!(
            now >= mandate.last_sweep_timestamp + mandate.sweep_period_seconds,
            SweepNovaError::SweepPeriodNotElapsed
        );
        require!(
            amount_to_sweep <= mandate.max_sweep_amount_per_period,
            SweepNovaError::SweepAmountExceedsMandateCap
        );

        mandate.last_sweep_timestamp = now;
        mandate.total_swept = mandate.total_swept.checked_add(amount_to_sweep).unwrap();

        // Perform delegated token transfer
        let cpi_accounts = Transfer {
            from: ctx.accounts.source_token_account.to_account_info(),
            to: ctx.accounts.destination_vault.to_account_info(),
            authority: ctx.accounts.vault_authority.to_account_info(),
        };
        let seeds = &[
            b"vault-authority".as_ref(),
            &[ctx.bumps.vault_authority],
        ];
        let signer = &[&seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            signer,
        );
        token::transfer(cpi_ctx, amount_to_sweep)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeProtocol<'info> {
    #[account(
        init,
        payer = admin,
        space = ProtocolConfig::LEN,
        seeds = [b"protocol-config"],
        bump
    )]
    pub protocol_config: Account<'info, ProtocolConfig>,
    #[account(mut)]
    pub admin: Signer<'info>,
    /// CHECK: Emergency multisig safe
    pub emergency_multisig: AccountInfo<'info>,
    /// CHECK: Fee treasury
    pub fee_treasury: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateUserVault<'info> {
    #[account(
        init,
        payer = owner,
        space = UserVault::LEN,
        seeds = [b"user-vault", owner.key().as_ref(), mint.key().as_ref()],
        bump
    )]
    pub user_vault: Account<'info, UserVault>,
    #[account(mut)]
    pub owner: Signer<'info>,
    /// CHECK: Mint pubkey
    pub mint: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(position_id: u64)]
pub struct DepositFixedPosition<'info> {
    #[account(mut)]
    pub user_vault: Account<'info, UserVault>,
    #[account(
        init,
        payer = owner,
        space = DepositPosition::LEN,
        seeds = [b"deposit-position", user_vault.key().as_ref(), position_id.to_le_bytes().as_ref()],
        bump
    )]
    pub deposit_position: Account<'info, DepositPosition>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ExecuteBoundedSweep<'info> {
    #[account(mut)]
    pub sweep_mandate: Account<'info, SweepMandate>,
    #[account(mut)]
    pub source_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub destination_vault: Account<'info, TokenAccount>,
    /// CHECK: PDA signer for delegated sweep
    #[account(seeds = [b"vault-authority"], bump)]
    pub vault_authority: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
}
