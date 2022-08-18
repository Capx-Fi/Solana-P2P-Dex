use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{CloseAccount, Mint, Token, TokenAccount, Transfer},
};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod dex {
    use super::*;

    pub fn deposit_token(ctx: Context<Deposit>, _amount: u64) -> Result<()> {
        let transfer_instruction = anchor_spl::token::Transfer {
            from: ctx.accounts.token_user_ata.to_account_info(),
            to: ctx.accounts.user_vault.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
        );

        anchor_spl::token::transfer(cpi_ctx, _amount)?;

        Ok(())
    }

    pub fn withdraw_token(ctx: Context<Withdraw>, _vault_bump: u8, _amount: u64) -> Result<()> {
        let transfer_instruction = anchor_spl::token::Transfer {
            from: ctx.accounts.user_vault.to_account_info(),
            to: ctx.accounts.token_user_ata.to_account_info(),
            authority: ctx.accounts.user_vault.to_account_info(),
        };

        let bump_vector = _vault_bump.to_le_bytes();
        let inner = vec![
            b"user-vault".as_ref(),
            ctx.accounts.token_mint.to_account_info().key.as_ref(),
            ctx.accounts.user.key.as_ref(),
            bump_vector.as_ref(),
        ];
        let outer = vec![inner.as_slice()];
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
            outer.as_slice(),
        );
        anchor_spl::token::transfer(cpi_ctx, _amount)?;

        Ok(())
    }

    pub fn init_order(
        ctx: Context<CreateOrderAccount>,
        _vault_bump: u8,
        _random: Pubkey,
        _token1_amt: u64,
        _token2_amt: u64,
        _expiry_date: u64,
    ) -> Result<()> {
        let now_ts = Clock::get().unwrap().unix_timestamp as u64;
        require!(now_ts < _expiry_date, CustomError::CannotBeInFuture);
        let user_info = &mut ctx.accounts.user_account;
        user_info.user = ctx.accounts.user.to_account_info().key();
        user_info.token1mint = ctx.accounts.token1_mint.to_account_info().key();
        user_info.token2mint = ctx.accounts.token2_mint.to_account_info().key();
        user_info.token1amt = _token1_amt;
        user_info.token2amt = _token2_amt;
        user_info.expirytime = _expiry_date;
        user_info.orderstatus = 1;
        user_info.bump = *ctx.bumps.get("user_account").unwrap();

        let transfer_instruction = anchor_spl::token::Transfer {
            from: ctx.accounts.user_vault.to_account_info(),
            to: ctx.accounts.order_vault.to_account_info(),
            authority: ctx.accounts.user_vault.to_account_info(),
        };
        let bump_vector = _vault_bump.to_le_bytes();
        let inner = vec![
            b"user-vault".as_ref(),
            user_info.token1mint.as_ref(),
            ctx.accounts.user.key.as_ref(),
            bump_vector.as_ref(),
        ];
        let outer = vec![inner.as_slice()];
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
            outer.as_slice(),
        );
        anchor_spl::token::transfer(cpi_ctx, _token1_amt)?;

        Ok(())
    }

    pub fn cancel_order(
        ctx: Context<CancelOrderAccount>,
        _vault_bump: u8,
        _order_bump: u8,
        _random: Pubkey,
        _token1_amt: u64,
        _token2_amt: u64,
        _expiry_date: u64,
    ) -> Result<()> {
        let user_info = &mut ctx.accounts.user_account;
        require!(user_info.orderstatus == 1, CustomError::CannotCancel);
        require!(
            ctx.accounts.token1_mint.to_account_info().key() == user_info.token1mint,
            CustomError::WrongMintGiven
        );
        user_info.orderstatus = 0;

        let transfer_instruction = anchor_spl::token::Transfer {
            from: ctx.accounts.order_vault.to_account_info(),
            to: ctx.accounts.user_vault.to_account_info(),
            authority: ctx.accounts.order_vault.to_account_info(),
        };
        let bump_vector = _order_bump.to_le_bytes();
        let inner = vec![
            b"order-vault".as_ref(),
            _random.as_ref(),
            ctx.accounts.user.key.as_ref(),
            bump_vector.as_ref(),
        ];
        let outer = vec![inner.as_slice()];
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
            outer.as_slice(),
        );
        anchor_spl::token::transfer(cpi_ctx, user_info.token1amt)?;

        Ok(())
    }

    pub fn accept_order(
        ctx: Context<AcceptOrderAccount>,
        _random: Pubkey,
        _intitiator: Pubkey,
        _vault_bump: u8,
        _order_bump: u8,
    ) -> Result<()> {
        let user_info = &mut ctx.accounts.user_account;
        require!(user_info.orderstatus == 1, CustomError::Invalid);
        user_info.orderstatus = 2;

        let transfer_instruction = anchor_spl::token::Transfer {
            from: ctx.accounts.order_vault.to_account_info(),
            to: ctx.accounts.user_vault21.to_account_info(),
            authority: ctx.accounts.order_vault.to_account_info(),
        };
        let bump_vector = _order_bump.to_le_bytes();
        let inner = vec![
            b"order-vault".as_ref(),
            _random.as_ref(),
            _intitiator.as_ref(),
            bump_vector.as_ref(),
        ];
        let outer = vec![inner.as_slice()];
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
            outer.as_slice(),
        );
        anchor_spl::token::transfer(cpi_ctx, user_info.token1amt)?;

        let transfer_instruction2 = anchor_spl::token::Transfer {
            from: ctx.accounts.user_vault22.to_account_info(),
            to: ctx.accounts.user_vault12.to_account_info(),
            authority: ctx.accounts.user_vault22.to_account_info(),
        };
        let bump_vector2 = _vault_bump.to_le_bytes();
        let inner2 = vec![
            b"user-vault".as_ref(),
            ctx.accounts.token2_mint.to_account_info().key.as_ref(),
            ctx.accounts.user.key.as_ref(),
            bump_vector2.as_ref(),
        ];
        let outer2 = vec![inner2.as_slice()];
        let cpi_ctx2 = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction2,
            outer2.as_slice(),
        );
        anchor_spl::token::transfer(cpi_ctx2, user_info.token2amt)?;

        Ok(())
    }
}

#[error_code]
pub enum CustomError {
    CannotCancel,
    CannotBeInFuture,
    Invalid,
    WrongMintGiven,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    pub token_mint: Account<'info, Mint>,
    #[account(
        init_if_needed,
        payer = user,
        seeds = [b"user-vault".as_ref(),token_mint.key().as_ref(),user.key().as_ref()],
        bump,
        token::mint = token_mint,
        token::authority = user_vault,
    )]
    pub user_vault: Box<Account<'info, TokenAccount>>,
    #[account(mut, constraint = token_user_ata.mint ==  token_mint.key(), constraint = token_user_ata.owner == user.key())]
    pub token_user_ata: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    pub token_mint: Account<'info, Mint>,
    #[account(
        init_if_needed,
        payer = user,
        seeds = [b"user-vault".as_ref(),token_mint.key().as_ref(),user.key().as_ref()],
        bump,
        token::mint = token_mint,
        token::authority = user_vault,
    )]
    pub user_vault: Box<Account<'info, TokenAccount>>,
    #[account(mut, constraint = token_user_ata.mint ==  token_mint.key(), constraint = token_user_ata.owner == user.key())]
    pub token_user_ata: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(vault_bump: u8,random : Pubkey)]
pub struct CreateOrderAccount<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        init,
        payer = user,
        space = 8 + 8 + 4 + 200 + 1 + 8, seeds = [b"order-account".as_ref(),random.as_ref(),user.key().as_ref()], bump
    )]
    pub user_account: Box<Account<'info, OrderAccount>>,
    pub token1_mint: Account<'info, Mint>,
    pub token2_mint: Account<'info, Mint>,
    #[account(
        mut,
        seeds = [b"user-vault".as_ref(),token1_mint.key().as_ref(),user.key().as_ref()],
        bump = vault_bump
    )]
    pub user_vault: Box<Account<'info, TokenAccount>>,
    #[account(
        init,
        payer = user,
        seeds = [b"order-vault".as_ref(),random.as_ref(),user.key().as_ref()],
        bump,
        token::mint = token1_mint,
        token::authority = order_vault,
    )]
    pub order_vault: Box<Account<'info, TokenAccount>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(vault_bump: u8,order_bump: u8,random : Pubkey)]
pub struct CancelOrderAccount<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        seeds = [b"order-account".as_ref(),random.as_ref(),user.key().as_ref()], bump=user_account.bump
    )]
    pub user_account: Box<Account<'info, OrderAccount>>,
    pub token1_mint: Account<'info, Mint>,
    #[account(
        mut,
        seeds = [b"user-vault".as_ref(),token1_mint.key().as_ref(),user.key().as_ref()],
        bump = vault_bump
    )]
    pub user_vault: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        seeds = [b"order-vault".as_ref(),random.as_ref(),user.key().as_ref()],
        bump = order_bump
    )]
    pub order_vault: Box<Account<'info, TokenAccount>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(random : Pubkey,initiator : Pubkey,vault_bump: u8,order_bump: u8)]
pub struct AcceptOrderAccount<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        seeds = [b"order-account".as_ref(),random.as_ref(),initiator.as_ref()], bump=user_account.bump
    )]
    pub user_account: Box<Account<'info, OrderAccount>>,
    pub token1_mint: Account<'info, Mint>,
    pub token2_mint: Account<'info, Mint>,
    #[account(
        mut,
        seeds = [b"user-vault".as_ref(),token2_mint.key().as_ref(),user.key().as_ref()],
        bump = vault_bump
    )]
    pub user_vault22: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        seeds = [b"order-vault".as_ref(),random.as_ref(),initiator.as_ref()],
        bump = order_bump
    )]
    pub order_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = user,
        seeds = [b"user-vault".as_ref(),token1_mint.key().as_ref(),user.key().as_ref()],
        bump,
        token::mint = token1_mint,
        token::authority = user_vault21,
    )]
    pub user_vault21: Box<Account<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = user,
        seeds = [b"user-vault".as_ref(),token2_mint.key().as_ref(),initiator.as_ref()],
        bump,
        token::mint = token2_mint,
        token::authority = user_vault12,
    )]
    pub user_vault12: Box<Account<'info, TokenAccount>>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[account]
#[derive(Default)]
pub struct OrderAccount {
    orderid: Pubkey,
    user: Pubkey,
    token1mint: Pubkey,
    token2mint: Pubkey,
    token1amt: u64,
    token2amt: u64,
    expirytime: u64,
    orderstatus: u8,
    bump: u8,
}
