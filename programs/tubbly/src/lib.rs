use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_program;

declare_id!("8JAZNVnYjPLhTdAQQkhY1EAhjjDeKxmLG7fZXn9xyZy4");

#[program]
pub mod tubbly {
    use super::*;

    /// Initialize the program with owner
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let state = &mut ctx.accounts.state;
        state.owner = ctx.accounts.owner.key();
        state.request_counter = 0;
        
        emit!(OwnershipChanged {
            prev_owner: Pubkey::default(),
            new_owner: ctx.accounts.owner.key(),
        });
        
        Ok(())
    }

    /// Submit a balance request
    pub fn submit(
        ctx: Context<Submit>,
        req_id: u128,
        amount: u64,
    ) -> Result<()> {
        let request = &mut ctx.accounts.request;
        
        // Check if request already exists
        require!(
            request.caller == Pubkey::default(),
            ErrorCode::RequestIdAlreadyUsed
        );
        
        // Set request data
        request.req_id = req_id;
        request.caller = ctx.accounts.user.key();
        request.balance = amount;
        request.is_active = true;
        
        emit!(Submission {
            req_id,
            caller: ctx.accounts.user.key(),
            amount,
        });
        
        Ok(())
    }

    /// Confirm a request (only owner)
    pub fn confirm(ctx: Context<Confirm>, req_id: u128) -> Result<()> {
        let state = &ctx.accounts.state;
        let request = &mut ctx.accounts.request;
        let user_account = &mut ctx.accounts.user_account;
        
        // Only owner can confirm
        require!(
            ctx.accounts.owner.key() == state.owner,
            ErrorCode::NotOwner
        );
        
        // Check if request exists and is active
        require!(
            request.is_active,
            ErrorCode::IncorrectRequestId
        );
        
        // Update user balance
        user_account.balance = user_account
            .balance
            .checked_add(request.balance)
            .ok_or(ErrorCode::BalanceOverflow)?;
        
        // Mark request as processed
        let amount = request.balance;
        request.is_active = false;
        request.balance = 0;
        request.caller = Pubkey::default();
        
        emit!(Confirmation {
            req_id,
            user: user_account.owner,
            amount,
        });
        
        Ok(())
    }

    /// Check balance of an account
    pub fn balance_of(ctx: Context<BalanceOf>) -> Result<u64> {
        Ok(ctx.accounts.user_account.balance)
    }

    /// Get request details (only owner)
    pub fn get_request(ctx: Context<GetRequest>) -> Result<RequestData> {
        let state = &ctx.accounts.state;
        let request = &ctx.accounts.request;
        
        // Only owner can view requests
        require!(
            ctx.accounts.viewer.key() == state.owner,
            ErrorCode::NotOwner
        );
        
        Ok(RequestData {
            req_id: request.req_id,
            caller: request.caller,
            balance: request.balance,
            is_active: request.is_active,
        })
    }

    /// Change ownership (only owner)
    pub fn change_ownership(ctx: Context<ChangeOwnership>) -> Result<()> {
        let state = &mut ctx.accounts.state;
        
        // Only current owner can change ownership
        require!(
            ctx.accounts.current_owner.key() == state.owner,
            ErrorCode::NotOwner
        );
        
        // New owner cannot be zero/default
        require!(
            ctx.accounts.new_owner.key() != Pubkey::default(),
            ErrorCode::NewOwnerIsZero
        );
        
        let prev_owner = state.owner;
        state.owner = ctx.accounts.new_owner.key();
        
        emit!(OwnershipChanged {
            prev_owner,
            new_owner: ctx.accounts.new_owner.key(),
        });
        
        Ok(())
    }
}

// ===== Contexts =====

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = owner,
        space = 8 + 32 + 8,
        seeds = [b"state"],
        bump
    )]
    pub state: Account<'info, State>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(req_id: u128)]
pub struct Submit<'info> {
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + 16 + 32 + 8 + 1,
        seeds = [b"request", req_id.to_le_bytes().as_ref()],
        bump
    )]
    pub request: Account<'info, Request>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(req_id: u128)]
pub struct Confirm<'info> {
    #[account(seeds = [b"state"], bump)]
    pub state: Account<'info, State>,
    
    #[account(
        mut,
        seeds = [b"request", req_id.to_le_bytes().as_ref()],
        bump
    )]
    pub request: Account<'info, Request>,
    
    #[account(
        init_if_needed,
        payer = owner,
        space = 8 + 32 + 8,
        seeds = [b"user", request.caller.as_ref()],
        bump
    )]
    pub user_account: Account<'info, UserAccount>,
    
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct BalanceOf<'info> {
    #[account(seeds = [b"user", user.key().as_ref()], bump)]
    pub user_account: Account<'info, UserAccount>,
    pub user: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(req_id: u128)]
pub struct GetRequest<'info> {
    #[account(seeds = [b"state"], bump)]
    pub state: Account<'info, State>,
    
    #[account(seeds = [b"request", req_id.to_le_bytes().as_ref()], bump)]
    pub request: Account<'info, Request>,
    
    pub viewer: Signer<'info>,
}

#[derive(Accounts)]
pub struct ChangeOwnership<'info> {
    #[account(mut, seeds = [b"state"], bump)]
    pub state: Account<'info, State>,
    pub current_owner: Signer<'info>,
    /// CHECK: New owner is validated in the function
    pub new_owner: AccountInfo<'info>,
}

// ===== State Structs =====

#[account]
pub struct State {
    pub owner: Pubkey,
    pub request_counter: u64,
}

#[account]
pub struct Request {
    pub req_id: u128,
    pub caller: Pubkey,
    pub balance: u64,
    pub is_active: bool,
}

#[account]
pub struct UserAccount {
    pub owner: Pubkey,
    pub balance: u64,
}

// ===== Events =====

#[event]
pub struct OwnershipChanged {
    pub prev_owner: Pubkey,
    pub new_owner: Pubkey,
}

#[event]
pub struct Submission {
    pub req_id: u128,
    pub caller: Pubkey,
    pub amount: u64,
}

#[event]
pub struct Confirmation {
    pub req_id: u128,
    pub user: Pubkey,
    pub amount: u64,
}

// ===== Return Types =====

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct RequestData {
    pub req_id: u128,
    pub caller: Pubkey,
    pub balance: u64,
    pub is_active: bool,
}

// ===== Errors =====

#[error_code]
pub enum ErrorCode {
    #[msg("Not owner")]
    NotOwner,
    
    #[msg("Request ID already used")]
    RequestIdAlreadyUsed,
    
    #[msg("Incorrect request ID")]
    IncorrectRequestId,
    
    #[msg("New owner is zero address")]
    NewOwnerIsZero,
    
    #[msg("Balance overflow")]
    BalanceOverflow,
}