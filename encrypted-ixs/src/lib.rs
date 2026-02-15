use arcis::*;

#[encrypted]
mod private_governance {
    use arcis::*;

    pub struct CurrentTally {
        // Current encrypted vote counts on-chain
        // 0: Yes Votes, 1: No Votes, 2: Abstain
        pub counts: [u64; 3], 
    }

    pub struct UserVote {
        // User's choice (1=Yes, 2=No, 3=Abstain)
        pub choice: u64,
        // User's weight (Token Balance)
        pub weight: u64,
    }

    pub struct UpdateResult {
        // Updated encrypted vote counts
        pub new_counts: [u64; 3],
    }

    #[instruction]
    pub fn cast_vote(
        tally_ctxt: Enc<Shared, CurrentTally>,
        vote_ctxt: Enc<Shared, UserVote>
    ) -> Enc<Shared, UpdateResult> {
        let tally = tally_ctxt.to_arcis();
        let vote = vote_ctxt.to_arcis();
        
        // Use Mux (Multiplexer) to allocate weight to the corresponding bucket
        // If choice == 1 (Yes), add weight to counts[0]
        // If choice == 2 (No), add weight to counts[1]
        // If choice == 3 (Abstain), add weight to counts[2]

        let is_yes = vote.choice == 1;
        let is_no = vote.choice == 2;
        let is_abstain = vote.choice == 3;

        // Calculate increments
        let add_yes = if is_yes { vote.weight } else { 0u64 };
        let add_no = if is_no { vote.weight } else { 0u64 };
        let add_abs = if is_abstain { vote.weight } else { 0u64 };

        // Homomorphic addition
        let new_yes = tally.counts[0] + add_yes;
        let new_no = tally.counts[1] + add_no;
        let new_abs = tally.counts[2] + add_abs;

        let result = UpdateResult {
            new_counts: [new_yes, new_no, new_abs],
        };

        // Return the updated encrypted state to the on-chain program
        tally_ctxt.owner.from_arcis(result)
    }
}