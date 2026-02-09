use arcis::*;

#[encrypted]
mod private_governance {
    use arcis::*;

    pub struct VotingBallot {
        /// The vote value: 1 for Yes, 0 for No (encrypted)
        pub choice: u32,
        /// Optional: Weight of the voter (e.g., governance token balance)
        pub weight: u32,
    }

    /// Fixed-size batch for FHE circuit compatibility
    pub struct VotingBatch {
        pub ballots: [VotingBallot; 2],
    }

    pub struct TallyResult {
        /// The aggregated total of all "Yes" votes weighted by their power
        pub total_yes_votes: u32,
        /// Proof of execution timestamp or nonce
        pub batch_id: u64,
    }

    /// Tally Votes: Homomorphically sum the batch of encrypted ballots
    #[instruction]
    pub fn tally_votes(
        input_ctxt: Enc<Shared, VotingBatch>,
        batch_id: u64
    ) -> Enc<Shared, TallyResult> {
        let input = input_ctxt.to_arcis();
        
        // Initialize tally with the first ballot's weighted vote
        let mut total_yes = input.ballots[0].choice * input.ballots[0].weight;

        // Add the second ballot (In FHE, we must use fixed iterations)
        total_yes = total_yes + (input.ballots[1].choice * input.ballots[1].weight);

        let result = TallyResult {
            total_yes_votes: total_yes,
            batch_id,
        };

        // Return the encrypted tally to the protocol authority
        input_ctxt.owner.from_arcis(result)
    }
}