use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_spl::token;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{ Approve, Mint, MintTo, Revoke, Token, TokenAccount },
};
use mpl_token_metadata::{
    ID as METADATA_PROGRAM_ID,
};

declare_id!("AxJjgFUbaQn8ypKgRy56EKco2cQ95oGEN7oJUdo7VCUY");

#[program]
pub mod anchor_nft_staking {

    use super::*;

    pub fn stake(ctx: Context<Stake>) -> Result<()> {
        require!(ctx.accounts.stake_state.stake_state == StakeState::Unstaked, StakeError::AlreadyStaked);

        let clock = Clock::get().unwrap();

        msg!("Approving delegate"); 
        let cpi_approve_program = ctx.accounts.token_program.to_account_info();
        let cpi_approve_accounts = Approve {
            to: ctx.accounts.nft_token_account.to_account_info(),
            delegate: ctx.accounts.program_authority.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };

        let cpi_approve_context = CpiContext::new(cpi_approve_program, cpi_approve_accounts);
        token::approve(cpi_approve_context, 1)?;

        msg!("Freezing token account");
        let authority_bump = *ctx.bumps.get("program_authority").unwrap();
        
        invoke_signed(
            &mpl_token_metadata::instruction::freeze_delegated_account(
                ctx.accounts.metadata_program.key(),
                ctx.accounts.program_authority.key(),
                ctx.accounts.nft_token_account.key(),
                ctx.accounts.nft_edition.key(),
                ctx.accounts.nft_mint.key(),
            ),
            &[
                ctx.accounts.metadata_program.to_account_info(),
                ctx.accounts.program_authority.to_account_info(),
                ctx.accounts.nft_token_account.to_account_info(),
                ctx.accounts.nft_edition.to_account_info(),
                ctx.accounts.nft_mint.to_account_info(),
            ],
            &[&[b"authority", &[authority_bump]]],
        )?;        

        ctx.accounts.stake_state.token_account = ctx.accounts.nft_token_account.key();
        ctx.accounts.stake_state.user_pubkey = ctx.accounts.user.key();
        ctx.accounts.stake_state.stake_state = StakeState::Staked;
        ctx.accounts.stake_state.stake_start_time = clock.unix_timestamp;
        ctx.accounts.stake_state.last_reddem_time = clock.unix_timestamp;
        ctx.accounts.stake_state.is_initialized = true;

        Ok(())
    }

    pub fn redeem(ctx: Context<Redeem>) -> Result<()> {
        require!(ctx.accounts.stake_state.is_initialized, StakeError::NotInitialized);
        require!(ctx.accounts.stake_state.stake_state == StakeState::Staked, StakeError::NotStaked);

        let clock = Clock::get()?;
        msg!("Stake last redeem {}",ctx.accounts.stake_state.last_reddem_time);

        let time_since_last_redeem = clock.unix_timestamp - ctx.accounts.stake_state.last_reddem_time;
        msg!("Time since last redeem {} Seconds",time_since_last_redeem);

        // Swap the next two lines out between prod/testing
        //let redeem_amount = (10 * i64::pow(10, 2) * time_since_last_redeem) / (24 * 60 * 60); // Prod
        let redeem_amount = 10000000; // Testing
        msg!("Eligible redeem amount {}",redeem_amount/100);

        msg!("Minting staking rewards");

        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(), 
                MintTo { 
                    mint: ctx.accounts.stake_mint.to_account_info(), 
                    to: ctx.accounts.user_stake_ata.to_account_info(), 
                    authority: ctx.accounts.stake_authority.to_account_info()
                    }, 
                &[&[
                        b"mint".as_ref(),
                        &[*ctx.bumps.get("stake_authority").unwrap()],
                ]]    
            ), 
            redeem_amount.try_into().unwrap()
        )?;

        ctx.accounts.stake_state.last_reddem_time = clock.unix_timestamp;
        ctx.accounts.stake_state.total_earned += redeem_amount as u64;

        msg!("Updated last redeem time {}", ctx.accounts.stake_state.last_reddem_time);
        Ok(())
    }

    pub fn unstake(ctx: Context<Unstake>) -> Result<()> {
        require!(ctx.accounts.stake_state.is_initialized, StakeError::NotInitialized);
        require!(ctx.accounts.stake_state.stake_state == StakeState::Staked, StakeError::NotStaked);

        msg!("Thawing token account");
        let authority_bump = *ctx.bumps.get("program_authority").unwrap();
        
        invoke_signed(
            &mpl_token_metadata::instruction::thaw_delegated_account(
                ctx.accounts.metadata_program.key(),
                ctx.accounts.program_authority.key(),
                ctx.accounts.nft_token_account.key(),
                ctx.accounts.nft_edition.key(),
                ctx.accounts.nft_mint.key(),
            ),
            &[
                ctx.accounts.metadata_program.to_account_info(),
                ctx.accounts.program_authority.to_account_info(),
                ctx.accounts.nft_token_account.to_account_info(),
                ctx.accounts.nft_edition.to_account_info(),
                ctx.accounts.nft_mint.to_account_info(),
            ],
            &[&[b"authority", &[authority_bump]]],
        )?;  


        msg!("Revoking delegate"); 
        let cpi_revoke_program = ctx.accounts.token_program.to_account_info();
        let cpi_revoke_accounts = Revoke {
            source: ctx.accounts.nft_token_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };

        let cpi_revoke_context = CpiContext::new(cpi_revoke_program, cpi_revoke_accounts);
        token::revoke(cpi_revoke_context)?;

        let clock = Clock::get()?;
        msg!("Stake last redeem {}",ctx.accounts.stake_state.last_reddem_time);

        let time_since_last_redeem = clock.unix_timestamp - ctx.accounts.stake_state.last_reddem_time;
        msg!("Time since last redeem {} Seconds",time_since_last_redeem);
        
        let redeem_amount = (10 * i64::pow(10, 2) * time_since_last_redeem) / (24 * 60 * 60);
        msg!("Eligible redeem amount {}",redeem_amount/100);

        msg!("Minting staking rewards");

        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(), 
                MintTo { 
                    mint: ctx.accounts.stake_mint.to_account_info(), 
                    to: ctx.accounts.user_stake_ata.to_account_info(), 
                    authority: ctx.accounts.stake_authority.to_account_info()
                    }, 
                &[&[
                        b"mint".as_ref(),
                        &[*ctx.bumps.get("stake_authority").unwrap()],
                ]]    
            ), 
            redeem_amount.try_into().unwrap()
        )?;

        ctx.accounts.stake_state.last_reddem_time = clock.unix_timestamp;
        msg!("Updated last redeem time {}", ctx.accounts.stake_state.last_reddem_time);

        ctx.accounts.stake_state.stake_state = StakeState::Unstaked;
        ctx.accounts.stake_state.total_earned += redeem_amount as u64;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut, 
        associated_token::mint = nft_mint, 
        associated_token::authority = user
    )]
    pub nft_token_account: Account<'info, TokenAccount>,
    pub nft_mint: Account<'info, Mint>,
    #[account(
        owner = METADATA_PROGRAM_ID,
    )]
    /// CHECK: Manual validation
    pub nft_edition: UncheckedAccount<'info>,
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + std::mem::size_of::<UserStakeInfo>(),
        seeds = [user.key().as_ref(), nft_token_account.key().as_ref()],
        bump 
    )]
    pub stake_state: Account<'info, UserStakeInfo>,
    /// CHECK: Manual validation
    #[account( mut, seeds = ["authority".as_bytes().as_ref()], bump )]    
    pub program_authority: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub metadata_program: Program<'info, Metadata>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Redeem<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        token::authority = user
    )]
    pub nft_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [user.key().as_ref(), nft_token_account.key().as_ref()],
        bump,
        constraint = *user.key == stake_state.user_pubkey,
        constraint = nft_token_account.key() == stake_state.token_account,
    )]
    pub stake_state: Account<'info, UserStakeInfo>,
    #[account(mut)]
    pub stake_mint : Account<'info, Mint>,
    /// CHECK: Manual validation
    #[account(seeds = ["mint".as_bytes().as_ref()], bump )]
    pub stake_authority: UncheckedAccount<'info>,
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = stake_mint,
        associated_token::authority = user,
    )]
    pub user_stake_ata: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Unstake<'info> {
#[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        token::authority=user
    )]
    pub nft_token_account: Account<'info, TokenAccount>,
    pub nft_mint: Account<'info, Mint>,
    /// CHECK: Manual validation
    #[account(owner = METADATA_PROGRAM_ID)]
    pub nft_edition: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [user.key().as_ref(), nft_token_account.key().as_ref()],
        bump,
        constraint = *user.key == stake_state.user_pubkey,
        constraint = nft_token_account.key() == stake_state.token_account
    )]
    pub stake_state: Account<'info, UserStakeInfo>,
    /// CHECK: manual check
    #[account(mut, seeds=["authority".as_bytes().as_ref()], bump)]
    pub program_authority: UncheckedAccount<'info>,
    #[account(mut)]
    pub stake_mint: Account<'info, Mint>,
    /// CHECK: manual validation
    #[account(seeds = ["mint".as_bytes().as_ref()], bump)]
    pub stake_authority: UncheckedAccount<'info>,
    #[account(
        init_if_needed,
        payer=user,
        associated_token::mint=stake_mint,
        associated_token::authority=user
    )]
    pub user_stake_ata: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub metadata_program: Program<'info, Metadata>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Clone)]
pub struct Metadata;


impl anchor_lang::Id for Metadata {
    fn id() -> Pubkey {
        METADATA_PROGRAM_ID
    }
}


#[account]
pub struct UserStakeInfo {
    pub token_account: Pubkey,
    pub stake_start_time: i64,
    pub last_reddem_time: i64,
    pub total_earned: u64,
    pub user_pubkey: Pubkey,
    pub stake_state: StakeState,
    pub is_initialized: bool,
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, PartialEq)]
pub enum StakeState {
    Unstaked,
    Staked
}

impl Default for StakeState {
    fn default() -> Self {
        StakeState::Unstaked
    }
}

#[error_code]
pub enum StakeError {
    #[msg("NFT is already in stake")]
    AlreadyStaked,
    #[msg("Stake account is not initialized")]
    NotInitialized,
    #[msg("NFT is not staked")]
    NotStaked,
}