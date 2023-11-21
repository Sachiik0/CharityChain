use anchor_lang::prelude::*;
use anchor_lang::solana_program::entrypoint::ProgramResult;

// Declare the program ID
declare_id!("DeSg615KHp13ghYULjsBrXjvUosYYenSPKXqyHawxY2f");

// Define the crowdfunding program module
#[program]
pub mod crowdfunding {
    use super::*;

    // Create a new campaign
    pub fn create(ctx: Context<Create>, name: String, description: String) -> ProgramResult {
        let campaign = &mut ctx.accounts.campaign;
        campaign.name = name;
        campaign.description = description;
        campaign.amount_donated = 0;
        campaign.admin = *ctx.accounts.user.key;
        Ok(())
    }

    // Withdraw funds from the campaign
    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> ProgramResult {
        let campaign = &mut ctx.accounts.campaign;
        let user = &mut ctx.accounts.user;

        // Check if the user is the campaign admin
        if campaign.admin != *user.key {
            return Err(ProgramError::IncorrectProgramId);
        }

        // Check if there are sufficient funds for withdrawal
        let rent_balance = Rent::get()?.minimum_balance(campaign.to_account_info().data_len());
        if **campaign.to_account_info().lamports.borrow() - rent_balance < amount {
            return Err(ProgramError::InsufficientFunds);
        }

        // Perform the withdrawal
        **campaign.to_account_info().try_borrow_mut_lamports()? -= amount;
        **user.to_account_info().try_borrow_mut_lamports()? += amount;

        Ok(())
    }

    // Donate funds to the campaign
    pub fn donate(ctx: Context<Donate>, amount: u64) -> ProgramResult {
        let campaign = &mut ctx.accounts.campaign;
    
        // Check if the campaign is finished
        if campaign.finished {
            return Err(ProgramError::InvalidAccountData);
            // Or handle it in a way appropriate for your use case
        }
    
        // Transfer SOL from the user to the campaign
        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.user.key(),
            &ctx.accounts.campaign.key(),
            amount,
        );
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.user.to_account_info(),
                ctx.accounts.campaign.to_account_info(),
            ],
        );
    
        // Update the amount donated
        (&mut ctx.accounts.campaign).amount_donated += amount;
    
        Ok(())
    }

    // Finish the campaign
    pub fn finish_campaign(ctx: Context<FinishCampaign>) -> ProgramResult {
        let campaign = &mut ctx.accounts.campaign;
        let user = &mut ctx.accounts.user;

        // Check if the user is the campaign admin
        if campaign.admin != *user.key {
            return Err(ProgramError::IncorrectProgramId);
        }

        // Mark the campaign as finished
        campaign.finished = true;

        Ok(())
    }
}

// Define the account structure for a Campaign
#[account]
pub struct Campaign {
    pub admin: Pubkey,
    pub name: String,
    pub description: String,
    pub amount_donated: u64,
    pub finished: bool,
}

// Define the account structure for creating a new Campaign
#[derive(Accounts)]
pub struct Create<'info> {
    #[account(init, payer = user, space = 90000, seeds = [b"CampaignSeed".as_ref(), &user.key().to_bytes()[..]], bump)]
    pub campaign: Account<'info, Campaign>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

// Define the account structure for withdrawing funds from a Campaign
#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub campaign: Account<'info, Campaign>,
    #[account(mut)]
    pub user: Signer<'info>,
}

// Define the account structure for donating to a Campaign
#[derive(Accounts)]
pub struct Donate<'info> {
    #[account(mut)]
    pub campaign: Account<'info, Campaign>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

// Define the account structure for finishing a Campaign
#[derive(Accounts)]
pub struct FinishCampaign<'info> {
    #[account(mut)]
    pub campaign: Account<'info, Campaign>,
    #[account(mut)]
    pub user: Signer<'info>,
}
