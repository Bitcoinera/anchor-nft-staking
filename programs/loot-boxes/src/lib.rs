use anchor_lang::prelude::*;
use anchor_nft_staking::UserStakeInfo;
use anchor_spl::token;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Burn, Mint, MintTo, Token, TokenAccount},
};

declare_id!("E1QecFyDBGpRczaBjmQ7o2ry2wzAX1MgcrQQajroayWc");

#[program]
pub mod loot_boxes {
    use super::*;

    pub fn open_lootbox(ctx: Context<OpenLootbox>, user_points_to_burn: u64) -> Result<()> {
        //require!( !ctx.accounts.lootbox_pointer.is_initialized || ctx.accounts.lootbox_pointer.is_claimed, LootboxErrors::LootboxAlreadyClaimed );

        let mut points = 10;

        loop {
            if points > user_points_to_burn {
                return err!(LootboxErrors::InvalidLootboxNumber);
            }
            if points == user_points_to_burn {
                require!(
                    ctx.accounts.stake_state.total_earned >= user_points_to_burn,
                    LootboxErrors::InvalidLootboxNumber
                );
                break;
            } else {
                points = points * 2;
            }
        }

        msg!("Buring tokens");

        token::burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Burn {
                    mint: ctx.accounts.stake_mint.to_account_info(),
                    from: ctx.accounts.user_stake_ata.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            user_points_to_burn * u64::pow(10, 6),
        )?;

        msg!("DEV tokens burned");

        let available_gear: Vec<Pubkey> = vec![
            "CudtPyZYtPrwJufP6k7gAzPYfV12CLdpu4MHXfLSig9" .parse::<Pubkey>() .unwrap(),
            "E6cg8XYnor3g58o2qP7N2DHV9JNDJgwpNDAh7QrSw72w" .parse::<Pubkey>() .unwrap(),
            "CeMJNozciBS8Ke9Y7saYbJeeaq2vSoEptgXSvoMBq1Q3" .parse::<Pubkey>() .unwrap(),
            "H6fpRxytdX6MpBAffyxMitsqoyhHqE71SwNPsEvt1oQ2" .parse::<Pubkey>() .unwrap(),
            "Hpvp9ZuFnK1E5aBxVM3thGqsybGNBTf6ucXT2ZEX7EPr" .parse::<Pubkey>() .unwrap(),
        ];

        let clock = Clock::get()?;
        let random_number = clock.unix_timestamp as u64 % available_gear.len() as u64;

        let gear_to_mint = available_gear[random_number as usize];

        ctx.accounts.lootbox_pointer.mint = gear_to_mint;
        ctx.accounts.lootbox_pointer.is_claimed = false;
        ctx.accounts.lootbox_pointer.is_initialized = true;

        Ok(())
    }

    pub fn claim_lootbox(ctx: Context<ClaimLootbox>) -> Result<()> {
        require!(
            ctx.accounts.lootbox_pointer.is_initialized,
            LootboxErrors::LootboxNotInitialized
        );
        require!(
            !ctx.accounts.lootbox_pointer.is_claimed,
            LootboxErrors::LootboxAlreadyClaimed
        );

        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.gear_mint.to_account_info(),
                    to: ctx.accounts.user_gear_ata.to_account_info(),
                    authority: ctx.accounts.gear_mint_authority.to_account_info(),
                },
                &[&[
                    b"mint".as_ref(),
                    &[*ctx.bumps.get("gear_mint_authority").unwrap()],
                ]],
            ),
            1,
        )?;

        ctx.accounts.lootbox_pointer.is_claimed = true;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct OpenLootbox<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        init_if_needed,
        payer = user,
        space = std::mem::size_of::<LootboxPointer>() + 8,
        seeds = ["lootbox".as_bytes(), user.key().as_ref()],
        bump
    )]
    pub lootbox_pointer: Account<'info, LootboxPointer>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    // Swap the next two lines out between prod/testing
    
    // #[account(
    //     mut,
    //     address="9p1pxtR5GKJLwGuM8fu9qDvyqv1sbPCDSz9wgBcjVbLW".parse::<Pubkey>().unwrap()
    // )]
    #[account(mut)] // For testing
    pub stake_mint: Account<'info, Mint>,
     #[account(
        mut,
        associated_token::mint=stake_mint,
        associated_token::authority=user
    )]
    pub user_stake_ata: Account<'info, TokenAccount>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    #[account(constraint = stake_state.user_pubkey == user.key())]
    pub stake_state: Account<'info, UserStakeInfo>,
}

#[derive(Accounts)]
pub struct ClaimLootbox<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        seeds = ["lootbox".as_bytes(), user.key().as_ref()],
        bump,
        constraint= lootbox_pointer.is_initialized,
    )]
    pub lootbox_pointer: Account<'info, LootboxPointer>,
    #[account(
        mut,
        constraint=lootbox_pointer.mint==gear_mint.key()
    )]
    pub gear_mint: Account<'info, Mint>,
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = gear_mint,
        associated_token::authority = user,
    )]
    pub user_gear_ata: Account<'info, TokenAccount>,
    /// CHECK: Mint authority
    #[account(
        seeds = ["mint".as_bytes()],
        bump,
    )]
    pub gear_mint_authority: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}



#[account]
pub struct LootboxPointer {
    mint: Pubkey,
    is_claimed: bool,
    is_initialized: bool,
}

#[error_code]
pub enum LootboxErrors {
    #[msg("You do not have enough tokens to open this lootbox")]
    NotEnoughTokens,
    #[msg("Invalid lootbox number")]
    InvalidLootboxNumber,
    #[msg("Lootbox already opened just claim it incase if you have not claimed it")]
    LootboxAlreadyClaimed,
    #[msg("Lootbox not initialized")]
    LootboxNotInitialized,
}
