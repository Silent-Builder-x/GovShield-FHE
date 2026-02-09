use anchor_lang::prelude::*;
use arcium_anchor::prelude::*;

const COMP_DEF_OFFSET_TALLY_VOTES: u32 = comp_def_offset("tally_votes");

declare_id!("8ZVcFcKkzj3NAWLGhcVKaLGhQVyQiPCbHnCqRzQeySkD");

#[arcium_program]
pub mod private_voting {
    use super::*;

    pub fn init_tally_votes_comp_def(ctx: Context<InitTallyVotesCompDef>) -> Result<()> {
        init_comp_def(ctx.accounts, None, None)?;
        Ok(())
    }

    pub fn cast_and_tally(
        ctx: Context<TallyVotes>,
        computation_offset: u64,
        choice_1: [u8; 32],
        weight_1: [u8; 32],
        choice_2: [u8; 32],
        weight_2: [u8; 32],
        pubkey: [u8; 32],
        nonce: u128,
    ) -> Result<()> {
        ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;
        
        let args = ArgBuilder::new()
            .x25519_pubkey(pubkey)
            .plaintext_u128(nonce)
            .encrypted_u32(choice_1)
            .encrypted_u32(weight_1)
            .encrypted_u32(choice_2)
            .encrypted_u32(weight_2)
            .build();

        queue_computation(
            ctx.accounts,
            computation_offset,
            args,
            vec![TallyVotesCallback::callback_ix(
                computation_offset,
                &ctx.accounts.mxe_account,
                &[]
            )?],
            1,
            0,
        )?;
        Ok(())
    }

    #[arcium_callback(encrypted_ix = "tally_votes")]
    pub fn tally_votes_callback(
        ctx: Context<TallyVotesCallback>,
        output: SignedComputationOutputs<TallyVotesOutput>,
    ) -> Result<()> {
        let _result = match output.verify_output(&ctx.accounts.cluster_account, &ctx.accounts.computation_account) {
            Ok(res) => res,
            Err(_) => return Err(ErrorCode::AbortedComputation.into()),
        };

        msg!("Governance Tally Successful! Computation finalized in MXE.");

        emit!(TallyEvent {
            batch_id: ctx.accounts.computation_account.key().to_bytes(),
            status: 1,
        });
        Ok(())
    }
}

#[queue_computation_accounts("tally_votes", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64)]
pub struct TallyVotes<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init_if_needed,
        space = 9,
        payer = payer,
        seeds = [&SIGN_PDA_SEED],
        bump,
        address = derive_sign_pda!(),
    )]
    pub sign_pda_account: Account<'info, ArciumSignerAccount>,
    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Account<'info, MXEAccount>,
    #[account(mut, address = derive_mempool_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: Internal Arcium mempool account, safe because it is validated by the Arcium program
    pub mempool_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_execpool_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: Internal Arcium execution pool account, safe because it is validated by the Arcium program
    pub executing_pool: UncheckedAccount<'info>,
    #[account(mut, address = derive_comp_pda!(computation_offset, mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: Internal Arcium computation tracking account, validated via PDA derivation
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_TALLY_VOTES))]
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

#[callback_accounts("tally_votes")]
#[derive(Accounts)]
pub struct TallyVotesCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_TALLY_VOTES))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Account<'info, MXEAccount>,
    /// CHECK: Internal computation account for verifying callback data
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_cluster_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    pub cluster_account: Account<'info, Cluster>,
    #[account(address = ::anchor_lang::solana_program::sysvar::instructions::ID)]
    /// CHECK: Instructions sysvar for cross-program verification
    pub instructions_sysvar: AccountInfo<'info>,
}

#[init_computation_definition_accounts("tally_votes", payer)]
#[derive(Accounts)]
pub struct InitTallyVotesCompDef<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut, address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut)]
    /// CHECK: Uninitialized Arcium computation definition account
    pub comp_def_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_mxe_lut_pda!(mxe_account.lut_offset_slot))]
    /// CHECK: Address lookup table for Arcium node communication
    pub address_lookup_table: UncheckedAccount<'info>,
    #[account(address = LUT_PROGRAM_ID)]
    /// CHECK: Standard Solana LUT program
    pub lut_program: UncheckedAccount<'info>,
    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

#[event]
pub struct TallyEvent {
    pub batch_id: [u8; 32],
    pub status: u8,
}

#[error_code]
pub enum ErrorCode {
    #[msg("The computation was aborted")]
    AbortedComputation,
    #[msg("Cluster not set")]
    ClusterNotSet,
}