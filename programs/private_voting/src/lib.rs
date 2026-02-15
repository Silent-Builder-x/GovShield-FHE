use anchor_lang::prelude::*;
use arcium_anchor::prelude::*;

const COMP_DEF_OFFSET_VOTE: u32 = comp_def_offset("cast_vote");

declare_id!("HZ62UwC77jaSMEYLJn2R4hWjrdVWWsJWzZ6BrxKHnfbY");

#[arcium_program]
pub mod private_voting {
    use super::*;

    pub fn init_config(ctx: Context<InitConfig>) -> Result<()> {
        init_comp_def(ctx.accounts, None, None)?;
        Ok(())
    }

    /// [New] Create an encrypted ballot box
    pub fn create_poll(ctx: Context<CreatePoll>, proposal_id: u64) -> Result<()> {
        let poll = &mut ctx.accounts.poll;
        poll.authority = ctx.accounts.authority.key();
        poll.proposal_id = proposal_id;
        // Initialize the encrypted counter to 0 (3 u64: Yes, No, Abstain)
        // Assume the frontend has generated ciphertext representing [0,0,0]
        poll.encrypted_tally = [[0u8; 32]; 3]; 
        poll.is_active = true;
        Ok(())
    }

    /// [Upgrade] Submit an encrypted vote
    /// Users encrypt Choice and Weight locally and send them to Arcium for aggregation
    pub fn cast_vote(
        ctx: Context<CastVote>,
        computation_offset: u64,
        encrypted_choice: [u8; 32], // User-encrypted choice (1, 2, 3)
        encrypted_weight: [u8; 32], // User-encrypted weight (Token Balance)
        pubkey: [u8; 32],
        nonce: u128,
    ) -> Result<()> {
        let poll = &ctx.accounts.poll;
        require!(poll.is_active, VotingError::PollClosed);

        ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;
        
        let mut builder = ArgBuilder::new()
            .x25519_pubkey(pubkey)
            .plaintext_u128(nonce);

        // 1. Inject the current on-chain encrypted state (Current Tally)
        for count in &poll.encrypted_tally {
            builder = builder.encrypted_u64(*count);
        }

        // 2. Inject the user's new vote (User Vote)
        builder = builder
            .encrypted_u64(encrypted_choice)
            .encrypted_u64(encrypted_weight);

        queue_computation(
            ctx.accounts,
            computation_offset,
            builder.build(),
            vec![CastVoteCallback::callback_ix(
                computation_offset,
                &ctx.accounts.mxe_account,
                &[]
            )?],
            1,
            0,
        )?;
        Ok(())
    }

    #[arcium_callback(encrypted_ix = "cast_vote")]
    pub fn cast_vote_callback(
        ctx: Context<CastVoteCallback>,
        output: SignedComputationOutputs<CastVoteOutput>,
    ) -> Result<()> {
        let o = match output.verify_output(&ctx.accounts.cluster_account, &ctx.accounts.computation_account) {
            Ok(CastVoteOutput { field_0 }) => field_0,
            Err(_) => return Err(ErrorCode::AbortedComputation.into()),
        };

        // Update the on-chain state to the new ciphertext totals
        let poll = &mut ctx.accounts.poll;
        
        // Arcis returns: new_counts [u64; 3]
        poll.encrypted_tally[0] = o.ciphertexts[0]; // New Yes
        poll.encrypted_tally[1] = o.ciphertexts[1]; // New No
        poll.encrypted_tally[2] = o.ciphertexts[2]; // New Abstain

        msg!("Vote aggregated confidentially. Tally updated.");
        
        emit!(VoteCastEvent {
            poll: poll.key(),
            timestamp: Clock::get()?.unix_timestamp,
        });
        Ok(())
    }

    /// [New] Close the poll and publish results (requires decryption logic)
    /// In Arcium, decryption usually requires additional private key operations or specific Reveal circuits
    /// Here, it is simplified to marking the state
    pub fn close_poll(ctx: Context<ClosePoll>) -> Result<()> {
        let poll = &mut ctx.accounts.poll;
        poll.is_active = false;
        msg!("Poll closed. Results ready for decryption.");
        Ok(())
    }
}

// --- Accounts ---

#[derive(Accounts)]
pub struct CreatePoll<'info> {
    #[account(
        init, 
        payer = authority, 
        space = 8 + 32 + 8 + (32 * 3) + 1, 
        seeds = [b"poll", authority.key().as_ref()],
        bump
    )]
    pub poll: Account<'info, PollAccount>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct PollAccount {
    pub authority: Pubkey,
    pub proposal_id: u64,
    // Stores encrypted counts for [Yes, No, Abstain]
    pub encrypted_tally: [[u8; 32]; 3],
    pub is_active: bool,
}

#[queue_computation_accounts("cast_vote", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64)]
pub struct CastVote<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub poll: Account<'info, PollAccount>, // Read and update
    
    #[account(init_if_needed, space = 9, payer = payer, seeds = [&SIGN_PDA_SEED], bump, address = derive_sign_pda!())]
    pub sign_pda_account: Account<'info, ArciumSignerAccount>,
    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut, address = derive_mempool_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: Mempool
    pub mempool_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_execpool_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: Execpool
    pub executing_pool: UncheckedAccount<'info>,
    #[account(mut, address = derive_comp_pda!(computation_offset, mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: Comp
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_VOTE))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(mut, address = derive_cluster_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    pub cluster_account: Account<'info, Cluster>,
    #[account(mut, address = ARCIUM_FEE_POOL_ACCOUNT_ADDRESS)]
    pub pool_account: Account<'info, FeePool>,
    #[account(mut, address = ARCIUM_CLOCK_ACCOUNT_ADDRESS)]
    pub clock_account: Account<'info, ClockAccount>,
    pub system_program: Program<'info, System>,
    pub arcium_program: Program<'info, Arcium>,
}

#[callback_accounts("cast_vote")]
#[derive(Accounts)]
pub struct CastVoteCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_VOTE))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    /// CHECK: Comp
    pub computation_account: UncheckedAccount<'info>,
    #[account(mut)]
    pub poll: Account<'info, PollAccount>,
    #[account(address = derive_cluster_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    pub cluster_account: Account<'info, Cluster>,
    #[account(address = ::anchor_lang::solana_program::sysvar::instructions::ID)]
    /// CHECK: Sysvar
    pub instructions_sysvar: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct ClosePoll<'info> {
    #[account(mut, has_one = authority)]
    pub poll: Account<'info, PollAccount>,
    pub authority: Signer<'info>,
}

#[init_computation_definition_accounts("cast_vote", payer)]
#[derive(Accounts)]
pub struct InitConfig<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut, address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut)]
    /// CHECK: Def
    pub comp_def_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_mxe_lut_pda!(mxe_account.lut_offset_slot))]
    /// CHECK: Lut
    pub address_lookup_table: UncheckedAccount<'info>,
    #[account(address = LUT_PROGRAM_ID)]
    /// CHECK: Lut Prog
    pub lut_program: UncheckedAccount<'info>,
    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

#[event]
pub struct VoteCastEvent {
    pub poll: Pubkey,
    pub timestamp: i64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Aborted")] AbortedComputation,
    #[msg("No Cluster")] ClusterNotSet,
}

#[error_code]
pub enum VotingError {
    #[msg("Poll is closed")] PollClosed,
}