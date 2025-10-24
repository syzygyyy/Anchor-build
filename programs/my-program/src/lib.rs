use anchor_lang::prelude::*;

declare_id!("HzGMFQqafY25HaZJ9s3H5m8rUZzppcgeZvEhD7D7Yoyg");

#[program]
pub mod my_program {
    use super::*;

    /// 初始化计数器账户
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let counter = &mut ctx.accounts.counter;
        // 关键：必须显式初始化所有字段
        counter.count = 0;
        counter.authority = ctx.accounts.user.key();
        msg!("计数器已初始化，初始值: {}", counter.count);
        Ok(())
    }

    /// 增加计数器的值
    pub fn increment(ctx: Context<Increment>) -> Result<()> {
        let counter = &mut ctx.accounts.counter;
        counter.count = counter.count.checked_add(1).ok_or(ErrorCode::Overflow)?;
        msg!("计数器增加，当前值: {}", counter.count);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = user,
        space = 8 + Counter::INIT_SPACE,
        seeds = [b"counter", user.key().as_ref()],
        bump
    )]
    pub counter: Account<'info, Counter>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Increment<'info> {
    #[account(
        mut,
        seeds = [b"counter", authority.key().as_ref()],
        bump,
        constraint = counter.authority == authority.key() @ ErrorCode::Unauthorized
    )]
    pub counter: Account<'info, Counter>,
    
    pub authority: Signer<'info>,
}

#[account]
#[derive(InitSpace)]
pub struct Counter {
    pub count: u64,
    pub authority: Pubkey,
}

#[error_code]
pub enum ErrorCode {
    #[msg("未授权的操作")]
    Unauthorized,
    #[msg("计数器溢出")]
    Overflow,
}
